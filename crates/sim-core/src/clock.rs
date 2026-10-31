//! The game clock. Time is counted in whole ticks (`TICKS_PER_MINUTE` to a game minute), so
//! every minute of every day falls on exactly one tick however long a game runs: what happens
//! on one tick of the day (births and ageing at midnight, the job market, rent) never misses a
//! day.

use crate::TICKS_PER_MINUTE;

/// Game starts on day 1 at 08:00.
const START_MINUTE: u64 = 8 * 60;
const MINUTES_PER_DAY: u64 = 24 * 60;
/// Ticks in a game day.
pub const TICKS_PER_DAY: u64 = MINUTES_PER_DAY * TICKS_PER_MINUTE;
/// Ticks from the midnight before day 1 to the start of the game.
const START_TICKS: u64 = START_MINUTE * TICKS_PER_MINUTE;

/// Ticks simulated per real step (one step = 1 / TICKS_PER_SECOND seconds) for each speed:
/// paused, normal, fast, faster, time-lapse, fastest.
pub const TICKS_PER_STEP: [u32; 6] = [0, 1, 3, 10, 30, 60];
pub const MAX_SPEED: u8 = 5;
/// Speed of "skip quiet hours" while the player's household sleeps or is out.
pub const AUTO_FAST_SPEED: u8 = 4;

/// Ticks since the midnight before day 1.
fn since_midnight(tick: u64) -> u64 {
    tick.saturating_add(START_TICKS)
}

pub fn total_minutes(tick: u64) -> f64 {
    since_midnight(tick) as f64 / TICKS_PER_MINUTE as f64
}

/// Day number, starting at 1.
pub fn day(tick: u64) -> u32 {
    (since_midnight(tick) / TICKS_PER_DAY + 1).min(u64::from(u32::MAX)) as u32
}

/// Day of the week, 0 = Monday (day 1 is a Monday).
pub fn weekday(day: u32) -> u32 {
    (day.max(1) - 1) % 7
}

/// Hour within the current day, `[0, 24)`.
pub fn hour(tick: u64) -> f32 {
    minute_of_day(tick) / 60.0
}

/// The tick at which `day` reaches `minute` (None if that is before the game started).
pub fn tick_at(day: u32, minute: f32) -> Option<u64> {
    let into_day = (f64::from(minute) * TICKS_PER_MINUTE as f64).round() as i64;
    let total = i64::from(day.max(1) - 1) * TICKS_PER_DAY as i64 + into_day - START_TICKS as i64;
    (total >= 0).then_some(total as u64)
}

/// Whether `tick` is the one tick of its day at `minute` (for what happens once a day).
pub fn at_minute(tick: u64, minute: f32) -> bool {
    tick_at(day(tick), minute) == Some(tick)
}

/// Minute within the current day, `[0, 1440)`.
pub fn minute_of_day(tick: u64) -> f32 {
    (since_midnight(tick) % TICKS_PER_DAY) as f32 / TICKS_PER_MINUTE as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MINUTES_PER_TICK;

    #[test]
    fn starts_at_eight_on_day_one() {
        assert_eq!(day(0), 1);
        assert_eq!(minute_of_day(0), 480.0);
    }

    #[test]
    fn weekdays_and_tick_lookup() {
        assert_eq!(weekday(1), 0);
        assert_eq!(weekday(8), 0);
        assert_eq!(weekday(7), 6);
        let t = tick_at(2, 9.0 * 60.0).unwrap();
        assert_eq!(day(t), 2);
        assert!((minute_of_day(t) - 540.0).abs() < 0.01);
        assert_eq!(tick_at(1, 60.0), None, "01:00 on day 1 is before the start");
    }

    #[test]
    fn rolls_over_midnight() {
        let ticks_to_midnight = (16.0 * 60.0 / MINUTES_PER_TICK) as u64;
        assert_eq!(day(ticks_to_midnight), 2);
        assert!(minute_of_day(ticks_to_midnight) < 0.01);
    }

    /// Each day's midnight (and each other minute something happens at) is one tick, which `day`
    /// puts on that day. The f32 clock missed midnight on days 1167-2331, 3497-4661 and 5827 on.
    #[test]
    fn every_day_hits_midnight_exactly_once() {
        for d in 2..=10_000u32 {
            let t = tick_at(d, 0.0).unwrap();
            assert_eq!(day(t), d, "midnight of day {d}");
            assert_eq!(minute_of_day(t), 0.0, "day {d}");
            assert_eq!(day(t - 1), d - 1, "the tick before day {d}'s midnight");
            assert!(at_minute(t, 0.0), "day {d}");
            assert!(
                !at_minute(t - 1, 0.0) && !at_minute(t + 1, 0.0),
                "only one tick of day {d} is midnight"
            );
            for minute in [
                0.05,
                6.0 * 60.0,
                9.5 * 60.0,
                12.0 * 60.0,
                23.9 * 60.0,
                1439.95,
            ] {
                let at = tick_at(d, minute).unwrap();
                assert_eq!(day(at), d, "{minute} on day {d}");
                assert!(at_minute(at, minute), "{minute} on day {d}");
                assert!(
                    !at_minute(at + 1, minute),
                    "{minute} on day {d}, a tick later"
                );
            }
        }
    }

    #[test]
    fn minutes_stay_within_the_day() {
        for d in [2u32, 1167, 2329, 6000, 1_000_000] {
            let t = tick_at(d, 0.0).unwrap();
            for t in t - 3..t + 3 {
                assert!(
                    (0.0..1440.0).contains(&minute_of_day(t)),
                    "day {d}, tick {t}"
                );
                assert!(hour(t) < 24.0);
                assert!(day(t + 1) >= day(t));
            }
        }
        assert_eq!(day(u64::MAX), u32::MAX, "no overflow on a huge tick");
    }
}
