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

//! General utility modules and helper functions for the EITE runtime.

pub mod array;
pub mod ascii;
pub mod bitwise;
pub mod logic;
pub mod math;
pub mod pred;
pub mod string;

/// Representation of the “generic” StageL values referenced by the JS.
/// We only add the variants actually needed by the converted functions here.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Str(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }
    pub fn as_int(&self) -> Option<i32> {
        if let Value::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Value::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn is_generic_primitive(&self) -> bool {
        matches!(self, Value::Bool(_) | Value::Int(_) | Value::Str(_))
    }
}

pub fn ne_values(a: &Value, b: &Value) -> bool {
    !eq_values(a, b)
}

/// Comparison helpers (implEq / implGt / implLt) simplified.
/// Generic equality only covers primitive subset implemented in Value above.
/// (The original JS allowed direct === which included string/int/bool.)
pub fn eq_values(a: &Value, b: &Value) -> bool {
    a == b
}
pub fn gt_int(a: i32, b: i32) -> bool {
    a > b
}
pub fn lt_int(a: i32, b: i32) -> bool {
    a < b
}
