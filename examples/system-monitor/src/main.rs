mod collector;

use std::time::Duration;

use collector::{MetricPoint, ProcessSample, SystemCollector, SystemSnapshot};
use gpui_framework::prelude::*;
use gpui_framework::raw::{self, Hsla};

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

enum MonitorState {
    Loading,
    Ready(SystemSnapshot),
    Error(String),
}

struct SystemMonitor {
    state: MonitorState,
}

impl SystemMonitor {
    fn new(window: &mut UiWindow, cx: &mut Context<Self>) -> Self {
        cx.spawn_in(window, async move |this, cx| {
            let mut collector = cx
                .background_spawn(async move { SystemCollector::new() })
                .await;
            loop {
                let (next_collector, result) = cx
                    .background_spawn(async move {
                        let mut collector = collector;
                        let result = collector.collect();
                        (collector, result)
                    })
                    .await;
                collector = next_collector;

                if this
                    .update(cx, |monitor, cx| {
                        monitor.state = match result {
                            Ok(snapshot) => MonitorState::Ready(snapshot),
                            Err(error) => MonitorState::Error(error.to_string()),
                        };
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }

                cx.background_executor().timer(REFRESH_INTERVAL).await;
            }
        })
        .detach();

        Self {
            state: MonitorState::Loading,
        }
    }

    fn status_text(&self) -> String {
        match &self.state {
            MonitorState::Loading => "Collecting system metrics…".into(),
            MonitorState::Ready(snapshot) => {
                format!(
                    "Updated sample {} · refreshes every 1 second",
                    snapshot.sample.saturating_add(1)
                )
            }
            MonitorState::Error(error) => format!("Refresh failed: {error} · retrying"),
        }
    }
}

impl Render for SystemMonitor {
    fn render(&mut self, _: &mut UiWindow, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let status = self.status_text();
        let content = raw::div()
            .id("monitor-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p(theme.spacing.lg);
        let content = match &self.state {
            MonitorState::Loading => content.child(state_panel(
                "Collecting system metrics…",
                "The first sample is being collected in the background.",
                theme.colors.muted,
                &theme,
            )),
            MonitorState::Error(error) => content.child(state_panel(
                "Metrics unavailable",
                error,
                theme.colors.danger,
                &theme,
            )),
            MonitorState::Ready(snapshot) => content.child(render_dashboard(snapshot, &theme)),
        };

        raw::div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .child(render_header(&status, &theme))
            .child(content)
    }
}

fn render_header(status: &str, theme: &Theme) -> impl IntoElement {
    raw::div()
        .flex()
        .items_center()
        .justify_between()
        .gap(theme.spacing.md)
        .p(theme.spacing.lg)
        .border_b_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.surface)
        .child(
            raw::div()
                .flex()
                .flex_col()
                .gap(theme.spacing.xs)
                .child(Text::new("System Monitor").heading())
                .child(Text::new("Live Linux CPU, memory, uptime, and process activity").muted()),
        )
        .child(
            raw::div()
                .text_color(theme.colors.muted)
                .child(status.to_owned()),
        )
}

fn render_dashboard(snapshot: &SystemSnapshot, theme: &Theme) -> impl IntoElement {
    let memory = format!(
        "{} / {}",
        format_bytes(snapshot.used_memory_bytes),
        format_bytes(snapshot.total_memory_bytes)
    );

    raw::div()
        .flex()
        .flex_col()
        .gap(theme.spacing.lg)
        .child(
            raw::div()
                .flex()
                .gap(theme.spacing.md)
                .child(metric_card(
                    "CPU",
                    format_percent(snapshot.cpu_percent),
                    "Global usage",
                    theme.colors.accent,
                    theme,
                ))
                .child(metric_card(
                    "Memory",
                    memory,
                    format_percent(memory_percent(snapshot)),
                    theme.colors.focus,
                    theme,
                ))
                .child(metric_card(
                    "Uptime",
                    format_uptime(snapshot.uptime_seconds),
                    "Since last boot",
                    theme.colors.selection,
                    theme,
                )),
        )
        .child(
            raw::div()
                .flex()
                .gap(theme.spacing.md)
                .child(history_panel(
                    "CPU history",
                    &snapshot.history,
                    |point| point.cpu_percent,
                    theme.colors.accent,
                    theme,
                ))
                .child(history_panel(
                    "Memory history",
                    &snapshot.history,
                    |point| point.memory_percent,
                    theme.colors.focus,
                    theme,
                )),
        )
        .child(process_panel(snapshot, theme))
}

fn metric_card(
    title: &str,
    value: String,
    detail: impl Into<raw::SharedString>,
    accent: Hsla,
    theme: &Theme,
) -> impl IntoElement {
    raw::div()
        .flex_1()
        .min_w(px(180.))
        .flex()
        .flex_col()
        .gap(theme.spacing.xs)
        .p(theme.spacing.md)
        .border_1()
        .border_color(accent)
        .rounded(theme.radius)
        .bg(theme.colors.surface)
        .child(Text::new(title).muted())
        .child(raw::div().text_size(px(24.)).child(value))
        .child(Text::new(detail).muted())
}

fn history_panel(
    title: &str,
    points: &[MetricPoint],
    value: fn(&MetricPoint) -> f32,
    color: Hsla,
    theme: &Theme,
) -> impl IntoElement {
    let current = points.last().map(value).unwrap_or(0.0);
    let bars = points.iter().map(|point| {
        let height = (value(point).clamp(0.0, 100.0) * 0.84).max(2.0);
        raw::div()
            .flex_1()
            .min_w(px(2.))
            .h(px(height))
            .rounded(px(2.))
            .bg(color)
    });

    raw::div()
        .flex_1()
        .min_w(px(260.))
        .h(px(156.))
        .flex()
        .flex_col()
        .gap(theme.spacing.sm)
        .p(theme.spacing.md)
        .border_1()
        .border_color(theme.colors.border)
        .rounded(theme.radius)
        .bg(theme.colors.surface)
        .child(
            raw::div()
                .flex()
                .items_center()
                .justify_between()
                .child(Text::new(title))
                .child(Text::new(format_percent(current)).muted()),
        )
        .child(
            raw::div()
                .flex()
                .items_end()
                .gap(px(2.))
                .h(px(100.))
                .when(points.is_empty(), |this| {
                    this.child(Text::new("Waiting for first sample").muted())
                })
                .children(bars),
        )
}

fn process_panel(snapshot: &SystemSnapshot, theme: &Theme) -> impl IntoElement {
    let processes = snapshot.processes.clone();
    let rows = processes
        .clone()
        .into_iter()
        .map(|process| process_row(process, theme));

    raw::div()
        .flex()
        .flex_col()
        .gap(theme.spacing.sm)
        .p(theme.spacing.md)
        .border_1()
        .border_color(theme.colors.border)
        .rounded(theme.radius)
        .bg(theme.colors.surface)
        .child(
            raw::div()
                .flex()
                .items_center()
                .justify_between()
                .child(Text::new("Processes").heading())
                .child(Text::new(format!("Top {} by CPU", processes.len())).muted()),
        )
        .child(process_header(theme))
        .child(
            raw::div()
                .id("process-list")
                .h(px(320.))
                .flex()
                .flex_col()
                .gap(px(1.))
                .overflow_y_scroll()
                .when(processes.is_empty(), |this| {
                    this.child(
                        raw::div()
                            .p(theme.spacing.md)
                            .text_color(theme.colors.muted)
                            .child("No processes are currently available."),
                    )
                })
                .children(rows),
        )
}

fn process_header(theme: &Theme) -> impl IntoElement {
    raw::div()
        .flex()
        .gap(theme.spacing.sm)
        .px(theme.spacing.sm)
        .py(theme.spacing.xs)
        .text_color(theme.colors.muted)
        .child(raw::div().w(px(64.)).child("PID"))
        .child(raw::div().flex_1().child("Process"))
        .child(raw::div().w(px(86.)).child("CPU"))
        .child(raw::div().w(px(110.)).child("Memory"))
}

fn process_row(process: ProcessSample, theme: &Theme) -> impl IntoElement {
    let cpu_color = if process.cpu_percent >= 50.0 {
        theme.colors.danger
    } else if process.cpu_percent >= 20.0 {
        theme.colors.focus
    } else {
        theme.colors.text
    };

    raw::div()
        .flex()
        .items_center()
        .gap(theme.spacing.sm)
        .px(theme.spacing.sm)
        .py(theme.spacing.xs)
        .border_t_1()
        .border_color(theme.colors.border)
        .child(
            raw::div()
                .w(px(64.))
                .text_color(theme.colors.muted)
                .child(process.pid.to_string()),
        )
        .child(
            raw::div()
                .flex_1()
                .min_w_0()
                .truncate()
                .child(if process.name.is_empty() {
                    "[unnamed]".to_owned()
                } else {
                    process.name
                }),
        )
        .child(
            raw::div()
                .w(px(86.))
                .text_color(cpu_color)
                .child(format_percent(process.cpu_percent)),
        )
        .child(
            raw::div()
                .w(px(110.))
                .text_color(theme.colors.muted)
                .child(format_bytes(process.memory_bytes)),
        )
}

fn state_panel(title: &str, detail: &str, color: Hsla, theme: &Theme) -> impl IntoElement {
    raw::div()
        .flex_1()
        .min_h(px(240.))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(theme.spacing.sm)
        .text_color(color)
        .child(Text::new(title).heading())
        .child(Text::new(detail).muted())
}

fn memory_percent(snapshot: &SystemSnapshot) -> f32 {
    if snapshot.total_memory_bytes == 0 {
        0.0
    } else {
        (snapshot.used_memory_bytes as f32 / snapshot.total_memory_bytes as f32 * 100.0)
            .clamp(0.0, 100.0)
    }
}

fn format_percent(value: f32) -> String {
    format!("{:.1}%", value)
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GIB {
        format!("{:.1} GiB", bytes / GIB)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes / MIB)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes / KIB)
    } else {
        format!("{} B", bytes as u64)
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = seconds / 3_600 % 24;
    let minutes = seconds / 60 % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn main() -> Result<(), SystemError> {
    App::new()
        .theme(ThemeMode::Dark)
        .window(
            Window::new("System Monitor").size(1_080., 760.),
            SystemMonitor::new,
        )
        .run()
}
