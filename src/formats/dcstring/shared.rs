// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Document-local lexical bindings and explicitly supplied enclosing scopes.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

pub use serde_json;

use crate::{DcChar, DcMixedDecode, DcMixedEncode, DcMixedReader, DcMstring};

/// Typed shared values whose definitions are persisted by the caller.
#[derive(Clone, Default)]
pub struct SharedValueContext {
    bindings: BTreeMap<String, Arc<dyn Any + Send + Sync>>,
    scopes: Vec<BTreeMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl std::fmt::Debug for SharedValueContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.scopes.iter().rev().chain(std::iter::once(&self.bindings)).flat_map(BTreeMap::keys)).finish()
    }
}

impl SharedValueContext {
    /// Reports whether an enclosing document supplied this binding.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    fn lookup(&self, name: &str) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.scopes.iter().rev().chain(std::iter::once(&self.bindings)).find_map(|scope| scope.get(name))
    }

    /// Opens a lexical scope without importing bindings from another document.
    pub fn enter_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    /// Discards the innermost scope and restores outer declarations.
    pub fn exit_scope(&mut self) -> Result<()> {
        self.scopes.pop().context("No reference scope to exit")?;
        Ok(())
    }

    /// Declares a value in the current scope; inner declarations may shadow.
    pub fn insert<T: Any + Send + Sync>(&mut self, name: impl Into<String>, value: Arc<T>) -> Result<()> {
        let name = name.into();
        ensure!(!name.is_empty(), "Shared binding names must not be empty");
        let scope = if let Some(scope) = self.scopes.last() { scope } else { &self.bindings };
        ensure!(!scope.contains_key(&name), "Duplicate shared binding {name:?}");
        if self.contains(&name) {
            warn_fmt!("Reference declaration {name:?} shadows an enclosing declaration");
        }
        let scope = if let Some(scope) = self.scopes.last_mut() { scope } else { &mut self.bindings };
        scope.insert(name, Arc::new(value));
        Ok(())
    }

    /// Finds a persisted definition with identical value and type.
    pub fn binding_for<T: Any + Send + Sync + PartialEq>(&self, value: &Arc<T>) -> Option<&str> {
        self.scopes.iter().rev().chain(std::iter::once(&self.bindings)).flat_map(BTreeMap::iter).find_map(|(name, binding)| {
            if !self.lookup(name).is_some_and(|visible| Arc::ptr_eq(visible, binding)) {
                return None;
            }
            let stored = binding.downcast_ref::<Arc<T>>()?;
            if Arc::ptr_eq(stored, value) || stored == value {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    /// Resolves a reference without recapturing or copying its definition.
    pub fn resolve<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>> {
        let binding = self.lookup(name).with_context(|| format!("Unresolved shared binding {name:?}"))?;
        let value = binding.downcast_ref::<Arc<T>>().with_context(|| format!("Wrong type for shared binding {name:?}"))?;
        Ok(Arc::clone(value))
    }
}

/// Emits a lexical scope boundary and opens its binding table.
pub fn encode_scope_begin(mst: &mut DcMstring, scope: &mut SharedValueContext) {
    mst.push_char(DcChar::from_short(541));
    scope.enter_scope();
}

/// Emits the closing boundary and discards declarations in that scope.
pub fn encode_scope_end(mst: &mut DcMstring, scope: &mut SharedValueContext) -> Result<()> {
    scope.exit_scope()?;
    mst.push_char(DcChar::from_short(542));
    Ok(())
}

/// Declares a concrete value using Dc 269 and registers it for later references.
pub fn encode_declaration<T>(mst: &mut DcMstring, scope: &mut SharedValueContext, name: &str, value: Arc<T>) -> Result<()>
where
    T: Any + Send + Sync + DcMixedEncode,
{
    encode_identifier(mst, name)?;
    mst.push_char(DcChar::from_short(269));
    value.encode_dc_mixed_with_context(mst, scope)?;
    scope.insert(name, value)
}

/// Contextual JSON serialization generated for explicitly shared fields.
pub trait SharedJson: Sized {
    fn encode_json_with_context(&self, context: &SharedValueContext) -> Result<serde_json::Value>;
    fn decode_json_with_context(value: serde_json::Value, context: &SharedValueContext) -> Result<Self>;
}

/// Encodes a shared JSON field using an unambiguous contextual envelope.
pub fn shared_to_json<T>(value: &Arc<T>, context: &SharedValueContext) -> Result<serde_json::Value>
where
    T: Any + Send + Sync + PartialEq + serde::Serialize,
{
    let (key, payload) = if let Some(name) = context.binding_for(value) {
        ("$dc_shared", serde_json::Value::String(name.to_owned()))
    } else {
        ("$dc_inline", serde_json::to_value(value)?)
    };
    Ok(serde_json::Value::Object(serde_json::Map::from_iter([(key.to_owned(), payload)])))
}

/// Decodes a contextual envelope, preserving registered Arc identity.
pub fn shared_from_json<T>(value: serde_json::Value, context: &SharedValueContext) -> Result<Arc<T>>
where
    T: Any + Send + Sync + serde::de::DeserializeOwned,
{
    let serde_json::Value::Object(mut envelope) = value else { bail!("Expected shared-value envelope"); };
    ensure!(envelope.len() == 1, "Invalid shared-value envelope");
    if let Some(value) = envelope.remove("$dc_shared") {
        let name = value.as_str().context("Shared binding must be a string")?;
        context.resolve(name)
    } else if let Some(value) = envelope.remove("$dc_inline") {
        Ok(Arc::new(serde_json::from_value(value)?))
    } else {
        bail!("Unknown shared-value envelope");
    }
}

/// Emits a nonempty Unicode identifier using Dc 270..271 framing.
pub fn encode_identifier(mst: &mut DcMstring, name: &str) -> Result<()> {
    ensure!(!name.is_empty(), "Identifiers must not be empty");
    mst.push_char(DcChar::from_short(270));
    for character in name.chars() {
        let character = DcChar::from_char(character);
        if character == DcChar::from_short(255) || character == DcChar::from_short(271) {
            mst.push_char(DcChar::from_short(255));
        }
        mst.push_char(character);
    }
    mst.push_char(DcChar::from_short(271));
    Ok(())
}

/// Reads a Unicode identifier without interpreting its name as an expression.
pub fn decode_identifier(reader: &mut DcMixedReader<'_>) -> Result<String> {
    reader.expect_short_dc(270)?;
    let mut name = String::new();
    loop {
        let mut character = reader.next_char()?;
        if character == DcChar::from_short(271) {
            ensure!(!name.is_empty(), "Identifiers must not be empty");
            return Ok(name);
        }
        if character == DcChar::from_short(255) {
            character = reader.next_char()?;
        }
        let codepoint = character.to_unicode().context("Expected Unicode binding name")?;
        name.push(char::from_u32(codepoint).context("Invalid Unicode binding name")?);
    }
}

/// Assigns a field from a lexically bound value using identifiers, not strings.
pub fn encode_shared_reference(mst: &mut DcMstring, field: &str, binding: &str) -> Result<()> {
    encode_identifier(mst, field)?;
    mst.push_char(DcChar::from_short(269));
    mst.push_char(DcChar::from_short(540));
    encode_identifier(mst, binding)
}

/// Reads an assignment whose value is a named reference.
pub fn decode_shared_reference(reader: &mut DcMixedReader<'_>) -> Result<(String, String)> {
    let field = decode_identifier(reader)?;
    reader.expect_short_dc(269)?;
    reader.expect_short_dc(540)?;
    Ok((field, decode_identifier(reader)?))
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, crate::DcMixed)]
    #[dc(begin = 321, end = 322)]
    struct Definition {
        #[dc(333)]
        value: u64,
    }

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, crate::DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SharedRecord {
        #[dc(nested = 321, shared = "definition")]
        definition: Option<Arc<Definition>>,
    }

    #[derive(Debug, PartialEq, Eq, crate::DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct NestedRecord {
        #[dc(333)]
        record: Box<SharedRecord>,
    }

    #[crate::ctb_test]
    fn test_shared_attribute_roundtrip() -> Result<()> {
        let definition = Arc::new(Definition { value: 42 });
        let record = SharedRecord { definition: Some(Arc::clone(&definition)) };
        crate::assert_dc_roundtrip(&record)?;
        let mut context = SharedValueContext::default();
        context.insert("source", Arc::clone(&definition))?;
        let mut encoded = DcMstring::new();
        record.encode_dc_mixed_with_context(&mut encoded, &context)?;
        let decoded = SharedRecord::decode_dc_mixed(&mut DcMixedReader::with_context(&encoded, &context))?;
        assert!(decoded.definition.as_ref().is_some_and(|value| Arc::ptr_eq(value, &definition)));
        assert!(SharedRecord::decode_dc_mixed(&mut DcMixedReader::new(&encoded)).is_err());
        let json = record.encode_json_with_context(&context)?;
        let decoded = SharedRecord::decode_json_with_context(json.clone(), &context)?;
        assert!(decoded.definition.as_ref().is_some_and(|value| Arc::ptr_eq(value, &definition)));
        assert!(SharedRecord::decode_json_with_context(json, &SharedValueContext::default()).is_err());
        let empty = SharedRecord { definition: None };
        assert_eq!(SharedRecord::decode_json_with_context(empty.encode_json_with_context(&context)?, &context)?, empty);
        let distinct = SharedRecord { definition: Some(Arc::new(Definition { value: 43 })) };
        let mut inline = DcMstring::new();
        distinct.encode_dc_mixed_with_context(&mut inline, &context)?;
        assert_eq!(SharedRecord::decode_dc_mixed(&mut DcMixedReader::new(&inline))?, distinct);
        let nested = NestedRecord { record: Box::new(record) };
        let mut encoded = DcMstring::new();
        nested.encode_dc_mixed_with_context(&mut encoded, &context)?;
        let decoded = NestedRecord::decode_dc_mixed(&mut DcMixedReader::with_context(&encoded, &context))?;
        assert!(decoded.record.definition.as_ref().is_some_and(|value| Arc::ptr_eq(value, &definition)));
        let array = [nested.record];
        let mut encoded = DcMstring::new();
        array.encode_dc_mixed_with_context(&mut encoded, &context)?;
        let decoded = <[Box<SharedRecord>; 1]>::decode_dc_mixed(&mut DcMixedReader::with_context(&encoded, &context))?;
        assert!(decoded.first().and_then(|record| record.definition.as_ref()).is_some_and(|value| Arc::ptr_eq(value, &definition)));
        let mut duplicate = DcMstring::new();
        duplicate.push_char(DcChar::from_short(401));
        encode_shared_reference(&mut duplicate, "definition", "source")?;
        encode_shared_reference(&mut duplicate, "definition", "source")?;
        duplicate.push_char(DcChar::from_short(402));
        assert!(SharedRecord::decode_dc_mixed(&mut DcMixedReader::with_context(&duplicate, &context)).is_err());
        let mut unknown = DcMstring::new();
        unknown.push_char(DcChar::from_short(401));
        encode_shared_reference(&mut unknown, "unknown", "source")?;
        unknown.push_char(DcChar::from_short(402));
        assert!(SharedRecord::decode_dc_mixed(&mut DcMixedReader::with_context(&unknown, &context)).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_shared_reference_edits() -> Result<()> {
        let definition = Arc::new(Definition { value: 42 });
        let record = SharedRecord { definition: Some(Arc::clone(&definition)) };
        let updated = SharedRecord { definition: Some(Arc::new(Definition { value: 999 })) };
        let mut context = SharedValueContext::default();
        context.insert("source", Arc::clone(&definition))?;
        let mut source = DcMstring::new();
        record.encode_dc_mixed_with_context(&mut source, &context)?;
        assert!(source.inspect::<SharedRecord>().diagnostic().is_some());
        let view = source.inspect_with_context::<SharedRecord>(&context);
        assert!(view.diagnostic().is_none());
        assert_eq!(view.fields().len(), 1);
        let revised = view.replace_field("definition", &updated)?;
        assert_eq!(revised.decode::<SharedRecord>()?, updated);
        let text = source.to_dc_string()?;
        assert_eq!(text.decode_with_context::<SharedRecord>(&context)?, record);
        let revised = text.inspect_with_context::<SharedRecord>(&context).replace_field("definition", &updated)?;
        assert_eq!(revised.decode::<SharedRecord>()?, updated);
        assert!(Arc::ptr_eq(&context.resolve::<Definition>("source")?, &definition));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_lexical_reference_scopes() -> Result<()> {
        let outer = Arc::new(Definition { value: 42 });
        let inner = Arc::new(Definition { value: 43 });
        let mut scope = SharedValueContext::default();
        let mut document = DcMstring::new();
        encode_scope_begin(&mut document, &mut scope);
        encode_declaration(&mut document, &mut scope, "source", Arc::clone(&outer))?;
        let outer_record = SharedRecord { definition: Some(Arc::clone(&outer)) };
        outer_record.encode_dc_mixed_with_context(&mut document, &scope)?;
        encode_scope_begin(&mut document, &mut scope);
        encode_declaration(&mut document, &mut scope, "source", Arc::clone(&inner))?;
        assert!(scope.binding_for(&outer).is_none());
        SharedRecord { definition: Some(Arc::clone(&inner)) }.encode_dc_mixed_with_context(&mut document, &scope)?;
        encode_scope_end(&mut document, &mut scope)?;
        assert!(Arc::ptr_eq(&scope.resolve::<Definition>("source")?, &outer));
        outer_record.encode_dc_mixed_with_context(&mut document, &scope)?;
        encode_scope_end(&mut document, &mut scope)?;
        assert!(scope.resolve::<Definition>("source").is_err());
        assert!(scope.exit_scope().is_err());
        let mut reader = DcMixedReader::new(&document);
        reader.enter_reference_scope()?;
        let mut syntax = reader.clone();
        assert_eq!(decode_identifier(&mut syntax)?, "source");
        syntax.expect_short_dc(269)?;
        syntax.expect_short_dc(321)?;
        let outer = reader.decode_declaration::<Definition>()?;
        let mut syntax = reader.clone();
        syntax.expect_short_dc(401)?;
        assert_eq!(decode_identifier(&mut syntax)?, "definition");
        syntax.expect_short_dc(269)?;
        syntax.expect_short_dc(540)?;
        assert_eq!(decode_identifier(&mut syntax)?, "source");
        syntax.expect_short_dc(402)?;
        let decoded = SharedRecord::decode_dc_mixed(&mut reader)?;
        assert!(decoded.definition.as_ref().is_some_and(|value| Arc::ptr_eq(value, &outer)));
        reader.enter_reference_scope()?;
        let inner = reader.decode_declaration::<Definition>()?;
        let decoded = SharedRecord::decode_dc_mixed(&mut reader)?;
        assert!(decoded.definition.as_ref().is_some_and(|value| Arc::ptr_eq(value, &inner)));
        reader.exit_reference_scope()?;
        let decoded = SharedRecord::decode_dc_mixed(&mut reader)?;
        assert!(decoded.definition.as_ref().is_some_and(|value| Arc::ptr_eq(value, &outer)));
        reader.exit_reference_scope()?;
        assert!(reader.resolve_shared::<Definition>("source").is_err());
        let mut next_document = DcMstring::new();
        next_document.push_char(DcChar::from_short(401));
        encode_shared_reference(&mut next_document, "definition", "source")?;
        next_document.push_char(DcChar::from_short(402));
        assert!(SharedRecord::decode_dc_mixed(&mut DcMixedReader::new(&next_document)).is_err());
        let mut identifier = DcMstring::new();
        encode_identifier(&mut identifier, "source.environment")?;
        let mut reader = DcMixedReader::new(&identifier);
        reader.expect_short_dc(270)?;
        assert_eq!(reader.next_char()?, DcChar::from_char('s'));
        assert_eq!(decode_identifier(&mut DcMixedReader::new(&identifier))?, "source.environment");
        assert!(encode_identifier(&mut DcMstring::new(), "").is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_shared_context_and_json() -> Result<()> {
        let original = Arc::new(42u64);
        let mut context = SharedValueContext::default();
        context.insert("source", Arc::clone(&original))?;
        let encoded = shared_to_json(&original, &context)?;
        assert_eq!(encoded.get("$dc_shared").and_then(serde_json::Value::as_str), Some("source"));
        let decoded = shared_from_json::<u64>(encoded.clone(), &context)?;
        assert!(Arc::ptr_eq(&original, &decoded));
        assert!(shared_from_json::<u64>(encoded, &SharedValueContext::default()).is_err());
        assert!(context.resolve::<String>("source").is_err());
        assert!(context.insert("source", Arc::new(9u64)).is_err());
        let distinct = Arc::new(43u64);
        let encoded = shared_to_json(&distinct, &context)?;
        assert_eq!(*shared_from_json::<u64>(encoded, &context)?, 43);
        assert_eq!(context.binding_for(&Arc::new(42u64)), Some("source"));
        Ok(())
    }
}