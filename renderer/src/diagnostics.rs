//! One bounded diagnostic budget shared by receive and render threads.
use std::{
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

const BURST: f64 = 20.0;
const PER_SECOND: f64 = 20.0;
const MAX_MESSAGE_BYTES: usize = 4096;

struct Bucket {
    tokens: f64,
    updated: Instant,
}

pub struct DiagnosticBudget {
    bucket: Mutex<Bucket>,
    suppressed: AtomicU64,
}

impl DiagnosticBudget {
    pub fn new(now: Instant) -> Self {
        Self {
            bucket: Mutex::new(Bucket {
                tokens: BURST,
                updated: now,
            }),
            suppressed: AtomicU64::new(0),
        }
    }

    pub fn allow(&self, now: Instant, critical: bool) -> bool {
        if critical {
            return true;
        }
        let mut bucket = self
            .bucket
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let elapsed = now.saturating_duration_since(bucket.updated).as_secs_f64();
        bucket.updated = bucket.updated.max(now);
        bucket.tokens = (bucket.tokens + elapsed * PER_SECOND).min(BURST);
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            self.suppressed.fetch_add(1, Ordering::Relaxed);
            false
        }
    }

    pub fn suppressed(&self) -> u64 {
        self.suppressed.load(Ordering::Relaxed)
    }
}

pub fn bounded_message(message: &str) -> &str {
    let mut end = message.len().min(MAX_MESSAGE_BYTES);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    &message[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, time::Duration};

    #[test]
    fn burst_refill_and_idle_credit_are_bounded() {
        let now = Instant::now();
        let budget = DiagnosticBudget::new(now);
        for _ in 0..20 {
            assert!(budget.allow(now, false));
        }
        assert!(!budget.allow(now, false));
        assert!(!budget.allow(now + Duration::from_millis(40), false));
        assert!(budget.allow(now + Duration::from_millis(51), false));
        let later = now + Duration::from_secs(3600);
        for _ in 0..20 {
            assert!(budget.allow(later, false));
        }
        assert!(!budget.allow(later, false));
        assert_eq!(budget.suppressed(), 3);
    }

    #[test]
    fn critical_diagnostics_bypass_and_do_not_consume_budget() {
        let now = Instant::now();
        let budget = DiagnosticBudget::new(now);
        for _ in 0..100 {
            assert!(budget.allow(now, true));
        }
        for _ in 0..20 {
            assert!(budget.allow(now, false));
        }
        assert!(!budget.allow(now, false));
        assert!(budget.allow(now, true));
        assert_eq!(budget.suppressed(), 1);
    }

    #[test]
    fn threads_share_one_fixed_size_budget() {
        let now = Instant::now();
        let budget = Arc::new(DiagnosticBudget::new(now));
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let budget = budget.clone();
                std::thread::spawn(move || (0..100).filter(|_| budget.allow(now, false)).count())
            })
            .collect();
        assert_eq!(
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .sum::<usize>(),
            20
        );
        assert_eq!(budget.suppressed(), 380);
    }

    #[test]
    fn utf8_messages_are_bounded_without_splitting_characters() {
        assert_eq!(bounded_message("short"), "short");
        let text = "a".repeat(4095) + "é";
        assert_eq!(bounded_message(&text).len(), 4095);
        assert_eq!(bounded_message(&"é".repeat(3000)).len(), 4096);
    }
}
