//! UI-independent formatting and parsing of field values.

use crate::field::DateTimeDisplay;
use std::fmt;
use time::PrimitiveDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

/// A duration split into the components edited by duration inputs.
///
/// Components keep the sign of the duration. Hours are not wrapped into days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DurationParts {
    /// Whole hours.
    pub hours: i64,
    /// Minutes within the hour.
    pub minutes: i64,
    /// Seconds within the minute.
    pub seconds: i64,
    /// Milliseconds within the second.
    pub milliseconds: i64,
}

impl DurationParts {
    /// Splits `duration` into its components.
    #[must_use]
    pub fn from_duration(duration: time::Duration) -> Self {
        Self {
            hours: duration.whole_hours(),
            minutes: duration.whole_minutes() % 60,
            seconds: duration.whole_seconds() % 60,
            milliseconds: i64::from(duration.subsec_milliseconds()),
        }
    }

    /// Combines the components into a duration.
    #[must_use]
    pub fn to_duration(self) -> time::Duration {
        time::Duration::hours(self.hours)
            + time::Duration::minutes(self.minutes)
            + time::Duration::seconds(self.seconds)
            + time::Duration::milliseconds(self.milliseconds)
    }

    /// Returns the hundredths of a second within the second.
    #[must_use]
    pub fn centiseconds(self) -> i64 {
        self.milliseconds / 10
    }

    /// Replaces the hundredths of a second, dropping any finer remainder.
    #[must_use]
    pub fn with_centiseconds(self, centiseconds: i64) -> Self {
        Self {
            milliseconds: centiseconds * 10,
            ..self
        }
    }
}

/// Error returned when a duration input cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationParseError {
    input: String,
}

impl fmt::Display for DurationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' is not a duration in the form HH:MM:SS or HH:MM:SS.cc",
            self.input
        )
    }
}

impl std::error::Error for DurationParseError {}

impl std::str::FromStr for DurationParts {
    type Err = DurationParseError;

    /// Parses `HH:MM`, `HH:MM:SS`, or `HH:MM:SS.cc`, optionally preceded by `-` for a negative
    /// duration. Minutes and seconds must be below 60.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let error = || DurationParseError {
            input: input.to_owned(),
        };
        let trimmed = input.trim();
        let (negative, unsigned) = match trimmed.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, trimmed),
        };
        let (clock, centiseconds) = match unsigned.split_once('.') {
            Some((clock, fraction)) if (1..=2).contains(&fraction.len()) => {
                let digits = parse_digits(fraction).ok_or_else(error)?;
                (
                    clock,
                    if fraction.len() == 1 {
                        digits * 10
                    } else {
                        digits
                    },
                )
            }
            Some(_) => return Err(error()),
            None => (unsigned, 0),
        };
        let parts = clock
            .split(':')
            .map(|part| parse_digits(part).ok_or_else(error))
            .collect::<Result<Vec<_>, _>>()?;
        let (hours, minutes, seconds) = match parts.as_slice() {
            [hours, minutes] => (*hours, *minutes, 0),
            [hours, minutes, seconds] => (*hours, *minutes, *seconds),
            _ => return Err(error()),
        };
        if !(0..60).contains(&minutes) || !(0..60).contains(&seconds) {
            return Err(error());
        }
        let parts = Self {
            hours,
            minutes,
            seconds,
            milliseconds: 0,
        }
        .with_centiseconds(centiseconds);
        Ok(if negative { parts.negated() } else { parts })
    }
}

/// Parses a non-empty run of ASCII digits, rejecting signs and other characters.
fn parse_digits(part: &str) -> Option<i64> {
    if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

impl DurationParts {
    /// Returns whether the parts describe a negative duration.
    fn is_negative(self) -> bool {
        self.hours < 0 || self.minutes < 0 || self.seconds < 0 || self.milliseconds < 0
    }

    /// Returns the parts of the duration with the opposite sign.
    fn negated(self) -> Self {
        Self {
            hours: -self.hours,
            minutes: -self.minutes,
            seconds: -self.seconds,
            milliseconds: -self.milliseconds,
        }
    }
}

impl fmt::Display for DurationParts {
    /// Formats as `HH:MM:SS.cc`, preceded by `-` for a negative duration.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{:02}:{:02}:{:02}.{:02}",
            if self.is_negative() { "-" } else { "" },
            self.hours.unsigned_abs(),
            self.minutes.unsigned_abs(),
            self.seconds.unsigned_abs(),
            self.centiseconds().unsigned_abs()
        )
    }
}

/// Formats a date-time for display.
///
/// [`DateTimeDisplay::IsoUtc`] interprets the value as UTC and renders RFC 3339.
/// [`DateTimeDisplay::LocalizedLocal`] currently renders a fixed `DD.MM.YYYY HH:MM` pattern.
#[must_use]
pub fn format_date_time(date_time: PrimitiveDateTime, display: DateTimeDisplay) -> String {
    match display {
        DateTimeDisplay::IsoUtc => date_time
            .assume_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| date_time.to_string()),
        // TODO: Format using the current user's locale.
        DateTimeDisplay::LocalizedLocal => date_time
            .format(format_description!("[day].[month].[year] [hour]:[minute]"))
            .unwrap_or_else(|_| date_time.to_string()),
    }
}

/// Formats a date-time in the `YYYY-MM-DDTHH:MM:SS` form used by text-based date-time inputs.
#[must_use]
pub fn format_date_time_input(date_time: PrimitiveDateTime) -> String {
    date_time
        .format(format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second]"
        ))
        .unwrap_or_else(|_| date_time.to_string())
}

/// Error returned when a date-time input cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeParseError {
    input: String,
}

impl fmt::Display for DateTimeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' is not a date-time in the form YYYY-MM-DDTHH:MM or YYYY-MM-DDTHH:MM:SS",
            self.input
        )
    }
}

impl std::error::Error for DateTimeParseError {}

/// Parses a date-time in the form `YYYY-MM-DDTHH:MM[:SS]`. A space may replace the `T`.
///
/// # Errors
///
/// Returns an error if `input` does not match one of the accepted forms or names an invalid date or
/// time.
pub fn parse_date_time_input(input: &str) -> Result<PrimitiveDateTime, DateTimeParseError> {
    let normalized = input.trim().replacen(' ', "T", 1);
    PrimitiveDateTime::parse(
        &normalized,
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
    )
    .or_else(|_| {
        PrimitiveDateTime::parse(
            &normalized,
            format_description!("[year]-[month]-[day]T[hour]:[minute]"),
        )
    })
    .map_err(|_| DateTimeParseError {
        input: input.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use time::macros::datetime;

    #[test]
    fn duration_parts_round_trip() {
        let duration = time::Duration::milliseconds(3 * 3_600_000 + 25 * 60_000 + 7_000 + 456);
        let parts = DurationParts::from_duration(duration);
        assert_that!(parts).is_equal_to(DurationParts {
            hours: 3,
            minutes: 25,
            seconds: 7,
            milliseconds: 456,
        });
        assert_that!(parts.to_duration()).is_equal_to(duration);
        assert_that!(parts.to_string()).is_equal_to("03:25:07.45".to_owned());
    }

    #[test]
    fn replacing_components_keeps_the_others() {
        let parts = DurationParts::from_duration(time::Duration::milliseconds(3_723_456));
        let changed = DurationParts {
            minutes: 59,
            ..parts
        }
        .with_centiseconds(9);
        assert_that!(changed.to_duration()).is_equal_to(time::Duration::milliseconds(
            3_600_000 + 59 * 60_000 + 3_000 + 90,
        ));
    }

    #[test]
    fn durations_parse_from_their_display_form() {
        let parts = "03:25:07.45"
            .parse::<DurationParts>()
            .expect("valid duration");
        assert_that!(parts.to_string()).is_equal_to("03:25:07.45".to_owned());
        assert_that!(
            "1:30"
                .parse::<DurationParts>()
                .map(DurationParts::to_duration)
        )
        .is_equal_to(Ok(time::Duration::minutes(90)));
        assert_that!(
            "00:00:01.5"
                .parse::<DurationParts>()
                .map(|it| it.milliseconds)
        )
        .is_equal_to(Ok(500));
        assert_that!("00:60:00".parse::<DurationParts>().is_err()).is_true();
        assert_that!("soon".parse::<DurationParts>().is_err()).is_true();
    }

    #[test]
    fn negative_durations_round_trip_through_their_display_form() {
        let duration = -time::Duration::milliseconds(90_500);
        let parts = DurationParts::from_duration(duration);

        assert_that!(parts.to_string()).is_equal_to("-00:01:30.50".to_owned());
        assert_that!(
            parts
                .to_string()
                .parse::<DurationParts>()
                .map(DurationParts::to_duration)
        )
        .is_equal_to(Ok(duration));
        assert_that!("--00:01:30".parse::<DurationParts>().is_err()).is_true();
    }

    #[test]
    fn date_times_format_for_display() {
        let date_time = datetime!(2026-10-05 13:07:09);
        assert_that!(format_date_time(date_time, DateTimeDisplay::IsoUtc))
            .is_equal_to("2026-10-05T13:07:09Z".to_owned());
        assert_that!(format_date_time(date_time, DateTimeDisplay::LocalizedLocal))
            .is_equal_to("05.10.2026 13:07".to_owned());
    }

    #[test]
    fn date_time_input_round_trips_and_accepts_short_forms() {
        let date_time = datetime!(2026-10-05 13:07:09);
        assert_that!(parse_date_time_input(&format_date_time_input(date_time)))
            .is_equal_to(Ok(date_time));
        assert_that!(parse_date_time_input("2026-10-05 13:07"))
            .is_equal_to(Ok(datetime!(2026-10-05 13:07)));
        assert_that!(parse_date_time_input("2026-13-05T13:07").is_err()).is_true();
        assert_that!(parse_date_time_input("yesterday").is_err()).is_true();
    }
}
