use std::{cmp::Ordering, collections::BinaryHeap};

use crate::protocol::Command;

pub const MAX_PENDING_COMMANDS: usize = 8192;
pub const MAX_PENDING_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_LOOKAHEAD_SECONDS: f64 = 3600.0;

#[derive(Debug, PartialEq, Eq)]
pub enum ScheduleError {
    TimeRange,
    CommandLimit,
    ByteLimit,
}

impl ScheduleError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::TimeRange => "future events must have finite times within one hour",
            Self::CommandLimit => {
                "future schedule is full (8192 commands including bundle containers)"
            }
            Self::ByteLimit => "future schedule payload budget exhausted (16 MiB)",
        }
    }
}

/// A command that must be applied on or after `target_time`, expressed in the
/// renderer process's monotonic clock domain.
#[derive(Debug)]
pub struct ScheduledCommand {
    pub command: Command,
    pub target_time: f64,
    pub sequence: u64,
}

impl PartialEq for ScheduledCommand {
    fn eq(&self, other: &Self) -> bool {
        self.target_time.to_bits() == other.target_time.to_bits() && self.sequence == other.sequence
    }
}

impl Eq for ScheduledCommand {}

impl PartialOrd for ScheduledCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max heap. Reverse both keys so the earliest timestamp
        // and then earliest packet-order sequence are popped first.
        other
            .target_time
            .total_cmp(&self.target_time)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

#[derive(Default)]
pub struct Scheduler {
    pending: BinaryHeap<ScheduledCommand>,
    command_count: usize,
    bytes: usize,
    rejected: u64,
}

impl Scheduler {
    pub fn push(&mut self, command: ScheduledCommand, now: f64) -> Result<(), ScheduleError> {
        let count = command.command.command_count();
        let bytes = std::mem::size_of::<ScheduledCommand>() + command.command.heap_bytes();
        let error = if !command.target_time.is_finite()
            || !now.is_finite()
            || command.target_time - now > MAX_LOOKAHEAD_SECONDS
        {
            Some(ScheduleError::TimeRange)
        } else if count > MAX_PENDING_COMMANDS.saturating_sub(self.command_count) {
            Some(ScheduleError::CommandLimit)
        } else if bytes > MAX_PENDING_BYTES.saturating_sub(self.bytes) {
            Some(ScheduleError::ByteLimit)
        } else {
            None
        };
        if let Some(error) = error {
            self.rejected = self.rejected.saturating_add(1);
            return Err(error);
        }
        self.command_count += count;
        self.bytes += bytes;
        self.pending.push(command);
        Ok(())
    }

    pub fn pop_due(&mut self, now: f64) -> Option<ScheduledCommand> {
        if self
            .pending
            .peek()
            .is_some_and(|command| command.target_time <= now)
        {
            // The peek above proves a value is present.
            let command = self.pending.pop().expect("scheduled command must exist");
            self.command_count -= command.command.command_count();
            self.bytes -= std::mem::size_of::<ScheduledCommand>() + command.command.heap_bytes();
            Some(command)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.command_count
    }

    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn rejected(&self) -> u64 {
        self.rejected
    }

    pub fn clear(&mut self) -> usize {
        let count = self.command_count;
        self.pending.clear();
        self.command_count = 0;
        self.bytes = 0;
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scheduled(target_time: f64, sequence: u64) -> ScheduledCommand {
        ScheduledCommand {
            command: Command::Quit,
            target_time,
            sequence,
        }
    }

    #[test]
    fn releases_only_due_events_in_timestamp_then_packet_order() {
        let mut scheduler = Scheduler::default();
        scheduler.push(scheduled(2.0, 3), 0.0).unwrap();
        scheduler.push(scheduled(1.0, 2), 0.0).unwrap();
        scheduler.push(scheduled(1.0, 1), 0.0).unwrap();

        assert!(scheduler.pop_due(0.9).is_none());
        assert_eq!(scheduler.pop_due(1.0).unwrap().sequence, 1);
        assert_eq!(scheduler.pop_due(1.0).unwrap().sequence, 2);
        assert_eq!(scheduler.len(), 1);
        assert_eq!(scheduler.pop_due(2.0).unwrap().sequence, 3);
        assert_eq!(scheduler.bytes(), 0);
    }

    #[test]
    fn rejects_excess_commands_and_recovers_after_clear() {
        let mut scheduler = Scheduler::default();
        for sequence in 0..MAX_PENDING_COMMANDS {
            scheduler.push(scheduled(10., sequence as u64), 0.).unwrap();
        }
        assert_eq!(
            scheduler.push(scheduled(10., 9999), 0.),
            Err(ScheduleError::CommandLimit)
        );
        assert_eq!(scheduler.rejected(), 1);
        assert_eq!(scheduler.clear(), MAX_PENDING_COMMANDS);
        assert_eq!(scheduler.bytes(), 0);
        scheduler.push(scheduled(10., 10000), 0.).unwrap();
    }

    #[test]
    fn bytes_and_nested_commands_are_charged_atomically() {
        let mut scheduler = Scheduler::default();
        let batch = Command::Batch {
            commands: (0..MAX_PENDING_COMMANDS).map(|_| Command::Quit).collect(),
            target_time: Some(1.),
        };
        assert_eq!(
            scheduler.push(
                ScheduledCommand {
                    command: batch,
                    target_time: 1.,
                    sequence: 1
                },
                0.
            ),
            Err(ScheduleError::CommandLimit)
        );
        assert_eq!(scheduler.len(), 0);
        let large = Command::WindowTitle {
            reply_to: crate::protocol::ReplyTarget::from_sender(
                "127.0.0.1:1".parse().unwrap(),
                true,
            ),
            title: "x".repeat(MAX_PENDING_BYTES),
        };
        assert_eq!(
            scheduler.push(
                ScheduledCommand {
                    command: large,
                    target_time: 1.,
                    sequence: 2
                },
                0.
            ),
            Err(ScheduleError::ByteLimit)
        );
        assert_eq!(scheduler.bytes(), 0);
    }

    #[test]
    fn rejects_invalid_or_excessive_lookahead() {
        let mut scheduler = Scheduler::default();
        for time in [f64::NAN, f64::INFINITY, MAX_LOOKAHEAD_SECONDS + 0.01] {
            assert_eq!(
                scheduler.push(scheduled(time, 1), 0.),
                Err(ScheduleError::TimeRange)
            );
        }
        scheduler
            .push(scheduled(MAX_LOOKAHEAD_SECONDS, 1), 0.)
            .unwrap();
    }
}
