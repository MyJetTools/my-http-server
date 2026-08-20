// This crate runs at compile time, so every crate it depends on is built a second time - for the
// host - next to the target build. Formatting a single timestamp is not worth compiling
// `rust-extensions` twice, so the date here is produced with std only.

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;
const MICROSECONDS_PER_SECOND: i64 = 1_000_000;

pub fn now_as_rfc3339() -> String {
    let unix_microseconds = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(since_epoch) => since_epoch.as_micros() as i64,
        Err(before_epoch) => -(before_epoch.duration().as_micros() as i64),
    };

    to_rfc3339(unix_microseconds)
}

/// UTC, microsecond precision - the same shape the previous
/// `DateTimeAsMicroseconds::now().to_rfc3339()` produced: `2025-08-20T09:15:42.123456+00:00`.
fn to_rfc3339(unix_microseconds: i64) -> String {
    let seconds = unix_microseconds.div_euclid(MICROSECONDS_PER_SECOND);
    let microseconds = unix_microseconds.rem_euclid(MICROSECONDS_PER_SECOND);

    let days = seconds.div_euclid(SECONDS_PER_DAY);
    let second_of_day = seconds.rem_euclid(SECONDS_PER_DAY);

    let (year, month, day) = civil_from_days(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}+00:00",
        year,
        month,
        day,
        second_of_day / 3600,
        second_of_day % 3600 / 60,
        second_of_day % 60,
        microseconds
    )
}

/// Days since 1970-01-01 -> (year, month, day). Howard Hinnant's `civil_from_days`: it shifts the
/// epoch to 0000-03-01 so that the leap day lands at the end of the year, which makes the
/// 400-year era arithmetic branchless.
fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let shifted = days_since_epoch + 719_468;

    let era = if shifted >= 0 { shifted } else { shifted - 146_096 } / 146_097;
    let day_of_era = shifted - era * 146_097; // [0, 146096]

    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365; // [0, 399]

    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // [0, 365]

    let month_position = (5 * day_of_year + 2) / 153; // [0, 11], where 0 is March

    let day = (day_of_year - (153 * month_position + 2) / 5 + 1) as u32; // [1, 31]

    let month = if month_position < 10 {
        month_position + 3
    } else {
        month_position - 9
    } as u32; // [1, 12]

    let year = if month <= 2 { year + 1 } else { year };

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::to_rfc3339;

    #[test]
    fn test_unix_epoch() {
        assert_eq!(to_rfc3339(0), "1970-01-01T00:00:00.000000+00:00");
    }

    #[test]
    fn test_date_time_with_microseconds() {
        assert_eq!(
            to_rfc3339(1_755_681_342_123_456),
            "2025-08-20T09:15:42.123456+00:00"
        );
    }

    #[test]
    fn test_today() {
        assert_eq!(
            to_rfc3339(1_787_236_625_000_001),
            "2026-08-20T14:37:05.000001+00:00"
        );
    }

    #[test]
    fn test_leap_day() {
        assert_eq!(
            to_rfc3339(1_709_164_800_000_000),
            "2024-02-29T00:00:00.000000+00:00"
        );
    }

    #[test]
    fn test_before_unix_epoch() {
        assert_eq!(to_rfc3339(-1), "1969-12-31T23:59:59.999999+00:00");
    }

    #[test]
    fn test_end_of_four_digit_years() {
        assert_eq!(
            to_rfc3339(253_402_300_799_000_000),
            "9999-12-31T23:59:59.000000+00:00"
        );
    }
}
