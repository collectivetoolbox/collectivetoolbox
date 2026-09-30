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

//! Primality testing and factor verification using Malachite.
//!
//! Provides deterministic and probabilistic primality testing for native
//! unsigned integers and arbitrary-precision `Natural` numbers.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use malachite::Natural;
use malachite::base::num::arithmetic::traits::{
    CheckedSub, DivExactAssign, DivisibleBy, ModPow, Parity,
};
use malachite::base::num::basic::traits::{One, Zero};
use malachite::base::num::factorization::traits::IsPrime as MalachiteIsPrime;

/// First 25 small prime numbers used for rapid trial division pre-filtering.
pub const SMALL_PRIMES: &[u32] = &[
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67,
    71, 73, 79, 83, 89, 97,
];

/// Deterministic prime testing bases for Miller-Rabin testing up to $2^{64}$.
pub const MILLER_RABIN_BASES: &[u32] = &[
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37,
];

/// Checks if a 64-bit integer is prime using Malachite's BPSW algorithm.
#[must_use]
pub fn is_prime_u64(n: u64) -> bool {
    n.is_prime()
}

/// Checks if an arbitrary-precision `Natural` number is prime using trial
/// division followed by Miller-Rabin probabilistic primality testing.
#[must_use]
pub fn is_prime_natural(n: &Natural) -> bool {
    let two = Natural::from(2_u32);
    if *n < two {
        return false;
    }
    if *n == two {
        return true;
    }
    if n.even() {
        return false;
    }

    // Trial division against small primes
    for prime in SMALL_PRIMES {
        let p_nat = Natural::from(*prime);
        if *n == p_nat {
            return true;
        }
        if n.divisible_by(&p_nat) {
            return false;
        }
    }

    // If n fits in u64, use fast deterministic check
    if let Ok(val) = u64::try_from(n) {
        return val.is_prime();
    }

    // Write n - 1 = 2^r * d with d odd
    let Some(n_minus_1) = n.checked_sub(&Natural::ONE) else {
        return false;
    };
    let mut d = n_minus_1.clone();
    let mut r: u64 = 0;
    while d.even() {
        d.div_exact_assign(&two);
        r = r.saturating_add(1);
    }

    // Miller-Rabin rounds using fixed prime bases
    for base in MILLER_RABIN_BASES {
        let a = Natural::from(*base);
        if a >= *n {
            break;
        }
        let mut x = (&a).mod_pow(&d, n);
        if x == Natural::ONE || x == n_minus_1 {
            continue;
        }

        let mut composite = true;
        let mut count: u64 = 1;
        while count < r {
            x = (&x).mod_pow(&two, n);
            if x == n_minus_1 {
                composite = false;
                break;
            }
            count = count.saturating_add(1);
        }

        if composite {
            return false;
        }
    }

    true
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

    #[crate::ctb_test]
    fn test_is_prime_natural_small_and_large() {
        assert!(!is_prime_natural(&Natural::from(0_u32)));
        assert!(!is_prime_natural(&Natural::from(1_u32)));
        assert!(is_prime_natural(&Natural::from(2_u32)));
        assert!(is_prime_natural(&Natural::from(3_u32)));
        assert!(!is_prime_natural(&Natural::from(4_u32)));
        assert!(is_prime_natural(&Natural::from(17_u32)));
        assert!(!is_prime_natural(&Natural::from(561_u32))); // Carmichael number

        // 32-bit FNV prime: 2^24 + 2^8 + 0x93 = 16777619
        assert!(is_prime_natural(&Natural::from(16_777_619_u32)));

        // 64-bit FNV prime: 2^40 + 2^8 + 0xb3 = 1099511628211
        assert!(is_prime_natural(&Natural::from(1_099_511_628_211_u64)));

        // 128-bit FNV prime: 2^88 + 2^8 + 0x3b
        let p128 = (Natural::ONE << 88_u32)
            + (Natural::ONE << 8_u32)
            + Natural::from(0x3b_u32);
        assert!(is_prime_natural(&p128));
    }
}
