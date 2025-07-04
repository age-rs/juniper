use std::{borrow::Cow, fmt, hash::Hash, slice, vec};

use arcstr::ArcStr;
use compact_str::CompactString;

use indexmap::IndexMap;

use crate::{
    executor::Variables,
    parser::Spanning,
    value::{DefaultScalarValue, Scalar, ScalarValue, ToScalarValue},
};

/// Type literal in a syntax tree.
///
/// This enum carries no semantic information and might refer to types that do not exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type<N = ArcStr> {
    /// `null`able named type, e.g. `String`.
    Named(N),

    /// `null`able list type, e.g. `[String]`.
    ///
    /// The list itself is `null`able, the containing [`Type`] might be non-`null`.
    List(Box<Type<N>>, Option<usize>),

    /// Non-`null` named type, e.g. `String!`.
    NonNullNamed(N),

    /// Non-`null` list type, e.g. `[String]!`.
    ///
    /// The list itself is non-`null`, the containing [`Type`] might be `null`able.
    NonNullList(Box<Type<N>>, Option<usize>),
}

impl<N: fmt::Display> fmt::Display for Type<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(n) => write!(f, "{n}"),
            Self::NonNullNamed(n) => write!(f, "{n}!"),
            Self::List(t, _) => write!(f, "[{t}]"),
            Self::NonNullList(t, _) => write!(f, "[{t}]!"),
        }
    }
}

impl<N: AsRef<str>> Type<N> {
    /// Returns the name of this named [`Type`].
    ///
    /// Only applies to named [`Type`]s. Lists will return [`None`].
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Named(n) | Self::NonNullNamed(n) => Some(n.as_ref()),
            Self::List(..) | Self::NonNullList(..) => None,
        }
    }

    /// Returns the innermost name of this [`Type`] by unpacking lists.
    ///
    /// All [`Type`] literals contain exactly one named type.
    #[must_use]
    pub fn innermost_name(&self) -> &str {
        match self {
            Self::Named(n) | Self::NonNullNamed(n) => n.as_ref(),
            Self::List(l, ..) | Self::NonNullList(l, ..) => l.innermost_name(),
        }
    }

    /// Indicates whether this [`Type`] can only represent non-`null` values.
    #[must_use]
    pub fn is_non_null(&self) -> bool {
        match self {
            Self::NonNullList(..) | Self::NonNullNamed(..) => true,
            Self::List(..) | Self::Named(..) => false,
        }
    }
}

/// A JSON-like value that can be passed into the query execution, either
/// out-of-band, or in-band as default variable values. These are _not_ constant
/// and might contain variables.
///
/// Lists and objects variants are _spanned_, i.e. they contain a reference to
/// their position in the source file, if available.
#[expect(missing_docs, reason = "self-explanatory")]
#[derive(Clone, Debug, PartialEq)]
pub enum InputValue<S = DefaultScalarValue> {
    Null,
    Scalar(S),
    Enum(String),
    Variable(String),
    List(Vec<Spanning<InputValue<S>>>),
    Object(Vec<(Spanning<String>, Spanning<InputValue<S>>)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinition<'a, S> {
    pub var_type: Spanning<Type<&'a str>>,
    pub default_value: Option<Spanning<InputValue<S>>>,
    pub directives: Option<Vec<Spanning<Directive<'a, S>>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arguments<'a, S> {
    pub items: Vec<(Spanning<&'a str>, Spanning<InputValue<S>>)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinitions<'a, S> {
    pub items: Vec<(Spanning<&'a str>, VariableDefinition<'a, S>)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field<'a, S> {
    pub alias: Option<Spanning<&'a str>>,
    pub name: Spanning<&'a str>,
    pub arguments: Option<Spanning<Arguments<'a, S>>>,
    pub directives: Option<Vec<Spanning<Directive<'a, S>>>>,
    pub selection_set: Option<Vec<Selection<'a, S>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentSpread<'a, S> {
    pub name: Spanning<&'a str>,
    pub directives: Option<Vec<Spanning<Directive<'a, S>>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragment<'a, S> {
    pub type_condition: Option<Spanning<&'a str>>,
    pub directives: Option<Vec<Spanning<Directive<'a, S>>>>,
    pub selection_set: Vec<Selection<'a, S>>,
}

/// Entry in a GraphQL selection set
///
/// This enum represents one of the three variants of a selection that exists
/// in GraphQL: a field, a fragment spread, or an inline fragment. Each of the
/// variants references their location in the query source.
///
/// ```text
/// {
///   field(withArg: 123) { subField }
///   ...fragmentSpread
///   ...on User {
///     inlineFragmentField
///   }
/// }
/// ```
#[expect(missing_docs, reason = "self-explanatory")]
#[derive(Clone, Debug, PartialEq)]
pub enum Selection<'a, S = DefaultScalarValue> {
    Field(Spanning<Field<'a, S>>),
    FragmentSpread(Spanning<FragmentSpread<'a, S>>),
    InlineFragment(Spanning<InlineFragment<'a, S>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Directive<'a, S> {
    pub name: Spanning<&'a str>,
    pub arguments: Option<Spanning<Arguments<'a, S>>>,
}

#[expect(missing_docs, reason = "self-explanatory")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[expect(missing_docs, reason = "self-explanatory")]
#[derive(Clone, Debug, PartialEq)]
pub struct Operation<'a, S> {
    pub operation_type: OperationType,
    pub name: Option<Spanning<&'a str>>,
    pub variable_definitions: Option<Spanning<VariableDefinitions<'a, S>>>,
    pub directives: Option<Vec<Spanning<Directive<'a, S>>>>,
    pub selection_set: Vec<Selection<'a, S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fragment<'a, S> {
    pub name: Spanning<&'a str>,
    pub type_condition: Spanning<&'a str>,
    pub directives: Option<Vec<Spanning<Directive<'a, S>>>>,
    pub selection_set: Vec<Selection<'a, S>>,
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub enum Definition<'a, S> {
    Operation(Spanning<Operation<'a, S>>),
    Fragment(Spanning<Fragment<'a, S>>),
}

#[doc(hidden)]
pub type Document<'a, S> = [Definition<'a, S>];
#[doc(hidden)]
pub type OwnedDocument<'a, S> = Vec<Definition<'a, S>>;

/// Parsing of an unstructured [`InputValue`] into a Rust data type.
///
/// The conversion _can_ fail, and must in that case return an [`Err`]. Thus, not restricted in the
/// definition of this trait, the returned [`Err`] should be convertible with the [`IntoFieldError`]
/// trait to fit well into the library machinery.
///
/// [`IntoFieldError`]: crate::IntoFieldError
pub trait FromInputValue<S = DefaultScalarValue>: Sized {
    /// Type of this conversion error.
    ///
    /// Thus, not restricted, it should be convertible with the [`IntoFieldError`] trait to fit well
    /// into the library machinery.
    ///
    /// [`IntoFieldError`]: crate::IntoFieldError
    type Error;

    /// Performs the conversion.
    fn from_input_value(v: &InputValue<S>) -> Result<Self, Self::Error>;

    /// Performs the conversion from an absent value (e.g. to distinguish
    /// between implicit and explicit `null`).
    ///
    /// The default implementation just calls [`from_input_value()`] as if an
    /// explicit `null` was provided.
    ///
    /// [`from_input_value()`]: FromInputValue::from_input_value
    fn from_implicit_null() -> Result<Self, Self::Error> {
        Self::from_input_value(&InputValue::<S>::Null)
    }
}

/// Losslessly clones a Rust data type into an [`InputValue`].
pub trait ToInputValue<S = DefaultScalarValue> {
    /// Performs the conversion.
    fn to_input_value(&self) -> InputValue<S>;
}

impl<S> InputValue<S> {
    /// Construct a `null` value.
    pub fn null() -> Self {
        Self::Null
    }

    /// Construct a scalar value.
    pub fn scalar<T: Into<S>>(v: T) -> Self {
        Self::Scalar(v.into())
    }

    /// Construct an enum value.
    pub fn enum_value<T: AsRef<str>>(s: T) -> Self {
        Self::Enum(s.as_ref().into())
    }

    /// Construct a variable value.
    pub fn variable<T: AsRef<str>>(v: T) -> Self {
        Self::Variable(v.as_ref().into())
    }

    /// Construct a [`Spanning::unlocated`] list.
    ///
    /// Convenience function to make each [`InputValue`] in the input vector
    /// not contain any location information. Can be used from [`ToInputValue`]
    /// implementations, where no source code position information is available.
    pub fn list(l: Vec<Self>) -> Self {
        Self::List(l.into_iter().map(Spanning::unlocated).collect())
    }

    /// Construct a located list.
    pub fn parsed_list(l: Vec<Spanning<Self>>) -> Self {
        Self::List(l)
    }

    /// Construct aa [`Spanning::unlocated`] object.
    ///
    /// Similarly to [`InputValue::list`] it makes each key and value in the
    /// given hash map not contain any location information.
    pub fn object<K>(o: IndexMap<K, Self>) -> Self
    where
        K: AsRef<str> + Eq + Hash,
    {
        Self::Object(
            o.into_iter()
                .map(|(k, v)| {
                    (
                        Spanning::unlocated(k.as_ref().into()),
                        Spanning::unlocated(v),
                    )
                })
                .collect(),
        )
    }

    /// Construct a located object.
    pub fn parsed_object(o: Vec<(Spanning<String>, Spanning<Self>)>) -> Self {
        Self::Object(o)
    }

    /// Resolves all variables of this [`InputValue`] to their actual `values`.
    ///
    /// If a variable is not present in the `values`:
    /// - Returns [`None`] in case this is an [`InputValue::Variable`].
    /// - Skips field in case of an [`InputValue::Object`] field.
    /// - Replaces with an [`InputValue::Null`] in case of an
    ///   [`InputValue::List`] element.
    ///
    /// This is done, because for an [`InputValue::Variable`] (or an
    /// [`InputValue::Object`] field) a default value can be used later, if it's
    /// provided. While on contrary, a single [`InputValue::List`] element
    /// cannot have a default value.
    #[must_use]
    pub fn into_const(self, values: &Variables<S>) -> Option<Self>
    where
        S: Clone,
    {
        match self {
            Self::Variable(v) => values.get(&v).cloned(),
            Self::List(l) => Some(Self::List(
                l.into_iter()
                    .map(|s| s.map(|v| v.into_const(values).unwrap_or_else(Self::null)))
                    .collect(),
            )),
            Self::Object(o) => Some(Self::Object(
                o.into_iter()
                    .filter_map(|(sk, sv)| sv.and_then(|v| v.into_const(values)).map(|sv| (sk, sv)))
                    .collect(),
            )),
            v => Some(v),
        }
    }

    /// Shorthand form of invoking [`FromInputValue::from_input_value()`].
    pub fn convert<T: FromInputValue<S>>(&self) -> Result<T, T::Error> {
        T::from_input_value(self)
    }

    /// Does the value represent a `null`?
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Does the value represent a variable?
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    /// View the underlying enum value, if present.
    pub fn as_enum_value(&self) -> Option<&str> {
        match self {
            Self::Enum(e) => Some(e.as_str()),
            _ => None,
        }
    }

    /// View the underlying scalar value, if present.
    pub fn as_scalar(&self) -> Option<&S> {
        match self {
            Self::Scalar(s) => Some(s),
            _ => None,
        }
    }

    /// Converts this [`InputValue`] to a [`Spanning::unlocated`] object value.
    ///
    /// This constructs a new [`IndexMap`] containing references to the keys
    /// and values of `self`.
    pub fn to_object_value(&self) -> Option<IndexMap<&str, &Self>> {
        match self {
            Self::Object(o) => Some(
                o.iter()
                    .map(|(sk, sv)| (sk.item.as_str(), &sv.item))
                    .collect(),
            ),
            _ => None,
        }
    }

    /// Converts this [`InputValue`] to a [`Spanning::unlocated`] list value.
    ///
    /// This constructs a new [`Vec`] containing references to the values of
    /// `self`.
    pub fn to_list_value(&self) -> Option<Vec<&Self>> {
        match self {
            Self::List(l) => Some(l.iter().map(|s| &s.item).collect()),
            _ => None,
        }
    }

    /// Recursively finds all variables
    pub fn referenced_variables(&self) -> Vec<&str> {
        match self {
            Self::Variable(name) => vec![name.as_str()],
            Self::List(l) => l
                .iter()
                .flat_map(|v| v.item.referenced_variables())
                .collect(),
            Self::Object(o) => o
                .iter()
                .flat_map(|(_, v)| v.item.referenced_variables())
                .collect(),
            _ => vec![],
        }
    }

    /// Compares equality with another [`InputValue``] ignoring any source
    /// position information.
    pub fn unlocated_eq(&self, other: &Self) -> bool
    where
        S: PartialEq,
    {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Scalar(s1), Self::Scalar(s2)) => s1 == s2,
            (Self::Enum(s1), Self::Enum(s2)) | (Self::Variable(s1), Self::Variable(s2)) => s1 == s2,
            (Self::List(l1), Self::List(l2)) => l1
                .iter()
                .zip(l2.iter())
                .all(|(v1, v2)| v1.item.unlocated_eq(&v2.item)),
            (Self::Object(o1), Self::Object(o2)) => {
                o1.len() == o2.len()
                    && o1.iter().all(|(sk1, sv1)| {
                        o2.iter().any(|(sk2, sv2)| {
                            sk1.item == sk2.item && sv1.item.unlocated_eq(&sv2.item)
                        })
                    })
            }
            _ => false,
        }
    }
}

impl<S: ScalarValue> fmt::Display for InputValue<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Scalar(s) => fmt::Display::fmt(<&Scalar<_>>::from(s), f),
            Self::Enum(v) => write!(f, "{v}"),
            Self::Variable(v) => write!(f, "${v}"),
            Self::List(v) => {
                write!(f, "[")?;
                for (i, spanning) in v.iter().enumerate() {
                    spanning.item.fmt(f)?;
                    if i < v.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Self::Object(o) => {
                write!(f, "{{")?;
                for (i, (k, v)) in o.iter().enumerate() {
                    write!(f, "{}: ", k.item)?;
                    v.item.fmt(f)?;
                    if i < o.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

/// Conversion into an [`InputValue`].
///
/// This trait exists to work around [orphan rules] and allow to specify custom efficient
/// conversions whenever some custom [`ScalarValue`] is involved
/// (`impl IntoInputValue<CustomScalarValue> for ForeignType` would work, while
/// `impl From<ForeignType> for InputValue<CustomScalarValue>` wound not).
///
/// This trait is used inside [`graphql_input_value!`] macro expansion and implementing it allows to
/// put values of the implementor type there.
///
/// [`graphql_input_value!`]: crate::graphql_input_value
/// [orphan rules]: https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
pub trait IntoInputValue<S> {
    /// Converts this value into an [`InputValue`].
    #[must_use]
    fn into_input_value(self) -> InputValue<S>;
}

impl<S> IntoInputValue<S> for InputValue<S> {
    fn into_input_value(self) -> Self {
        self
    }
}

impl<T, S> IntoInputValue<S> for Option<T>
where
    T: IntoInputValue<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        match self {
            Some(v) => v.into_input_value(),
            None => InputValue::Null,
        }
    }
}

impl<T, S> IntoInputValue<S> for &T
where
    T: ToScalarValue<S> + ?Sized,
{
    fn into_input_value(self) -> InputValue<S> {
        InputValue::Scalar(self.to_scalar_value())
    }
}

impl<S> IntoInputValue<S> for String
where
    String: Into<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        InputValue::Scalar(self.into())
    }
}

impl<S> IntoInputValue<S> for Cow<'_, str>
where
    for<'a> &'a str: IntoInputValue<S>,
    String: IntoInputValue<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        match self {
            Cow::Borrowed(s) => s.into_input_value(),
            Cow::Owned(s) => s.into_input_value(),
        }
    }
}

impl<S: ScalarValue> IntoInputValue<S> for ArcStr
where
    ArcStr: ToScalarValue<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        InputValue::Scalar(self.to_scalar_value())
    }
}

impl<S: ScalarValue> IntoInputValue<S> for CompactString
where
    CompactString: ToScalarValue<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        InputValue::Scalar(self.to_scalar_value())
    }
}

impl<S> IntoInputValue<S> for i32
where
    i32: ToScalarValue<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        InputValue::Scalar(self.to_scalar_value())
    }
}

impl<S> IntoInputValue<S> for f64
where
    f64: ToScalarValue<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        InputValue::Scalar(self.to_scalar_value())
    }
}

impl<S> IntoInputValue<S> for bool
where
    bool: ToScalarValue<S>,
{
    fn into_input_value(self) -> InputValue<S> {
        InputValue::Scalar(self.to_scalar_value())
    }
}

impl<'a, S> Arguments<'a, S> {
    pub fn into_iter(self) -> vec::IntoIter<(Spanning<&'a str>, Spanning<InputValue<S>>)> {
        self.items.into_iter()
    }

    pub fn iter(&self) -> slice::Iter<'_, (Spanning<&'a str>, Spanning<InputValue<S>>)> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<'_, (Spanning<&'a str>, Spanning<InputValue<S>>)> {
        self.items.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, key: &str) -> Option<&Spanning<InputValue<S>>> {
        self.items
            .iter()
            .filter(|&(k, _)| k.item == key)
            .map(|(_, v)| v)
            .next()
    }
}

impl<'a, S> VariableDefinitions<'a, S> {
    pub fn iter(&self) -> slice::Iter<'_, (Spanning<&'a str>, VariableDefinition<'a, S>)> {
        self.items.iter()
    }
}

#[cfg(test)]
mod spec_input_value_fmt {
    use crate::graphql_input_value;

    use super::InputValue;

    #[test]
    fn correct() {
        let value: InputValue = graphql_input_value!(null);
        assert_eq!(value.to_string(), "null");

        let value: InputValue = graphql_input_value!(123);
        assert_eq!(value.to_string(), "123");

        let value: InputValue = graphql_input_value!(12.3);
        assert_eq!(value.to_string(), "12.3");

        let value: InputValue = graphql_input_value!("FOO");
        assert_eq!(value.to_string(), "\"FOO\"");

        let value: InputValue = graphql_input_value!(true);
        assert_eq!(value.to_string(), "true");

        let value: InputValue = graphql_input_value!(BAR);
        assert_eq!(value.to_string(), "BAR");

        let value: InputValue = graphql_input_value!(@baz);
        assert_eq!(value.to_string(), "$baz");

        let value: InputValue = graphql_input_value!([1, 2]);
        assert_eq!(value.to_string(), "[1, 2]");

        let value: InputValue = graphql_input_value!({"foo": 1,"bar": 2});
        assert_eq!(value.to_string(), "{foo: 1, bar: 2}");
    }
}
