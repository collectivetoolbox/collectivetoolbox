use serde_json::{Map, Value};

use crate::json::maybe_value::{MaybeField, MaybeOption, MaybeValue};

/// A small abstraction over patch field types (`MaybeOption<T>` vs
/// `MaybeValue<T>`) so form-to-patch helpers can infer the correct return type
/// from assignment context.
pub trait PatchFieldFromForm<T>: Sized {
    /// Field omitted / not provided.
    fn from_form_missing() -> Self;

    /// "Reset to default" (typically a checked "use default" checkbox).
    ///
    /// This should *not* force a JSON `null` to be written; instead it should
    /// behave like "no override".
    fn from_form_defaulted() -> Self;

    /// Field provided with a concrete value.
    fn from_form_value(v: T) -> Self;
}

impl<T> PatchFieldFromForm<T> for MaybeOption<T> {
    fn from_form_missing() -> Self {
        Self::Missing
    }

    fn from_form_defaulted() -> Self {
        // "Use default" means "remove override". We represent that as
        // `Missing` so the patch layer does not inject `null` keys.
        Self::Missing
    }

    fn from_form_value(v: T) -> Self {
        Self::Value(v)
    }
}

impl<T> PatchFieldFromForm<T> for MaybeValue<T> {
    fn from_form_missing() -> Self {
        Self::Missing
    }

    fn from_form_defaulted() -> Self {
        // `MaybeValue<T>` cannot represent null/default explicitly; treat as
        // "no change".
        Self::Missing
    }

    fn from_form_value(v: T) -> Self {
        Self::Value(v)
    }
}

/// Convert a form `Option<bool>` (checkbox) plus a "use default" flag into a
/// patch field (`MaybeOption<bool>` or `MaybeValue<bool>`).
pub fn bool_to_patch<P>(default_checked: bool, v: Option<bool>) -> P
where
    P: PatchFieldFromForm<bool>,
{
    if default_checked {
        P::from_form_defaulted()
    } else if let Some(v) = v {
        P::from_form_value(v)
    } else {
        P::from_form_missing()
    }
}

/// Convert a form `Option<String>` plus a "use default" flag into a patch field
/// (`MaybeOption<String>` or `MaybeValue<String>`).
///
/// Empty/whitespace-only strings are treated as "not provided" to avoid
/// accidental clears when a textbox is left empty.
pub fn string_to_patch<P>(default_checked: bool, v: Option<String>) -> P
where
    P: PatchFieldFromForm<String>,
{
    if default_checked {
        return P::from_form_defaulted();
    }

    let Some(v) = v else {
        return P::from_form_missing();
    };

    if v.trim().is_empty() {
        P::from_form_missing()
    } else {
        P::from_form_value(v)
    }
}

/// Convert a form `Option<String>` containing a u16 (e.g., a port) plus a "use
/// default" flag into a patch field (`MaybeOption<u16>` or `MaybeValue<u16>`).
///
/// Returns an error if the string is non-empty and not a valid u16.
pub fn u16_string_to_patch<P>(
    default_checked: bool,
    v: Option<String>,
) -> anyhow::Result<P>
where
    P: PatchFieldFromForm<u16>,
{
    if default_checked {
        return Ok(P::from_form_defaulted());
    }

    let Some(v) = v else {
        return Ok(P::from_form_missing());
    };

    let trimmed = v.trim();
    if trimmed.is_empty() {
        return Ok(P::from_form_defaulted());
    }

    let n = trimmed.parse::<u16>()?;
    if n == 0 {
        Ok(P::from_form_defaulted())
    } else {
        Ok(P::from_form_value(n))
    }
}

pub fn apply_bool<M>(map: &mut Map<String, Value>, key: &str, v: &M)
where
    M: MaybeField<bool>,
{
    if v.is_missing() {
        // Do nothing
    } else if v.is_absent() {
        map.insert(key.to_string(), Value::Null);
    } else if let Some(b) = v.as_value() {
        map.insert(key.to_string(), Value::Bool(*b));
    }
}

pub fn apply_string<M>(map: &mut Map<String, Value>, key: &str, v: &M)
where
    M: MaybeField<String>,
{
    if v.is_missing() {
        // Do nothing
    } else if v.is_absent() {
        map.insert(key.to_string(), Value::Null);
    } else if let Some(s) = v.as_value() {
        map.insert(key.to_string(), Value::String(s.clone()));
    }
}

pub fn apply_u16<M>(map: &mut Map<String, Value>, key: &str, v: &M)
where
    M: MaybeField<u16>,
{
    if v.is_missing() {
        // Do nothing
    } else if v.is_absent() {
        map.insert(key.to_string(), Value::Null);
    } else if let Some(n) = v.as_value() {
        map.insert(
            key.to_string(),
            Value::Number(serde_json::Number::from(u64::from(*n))),
        );
    }
}

pub fn apply_u32<M>(map: &mut Map<String, Value>, key: &str, v: &M)
where
    M: MaybeField<u32>,
{
    if v.is_missing() {
        // Do nothing
    } else if v.is_absent() {
        map.insert(key.to_string(), Value::Null);
    } else if let Some(n) = v.as_value() {
        map.insert(
            key.to_string(),
            Value::Number(serde_json::Number::from(u64::from(*n))),
        );
    }
}

pub fn apply_u64_vec<M>(map: &mut Map<String, Value>, key: &str, v: &M)
where
    M: MaybeField<Vec<u64>>,
{
    if v.is_missing() {
        // Do nothing
    } else if v.is_absent() {
        map.insert(key.to_string(), Value::Null);
    } else if let Some(vec) = v.as_value() {
        let arr = vec.iter().copied().map(Value::from).collect::<Vec<Value>>();
        map.insert(key.to_string(), Value::Array(arr));
    }
}

pub fn apply_serde<M, T>(map: &mut Map<String, Value>, key: &str, v: &M)
where
    M: MaybeField<T>,
    T: serde::Serialize,
{
    if v.is_missing() {
        // Do nothing
    } else if v.is_absent() {
        map.insert(key.to_string(), Value::Null);
    } else if let Some(val) = v.as_value() {
        if let Ok(json_val) = serde_json::to_value(val) {
            map.insert(key.to_string(), json_val);
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use crate::json::maybe_value::{MaybeOption, MaybeValue};
    use anyhow::{Result, ensure};

    #[crate::ctb_test]
    fn apply_null_inserts_json_null() -> Result<()> {
        let mut map = Map::<String, Value>::new();

        let v = MaybeOption::<String>::Null;
        apply_string(&mut map, "k", &v);
        ensure!(map.get("k") == Some(&Value::Null));

        map.clear();
        let v = MaybeOption::<u16>::Null;
        apply_u16(&mut map, "k", &v);
        ensure!(map.get("k") == Some(&Value::Null));

        map.clear();
        let v = MaybeOption::<Vec<u64>>::Null;
        apply_u64_vec(&mut map, "k", &v);
        ensure!(map.get("k") == Some(&Value::Null));

        map.clear();
        let v = MaybeValue::<bool>::Missing;
        apply_bool(&mut map, "k", &v);
        ensure!(!map.contains_key("k"));

        Ok(())
    }

    #[crate::ctb_test]
    fn form_helpers_infer_return_type() -> Result<()> {
        let v: MaybeOption<String> =
            string_to_patch(true, Some("x".to_string()));
        ensure!(v == MaybeOption::Missing);

        let v: MaybeValue<String> =
            string_to_patch(true, Some("x".to_string()));
        ensure!(v == MaybeValue::Missing);

        let v: MaybeOption<bool> = bool_to_patch(false, Some(true));
        ensure!(v == MaybeOption::Value(true));

        let v: MaybeValue<bool> = bool_to_patch(false, None);
        ensure!(v == MaybeValue::Missing);

        Ok(())
    }

    #[crate::ctb_test]
    fn u16_string_to_patch_validates() -> Result<()> {
        let v: MaybeOption<u16> =
            u16_string_to_patch(false, Some("123".to_string()))?;
        ensure!(v == MaybeOption::Value(123));

        let v: MaybeOption<u16> =
            u16_string_to_patch(false, Some(String::new()))?;
        ensure!(v == MaybeOption::Missing);

        let err: std::result::Result<MaybeOption<u16>, _> =
            u16_string_to_patch(false, Some("nope".to_string()));
        ensure!(err.is_err());

        Ok(())
    }
}
