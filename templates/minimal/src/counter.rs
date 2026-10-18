/// UI-independent state, so transitions can be tested without opening a window.
#[derive(Default)]
pub struct Counter {
    value: u64,
}

impl Counter {
    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value = self.value.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::Counter;

    #[test]
    fn counter_starts_at_zero() {
        assert_eq!(Counter::default().value(), 0);
    }

    #[test]
    fn each_increment_advances_the_counter() {
        let mut counter = Counter::default();
        for expected in 1..=5 {
            counter.increment();
            assert_eq!(counter.value(), expected);
        }
    }

    #[test]
    fn increment_does_not_wrap_at_the_limit() {
        let mut counter = Counter { value: u64::MAX };
        counter.increment();
        assert_eq!(counter.value(), u64::MAX);
    }
}
