// `pkg_compile_date_time!` is not used anywhere in this workspace, so nothing else type-checks its
// expansion. It now formats the timestamp with std only (no rust-extensions at build stage), which
// makes a shape check worth having.

#[test]
fn test_compile_date_time_is_rfc3339() {
    const COMPILED_AT: &'static str = my_http_server::macros::pkg_compile_date_time!();

    // 2026-08-20T14:37:05.000001+00:00
    assert_eq!(COMPILED_AT.len(), 32, "unexpected value: {}", COMPILED_AT);

    let (date, rest) = COMPILED_AT.split_once('T').unwrap();
    let (time, offset) = rest.split_once('+').unwrap();
    let (time, microseconds) = time.split_once('.').unwrap();

    assert_eq!(offset, "00:00");
    assert_eq!(microseconds.len(), 6);
    assert!(microseconds.chars().all(|c| c.is_ascii_digit()));

    let date: Vec<&str> = date.split('-').collect();
    let time: Vec<&str> = time.split(':').collect();

    let year: u32 = date[0].parse().unwrap();
    let month: u32 = date[1].parse().unwrap();
    let day: u32 = date[2].parse().unwrap();

    let hour: u32 = time[0].parse().unwrap();
    let minute: u32 = time[1].parse().unwrap();
    let second: u32 = time[2].parse().unwrap();

    assert!(year >= 2026, "compiled in the past: {}", COMPILED_AT);
    assert!((1..=12).contains(&month));
    assert!((1..=31).contains(&day));
    assert!(hour <= 23);
    assert!(minute <= 59);
    assert!(second <= 59);
}
