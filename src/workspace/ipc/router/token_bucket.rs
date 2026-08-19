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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::types::ConnectionId;
use tokio::time::Instant;

/// Key for rate limiting (connection, service, method).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(super) struct RateKey {
    conn_id: ConnectionId,
    service: String,
    method: String,
}

impl RateKey {
    pub(super) fn bytes(
        conn_id: ConnectionId,
        service: String,
        method: String,
    ) -> Self {
        Self {
            conn_id,
            service,
            method,
        }
    }
}

/// Simple token bucket for rate limiting.
#[derive(Debug)]
pub(super) struct TokenBucket {
    capacity: u128,
    tokens: u128,
    last_refill: Instant,
    rate_bytes_per_sec: u64,
}

impl TokenBucket {
    pub(super) fn new(rate: u64, capacity: u64, now: Instant) -> Self {
        Self {
            capacity: u128::from(capacity),
            tokens: u128::from(capacity),
            last_refill: now,
            rate_bytes_per_sec: rate,
        }
    }

    pub(super) fn try_take(&mut self, amount: u64, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last_refill);
        let elapsed_ns = elapsed.as_nanos();
        // Reason for fallback: token refill division calculation overflow adds 0 tokens to bucket capacity
        let added_tokens = elapsed_ns
            .saturating_mul(u128::from(self.rate_bytes_per_sec))
            .checked_div(1_000_000_000)
            .unwrap_or(0);
        self.tokens =
            self.tokens.saturating_add(added_tokens).min(self.capacity);
        self.last_refill = now;

        let amount_u128 = u128::from(amount);
        if self.tokens >= amount_u128 {
            self.tokens = self.tokens.saturating_sub(amount_u128);
            true
        } else {
            false
        }
    }
}
