use std::{
    convert::{AsMut, AsRef},
    fmt,
    ops::{Deref, DerefMut},
};

use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde_json::Value;

pub const NULLABLE: &str = "nullable";
pub const X_EMBEDDED_RESOURCE: &str = "x-kubernetes-embedded-resource";
pub const X_INT_OR_STRING: &str = "x-kubernetes-int-or-string";
pub const X_PRESERVE_UNKNOWN_FIELDS: &str = "x-kubernetes-preserve-unknown-fields";

#[derive(Debug)]
pub struct Nullable<T>(T);

impl<T> Nullable<T> {
    #[must_use]
    pub const fn new(inner: T) -> Self { Self(inner) }

    #[inline]
    pub fn into_inner(self) -> T { self.0 }
}

impl<T> JsonSchema for Nullable<T>
where
    T: JsonSchema,
{
    fn is_referenceable() -> bool { T::is_referenceable() }

    fn schema_name() -> String { T::schema_name() }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = T::json_schema(gen);
        if let Schema::Object(ref mut obj) = schema {
            obj.extensions.insert(NULLABLE.to_string(), Value::Bool(true));
        }

        schema
    }
}

impl<T> From<T> for Nullable<T> {
    fn from(value: T) -> Self { Self(value) }
}

impl<T> fmt::Display for Nullable<T>
where
    T: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl<T> Deref for Nullable<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> DerefMut for Nullable<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T> AsRef<T> for Nullable<T> {
    #[inline]
    fn as_ref(&self) -> &T { &self.0 }
}

impl<T> AsMut<T> for Nullable<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T { &mut self.0 }
}

#[derive(Debug)]
pub struct NonNullable<T>(T);

impl<T> NonNullable<T> {
    #[must_use]
    pub const fn new(inner: T) -> Self { Self(inner) }

    #[inline]
    pub fn into_inner(self) -> T { self.0 }
}

impl<T> JsonSchema for NonNullable<T>
where
    T: JsonSchema,
{
    fn is_referenceable() -> bool { T::is_referenceable() }

    fn schema_name() -> String { T::schema_name() }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = T::json_schema(gen);
        if let Schema::Object(ref mut obj) = schema {
            obj.extensions.remove(NULLABLE);
        }

        schema
    }
}

impl<T> From<T> for NonNullable<T> {
    fn from(value: T) -> Self { Self(value) }
}

impl<T> fmt::Display for NonNullable<T>
where
    T: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl<T> Deref for NonNullable<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> DerefMut for NonNullable<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T> AsRef<T> for NonNullable<T> {
    #[inline]
    fn as_ref(&self) -> &T { &self.0 }
}

impl<T> AsMut<T> for NonNullable<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T { &mut self.0 }
}
