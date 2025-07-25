//! GraphQL support for [`time`] crate types.
//!
//! # Supported types
//!
//! | Rust type             | Format                | GraphQL scalar        |
//! |-----------------------|-----------------------|-----------------------|
//! | [`Date`]              | `yyyy-MM-dd`          | [`LocalDate`][s1]     |
//! | [`Time`]              | `HH:mm[:ss[.SSS]]`    | [`LocalTime`][s2]     |
//! | [`PrimitiveDateTime`] | `yyyy-MM-ddTHH:mm:ss` | [`LocalDateTime`][s3] |
//! | [`OffsetDateTime`]    | [RFC 3339] string     | [`DateTime`][s4]      |
//! | [`UtcOffset`]         | `±hh:mm`              | [`UtcOffset`][s5]     |
//!
//! [`Date`]: time::Date
//! [`OffsetDateTime`]: time::OffsetDateTime
//! [`PrimitiveDateTime`]: time::PrimitiveDateTime
//! [`Time`]: time::Time
//! [`UtcOffset`]: time::UtcOffset
//! [RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
//! [s1]: https://graphql-scalars.dev/docs/scalars/local-date
//! [s2]: https://graphql-scalars.dev/docs/scalars/local-time
//! [s3]: https://graphql-scalars.dev/docs/scalars/local-date-time
//! [s4]: https://graphql-scalars.dev/docs/scalars/date-time
//! [s5]: https://graphql-scalars.dev/docs/scalars/utc-offset

use std::{
    fmt::{self, Display},
    io, str,
};
use time::{
    format_description::{BorrowedFormatItem, well_known::Rfc3339},
    macros::format_description,
};

use crate::graphql_scalar;

/// Date in the proleptic Gregorian calendar (without time zone).
///
/// Represents a description of the date (as used for birthdays, for example).
/// It cannot represent an instant on the time-line.
///
/// [`LocalDate` scalar][1] compliant.
///
/// See also [`time::Date`][2] for details.
///
/// [1]: https://graphql-scalars.dev/docs/scalars/local-date
/// [2]: https://docs.rs/time/*/time/struct.Date.html
#[graphql_scalar]
#[graphql(
    with = local_date,
    parse_token(String),
    specified_by_url = "https://graphql-scalars.dev/docs/scalars/local-date",
)]
pub type LocalDate = time::Date;

mod local_date {
    use super::*;

    /// Format of a [`LocalDate` scalar][1].
    ///
    /// [1]: https://graphql-scalars.dev/docs/scalars/local-date
    const FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

    impl Display for LazyFmt<&LocalDate> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0
                .format_into(&mut IoAdapter(f), FORMAT)
                .map_err(|_| fmt::Error)
                .map(drop)
        }
    }

    pub(super) fn to_output(v: &LocalDate) -> impl Display {
        LazyFmt(v)
    }

    pub(super) fn from_input(s: &str) -> Result<LocalDate, Box<str>> {
        LocalDate::parse(s, FORMAT).map_err(|e| format!("Invalid `LocalDate`: {e}").into())
    }
}

/// Clock time within a given date (without time zone) in `HH:mm[:ss[.SSS]]`
/// format.
///
/// All minutes are assumed to have exactly 60 seconds; no attempt is made to
/// handle leap seconds (either positive or negative).
///
/// [`LocalTime` scalar][1] compliant.
///
/// See also [`time::Time`][2] for details.
///
/// [1]: https://graphql-scalars.dev/docs/scalars/local-time
/// [2]: https://docs.rs/time/*/time/struct.Time.html
#[graphql_scalar]
#[graphql(
    with = local_time,
    parse_token(String),
    specified_by_url = "https://graphql-scalars.dev/docs/scalars/local-time",
)]
pub type LocalTime = time::Time;

mod local_time {
    use super::*;

    /// Full format of a [`LocalTime` scalar][1].
    ///
    /// [1]: https://graphql-scalars.dev/docs/scalars/local-time
    const FORMAT: &[BorrowedFormatItem<'_>] =
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]");

    /// Format of a [`LocalTime` scalar][1] without milliseconds.
    ///
    /// [1]: https://graphql-scalars.dev/docs/scalars/local-time
    const FORMAT_NO_MILLIS: &[BorrowedFormatItem<'_>] =
        format_description!("[hour]:[minute]:[second]");

    /// Format of a [`LocalTime` scalar][1] without seconds.
    ///
    /// [1]: https://graphql-scalars.dev/docs/scalars/local-time
    const FORMAT_NO_SECS: &[BorrowedFormatItem<'_>] = format_description!("[hour]:[minute]");

    impl Display for LazyFmt<&LocalTime> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0
                .format_into(
                    &mut IoAdapter(f),
                    if self.0.millisecond() == 0 {
                        FORMAT_NO_MILLIS
                    } else {
                        FORMAT
                    },
                )
                .map_err(|_| fmt::Error)
                .map(drop)
        }
    }

    pub(super) fn to_output(v: &LocalTime) -> impl Display {
        LazyFmt(v)
    }

    pub(super) fn from_input(s: &str) -> Result<LocalTime, Box<str>> {
        // First, try to parse the most used format.
        // At the end, try to parse the full format for the parsing error to be most informative.
        LocalTime::parse(s, FORMAT_NO_MILLIS)
            .or_else(|_| LocalTime::parse(s, FORMAT_NO_SECS))
            .or_else(|_| LocalTime::parse(s, FORMAT))
            .map_err(|e| format!("Invalid `LocalTime`: {e}").into())
    }
}

/// Combined date and time (without time zone) in `yyyy-MM-ddTHH:mm:ss` format.
///
/// [`LocalDateTime` scalar][1] compliant.
///
/// See also [`time::PrimitiveDateTime`][2] for details.
///
/// [1]: https://graphql-scalars.dev/docs/scalars/local-date-time
/// [2]: https://docs.rs/time/*/time/struct.PrimitiveDateTime.html
#[graphql_scalar]
#[graphql(
    with = local_date_time,
    parse_token(String),
    specified_by_url = "https://graphql-scalars.dev/docs/scalars/local-date-time",
)]
pub type LocalDateTime = time::PrimitiveDateTime;

mod local_date_time {
    use super::*;

    /// Format of a [`LocalDateTime` scalar][1].
    ///
    /// [1]: https://graphql-scalars.dev/docs/scalars/local-date-time
    const FORMAT: &[BorrowedFormatItem<'_>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");

    impl Display for LazyFmt<&LocalDateTime> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0
                .format_into(&mut IoAdapter(f), FORMAT)
                .map_err(|_| fmt::Error)
                .map(drop)
        }
    }

    pub(super) fn to_output(v: &LocalDateTime) -> impl Display {
        LazyFmt(v)
    }

    pub(super) fn from_input(s: &str) -> Result<LocalDateTime, Box<str>> {
        LocalDateTime::parse(s, FORMAT).map_err(|e| format!("Invalid `LocalDateTime`: {e}").into())
    }
}

/// Combined date and time (with time zone) in [RFC 3339][0] format.
///
/// Represents a description of an exact instant on the time-line (such as the
/// instant that a user account was created).
///
/// [`DateTime` scalar][1] compliant.
///
/// See also [`time::OffsetDateTime`][2] for details.
///
/// [0]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
/// [1]: https://graphql-scalars.dev/docs/scalars/date-time
/// [2]: https://docs.rs/time/*/time/struct.OffsetDateTime.html
#[graphql_scalar]
#[graphql(
    with = date_time,
    parse_token(String),
    specified_by_url = "https://graphql-scalars.dev/docs/scalars/date-time",
)]
pub type DateTime = time::OffsetDateTime;

mod date_time {
    use super::*;

    impl Display for LazyFmt<&DateTime> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0
                .to_offset(UtcOffset::UTC)
                .format_into(&mut IoAdapter(f), &Rfc3339)
                .map_err(|_| fmt::Error)
                .map(drop)
        }
    }

    pub(super) fn to_output(v: &DateTime) -> impl Display {
        LazyFmt(v)
    }

    pub(super) fn from_input(s: &str) -> Result<DateTime, Box<str>> {
        DateTime::parse(s, &Rfc3339)
            .map(|dt| dt.to_offset(UtcOffset::UTC))
            .map_err(|e| format!("Invalid `DateTime`: {e}").into())
    }
}

/// Format of a [`UtcOffset` scalar][1].
///
/// [1]: https://graphql-scalars.dev/docs/scalars/utc-offset
const UTC_OFFSET_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[offset_hour sign:mandatory]:[offset_minute]");

/// Offset from UTC in `±hh:mm` format. See [list of database time zones][0].
///
/// [`UtcOffset` scalar][1] compliant.
///
/// See also [`time::UtcOffset`][2] for details.
///
/// [0]: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
/// [1]: https://graphql-scalars.dev/docs/scalars/utc-offset
/// [2]: https://docs.rs/time/*/time/struct.UtcOffset.html
#[graphql_scalar]
#[graphql(
    with = utc_offset,
    parse_token(String),
    specified_by_url = "https://graphql-scalars.dev/docs/scalars/utc-offset",
)]
pub type UtcOffset = time::UtcOffset;

mod utc_offset {
    use super::*;

    impl Display for LazyFmt<&UtcOffset> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0
                .format_into(&mut IoAdapter(f), UTC_OFFSET_FORMAT)
                .map_err(|_| fmt::Error)
                .map(drop)
        }
    }

    pub(super) fn to_output(v: &UtcOffset) -> impl Display {
        LazyFmt(v)
    }

    pub(super) fn from_input(s: &str) -> Result<UtcOffset, Box<str>> {
        UtcOffset::parse(s, UTC_OFFSET_FORMAT)
            .map_err(|e| format!("Invalid `UtcOffset`: {e}").into())
    }
}

// TODO: Remove once time-rs/time#375 is resolved:
//       https://github.com/time-rs/time/issues/375
/// [`io::Write`] adapter for [`fmt::Formatter`].
///
/// Required because [`time`] crate cannot write to [`fmt::Write`], only to [`io::Write`].
struct IoAdapter<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl io::Write for IoAdapter<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = str::from_utf8(buf).map_err(io::Error::other)?;
        match self.0.write_str(s) {
            Ok(_) => Ok(s.len()),
            Err(e) => Err(io::Error::other(e)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Wrapper over [`time`] crate types to [`Display`] their format lazily.
struct LazyFmt<T>(T);

#[cfg(test)]
mod local_date_test {
    use time::macros::date;

    use crate::{FromInputValue as _, InputValue, ToInputValue as _, graphql_input_value};

    use super::LocalDate;

    #[test]
    fn parses_correct_input() {
        for (raw, expected) in [
            ("1996-12-19", date!(1996 - 12 - 19)),
            ("1564-01-30", date!(1564 - 01 - 30)),
        ] {
            let input: InputValue = graphql_input_value!((raw));
            let parsed = LocalDate::from_input_value(&input);

            assert!(
                parsed.is_ok(),
                "failed to parse `{raw}`: {:?}",
                parsed.unwrap_err(),
            );
            assert_eq!(parsed.unwrap(), expected, "input: {raw}");
        }
    }

    #[test]
    fn fails_on_invalid_input() {
        for input in [
            graphql_input_value!("1996-13-19"),
            graphql_input_value!("1564-01-61"),
            graphql_input_value!("2021-11-31"),
            graphql_input_value!("11-31"),
            graphql_input_value!("2021-11"),
            graphql_input_value!("2021"),
            graphql_input_value!("31"),
            graphql_input_value!("i'm not even a date"),
            graphql_input_value!(2.32),
            graphql_input_value!(1),
            graphql_input_value!(null),
            graphql_input_value!(false),
        ] {
            let input: InputValue = input;
            let parsed = LocalDate::from_input_value(&input);

            assert!(parsed.is_err(), "allows input: {input:?}");
        }
    }

    #[test]
    fn formats_correctly() {
        for (val, expected) in [
            (date!(1996 - 12 - 19), graphql_input_value!("1996-12-19")),
            (date!(1564 - 01 - 30), graphql_input_value!("1564-01-30")),
            (date!(2020 - W 01 - 3), graphql_input_value!("2020-01-01")),
            (date!(2020 - 001), graphql_input_value!("2020-01-01")),
        ] {
            let actual: InputValue = val.to_input_value();

            assert_eq!(actual, expected, "on value: {val}");
        }
    }
}

#[cfg(test)]
mod local_time_test {
    use time::macros::time;

    use crate::{FromInputValue as _, InputValue, ToInputValue as _, graphql_input_value};

    use super::LocalTime;

    #[test]
    fn parses_correct_input() {
        for (raw, expected) in [
            ("14:23:43", time!(14:23:43)),
            ("14:00:00", time!(14:00)),
            ("14:00", time!(14:00)),
            ("14:32", time!(14:32:00)),
            ("14:00:00.000", time!(14:00)),
            ("14:23:43.345", time!(14:23:43.345)),
        ] {
            let input: InputValue = graphql_input_value!((raw));
            let parsed = LocalTime::from_input_value(&input);

            assert!(
                parsed.is_ok(),
                "failed to parse `{raw}`: {:?}",
                parsed.unwrap_err(),
            );
            assert_eq!(parsed.unwrap(), expected, "input: {raw}");
        }
    }

    #[test]
    fn fails_on_invalid_input() {
        for input in [
            graphql_input_value!("12"),
            graphql_input_value!("12:"),
            graphql_input_value!("56:34:22"),
            graphql_input_value!("23:78:43"),
            graphql_input_value!("23:78:"),
            graphql_input_value!("23:18:99"),
            graphql_input_value!("23:18:22.4351"),
            graphql_input_value!("23:18:22."),
            graphql_input_value!("23:18:22.3"),
            graphql_input_value!("23:18:22.03"),
            graphql_input_value!("22.03"),
            graphql_input_value!("24:00"),
            graphql_input_value!("24:00:00"),
            graphql_input_value!("24:00:00.000"),
            graphql_input_value!("i'm not even a time"),
            graphql_input_value!(2.32),
            graphql_input_value!(1),
            graphql_input_value!(null),
            graphql_input_value!(false),
        ] {
            let input: InputValue = input;
            let parsed = LocalTime::from_input_value(&input);

            assert!(parsed.is_err(), "allows input: {input:?}");
        }
    }

    #[test]
    fn formats_correctly() {
        for (val, expected) in [
            (time!(1:02:03.004_005), graphql_input_value!("01:02:03.004")),
            (time!(0:00), graphql_input_value!("00:00:00")),
            (time!(12:00 pm), graphql_input_value!("12:00:00")),
            (time!(1:02:03), graphql_input_value!("01:02:03")),
        ] {
            let actual: InputValue = val.to_input_value();

            assert_eq!(actual, expected, "on value: {val}");
        }
    }
}

#[cfg(test)]
mod local_date_time_test {
    use time::macros::datetime;

    use crate::{FromInputValue as _, InputValue, ToInputValue as _, graphql_input_value};

    use super::LocalDateTime;

    #[test]
    fn parses_correct_input() {
        for (raw, expected) in [
            ("1996-12-19T14:23:43", datetime!(1996-12-19 14:23:43)),
            ("1564-01-30T14:00:00", datetime!(1564-01-30 14:00)),
        ] {
            let input: InputValue = graphql_input_value!((raw));
            let parsed = LocalDateTime::from_input_value(&input);

            assert!(
                parsed.is_ok(),
                "failed to parse `{raw}`: {:?}",
                parsed.unwrap_err(),
            );
            assert_eq!(parsed.unwrap(), expected, "input: {raw}");
        }
    }

    #[test]
    fn fails_on_invalid_input() {
        for input in [
            graphql_input_value!("12"),
            graphql_input_value!("12:"),
            graphql_input_value!("56:34:22"),
            graphql_input_value!("56:34:22.000"),
            graphql_input_value!("1996-12-1914:23:43"),
            graphql_input_value!("1996-12-19 14:23:43"),
            graphql_input_value!("1996-12-19Q14:23:43"),
            graphql_input_value!("1996-12-19T14:23:43Z"),
            graphql_input_value!("1996-12-19T14:23:43.543"),
            graphql_input_value!("1996-12-19T14:23"),
            graphql_input_value!("1996-12-19T14:23:1"),
            graphql_input_value!("1996-12-19T14:23:"),
            graphql_input_value!("1996-12-19T23:78:43"),
            graphql_input_value!("1996-12-19T23:18:99"),
            graphql_input_value!("1996-12-19T24:00:00"),
            graphql_input_value!("1996-12-19T99:02:13"),
            graphql_input_value!("i'm not even a datetime"),
            graphql_input_value!(2.32),
            graphql_input_value!(1),
            graphql_input_value!(null),
            graphql_input_value!(false),
        ] {
            let input: InputValue = input;
            let parsed = LocalDateTime::from_input_value(&input);

            assert!(parsed.is_err(), "allows input: {input:?}");
        }
    }

    #[test]
    fn formats_correctly() {
        for (val, expected) in [
            (
                datetime!(1996-12-19 12:00 am),
                graphql_input_value!("1996-12-19T00:00:00"),
            ),
            (
                datetime!(1564-01-30 14:00),
                graphql_input_value!("1564-01-30T14:00:00"),
            ),
        ] {
            let actual: InputValue = val.to_input_value();

            assert_eq!(actual, expected, "on value: {val}");
        }
    }
}

#[cfg(test)]
mod date_time_test {
    use time::macros::datetime;

    use crate::{FromInputValue as _, InputValue, ToInputValue as _, graphql_input_value};

    use super::DateTime;

    #[test]
    fn parses_correct_input() {
        for (raw, expected) in [
            (
                "2014-11-28T21:00:09+09:00",
                datetime!(2014-11-28 21:00:09 +9),
            ),
            ("2014-11-28T21:00:09Z", datetime!(2014-11-28 21:00:09 +0)),
            ("2014-11-28 21:00:09z", datetime!(2014-11-28 21:00:09 +0)),
            (
                "2014-11-28T21:00:09+00:00",
                datetime!(2014-11-28 21:00:09 +0),
            ),
            (
                "2014-11-28T21:00:09.05+09:00",
                datetime!(2014-11-28 12:00:09.05 +0),
            ),
            (
                "2014-11-28 21:00:09.05+09:00",
                datetime!(2014-11-28 12:00:09.05 +0),
            ),
        ] {
            let input: InputValue = graphql_input_value!((raw));
            let parsed = DateTime::from_input_value(&input);

            assert!(
                parsed.is_ok(),
                "failed to parse `{raw}`: {:?}",
                parsed.unwrap_err(),
            );
            assert_eq!(parsed.unwrap(), expected, "input: {raw}");
        }
    }

    #[test]
    fn fails_on_invalid_input() {
        for input in [
            graphql_input_value!("12"),
            graphql_input_value!("12:"),
            graphql_input_value!("56:34:22"),
            graphql_input_value!("56:34:22.000"),
            graphql_input_value!("1996-12-1914:23:43"),
            graphql_input_value!("1996-12-19T14:23:43"),
            graphql_input_value!("1996-12-19T14:23:43ZZ"),
            graphql_input_value!("1996-12-19T14:23:43.543"),
            graphql_input_value!("1996-12-19T14:23"),
            graphql_input_value!("1996-12-19T14:23:1"),
            graphql_input_value!("1996-12-19T14:23:"),
            graphql_input_value!("1996-12-19T23:78:43Z"),
            graphql_input_value!("1996-12-19T23:18:99Z"),
            graphql_input_value!("1996-12-19T24:00:00Z"),
            graphql_input_value!("1996-12-19T99:02:13Z"),
            graphql_input_value!("1996-12-19T99:02:13Z"),
            graphql_input_value!("1996-12-19T12:02:13+4444444"),
            graphql_input_value!("i'm not even a datetime"),
            graphql_input_value!(2.32),
            graphql_input_value!(1),
            graphql_input_value!(null),
            graphql_input_value!(false),
        ] {
            let input: InputValue = input;
            let parsed = DateTime::from_input_value(&input);

            assert!(parsed.is_err(), "allows input: {input:?}");
        }
    }

    #[test]
    fn formats_correctly() {
        for (val, expected) in [
            (
                datetime!(1996-12-19 12:00 am UTC),
                graphql_input_value!("1996-12-19T00:00:00Z"),
            ),
            (
                datetime!(1564-01-30 14:00 +9),
                graphql_input_value!("1564-01-30T05:00:00Z"),
            ),
        ] {
            let actual: InputValue = val.to_input_value();

            assert_eq!(actual, expected, "on value: {val}");
        }
    }
}

#[cfg(test)]
mod utc_offset_test {
    use time::macros::offset;

    use crate::{FromInputValue as _, InputValue, ToInputValue as _, graphql_input_value};

    use super::UtcOffset;

    #[test]
    fn parses_correct_input() {
        for (raw, expected) in [
            ("+00:00", offset!(+0)),
            ("-00:00", offset!(-0)),
            ("+10:00", offset!(+10)),
            ("-07:30", offset!(-7:30)),
            ("+14:00", offset!(+14)),
            ("-12:00", offset!(-12)),
        ] {
            let input: InputValue = graphql_input_value!((raw));
            let parsed = UtcOffset::from_input_value(&input);

            assert!(
                parsed.is_ok(),
                "failed to parse `{raw}`: {:?}",
                parsed.unwrap_err(),
            );
            assert_eq!(parsed.unwrap(), expected, "input: {raw}");
        }
    }

    #[test]
    fn fails_on_invalid_input() {
        for input in [
            graphql_input_value!("12"),
            graphql_input_value!("12:"),
            graphql_input_value!("12:00"),
            graphql_input_value!("+12:"),
            graphql_input_value!("+12:0"),
            graphql_input_value!("+12:00:34"),
            graphql_input_value!("+12"),
            graphql_input_value!("-12"),
            graphql_input_value!("-12:"),
            graphql_input_value!("-12:0"),
            graphql_input_value!("-12:00:32"),
            graphql_input_value!("-999:00"),
            graphql_input_value!("+999:00"),
            graphql_input_value!("i'm not even an offset"),
            graphql_input_value!(2.32),
            graphql_input_value!(1),
            graphql_input_value!(null),
            graphql_input_value!(false),
        ] {
            let input: InputValue = input;
            let parsed = UtcOffset::from_input_value(&input);

            assert!(parsed.is_err(), "allows input: {input:?}");
        }
    }

    #[test]
    fn formats_correctly() {
        for (val, expected) in [
            (offset!(+1), graphql_input_value!("+01:00")),
            (offset!(+0), graphql_input_value!("+00:00")),
            (offset!(-2:30), graphql_input_value!("-02:30")),
        ] {
            let actual: InputValue = val.to_input_value();

            assert_eq!(actual, expected, "on value: {val}");
        }
    }
}

#[cfg(test)]
mod integration_test {
    use time::macros::{date, datetime, offset, time};

    use crate::{
        execute, graphql_object, graphql_value, graphql_vars,
        schema::model::RootNode,
        types::scalars::{EmptyMutation, EmptySubscription},
    };

    use super::{DateTime, LocalDate, LocalDateTime, LocalTime, UtcOffset};

    #[tokio::test]
    async fn serializes() {
        struct Root;

        #[graphql_object]
        impl Root {
            fn local_date() -> LocalDate {
                date!(2015 - 03 - 14)
            }

            fn local_time() -> LocalTime {
                time!(16:07:08)
            }

            fn local_date_time() -> LocalDateTime {
                datetime!(2016-07-08 09:10:11)
            }

            fn date_time() -> DateTime {
                datetime!(1996-12-19 16:39:57 -8)
            }

            fn utc_offset() -> UtcOffset {
                offset!(+11:30)
            }
        }

        const DOC: &str = r#"{
            localDate
            localTime
            localDateTime
            dateTime,
            utcOffset,
        }"#;

        let schema = RootNode::new(
            Root,
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        );

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({
                    "localDate": "2015-03-14",
                    "localTime": "16:07:08",
                    "localDateTime": "2016-07-08T09:10:11",
                    "dateTime": "1996-12-20T00:39:57Z",
                    "utcOffset": "+11:30",
                }),
                vec![],
            )),
        );
    }
}
