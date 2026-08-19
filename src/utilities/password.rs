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

//! Password hashing and verification utilities using the Argon2 algorithm.

use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};
use zeroize::ZeroizeOnDrop;

pub const TEST_USER_PASS: &str =
    "test_password_523ed9db-1885-439d-83d5-61c869e5223d";
pub const TEST_USER_PHC: &str = "$argon2id$v=19$m=19456,t=2,p=1$RXiQwP6rcmLh98qumvyu0g$Pc9dem17Dgwz6uoiCuceMh+VYQxZ8WqSK36gfpAZYXY";

#[derive(ZeroizeOnDrop, PartialEq, Clone)]
pub struct Password {
    pub password: Vec<u8>,
}

impl Password {
    pub fn as_string_not_zeroizing(&self) -> String {
        self.password.iter().map(|&b| char::from(b)).collect()
    }

    pub fn from_string(s: &str) -> Self {
        Password {
            password: s.as_bytes().to_vec(),
        }
    }
}

pub fn hash(password: &Password) -> Result<String> {
    // If in debug or test and password is TEST_USER_PASS use hardcoded string
    #[cfg(any(debug_assertions, test))]
    {
        if password.password == TEST_USER_PASS.as_bytes() {
            return Ok(TEST_USER_PHC.to_string());
        }
    }
    let salt = SaltString::generate(&mut OsRng);
    let phc = Argon2::default()
        .hash_password(&password.password, &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {e}"))?;
    Ok(phc.to_string())
}

pub fn verify(password: &Password, hash: &str) -> Result<bool> {
    // If in debug or test and password is TEST_USER_PASS accept
    #[cfg(any(debug_assertions, test))]
    {
        if password.password == TEST_USER_PASS.as_bytes() {
            return Ok(true);
        }
    }
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(&password.password, &parsed_hash)
        .is_ok())
}
