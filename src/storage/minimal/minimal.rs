#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;
pub mod xkb;

static LICENSE_FILE: &[u8] = include_bytes!("../../../LICENSE.md");

pub fn get_license_text() -> String {
    String::from_utf8_lossy(LICENSE_FILE).into_owned()
}

pub fn get_license_boilerplate_line1() -> String {
    "This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.".to_string()
}

pub fn get_license_boilerplate_line2() -> String {
    "This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.".to_string()
}

pub fn get_license_boilerplate_line3() -> String {
    "You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.".to_string()
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
mod tests {}
