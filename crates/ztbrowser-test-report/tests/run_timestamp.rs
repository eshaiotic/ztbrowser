//! The start time of a run, read from nextest's JUnit timestamp with its UTC offset.

use ztbrowser_test_report::run_timestamp::RunTimestamp;

struct TimestampScenario {
    text: String,
}

impl TimestampScenario {
    fn given_timestamp(text: &str) -> Self {
        Self { text: text.to_owned() }
    }

    fn when_parsed(&self) -> TimestampOutcome {
        TimestampOutcome { timestamp: RunTimestamp::parse(&self.text) }
    }
}

struct TimestampOutcome {
    timestamp: Option<RunTimestamp>,
}

impl TimestampOutcome {
    fn then_is_unix_second(&self, expected: i64) {
        assert_eq!(self.timestamp.as_ref().map(RunTimestamp::unix_seconds), Some(expected));
    }

    fn then_file_stamp_is(&self, expected: &str) {
        assert_eq!(self.timestamp.as_ref().map(RunTimestamp::file_stamp).as_deref(), Some(expected));
    }

    fn then_label_is(&self, expected: &str) {
        assert_eq!(self.timestamp.as_ref().map(RunTimestamp::polish_label).as_deref(), Some(expected));
    }

    fn then_age_after_seconds_is(&self, elapsed: i64, expected: &str) {
        let timestamp = self.timestamp.as_ref().expect("timestamp should parse");
        assert_eq!(timestamp.describe_age(timestamp.unix_seconds() + elapsed), expected);
    }

    fn then_is_rejected(&self) {
        assert_eq!(self.timestamp, None);
    }
}

#[test]
fn should_take_the_offset_into_account_for_the_absolute_instant() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04.102+01:00")
        .when_parsed()
        .then_is_unix_second(1_793_693_644);
}

#[test]
fn should_read_utc_written_as_z() {
    TimestampScenario::given_timestamp("2026-11-03T08:14:04Z").when_parsed().then_is_unix_second(1_793_693_644);
}

#[test]
fn should_handle_a_leap_day_and_a_negative_offset() {
    TimestampScenario::given_timestamp("2024-02-29T23:59:59-05:30").when_parsed().then_is_unix_second(1_709_270_999);
}

#[test]
fn should_count_backwards_before_1970() {
    TimestampScenario::given_timestamp("1969-12-31T23:00:00+00:00").when_parsed().then_is_unix_second(-3600);
}

#[test]
fn should_name_the_report_file_after_the_local_time_of_the_run() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04.102+01:00")
        .when_parsed()
        .then_file_stamp_is("20261103-0914");
}

#[test]
fn should_label_the_run_with_a_polish_month_abbreviation() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04.102+01:00")
        .when_parsed()
        .then_label_is("3 lis 2026, 09:14");
}

#[test]
fn should_describe_a_run_less_than_a_minute_ago_as_just_now() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04+01:00")
        .when_parsed()
        .then_age_after_seconds_is(59, "przed chwilą");
}

#[test]
fn should_describe_a_recent_run_in_minutes() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04+01:00")
        .when_parsed()
        .then_age_after_seconds_is(7 * 60, "7 min temu");
}

#[test]
fn should_describe_a_run_from_yesterday_in_the_singular() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04+01:00")
        .when_parsed()
        .then_age_after_seconds_is(30 * 3600, "1 dzień temu");
}

#[test]
fn should_describe_older_runs_in_days() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04+01:00")
        .when_parsed()
        .then_age_after_seconds_is(3 * 86_400, "3 dni temu");
}

#[test]
fn should_reject_a_timestamp_without_an_offset() {
    TimestampScenario::given_timestamp("2026-11-03T09:14:04").when_parsed().then_is_rejected();
}

#[test]
fn should_reject_an_impossible_month() {
    TimestampScenario::given_timestamp("2026-13-03T09:14:04Z").when_parsed().then_is_rejected();
}
