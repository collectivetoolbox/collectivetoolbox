use serde::{Deserialize, Serialize};

/// Represents an input value that may be missing, explicitly cleared, or set.
///
/// This is primarily used for partial updates (PATCH-like behavior), where
/// omitted fields should preserve their existing persisted representation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MaybeOption<T> {
    /// The field was not provided.
    #[default]
    Missing,
    /// The field was explicitly provided as `null`. Whether it will be
    /// defaulted is a higher-level concern.
    Null,
    /// The field was provided with a value.
    Value(T),
}

impl<T> MaybeOption<T> {
    pub fn missing() -> Self {
        Self::Missing
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn value(v: T) -> Self {
        Self::Value(v)
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn is_absent(&self) -> bool {
        matches!(self, Self::Missing | Self::Null)
    }
}

impl<T> From<T> for MaybeOption<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MaybeValue<T> {
    /// The field was not provided.
    #[default]
    Missing,
    /// The field was provided with a value.
    Value(T),
}

impl<T> MaybeValue<T> {
    pub fn missing() -> Self {
        Self::Missing
    }

    pub fn value(v: T) -> Self {
        Self::Value(v)
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn is_absent(&self) -> bool {
        matches!(self, Self::Missing)
    }
}

impl<T> From<T> for MaybeValue<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

/// A small abstraction so helpers can work with both `MaybeOption<T>` (nullable)
/// and `MaybeValue<T>` (non-nullable).
pub trait MaybeField<T> {
    /// Returns `Some(&T)` only when a value is explicitly present.
    fn as_value(&self) -> Option<&T>;

    /// Returns `true` if the field is absent (missing, and/or null depending on
    /// the type).
    fn is_absent(&self) -> bool;

    /// Returns `true` if the field is missing (not provided).
    fn is_missing(&self) -> bool;
}

impl<T> MaybeField<T> for MaybeOption<T> {
    fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(v) => Some(v),
            Self::Missing | Self::Null => None,
        }
    }

    fn is_absent(&self) -> bool {
        Self::is_absent(self)
    }

    fn is_missing(&self) -> bool {
        Self::is_missing(self)
    }
}

impl<T> MaybeField<T> for MaybeValue<T> {
    fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(v) => Some(v),
            Self::Missing => None,
        }
    }

    fn is_absent(&self) -> bool {
        Self::is_absent(self)
    }

    fn is_missing(&self) -> bool {
        Self::is_missing(self)
    }
}

impl<T> Serialize for MaybeOption<T>
where
    T: Serialize,
{
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Missing | Self::Null => {
                Option::<&T>::None.serialize(serializer)
            }
            Self::Value(v) => Option::<&T>::Some(v).serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for MaybeOption<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt = Option::<T>::deserialize(deserializer)?;
        Ok(match opt {
            None => Self::Null,
            Some(v) => Self::Value(v),
        })
    }
}

impl<T> Serialize for MaybeValue<T>
where
    T: Serialize,
{
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Missing => Option::<&T>::None.serialize(serializer),
            Self::Value(v) => Option::<&T>::Some(v).serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for MaybeValue<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt = Option::<T>::deserialize(deserializer)?;
        match opt {
            Some(v) => Ok(Self::Value(v)),
            None => Err(serde::de::Error::custom(
                "null is not allowed for this field",
            )),
        }
    }
}

pub fn bool_or_default<M>(v: &M, default: bool) -> bool
where
    M: MaybeField<bool>,
{
    v.as_value().copied().unwrap_or(default)
}

pub fn str_or_default<M>(v: &M, default: &str) -> String
where
    M: MaybeField<String>,
{
    match v.as_value() {
        Some(s) => s.clone(),
        None => default.to_string(),
    }
}

pub fn str_or_empty<M>(v: &M) -> String
where
    M: MaybeField<String>,
{
    match v.as_value() {
        Some(s) => s.clone(),
        None => String::new(),
    }
}

pub fn u16_or_empty<M>(v: &M) -> Option<u16>
where
    M: MaybeField<u16>,
{
    v.as_value().copied()
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use anyhow::{Result, ensure};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize)]
    struct In {
        #[serde(default)]
        a: MaybeOption<u32>,
        #[serde(default)]
        b: MaybeValue<u32>,
    }

    #[derive(Debug, Default, Serialize)]
    struct Out {
        #[serde(default, skip_serializing_if = "MaybeOption::is_missing")]
        a: MaybeOption<u32>,
        #[serde(default, skip_serializing_if = "MaybeValue::is_missing")]
        b: MaybeValue<u32>,
    }

    #[crate::ctb_test]
    fn missing_vs_null_vs_value_deser() -> Result<()> {
        let v: In = serde_json::from_str("{}")?;
        ensure!(v.a == MaybeOption::Missing);
        ensure!(v.b == MaybeValue::Missing);

        let v: In = serde_json::from_str(r#"{"a":null}"#)?;
        ensure!(v.a == MaybeOption::Null);
        ensure!(v.b == MaybeValue::Missing);

        let err = serde_json::from_str::<In>(r#"{"b":null}"#).err();
        ensure!(err.is_some());

        let v: In = serde_json::from_str(r#"{"a":1,"b":2}"#)?;
        ensure!(v.a == MaybeOption::Value(1));
        ensure!(v.b == MaybeValue::Value(2));

        Ok(())
    }

    #[crate::ctb_test]
    fn skip_serializing_if_missing_works() -> Result<()> {
        let out = Out::default();
        let s = serde_json::to_string(&out)?;
        ensure!(s == "{}");

        let out = Out {
            a: MaybeOption::Null,
            b: MaybeValue::Missing,
        };
        let s = serde_json::to_string(&out)?;
        ensure!(s == r#"{"a":null}"#);

        let out = Out {
            a: MaybeOption::Missing,
            b: MaybeValue::Value(5),
        };
        let s = serde_json::to_string(&out)?;
        ensure!(s == r#"{"b":5}"#);

        Ok(())
    }

    #[crate::ctb_test]
    fn helpers_work_for_both_types() -> Result<()> {
        let v = MaybeOption::Null;
        ensure!(bool_or_default(&v, true));

        let v = MaybeValue::Missing;
        ensure!(str_or_default(&v, "x") == "x");

        let v = MaybeValue::Value("y".to_string());
        ensure!(str_or_empty(&v) == "y");

        let v = MaybeOption::Missing;
        ensure!(u16_or_empty(&v).is_none());

        Ok(())
    }
}
