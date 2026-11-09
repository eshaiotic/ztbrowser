//! When a test run started, in the form cargo-nextest writes into its JUnit file: local time with its UTC
//! offset, for example `2026-11-03T09:14:04.102+01:00`. Keeping the offset from the file means the report
//! needs no time zone database: the local date and time are read straight from the text, and the offset
//! gives the absolute instant used to sort runs and to say how long ago one ran.

const SECONDS_PER_DAY: i64 = 86_400;
const POLISH_MONTHS: [&str; 12] = ["sty", "lut", "mar", "kwi", "maj", "cze", "lip", "sie", "wrz", "paź", "lis", "gru"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunTimestamp {
    text: String,
    unix_seconds: i64,
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
}

impl RunTimestamp {
    /// Reads `YYYY-MM-DDTHH:MM:SS[.fraction](Z|±HH:MM)`; anything else is `None`.
    pub fn parse(text: &str) -> Option<Self> {
        let field = |start: usize, end: usize| text.get(start..end)?.parse::<i64>().ok();
        let separators_match = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')]
            .iter()
            .all(|&(index, separator)| text.as_bytes().get(index) == Some(&separator));
        if !separators_match {
            return None;
        }
        let (year, month, day) = (field(0, 4)?, field(5, 7)?, field(8, 10)?);
        let (hour, minute, second) = (field(11, 13)?, field(14, 16)?, field(17, 19)?);
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
            return None;
        }
        let offset_seconds = Self::offset_seconds(Self::after_fraction(text.get(19..)?))?;
        let local_seconds =
            Self::days_from_civil(year, month, day) * SECONDS_PER_DAY + hour * 3600 + minute * 60 + second;
        Some(Self {
            text: text.to_owned(),
            unix_seconds: local_seconds - offset_seconds,
            year,
            month,
            day,
            hour,
            minute,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn unix_seconds(&self) -> i64 {
        self.unix_seconds
    }

    /// `YYYYMMDD-HHmm` in the run's local time, for the report's file name.
    pub fn file_stamp(&self) -> String {
        format!("{:04}{:02}{:02}-{:02}{:02}", self.year, self.month, self.day, self.hour, self.minute)
    }

    /// `3 lis 2026, 09:14`.
    pub fn polish_label(&self) -> String {
        let month = usize::try_from(self.month - 1).ok().and_then(|index| POLISH_MONTHS.get(index)).unwrap_or(&"?");
        format!("{} {month} {}, {:02}:{:02}", self.day, self.year, self.hour, self.minute)
    }

    /// How long before `now_unix_seconds` the run started, in Polish: `przed chwilą`, `5 min temu`, `3 dni temu`.
    pub fn describe_age(&self, now_unix_seconds: i64) -> String {
        let minutes = (now_unix_seconds - self.unix_seconds).div_euclid(60);
        if minutes < 1 {
            return "przed chwilą".to_owned();
        }
        if minutes < 60 {
            return format!("{minutes} min temu");
        }
        let hours = minutes / 60;
        if hours < 24 {
            return format!("{hours} h temu");
        }
        match hours / 24 {
            1 => "1 dzień temu".to_owned(),
            days => format!("{days} dni temu"),
        }
    }

    fn after_fraction(rest: &str) -> &str {
        match rest.strip_prefix('.') {
            Some(fraction) => fraction.trim_start_matches(|character: char| character.is_ascii_digit()),
            None => rest,
        }
    }

    fn offset_seconds(offset: &str) -> Option<i64> {
        if offset == "Z" {
            return Some(0);
        }
        let sign = match offset.as_bytes().first()? {
            b'+' => 1,
            b'-' => -1,
            _ => return None,
        };
        if offset.len() != 6 || offset.as_bytes().get(3) != Some(&b':') {
            return None;
        }
        let hours = offset.get(1..3)?.parse::<i64>().ok()?;
        let minutes = offset.get(4..6)?.parse::<i64>().ok()?;
        Some(sign * (hours * 3600 + minutes * 60))
    }

    /// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's `days_from_civil`).
    fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
        let year = if month <= 2 { year - 1 } else { year };
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let month_from_march = (month + 9) % 12;
        let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }
}
