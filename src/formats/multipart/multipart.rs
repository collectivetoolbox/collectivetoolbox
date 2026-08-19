// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! Multipart form utilities. The meat of this is the `build_multipart`
//! function. The serialization code was needed to make it be able to tell a
//! `Vec<u8>` apart from other arrays. `Vec<u8>` are specifically assumed to be
//! file uploads.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, anyhow};
use reqwest::blocking::multipart::Part;

use crate::utilities::reader::reader_to_bytes;
use crate::utilities::serde_value::to_html_form_string;
use serde::Serialize;

/// Internal captured representation of a struct field for multipart building.
enum CapturedField {
    Bytes(Vec<u8>),
    Array(Vec<serde_json::Value>),
    Scalar(serde_json::Value),
}

/// Internal serializer error used to implement `serde::ser::Error` without panicking.
#[derive(Debug)]
struct CaptureError(anyhow::Error);

impl CaptureError {
    pub fn custom(msg: impl std::fmt::Display) -> Self {
        CaptureError(anyhow!(msg.to_string()))
    }
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CaptureError {}

impl serde::ser::Error for CaptureError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        CaptureError(anyhow::anyhow!(msg.to_string()))
    }
}

use serde::ser::{SerializeSeq, SerializeStruct, Serializer};

/// Serializer that captures a struct into a key -> `CapturedField` map.
struct TopSerializer<'a> {
    out: &'a mut std::collections::BTreeMap<String, CapturedField>,
}

impl<'a> Serializer for TopSerializer<'a> {
    type Ok = ();
    type Error = CaptureError;
    type SerializeSeq = serde::ser::Impossible<(), CaptureError>; // Changed to avoid type mismatch since sequences are not supported at top-level
    type SerializeTuple = serde::ser::Impossible<(), CaptureError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), CaptureError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), CaptureError>;
    type SerializeMap = serde::ser::Impossible<(), CaptureError>;
    type SerializeStruct = StructCapture<'a>;
    type SerializeStructVariant = serde::ser::Impossible<(), CaptureError>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(StructCapture { out: self.out })
    }

    // All other entry points at top-level are unsupported; we expect a struct.
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_some<T: ?Sized + Serialize>(
        self,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(CaptureError::custom("expected struct at top-level"))
    }
}

struct StructCapture<'a> {
    out: &'a mut std::collections::BTreeMap<String, CapturedField>,
}

impl SerializeStruct for StructCapture<'_> {
    type Ok = ();
    type Error = CaptureError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let captured = FieldSerializer::capture(value)?;
        self.out.insert(key.to_string(), captured);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Serializer that captures a single field into a `CapturedField`.
struct FieldSerializer;

impl FieldSerializer {
    fn capture<T: ?Sized + Serialize>(
        value: &T,
    ) -> Result<CapturedField, CaptureError> {
        value.serialize(Self)
    }
}

impl Serializer for FieldSerializer {
    type Ok = CapturedField;
    type Error = CaptureError;
    type SerializeSeq = SeqCapture;
    type SerializeTuple = serde::ser::Impossible<CapturedField, CaptureError>;
    type SerializeTupleStruct =
        serde::ser::Impossible<CapturedField, CaptureError>;
    type SerializeTupleVariant =
        serde::ser::Impossible<CapturedField, CaptureError>;
    type SerializeMap = serde::ser::Impossible<CapturedField, CaptureError>;
    type SerializeStruct = serde::ser::Impossible<CapturedField, CaptureError>;
    type SerializeStructVariant =
        serde::ser::Impossible<CapturedField, CaptureError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Bool(v)))
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Number(v.into())))
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let n = serde_json::Number::from_f64(f64::from(v))
            .ok_or_else(|| CaptureError::custom("invalid f32"))?;
        Ok(CapturedField::Scalar(serde_json::Value::Number(n)))
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let n = serde_json::Number::from_f64(v)
            .ok_or_else(|| CaptureError::custom("invalid f64"))?;
        Ok(CapturedField::Scalar(serde_json::Value::Number(n)))
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::String(
            v.to_string(),
        )))
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::String(
            v.to_string(),
        )))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Bytes(v.to_vec()))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Null))
    }
    fn serialize_some<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Represent Option<T>::Some as the inner value
        FieldSerializer::capture(value)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Null))
    }
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::Null))
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CapturedField::Scalar(serde_json::Value::String(
            variant.to_string(),
        )))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        FieldSerializer::capture(value)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Represent as a simple map { variant: value } as a scalar JSON object
        let mut map = serde_json::Map::new();
        map.insert(
            variant.to_string(),
            serde_json::to_value(value)
                .map_err(|e| CaptureError(anyhow!(e)))?,
        );
        Ok(CapturedField::Scalar(serde_json::Value::Object(map)))
    }
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqCapture { elems: Vec::new() })
    }
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error> {
        Err(CaptureError::custom("tuple not supported"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(CaptureError::custom("tuple struct not supported"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(CaptureError::custom("tuple variant not supported"))
    }
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        // Fallback: capture as scalar JSON object
        Err(CaptureError::custom(
            "map capture at field level not supported",
        ))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(CaptureError::custom("nested struct capture not supported"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(CaptureError::custom("struct variant not supported"))
    }
}

enum SeqElem {
    U8(u8),
    Other(serde_json::Value),
}

struct SeqCapture {
    elems: Vec<SeqElem>,
}

impl SerializeSeq for SeqCapture {
    type Ok = CapturedField;
    type Error = CaptureError;

    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        // Capture each element, distinguishing u8 from everything else.
        value.serialize(ElemCapture {
            elems: &mut self.elems,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // If all elements are U8, treat the whole sequence as bytes; else as a
        // JSON array.
        //
        // Important: for an empty sequence, we *cannot* infer the element type.
        // Treating it as bytes would cause unrelated empty Vecs (e.g.
        // `Vec<u64>`) to serialize as an empty multipart field, which strict
        // typed-multipart parsing will (correctly) reject.
        if self.elems.is_empty() {
            return Ok(CapturedField::Array(Vec::new()));
        }

        if self.elems.iter().all(|e| matches!(e, SeqElem::U8(_))) {
            let bytes = self
                .elems
                .into_iter()
                .map(|e| match e {
                    SeqElem::U8(b) => b,
                    SeqElem::Other(_v) => 0,
                })
                .collect::<Vec<u8>>();
            Ok(CapturedField::Bytes(bytes))
        } else {
            let arr = self
                .elems
                .into_iter()
                .map(|e| match e {
                    SeqElem::U8(b) => serde_json::Value::Number(b.into()),
                    SeqElem::Other(v) => v,
                })
                .collect::<Vec<_>>();
            Ok(CapturedField::Array(arr))
        }
    }
}

struct ElemCapture<'a> {
    elems: &'a mut Vec<SeqElem>,
}

impl Serializer for ElemCapture<'_> {
    type Ok = ();
    type Error = CaptureError;
    type SerializeSeq = serde::ser::Impossible<(), CaptureError>;
    type SerializeTuple = serde::ser::Impossible<(), CaptureError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), CaptureError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), CaptureError>;
    type SerializeMap = serde::ser::Impossible<(), CaptureError>;
    type SerializeStruct = serde::ser::Impossible<(), CaptureError>;
    type SerializeStructVariant = serde::ser::Impossible<(), CaptureError>;

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.elems.push(SeqElem::U8(v));
        Ok(())
    }

    // For all other primitives, push as a JSON value for later to_html_form_string conversion.
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.elems.push(SeqElem::Other(serde_json::Value::Bool(v)));
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(v.into())));
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(v.into())));
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(v.into())));
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(v.into())));
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(v.into())));
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(v.into())));
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(v.into())));
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let n = serde_json::Number::from_f64(f64::from(v))
            .ok_or_else(|| CaptureError::custom("invalid f32"))?;
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(n)));
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let n = serde_json::Number::from_f64(v)
            .ok_or_else(|| CaptureError::custom("invalid f64"))?;
        self.elems
            .push(SeqElem::Other(serde_json::Value::Number(n)));
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::String(v.to_string())));
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.elems
            .push(SeqElem::Other(serde_json::Value::String(v.to_string())));
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        // Represent as JSON array of numbers when used as element; bytes here is uncommon.
        let arr = v
            .iter()
            .copied()
            .map(|b| serde_json::Value::Number(b.into()))
            .collect();
        self.elems
            .push(SeqElem::Other(serde_json::Value::Array(arr)));
        Ok(())
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.elems.push(SeqElem::Other(serde_json::Value::Null));
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Represent as the inner value
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.elems.push(SeqElem::Other(serde_json::Value::Null));
        Ok(())
    }
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.elems.push(SeqElem::Other(serde_json::Value::Null));
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.elems.push(SeqElem::Other(serde_json::Value::String(
            variant.to_string(),
        )));
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Serialize inner as JSON value
        let v = serde_json::to_value(value)
            .map_err(|e| CaptureError(anyhow!(e)))?;
        self.elems.push(SeqElem::Other(v));
        Ok(())
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = serde_json::Map::new();
        map.insert(
            variant.to_string(),
            serde_json::to_value(value)
                .map_err(|e| CaptureError(anyhow!(e)))?,
        );
        self.elems
            .push(SeqElem::Other(serde_json::Value::Object(map)));
        Ok(())
    }
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        // Nested sequences as elements are uncommon; represent as error to keep implementation small.
        Err(CaptureError::custom("nested sequences not supported"))
    }
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple, Self::Error> {
        Err(CaptureError::custom("tuple elements not supported"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(CaptureError::custom("tuple struct elements not supported"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(CaptureError::custom("tuple variant elements not supported"))
    }
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        Err(CaptureError::custom("map elements not supported"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(CaptureError::custom("struct elements not supported"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(CaptureError::custom(
            "struct variant elements not supported",
        ))
    }
}

pub fn build_multipart<T: Serialize>(data: &T) -> Result<(Vec<u8>, String)> {
    // Build multipart body
    let mut multipart = reqwest::blocking::multipart::Form::new();

    // Capture fields with type-aware Vec<u8> detection.
    let mut fields = std::collections::BTreeMap::<String, CapturedField>::new();
    data.serialize(TopSerializer { out: &mut fields })
        .map_err(|e| anyhow!(e.to_string()))?;

    for (k, captured) in fields {
        match captured {
            CapturedField::Bytes(bytes) => {
                let bytes_part = Part::bytes(bytes);
                multipart = multipart.part(k.clone(), bytes_part);
            }
            CapturedField::Array(arr) => {
                for item in arr {
                    let val = to_html_form_string(&item)?;
                    if let Some(v_str) = val {
                        multipart = multipart.text(k.clone(), v_str);
                    }
                    // Skip None values
                }
            }
            CapturedField::Scalar(v) => {
                // Convert serde_json::Value to a string representation suitable for HTML forms
                let val = to_html_form_string(&v)?;
                if val.is_none() {
                    // Skip None values (e.g., unchecked checkboxes)
                    continue;
                }
                if let Some(vs) = val {
                    multipart = multipart.text(k, vs);
                } else {
                    return Err(anyhow!(
                        "Unsupported value type for field {k}: {v:?}"
                    ));
                }
            }
        }
    }

    let boundary = multipart.boundary().to_owned();
    let reader = multipart.into_reader();
    let buffer = reader_to_bytes(reader)?;
    let content_type = format!("multipart/form-data; boundary={boundary}");
    Ok((buffer, content_type))
}

#[cfg(test)]
#[expect(
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
    use serde::Serialize;

    #[crate::ctb_test]
    fn test_build_multipart() {
        #[derive(Serialize)]
        struct TestData {
            bytes_array: Vec<u8>,
            admin_users: Vec<u64>,
            fixed_port: u16,
            server_address: String,
        }

        let data = TestData {
            bytes_array: vec![1, 2, 3],
            admin_users: vec![12, 13],
            fixed_port: 5678,
            server_address: "bar".to_string(),
        };

        let result = build_multipart(&data);
        assert!(result.is_ok());
        let (buffer, content_type) = result.unwrap();
        assert!(content_type.starts_with("multipart/form-data; boundary="));
        // The buffer should contain the fields; exact boundary varies, so check for presence
        let buffer_str = String::from_utf8_lossy(&buffer);
        assert!(
            buffer_str.contains("\"bytes_array\"\r\n\r\n\x01\x02\x03"),
            "Buffer not contains expected: {buffer_str}"
        );
        assert!(
            buffer_str.contains("\"admin_users\"\r\n\r\n12"),
            "Buffer not contains expected: {buffer_str}"
        );
        assert!(
            buffer_str.contains("\"admin_users\"\r\n\r\n13"),
            "Buffer not contains expected: {buffer_str}"
        );
        assert!(
            buffer_str.contains("\"fixed_port\"\r\n\r\n5678"),
            "Buffer not contains expected: {buffer_str}"
        );
        assert!(
            buffer_str.contains("\"server_address\"\r\n\r\nbar"),
            "Buffer not contains expected: {buffer_str}"
        );
    }

    #[crate::ctb_test]
    fn test_build_multipart_omits_empty_vec_fields() {
        #[derive(Serialize)]
        struct TestData {
            admin_users: Vec<u64>,
            fixed_port: u16,
        }

        let data = TestData {
            admin_users: Vec::new(),
            fixed_port: 5678,
        };

        let (buffer, _content_type) = build_multipart(&data).unwrap();
        let buffer_str = String::from_utf8_lossy(&buffer);

        // Empty Vecs should serialize like browsers typically do for checkbox
        // groups: the field is absent.
        assert!(!buffer_str.contains("\"admin_users\""), "{buffer_str}");
        assert!(buffer_str.contains("\"fixed_port\""), "{buffer_str}");
    }
}
