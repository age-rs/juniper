//! Tests for `#[graphql_scalar]` macro placed on [`DeriveInput`].
//!
//! [`DeriveInput`]: syn::DeriveInput

pub mod common;

use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use juniper::{
    ParseScalarResult, ParseScalarValue, Scalar, ScalarToken, ScalarValue, execute, graphql_object,
    graphql_scalar, graphql_value, graphql_vars,
};

use self::common::{
    MyScalarValue,
    util::{schema, schema_with_scalar},
};

// Override `std::prelude` items to check whether macros expand hygienically.
use self::common::hygiene::*;

mod trivial {
    use super::*;

    #[graphql_scalar]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }

        fn parse_token<S: ScalarValue>(t: ScalarToken<'_>) -> ParseScalarResult<S> {
            <i32 as ParseScalarValue<S>>::from_str(t)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod transparent {
    use super::*;

    #[graphql_scalar]
    #[graphql(transparent)]
    struct Counter(i32);

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod transparent_with_resolver {
    use super::*;

    #[graphql_scalar]
    #[graphql_scalar(
        transparent,
        to_output_with = Self::to_output,
    )]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0 + 1
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 1}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod all_custom_resolvers {
    use super::*;

    #[graphql_scalar]
    #[graphql(
        to_output_with = to_output,
        from_input_with = Counter,
    )]
    #[graphql(parse_token_with = parse_token)]
    struct Counter(i32);

    fn to_output(v: &Counter) -> i32 {
        v.0
    }

    fn parse_token<S: ScalarValue>(value: ScalarToken<'_>) -> ParseScalarResult<S> {
        <i32 as ParseScalarValue<S>>::from_str(value)
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod explicit_name {
    use super::*;

    #[graphql_scalar]
    #[graphql(name = "Counter")]
    struct CustomCounter(i32);

    impl CustomCounter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }

        fn parse_token<S: ScalarValue>(value: ScalarToken<'_>) -> ParseScalarResult<S> {
            <i32 as ParseScalarValue<S>>::from_str(value)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: CustomCounter) -> CustomCounter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod delegated_parse_token {
    use super::*;

    #[graphql_scalar]
    #[graphql(parse_token(i32))]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod multiple_delegated_parse_token {
    use super::*;

    #[graphql_scalar]
    #[graphql(parse_token(prelude::String, i32))]
    enum StringOrInt {
        String(prelude::String),
        Int(i32),
    }

    impl StringOrInt {
        fn to_output<S: ScalarValue>(&self) -> S {
            match self {
                Self::String(s) => S::from_displayable(s),
                Self::Int(i) => (*i).into(),
            }
        }

        fn from_input(v: &Scalar<impl ScalarValue>) -> prelude::Result<Self, prelude::Box<str>> {
            v.try_to_string()
                .map(Self::String)
                .or_else(|| v.try_to_int().map(Self::Int))
                .ok_or_else(|| format!("Expected `String` or `Int`, found: {v}").into())
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn string_or_int(value: StringOrInt) -> StringOrInt {
            value
        }
    }

    #[tokio::test]
    async fn resolves_string() {
        const DOC: &str = r#"{ stringOrInt(value: "test") }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"stringOrInt": "test"}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_int() {
        const DOC: &str = r#"{ stringOrInt(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"stringOrInt": 0}), vec![])),
        );
    }
}

mod where_attribute {
    use super::*;

    #[graphql_scalar]
    #[graphql(
        to_output_with = to_output,
        from_input_with = from_input,
        parse_token(prelude::String),
        where(Tz: From<Utc>, Tz::Offset: fmt::Display),
        specified_by_url = "https://tools.ietf.org/html/rfc3339",
    )]
    struct CustomDateTime<Tz: TimeZone>(DateTime<Tz>);

    fn to_output<Tz>(v: &CustomDateTime<Tz>) -> prelude::String
    where
        Tz: From<Utc> + TimeZone,
        Tz::Offset: fmt::Display,
    {
        v.0.to_rfc3339()
    }

    fn from_input<Tz>(s: &str) -> prelude::Result<CustomDateTime<Tz>, prelude::Box<str>>
    where
        Tz: From<Utc> + TimeZone,
        Tz::Offset: fmt::Display,
    {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| CustomDateTime(dt.with_timezone(&Tz::from(Utc))))
            .map_err(|e| format!("Failed to parse `CustomDateTime`: {e}").into())
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn date_time(value: CustomDateTime<Utc>) -> CustomDateTime<Utc> {
            value
        }
    }

    #[tokio::test]
    async fn resolves_custom_date_time() {
        const DOC: &str = r#"{ dateTime(value: "1996-12-19T16:39:57-08:00") }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"dateTime": "1996-12-20T00:39:57+00:00"}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn has_specified_by_url() {
        const DOC: &str = r#"{
            __type(name: "CustomDateTime") {
                specifiedByUrl
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"specifiedByUrl": "https://tools.ietf.org/html/rfc3339"}}),
                vec![],
            )),
        );
    }
}

mod with_self {
    use super::*;

    #[graphql_scalar]
    #[graphql(with = Self)]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }

        fn parse_token<S: ScalarValue>(value: ScalarToken<'_>) -> ParseScalarResult<S> {
            <i32 as ParseScalarValue<S>>::from_str(value)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod with_module {
    use super::*;

    #[graphql_scalar]
    #[graphql(
        with = custom_date_time,
        parse_token(prelude::String),
        where(Tz: From<Utc>, Tz::Offset: fmt::Display),
        specified_by_url = "https://tools.ietf.org/html/rfc3339",
    )]
    struct CustomDateTime<Tz: TimeZone>(DateTime<Tz>);

    mod custom_date_time {
        use super::*;

        pub(super) fn to_output<Tz>(v: &CustomDateTime<Tz>) -> prelude::String
        where
            Tz: From<Utc> + TimeZone,
            Tz::Offset: fmt::Display,
        {
            v.0.to_rfc3339()
        }

        pub(super) fn from_input<Tz>(
            s: &str,
        ) -> prelude::Result<CustomDateTime<Tz>, prelude::Box<str>>
        where
            Tz: From<Utc> + TimeZone,
            Tz::Offset: fmt::Display,
        {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| CustomDateTime(dt.with_timezone(&Tz::from(Utc))))
                .map_err(|e| format!("Failed to parse `CustomDateTime`: {e}").into())
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn date_time(value: CustomDateTime<Utc>) -> CustomDateTime<Utc> {
            value
        }
    }

    #[tokio::test]
    async fn resolves_custom_date_time() {
        const DOC: &str = r#"{ dateTime(value: "1996-12-19T16:39:57-08:00") }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"dateTime": "1996-12-20T00:39:57+00:00"}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn has_specified_by_url() {
        const DOC: &str = r#"{
            __type(name: "CustomDateTime") {
                specifiedByUrl
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"specifiedByUrl": "https://tools.ietf.org/html/rfc3339"}}),
                vec![],
            )),
        );
    }
}

mod description_from_doc_comment {
    use super::*;

    /// Description
    #[graphql_scalar]
    #[graphql(parse_token(i32))]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"description": "Description"}}),
                vec![],
            )),
        );
    }
}

mod description_from_attribute {
    use super::*;

    /// Doc comment
    #[graphql_scalar]
    #[graphql(description = "Description from attribute", parse_token(i32))]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"description": "Description from attribute"}}),
                vec![],
            )),
        );
    }
}

mod custom_scalar {
    use super::*;

    /// Description
    #[graphql_scalar]
    #[graphql(scalar = MyScalarValue, parse_token(i32))]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }
    }

    struct QueryRoot;

    #[graphql_object(scalar = MyScalarValue)]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"description": "Description"}}),
                vec![],
            )),
        );
    }
}

mod generic_scalar {
    use super::*;

    /// Description
    #[graphql_scalar]
    #[graphql(scalar = S: ScalarValue, parse_token(i32))]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"description": "Description"}}),
                vec![]
            )),
        );
    }
}

mod bounded_generic_scalar {
    use super::*;

    #[graphql_scalar]
    #[graphql(scalar = S: ScalarValue + prelude::Clone, parse_token(i32))]
    struct Counter(i32);

    impl Counter {
        fn to_output(&self) -> i32 {
            self.0
        }

        fn from_input(i: i32) -> Self {
            Self(i)
        }
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }
}
