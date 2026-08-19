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

// ---------------
// Booleans (logic gates)
// ---------------

pub fn or(a: bool, b: bool) -> bool {
    a || b
}
pub fn nor(a: bool, b: bool) -> bool {
    !(a || b)
}
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}
pub fn xor(a: bool, b: bool) -> bool {
    (a || b) && !(a && b)
}
pub fn xnor(a: bool, b: bool) -> bool {
    !xor(a, b)
}
pub fn is_true(v: bool) -> bool {
    v
}
pub fn is_false(v: bool) -> bool {
    !v
}
