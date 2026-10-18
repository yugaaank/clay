use std::{collections::VecDeque, fmt};

use sysinfo::{
    CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System,
};

pub(crate) const HISTORY_LIMIT: usize = 60;
pub(crate) const PROCESS_LIMIT: usize = 50;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MetricPoint {
    pub(crate) sample: u64,
    pub(crate) cpu_percent: f32,
    pub(crate) memory_percent: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProcessSample {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) cpu_percent: f32,
    pub(crate) memory_bytes: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SystemSnapshot {
    pub(crate) sample: u64,
    pub(crate) cpu_percent: f32,
    pub(crate) used_memory_bytes: u64,
    pub(crate) total_memory_bytes: u64,
    pub(crate) uptime_seconds: u64,
    pub(crate) history: Vec<MetricPoint>,
    pub(crate) processes: Vec<ProcessSample>,
}

#[derive(Clone, Debug)]
pub(crate) struct MetricsCollector {
    history: VecDeque<MetricPoint>,
    history_limit: usize,
    process_limit: usize,
    next_sample: u64,
}

impl MetricsCollector {
    pub(crate) fn new(history_limit: usize, process_limit: usize) -> Self {
        Self {
            history: VecDeque::new(),
            history_limit,
            process_limit,
            next_sample: 0,
        }
    }

    pub(crate) fn record(
        &mut self,
        cpu_percent: f32,
        used_memory_bytes: u64,
        total_memory_bytes: u64,
        uptime_seconds: u64,
        mut processes: Vec<ProcessSample>,
    ) -> SystemSnapshot {
        let cpu_percent = bounded_percent(cpu_percent);
        let memory_percent = if total_memory_bytes == 0 {
            0.0
        } else {
            bounded_percent(used_memory_bytes as f32 / total_memory_bytes as f32 * 100.0)
        };
        let point = MetricPoint {
            sample: self.next_sample,
            cpu_percent,
            memory_percent,
        };
        self.next_sample = self.next_sample.saturating_add(1);
        if self.history_limit == 0 {
            self.history.clear();
        } else {
            self.history.push_back(point.clone());
            while self.history.len() > self.history_limit {
                self.history.pop_front();
            }
        }

        sort_and_limit_processes(&mut processes, self.process_limit);

        SystemSnapshot {
            sample: point.sample,
            cpu_percent,
            used_memory_bytes,
            total_memory_bytes,
            uptime_seconds,
            history: self.history.iter().cloned().collect(),
            processes,
        }
    }
}

pub(crate) struct SystemCollector {
    system: System,
    metrics: MetricsCollector,
}

impl SystemCollector {
    pub(crate) fn new() -> Self {
        let process_refresh = ProcessRefreshKind::new().with_cpu().with_memory();
        let refresh_kind = RefreshKind::new()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::new().with_cpu_usage())
            .with_processes(process_refresh);
        Self {
            system: System::new_with_specifics(refresh_kind),
            metrics: MetricsCollector::new(HISTORY_LIMIT, PROCESS_LIMIT),
        }
    }

    pub(crate) fn collect(&mut self) -> Result<SystemSnapshot, CollectorError> {
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            return Err(CollectorError::UnsupportedPlatform);
        }

        self.system.refresh_memory();
        self.system.refresh_cpu_usage();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            ProcessRefreshKind::new().with_cpu().with_memory(),
        );

        if self.system.cpus().is_empty() {
            return Err(CollectorError::NoCpuData);
        }
        let total_memory_bytes = self.system.total_memory();
        if total_memory_bytes == 0 {
            return Err(CollectorError::NoMemoryData);
        }

        let processes = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessSample {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                cpu_percent: nonnegative_finite(process.cpu_usage()),
                memory_bytes: process.memory(),
            })
            .collect();

        Ok(self.metrics.record(
            self.system.global_cpu_usage(),
            self.system.used_memory(),
            total_memory_bytes,
            System::uptime(),
            processes,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollectorError {
    UnsupportedPlatform,
    NoCpuData,
    NoMemoryData,
}

impl fmt::Display for CollectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => formatter.write_str("system metrics are unsupported"),
            Self::NoCpuData => formatter.write_str("CPU metrics are unavailable"),
            Self::NoMemoryData => formatter.write_str("memory metrics are unavailable"),
        }
    }
}

fn bounded_percent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

fn nonnegative_finite(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn sort_and_limit_processes(processes: &mut Vec<ProcessSample>, limit: usize) {
    processes.sort_by(|left, right| {
        right
            .cpu_percent
            .total_cmp(&left.cpu_percent)
            .then_with(|| right.memory_bytes.cmp(&left.memory_bytes))
            .then_with(|| left.pid.cmp(&right.pid))
    });
    processes.truncate(limit);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(pid: u32, cpu_percent: f32, memory_bytes: u64) -> ProcessSample {
        ProcessSample {
            pid,
            name: format!("process-{pid}"),
            cpu_percent,
            memory_bytes,
        }
    }

    #[test]
    fn history_keeps_only_the_newest_points() {
        let mut collector = MetricsCollector::new(3, PROCESS_LIMIT);

        for sample in 0..5 {
            let snapshot = collector.record(sample as f32, 25, 100, sample, Vec::new());
            if sample == 4 {
                assert_eq!(
                    snapshot
                        .history
                        .iter()
                        .map(|point| point.sample)
                        .collect::<Vec<_>>(),
                    vec![2, 3, 4]
                );
            }
        }
    }

    #[test]
    fn zero_history_limit_does_not_retain_samples() {
        let mut collector = MetricsCollector::new(0, PROCESS_LIMIT);
        let snapshot = collector.record(50.0, 1, 2, 0, Vec::new());

        assert!(snapshot.history.is_empty());
        assert_eq!(snapshot.sample, 0);
    }

    #[test]
    fn processes_sort_by_cpu_then_memory_then_pid_and_apply_limit() {
        let mut collector = MetricsCollector::new(HISTORY_LIMIT, 3);
        let snapshot = collector.record(
            10.0,
            1,
            2,
            0,
            vec![
                process(7, 12.0, 10),
                process(5, 40.0, 20),
                process(3, 40.0, 20),
                process(9, 40.0, 30),
                process(1, 2.0, 100),
            ],
        );

        assert_eq!(
            snapshot
                .processes
                .iter()
                .map(|process| process.pid)
                .collect::<Vec<_>>(),
            vec![9, 3, 5]
        );
    }

    #[test]
    fn equal_process_values_have_deterministic_pid_order() {
        let mut collector = MetricsCollector::new(HISTORY_LIMIT, PROCESS_LIMIT);
        let snapshot = collector.record(
            10.0,
            1,
            2,
            0,
            vec![
                process(8, 20.0, 10),
                process(2, 20.0, 10),
                process(5, 20.0, 10),
            ],
        );

        assert_eq!(
            snapshot
                .processes
                .iter()
                .map(|process| process.pid)
                .collect::<Vec<_>>(),
            vec![2, 5, 8]
        );
    }
}
