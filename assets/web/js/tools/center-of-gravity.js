// @license magnet:?xt=urn:btih:0b31508aeb0634b347b8270c7bee4d411b5d4109&dn=agpl-3.0.txt AGPL-3.0

import { rpcCall } from "./rpc.js";

/**
 * Executes a call to the Center of Gravity JSON-RPC API.
 *
 * @param {string} funcName The function to call on the backend.
 * @param {any[]} [args=[]] The function arguments.
 * @returns {Promise<any>} The result.
 */
async function cgCall(funcName, args = []) {
    return await rpcCall("/api/rpc/center_of_gravity", funcName, args);
}

/**
 * Initializes the Center of Gravity calculator event handlers.
 */
export function initCenterOfGravity() {
    const btnCalculate = document.getElementById("Calculate");
    const btnReset = document.getElementById("Reset");

    btnCalculate?.addEventListener("click", (e) => {
        e.preventDefault();
        calculateCg();
    });

    btnReset?.addEventListener("click", (e) => {
        e.preventDefault();
        resetFields();
    });
}

/**
 * Reads form input values, sends them to the backend RPC endpoint, and populates calculated outputs.
 */
async function calculateCg() {
    const emptyWt = parseFloat(getInputValue("EmptyWt")) || 0;
    const emptyCg = parseFloat(getInputValue("EmptyCg")) || 0;
    const datumVal = getInputValue("Datum");
    const datum = datumVal !== "" ? parseFloat(datumVal) : null;

    const frontPilotWt = parseFloat(getInputValue("FrontPilotWt")) || 0;
    const frontPilotArmVal = getInputValue("FrontPilotArm");
    const frontPilotArm = frontPilotArmVal !== "" ? parseFloat(frontPilotArmVal) : null;
    const frontDistVal = getInputValue("FrontPilotDistanceInFrontOfDatum");
    const frontDist = frontDistVal !== "" ? parseFloat(frontDistVal) : null;

    const rearPilotWt = parseFloat(getInputValue("RearPilotWt")) || 0;
    const rearPilotArmVal = getInputValue("RearPilotArm");
    const rearPilotArm = rearPilotArmVal !== "" ? parseFloat(rearPilotArmVal) : null;
    const rearDistVal = getInputValue("RearPilotDistanceBehindDatum");
    const rearDist = rearDistVal !== "" ? parseFloat(rearDistVal) : null;

    const ballastWt = parseFloat(getInputValue("BallastWt")) || 0;
    const ballastArm = parseFloat(getInputValue("BallastArm")) || 0;

    const inputPayload = {
        empty_weight: emptyWt,
        empty_cg: emptyCg,
        datum: datum,
        front_pilot_weight: frontPilotWt,
        front_pilot_arm: frontPilotArm,
        front_pilot_distance_in_front_of_datum: frontDist,
        rear_pilot_weight: rearPilotWt,
        rear_pilot_arm: rearPilotArm,
        rear_pilot_distance_behind_datum: rearDist,
        ballast_weight: ballastWt,
        ballast_arm: ballastArm
    };

    try {
        const result = await cgCall("calculateCenterOfGravity", [inputPayload]);

        setInputValue("EmptyMoment", result.empty_moment);
        if (result.front_pilot_arm !== undefined && frontPilotArmVal === "") {
            setInputValue("FrontPilotArm", result.front_pilot_arm);
        }
        setInputValue("FrontPilotMoment", result.front_pilot_moment);

        if (result.rear_pilot_arm !== undefined && rearPilotArmVal === "") {
            setInputValue("RearPilotArm", result.rear_pilot_arm);
        }
        setInputValue("RearPilotMoment", result.rear_pilot_moment);

        setInputValue("BallastMoment", result.ballast_moment);
        setInputValue("TotalMoment", result.total_moment);
        setInputValue("TotalWt", result.total_weight);
        setInputValue("LoadedCg", result.loaded_cg);
    } catch (err) {
        alert(`Calculation error: ${/** @type {Error} */ (err).message}`);
    }
}

/**
 * Resets all input and calculated fields.
 */
function resetFields() {
    const ids = [
        "EmptyWt", "EmptyCg", "FrontPilotWt", "FrontPilotArm",
        "FrontPilotDistanceInFrontOfDatum", "RearPilotWt", "RearPilotArm",
        "RearPilotDistanceBehindDatum", "Datum", "BallastWt", "BallastArm",
        "EmptyMoment", "FrontPilotMoment", "RearPilotMoment", "BallastMoment",
        "TotalMoment", "TotalWt", "LoadedCg"
    ];
    for (const id of ids) {
        const el = /** @type {HTMLInputElement | null} */ (document.getElementById(id));
        if (el) {
            el.value = "";
        }
    }
}

/**
 * Gets value of an input element by ID.
 *
 * @param {string} id
 * @returns {string}
 */
function getInputValue(id) {
    const el = /** @type {HTMLInputElement | null} */ (document.getElementById(id));
    return el ? el.value.trim() : "";
}

/**
 * Sets value of an input element by ID.
 *
 * @param {string} id
 * @param {number | string} val
 */
function setInputValue(id, val) {
    const el = /** @type {HTMLInputElement | null} */ (document.getElementById(id));
    if (el) {
        el.value = typeof val === "number" ? (Number.isInteger(val) ? String(val) : val.toFixed(4)) : String(val);
    }
}

// Auto-initialize when loaded
if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initCenterOfGravity);
} else {
    initCenterOfGravity();
}
