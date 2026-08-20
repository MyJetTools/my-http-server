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
    use super::{civil_from_days, to_rfc3339};

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
    fn test_leap_day_of_a_leap_century() {
        // 1600 and 2000 are divisible by 400 - leap.
        assert_eq!(
            to_rfc3339(-11_670_998_400_000_000),
            "1600-02-29T00:00:00.000000+00:00"
        );

        assert_eq!(
            to_rfc3339(951_868_799_000_000),
            "2000-02-29T23:59:59.000000+00:00"
        );
    }

    #[test]
    fn test_february_of_a_non_leap_century() {
        // 1700, 1900 and 2100 are divisible by 100 but not by 400 - February has 28 days,
        // so the day right after it is March 1st.
        assert_eq!(
            to_rfc3339(-8_515_238_401_000_000),
            "1700-02-28T23:59:59.000000+00:00"
        );
        assert_eq!(
            to_rfc3339(-8_515_238_400_000_000),
            "1700-03-01T00:00:00.000000+00:00"
        );

        assert_eq!(
            to_rfc3339(-2_203_934_400_000_000),
            "1900-02-28T12:00:00.000000+00:00"
        );
        assert_eq!(
            to_rfc3339(-2_203_891_200_000_000),
            "1900-03-01T00:00:00.000000+00:00"
        );

        assert_eq!(
            to_rfc3339(4_107_456_000_000_000),
            "2100-02-28T00:00:00.000000+00:00"
        );
        assert_eq!(
            to_rfc3339(4_107_542_400_000_000),
            "2100-03-01T00:00:00.000000+00:00"
        );
    }

    /// Every single day of 1600-01-01..2500-01-01 against a day-by-day walk that applies the
    /// Gregorian leap rule directly - an implementation that shares nothing with
    /// `civil_from_days`. The range holds every leap-rule case there is: leap centuries
    /// (1600, 2000, 2400) and non-leap ones (1700, 1800, 1900, 2100, 2200, 2300).
    #[test]
    fn test_every_day_of_nine_centuries() {
        fn is_leap_year(year: i64) -> bool {
            year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
        }

        fn days_in_month(year: i64, month: u32) -> u32 {
            match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    if is_leap_year(year) {
                        29
                    } else {
                        28
                    }
                }
                _ => panic!("month out of range: {}", month),
            }
        }

        const FIRST_DAY: i64 = -135_140; // 1600-01-01
        const LAST_DAY: i64 = 193_579; // 2500-01-01

        let (mut year, mut month, mut day) = (1600i64, 1u32, 1u32);
        let mut leap_days = 0;

        for days_since_epoch in FIRST_DAY..=LAST_DAY {
            assert_eq!(
                civil_from_days(days_since_epoch),
                (year, month, day),
                "mismatch at day {}",
                days_since_epoch
            );

            if (month, day) == (2, 29) {
                leap_days += 1;
            }

            day += 1;
            if day > days_in_month(year, month) {
                day = 1;
                month += 1;

                if month > 12 {
                    month = 1;
                    year += 1;
                }
            }
        }

        // The walk really covered the whole range...
        assert_eq!((year, month, day), (2500, 1, 2));
        // ...and met every leap year of 1600..=2499: 900/4 = 225, minus the six centuries that are
        // not divisible by 400.
        assert_eq!(leap_days, 219);
    }

    #[test]
    fn test_end_of_four_digit_years() {
        assert_eq!(
            to_rfc3339(253_402_300_799_000_000),
            "9999-12-31T23:59:59.000000+00:00"
        );
    }
}
