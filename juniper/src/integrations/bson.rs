//! GraphQL support for [`bson`] crate types.
//!
//! # Supported types
//!
//! | Rust type         | Format            | GraphQL scalar   |
//! |-------------------|-------------------|------------------|
//! | [`oid::ObjectId`] | HEX string        | [`ObjectID`][s1] |
//! | [`DateTime`]      | [RFC 3339] string | [`DateTime`][s4] |
//!
//! [`DateTime`]: bson::DateTime
//! [`oid::ObjectId`]: bson::oid::ObjectId
//! [RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
//! [s1]: https://graphql-scalars.dev/docs/scalars/object-id
//! [s4]: https://graphql-scalars.dev/docs/scalars/date-time

use crate::graphql_scalar;

// TODO: Try remove on upgrade of `bson` crate.
mod for_minimal_versions_check_only {
    use tap as _;
}

/// [BSON ObjectId][0] represented as a HEX string.
///
/// [`ObjectID` scalar][1] compliant.
///
/// See also [`bson::oid::ObjectId`][2] for details.
///
/// [0]: https://www.mongodb.com/docs/manual/reference/bson-types#objectid
/// [1]: https://graphql-scalars.dev/docs/scalars/object-id
/// [2]: https://docs.rs/bson/*/bson/oid/struct.ObjectId.html
#[graphql_scalar]
#[graphql(
    name = "ObjectID",
    with = object_id,
    parse_token(String),
    specified_by_url = "https://graphql-scalars.dev/docs/scalars/object-id",
)]
type ObjectId = bson::oid::ObjectId;

mod object_id {
    use super::ObjectId;

    pub(super) fn to_output(v: &ObjectId) -> String {
        v.to_hex()
    }

    pub(super) fn from_input(s: &str) -> Result<ObjectId, Box<str>> {
        ObjectId::parse_str(s).map_err(|e| format!("Failed to parse `ObjectID`: {e}").into())
    }
}

/// [BSON date][3] in [RFC 3339][0] format.
///
/// [BSON datetimes][3] have millisecond precision and are always in UTC (inputs with other
/// timezones are coerced).
///
/// [`DateTime` scalar][1] compliant.
///
/// See also [`bson::DateTime`][2] for details.
///
/// [0]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
/// [1]: https://graphql-scalars.dev/docs/scalars/date-time
/// [2]: https://docs.rs/bson/*/bson/struct.DateTime.html
/// [3]: https://www.mongodb.com/docs/manual/reference/bson-types#date
#[graphql_scalar]
#[graphql(
    with = date_time,
    parse_token(String),
    specified_by_url = "https://graphql-scalars.dev/docs/scalars/date-time",
)]
type DateTime = bson::DateTime;

mod date_time {
    use super::DateTime;

    pub(super) fn to_output(v: &DateTime) -> String {
        (*v).try_to_rfc3339_string()
            .unwrap_or_else(|e| panic!("failed to format `DateTime` as RFC 3339: {e}"))
    }

    pub(super) fn from_input(s: &str) -> Result<DateTime, Box<str>> {
        DateTime::parse_rfc3339_str(s)
            .map_err(|e| format!("Failed to parse `DateTime`: {e}").into())
    }
}

#[cfg(test)]
mod test {
    use bson::oid::ObjectId;

    use crate::{FromInputValue, InputValue, graphql_input_value};

    #[test]
    fn objectid_from_input() {
        let raw = "53e37d08776f724e42000000";
        let input: InputValue = graphql_input_value!((raw));

        let parsed: ObjectId = FromInputValue::from_input_value(&input).unwrap();
        let id = ObjectId::parse_str(raw).unwrap();

        assert_eq!(parsed, id);
    }
}

#[cfg(test)]
mod date_time_test {
    use crate::{FromInputValue as _, InputValue, ToInputValue as _, graphql_input_value};

    use super::DateTime;

    #[test]
    fn parses_correct_input() {
        for (raw, expected) in [
            (
                "2014-11-28T21:00:09+09:00",
                DateTime::builder()
                    .year(2014)
                    .month(11)
                    .day(28)
                    .hour(12)
                    .second(9)
                    .build()
                    .unwrap(),
            ),
            (
                "2014-11-28T21:00:09Z",
                DateTime::builder()
                    .year(2014)
                    .month(11)
                    .day(28)
                    .hour(21)
                    .second(9)
                    .build()
                    .unwrap(),
            ),
            (
                "2014-11-28 21:00:09z",
                DateTime::builder()
                    .year(2014)
                    .month(11)
                    .day(28)
                    .hour(21)
                    .second(9)
                    .build()
                    .unwrap(),
            ),
            (
                "2014-11-28T21:00:09+00:00",
                DateTime::builder()
                    .year(2014)
                    .month(11)
                    .day(28)
                    .hour(21)
                    .second(9)
                    .build()
                    .unwrap(),
            ),
            (
                "2014-11-28T21:00:09.05+09:00",
                DateTime::builder()
                    .year(2014)
                    .month(11)
                    .day(28)
                    .hour(12)
                    .second(9)
                    .millisecond(50)
                    .build()
                    .unwrap(),
            ),
            (
                "2014-11-28 21:00:09.05+09:00",
                DateTime::builder()
                    .year(2014)
                    .month(11)
                    .day(28)
                    .hour(12)
                    .second(9)
                    .millisecond(50)
                    .build()
                    .unwrap(),
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
                DateTime::builder()
                    .year(1996)
                    .month(12)
                    .day(19)
                    .hour(12)
                    .build()
                    .unwrap(),
                graphql_input_value!("1996-12-19T12:00:00Z"),
            ),
            (
                DateTime::builder()
                    .year(1564)
                    .month(1)
                    .day(30)
                    .hour(5)
                    .minute(3)
                    .second(3)
                    .millisecond(1)
                    .build()
                    .unwrap(),
                graphql_input_value!("1564-01-30T05:03:03.001Z"),
            ),
        ] {
            let actual: InputValue = val.to_input_value();

            assert_eq!(actual, expected, "on value: {val}");
        }
    }
}
