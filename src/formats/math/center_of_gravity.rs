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

//! Center of gravity (CG) and moment calculations for aircraft weight and
//! balance determination.
//!
//! Provides functions and data structures to compute moments, resolve pilot
//! station arms (directly or relative to a reference datum), and calculate the
//! loaded center of gravity across empty weight, occupants, and ballast.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

/// Inputs for aircraft center of gravity and moment calculations.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CenterOfGravityInput {
    /// Empty weight of the aircraft.
    pub empty_weight: f64,
    /// Empty center of gravity / arm of the aircraft.
    pub empty_cg: f64,
    /// Reference datum position, if calculating pilot arms relative to datum.
    pub datum: Option<f64>,
    /// Weight of the front pilot / occupant.
    pub front_pilot_weight: f64,
    /// Explicit arm for the front pilot (used when datum is not provided).
    pub front_pilot_arm: Option<f64>,
    /// Distance of front pilot in front of datum (used when datum is provided).
    pub front_pilot_distance_in_front_of_datum: Option<f64>,
    /// Weight of the rear pilot / occupant.
    pub rear_pilot_weight: f64,
    /// Explicit arm for the rear pilot (used when datum is not provided).
    pub rear_pilot_arm: Option<f64>,
    /// Distance of rear pilot behind datum (used when datum is provided).
    pub rear_pilot_distance_behind_datum: Option<f64>,
    /// Ballast weight.
    pub ballast_weight: f64,
    /// Ballast arm.
    pub ballast_arm: f64,
}

/// Results of the center of gravity and moment evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct CenterOfGravityResult {
    /// Empty moment (empty weight $\times$ empty CG).
    pub empty_moment: f64,
    /// Front pilot station arm.
    pub front_pilot_arm: f64,
    /// Front pilot moment (front pilot weight $\times$ front pilot arm).
    pub front_pilot_moment: f64,
    /// Rear pilot station arm.
    pub rear_pilot_arm: f64,
    /// Rear pilot moment (rear pilot weight $\times$ rear pilot arm).
    pub rear_pilot_moment: f64,
    /// Ballast moment (ballast weight $\times$ ballast arm).
    pub ballast_moment: f64,
    /// Total combined moment of all stations.
    pub total_moment: f64,
    /// Total combined weight of all stations.
    pub total_weight: f64,
    /// Loaded center of gravity position (total moment $/$ total weight).
    pub loaded_cg: f64,
}

/// Calculates the moment of a station given its weight and arm ($weight \times arm$).
pub fn calculate_moment(weight: f64, arm: f64) -> f64 {
    weight * arm
}

/// Calculates front pilot arm given datum and distance in front of datum
/// ($datum - distance$).
pub fn front_pilot_arm_from_datum(datum: f64, distance_in_front: f64) -> f64 {
    datum - distance_in_front
}

/// Calculates rear pilot arm given datum and distance behind datum
/// ($datum + distance$).
pub fn rear_pilot_arm_from_datum(datum: f64, distance_behind: f64) -> f64 {
    datum + distance_behind
}

/// Calculates the loaded center of gravity from total moment and total weight.
/// Returns an error if total weight is zero or non-finite.
pub fn calculate_loaded_cg(total_moment: f64, total_weight: f64) -> Result<f64> {
    if total_weight == 0.0 {
        bail!("Total weight cannot be zero");
    }
    let cg = total_moment / total_weight;
    if !cg.is_finite() {
        bail!("Calculated center of gravity is not finite: {cg}");
    }
    Ok(cg)
}

/// Evaluates aircraft weight, moments, and loaded center of gravity from inputs.
pub fn calculate_center_of_gravity(
    input: &CenterOfGravityInput,
) -> Result<CenterOfGravityResult> {
    let (front_pilot_arm, rear_pilot_arm) = if let Some(datum) = input.datum {
        let (Some(front_dist), Some(rear_dist)) = (
            input.front_pilot_distance_in_front_of_datum,
            input.rear_pilot_distance_behind_datum,
        ) else {
            bail!(
                "If providing Datum, must provide Pilot Distances Relative to Datum"
            );
        };
        (
            front_pilot_arm_from_datum(datum, front_dist),
            rear_pilot_arm_from_datum(datum, rear_dist),
        )
    } else {
        let (Some(front_arm), Some(rear_arm)) =
            (input.front_pilot_arm, input.rear_pilot_arm)
        else {
            bail!("If not providing Pilot Arms, must provide Datum");
        };
        (front_arm, rear_arm)
    };

    let empty_moment = calculate_moment(input.empty_weight, input.empty_cg);
    let front_pilot_moment =
        calculate_moment(input.front_pilot_weight, front_pilot_arm);
    let rear_pilot_moment =
        calculate_moment(input.rear_pilot_weight, rear_pilot_arm);
    let ballast_moment =
        calculate_moment(input.ballast_weight, input.ballast_arm);

    let total_moment = empty_moment
        + front_pilot_moment
        + rear_pilot_moment
        + ballast_moment;
    let total_weight = input.empty_weight
        + input.front_pilot_weight
        + input.rear_pilot_weight
        + input.ballast_weight;

    let loaded_cg = calculate_loaded_cg(total_moment, total_weight)?;

    Ok(CenterOfGravityResult {
        empty_moment,
        front_pilot_arm,
        front_pilot_moment,
        rear_pilot_arm,
        rear_pilot_moment,
        ballast_moment,
        total_moment,
        total_weight,
        loaded_cg,
    })
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

    #[crate::ctb_test]
    fn test_calculate_moment() {
        assert_eq!(calculate_moment(100.0, 50.0), 5000.0);
        assert_eq!(calculate_moment(0.0, 50.0), 0.0);
    }

    #[crate::ctb_test]
    fn test_arms_from_datum() {
        assert_eq!(front_pilot_arm_from_datum(100.0, 20.0), 80.0);
        assert_eq!(rear_pilot_arm_from_datum(100.0, 30.0), 130.0);
    }

    #[crate::ctb_test]
    fn test_calculate_loaded_cg() {
        assert_eq!(calculate_loaded_cg(10000.0, 100.0).unwrap(), 100.0);
        assert!(calculate_loaded_cg(10000.0, 0.0).is_err());
    }

    #[crate::ctb_test]
    fn test_cg_with_explicit_arms() {
        let input = CenterOfGravityInput {
            empty_weight: 1000.0,
            empty_cg: 80.0,
            datum: None,
            front_pilot_weight: 180.0,
            front_pilot_arm: Some(70.0),
            front_pilot_distance_in_front_of_datum: None,
            rear_pilot_weight: 170.0,
            rear_pilot_arm: Some(110.0),
            rear_pilot_distance_behind_datum: None,
            ballast_weight: 50.0,
            ballast_arm: 40.0,
        };

        let result = calculate_center_of_gravity(&input).unwrap();
        assert_eq!(result.empty_moment, 80000.0);
        assert_eq!(result.front_pilot_arm, 70.0);
        assert_eq!(result.front_pilot_moment, 12600.0);
        assert_eq!(result.rear_pilot_arm, 110.0);
        assert_eq!(result.rear_pilot_moment, 18700.0);
        assert_eq!(result.ballast_moment, 2000.0);
        assert_eq!(result.total_moment, 113300.0);
        assert_eq!(result.total_weight, 1400.0);
        let expected_cg = 113300.0 / 1400.0;
        assert!((result.loaded_cg - expected_cg).abs() < 1e-9);
    }

    #[crate::ctb_test]
    fn test_cg_with_datum() {
        let input = CenterOfGravityInput {
            empty_weight: 1000.0,
            empty_cg: 80.0,
            datum: Some(90.0),
            front_pilot_weight: 180.0,
            front_pilot_arm: None,
            front_pilot_distance_in_front_of_datum: Some(20.0),
            rear_pilot_weight: 170.0,
            rear_pilot_arm: None,
            rear_pilot_distance_behind_datum: Some(20.0),
            ballast_weight: 50.0,
            ballast_arm: 40.0,
        };

        let result = calculate_center_of_gravity(&input).unwrap();
        assert_eq!(result.front_pilot_arm, 70.0);
        assert_eq!(result.rear_pilot_arm, 110.0);
        assert_eq!(result.total_moment, 113300.0);
        assert_eq!(result.total_weight, 1400.0);
    }

    #[crate::ctb_test]
    fn test_cg_validation_errors() {
        // Missing datum when arms not provided
        let input_no_arms = CenterOfGravityInput {
            empty_weight: 1000.0,
            empty_cg: 80.0,
            datum: None,
            front_pilot_weight: 180.0,
            front_pilot_arm: None,
            front_pilot_distance_in_front_of_datum: None,
            rear_pilot_weight: 170.0,
            rear_pilot_arm: None,
            rear_pilot_distance_behind_datum: None,
            ballast_weight: 50.0,
            ballast_arm: 40.0,
        };
        assert!(calculate_center_of_gravity(&input_no_arms).is_err());

        // Datum provided but missing relative distances
        let input_missing_rel = CenterOfGravityInput {
            empty_weight: 1000.0,
            empty_cg: 80.0,
            datum: Some(100.0),
            front_pilot_weight: 180.0,
            front_pilot_arm: None,
            front_pilot_distance_in_front_of_datum: Some(10.0),
            rear_pilot_weight: 170.0,
            rear_pilot_arm: None,
            rear_pilot_distance_behind_datum: None,
            ballast_weight: 50.0,
            ballast_arm: 40.0,
        };
        assert!(calculate_center_of_gravity(&input_missing_rel).is_err());

        // Zero total weight
        let input_zero_weight = CenterOfGravityInput {
            empty_weight: 0.0,
            empty_cg: 0.0,
            datum: None,
            front_pilot_weight: 0.0,
            front_pilot_arm: Some(10.0),
            front_pilot_distance_in_front_of_datum: None,
            rear_pilot_weight: 0.0,
            rear_pilot_arm: Some(20.0),
            rear_pilot_distance_behind_datum: None,
            ballast_weight: 0.0,
            ballast_arm: 0.0,
        };
        assert!(calculate_center_of_gravity(&input_zero_weight).is_err());
    }
}
