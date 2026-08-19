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

//! JSON-RPC dispatcher for aircraft Center of Gravity and moment calculations.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use serde_json::Value;

/// Dispatches a Center of Gravity RPC method call.
pub async fn handle_center_of_gravity_call(
    func: &str,
    args: &[Value],
) -> anyhow::Result<Value> {
    use ctb_formats_math::center_of_gravity::*;

    match func {
        "calculateCenterOfGravity" => {
            let input_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: input"))?;

            // Reason for fallback: optional weight, arm, and CG inputs default to 0.0 when omitted from payload.
            let empty_weight = input_val
                .get("empty_weight")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            // Reason for fallback: optional weight, arm, and CG inputs default to 0.0 when omitted from payload.
            let empty_cg = input_val
                .get("empty_cg")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let datum = input_val.get("datum").and_then(Value::as_f64);
            // Reason for fallback: optional weight, arm, and CG inputs default to 0.0 when omitted from payload.
            let front_pilot_weight = input_val
                .get("front_pilot_weight")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let front_pilot_arm =
                input_val.get("front_pilot_arm").and_then(Value::as_f64);
            let front_pilot_distance_in_front_of_datum = input_val
                .get("front_pilot_distance_in_front_of_datum")
                .and_then(Value::as_f64);
            // Reason for fallback: optional weight, arm, and CG inputs default to 0.0 when omitted from payload.
            let rear_pilot_weight = input_val
                .get("rear_pilot_weight")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let rear_pilot_arm =
                input_val.get("rear_pilot_arm").and_then(Value::as_f64);
            let rear_pilot_distance_behind_datum = input_val
                .get("rear_pilot_distance_behind_datum")
                .and_then(Value::as_f64);
            // Reason for fallback: optional weight, arm, and CG inputs default to 0.0 when omitted from payload.
            let ballast_weight = input_val
                .get("ballast_weight")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            // Reason for fallback: optional weight, arm, and CG inputs default to 0.0 when omitted from payload.
            let ballast_arm = input_val
                .get("ballast_arm")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);

            let input = CenterOfGravityInput {
                empty_weight,
                empty_cg,
                datum,
                front_pilot_weight,
                front_pilot_arm,
                front_pilot_distance_in_front_of_datum,
                rear_pilot_weight,
                rear_pilot_arm,
                rear_pilot_distance_behind_datum,
                ballast_weight,
                ballast_arm,
            };

            let res = calculate_center_of_gravity(&input)?;
            Ok(serde_json::json!({
                "empty_moment": res.empty_moment,
                "front_pilot_arm": res.front_pilot_arm,
                "front_pilot_moment": res.front_pilot_moment,
                "rear_pilot_arm": res.rear_pilot_arm,
                "rear_pilot_moment": res.rear_pilot_moment,
                "ballast_moment": res.ballast_moment,
                "total_moment": res.total_moment,
                "total_weight": res.total_weight,
                "loaded_cg": res.loaded_cg
            }))
        }
        "calculateMoment" => {
            let weight = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: weight"))?;
            let arm = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: arm"))?;
            Ok(serde_json::to_value(calculate_moment(weight, arm))?)
        }
        "calculateLoadedCg" => {
            let total_moment = args
                .first()
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: total_moment"))?;
            let total_weight = args
                .get(1)
                .and_then(Value::as_f64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: total_weight"))?;
            Ok(serde_json::to_value(calculate_loaded_cg(
                total_moment,
                total_weight,
            )?)?)
        }
        _ => anyhow::bail!("Unknown center of gravity function: {func}"),
    }
}
