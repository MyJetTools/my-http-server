use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use rust_extensions::date_time::DateTimeAsMicroseconds;

/// Refuses a decrypted request body that is a replay: one whose timestamp is too far from the
/// server's clock, or whose salt was already accepted. Runs for encrypted bodies only: a plain one
/// never takes its lock.
///
/// A body passes the clock check while its timestamp is within `max_clock_skew` of the server's
/// clock, in the past or in the future. So its salt is kept, with its timestamp, until
/// `timestamp + max_clock_skew` is behind the server's clock: from then on the clock check refuses
/// a copy by itself. Every encrypted body that comes forgets the pairs that have run out first, so
/// what is kept is bounded by the bodies accepted in the last `2 × max_clock_skew`.
pub(crate) struct ReplayGuard {
    max_clock_skew: Duration,
    accepted: Mutex<AcceptedSalts>,
}

/// Why a body was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Replay {
    /// The sender's clock is `off` from the server's, more than `max` either way.
    ClockOff { off: Duration, max: Duration },
    /// A body with this salt was already accepted.
    SaltSeen,
}

impl ReplayGuard {
    pub(crate) fn new(max_clock_skew: Duration) -> Self {
        Self {
            max_clock_skew,
            accepted: Mutex::new(AcceptedSalts::default()),
        }
    }

    pub(crate) fn max_clock_skew(&self) -> Duration {
        self.max_clock_skew
    }

    /// Accepts a body sent at `timestamp` under `salt`, once. The clock is checked first, so a
    /// body refused for its clock is never remembered. The check and the remembering are one step
    /// under one lock: of two copies that arrive at once, one passes.
    pub(crate) fn accept(
        &self,
        salt: [u8; 16],
        timestamp: DateTimeAsMicroseconds,
    ) -> Result<(), Replay> {
        self.accept_at(salt, timestamp, DateTimeAsMicroseconds::now())
    }

    fn accept_at(
        &self,
        salt: [u8; 16],
        timestamp: DateTimeAsMicroseconds,
        now: DateTimeAsMicroseconds,
    ) -> Result<(), Replay> {
        let off = clock_off(timestamp, now);

        if off > self.max_clock_skew {
            return Err(Replay::ClockOff {
                off,
                max: self.max_clock_skew,
            });
        }

        let window = i64::try_from(self.max_clock_skew.as_micros()).unwrap_or(i64::MAX);

        let accepted = self
            .accepted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .accept(
                salt,
                timestamp.unix_microseconds,
                now.unix_microseconds,
                window,
            );

        if accepted {
            Ok(())
        } else {
            Err(Replay::SaltSeen)
        }
    }
}

/// How far the sender's clock is from the server's, either way. `abs_diff`: a timestamp of
/// `i64::MIN` is far off, not an overflow.
fn clock_off(timestamp: DateTimeAsMicroseconds, now: DateTimeAsMicroseconds) -> Duration {
    Duration::from_micros(now.unix_microseconds.abs_diff(timestamp.unix_microseconds))
}

/// The salts of the accepted bodies, with their timestamps: the earliest timestamp on top.
#[derive(Default)]
struct AcceptedSalts {
    salts: HashSet<[u8; 16]>,
    by_timestamp: BinaryHeap<Reverse<(i64, [u8; 16])>>,
}

impl AcceptedSalts {
    /// `true` the first time `salt` comes, `false` for every copy while it is kept. All times are
    /// unix microseconds, `window` the max clock skew.
    ///
    /// First forgets every salt whose timestamp is more than `window` behind `now`: a copy of its
    /// body would fail the clock check now, so remembering it serves nothing. A body exactly
    /// `window` behind still passes the clock check, so its salt is still kept.
    fn accept(&mut self, salt: [u8; 16], timestamp: i64, now: i64, window: i64) -> bool {
        while let Some(&Reverse((oldest, old_salt))) = self.by_timestamp.peek() {
            if oldest.saturating_add(window) >= now {
                break;
            }

            self.salts.remove(&old_salt);
            self.by_timestamp.pop();
        }

        if !self.salts.insert(salt) {
            return false;
        }

        self.by_timestamp.push(Reverse((timestamp, salt)));
        true
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.salts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: i64 = 1_000_000;
    const NOW: i64 = 1_700_000_000 * SECOND;

    fn at(unix_microseconds: i64) -> DateTimeAsMicroseconds {
        DateTimeAsMicroseconds::new(unix_microseconds)
    }

    fn guard() -> ReplayGuard {
        ReplayGuard::new(Duration::from_secs(10))
    }

    #[test]
    fn the_clock_is_off_either_way() {
        for (timestamp, off) in [
            (NOW, Duration::ZERO),
            (NOW - 10 * SECOND, Duration::from_secs(10)),
            (NOW + 10 * SECOND, Duration::from_secs(10)),
            (NOW - 1, Duration::from_micros(1)),
        ] {
            assert_eq!(clock_off(at(timestamp), at(NOW)), off, "{timestamp}");
        }

        // Far off, never an overflow.
        for timestamp in [i64::MIN, i64::MAX] {
            assert!(clock_off(at(timestamp), at(NOW)) > Duration::from_secs(1 << 40));
        }
    }

    #[test]
    fn a_body_within_the_window_is_accepted_once() {
        let guard = guard();

        for (salt, timestamp) in [(1u8, NOW), (2, NOW - 10 * SECOND), (3, NOW + 10 * SECOND)] {
            assert_eq!(guard.accept_at([salt; 16], at(timestamp), at(NOW)), Ok(()));
            assert_eq!(
                guard.accept_at([salt; 16], at(timestamp), at(NOW)),
                Err(Replay::SaltSeen),
                "{salt}"
            );
        }
    }

    #[test]
    fn a_clock_off_more_than_the_window_is_refused_and_not_remembered() {
        let guard = guard();

        for timestamp in [NOW - 10 * SECOND - 1, NOW + 10 * SECOND + 1, i64::MIN, 0] {
            match guard.accept_at([7; 16], at(timestamp), at(NOW)) {
                Err(Replay::ClockOff { off, max }) => {
                    assert!(off > Duration::from_secs(10), "{timestamp}");
                    assert_eq!(max, Duration::from_secs(10));
                }
                other => panic!("{timestamp}: {other:?}"),
            }
        }

        assert_eq!(guard.accepted.lock().unwrap().len(), 0);
        assert_eq!(guard.accept_at([7; 16], at(NOW), at(NOW)), Ok(()));
    }

    /// A copy is refused by its salt for as long as its clock would let it through, and by its
    /// clock after that - there is no moment in between when it passes.
    #[test]
    fn a_copy_never_passes() {
        let guard = guard();
        let timestamp = NOW + 10 * SECOND;

        assert_eq!(guard.accept_at([1; 16], at(timestamp), at(NOW)), Ok(()));

        for now in [NOW, NOW + 10 * SECOND, timestamp + 10 * SECOND] {
            assert_eq!(
                guard.accept_at([1; 16], at(timestamp), at(now)),
                Err(Replay::SaltSeen),
                "{now}"
            );
        }

        assert!(matches!(
            guard.accept_at([1; 16], at(timestamp), at(timestamp + 10 * SECOND + 1)),
            Err(Replay::ClockOff { .. })
        ));
    }

    #[test]
    fn a_salt_is_forgotten_once_its_timestamp_is_out_of_the_window() {
        let mut accepted = AcceptedSalts::default();
        let window = 10 * SECOND;

        assert!(accepted.accept([1; 16], NOW, NOW, window));
        assert!(accepted.accept([2; 16], NOW + 5 * SECOND, NOW, window));

        // Exactly the window behind: kept.
        assert!(!accepted.accept([1; 16], NOW, NOW + window, window));

        // Past it: the first is forgotten, the second, sent later, is not.
        assert!(accepted.accept([3; 16], NOW + 11 * SECOND, NOW + window + 1, window));
        assert_eq!(accepted.len(), 2);
        assert!(!accepted.accept([2; 16], NOW + 5 * SECOND, NOW + window + 1, window));

        // Every encrypted body forgets what has run out, whatever the order the salts came in.
        assert!(accepted.accept([4; 16], NOW + 60 * SECOND, NOW + 60 * SECOND, window));
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted.by_timestamp.len(), 1);
    }
}
