// @license magnet:?xt=urn:btih:0b31508aeb0634b347b8270c7bee4d411b5d4109&dn=agpl-3.0.txt AGPL-3.0

import { rpcCall } from "./rpc.js";
import { openModal } from "../modals.js";

/**
 * Executes a call to the Calculator JSON-RPC API.
 *
 * @param {string} funcName The function to call on the backend.
 * @param {any[]} [args=[]] The function arguments.
 * @returns {Promise<any>} The result.
 */
async function calcCall(funcName, args = []) {
    return await rpcCall("/api/rpc/calculator", funcName, args);
}

// State variables
let isLocked = false;
let currentOp = "+";
let rpsScores = { wins: 0, draws: 0, losses: 0 };
let sixR2Iterations = 0;

/**
 * Initializes the Calculator tool event handlers and UI state.
 */
export function initCalculator() {
    setupMenubar();
    setupMakeTab();
    setupPrimeTab();
    setupRandomTab();
    setupSqrtTab();
    setupTemperatureTab();
    setupPerimeterTab();
    setupConstantsTab();
    setupAreaTab();
    setupRpsModal();
    setupSixR2Modal();
}

/**
 * Sets up Menubar action bindings.
 */
function setupMenubar() {
    document.getElementById("menuFileNew")?.addEventListener("click", () => {
        if (isLocked) {
            alert("Session is locked.");
            return;
        }
        clearAllInputs();
    });

    document.getElementById("menuFileClearAll")?.addEventListener("click", () => {
        if (isLocked) {
            alert("Session is locked.");
            return;
        }
        clearAllInputs();
    });

    document.getElementById("menuFileLock")?.addEventListener("click", () => {
        const lockChk = /** @type {HTMLInputElement | null} */ (document.getElementById("chkLock"));
        if (lockChk) {
            lockChk.checked = !lockChk.checked;
            isLocked = lockChk.checked;
        }
    });

    document.getElementById("menuFileQuit")?.addEventListener("click", () => {
        if (isLocked) {
            alert("Session is locked.");
            return;
        }
        window.location.href = "/";
    });

    document.getElementById("menuSidecarRps")?.addEventListener("click", () => {
        const modalEl = document.getElementById("diaRps");
        if (modalEl) {
            openModal(modalEl);
        }
    });

    document.getElementById("menuSidecar6r2")?.addEventListener("click", () => {
        const modalEl = document.getElementById("diaSixR2");
        if (modalEl) {
            openModal(modalEl);
        }
    });

    document.getElementById("menuHelpAssistance")?.addEventListener("click", () => {
        switchTab("tab-assistance");
    });
}

/**
 * Switches active tab in the main tab group.
 *
 * @param {string} tabId The target tab ID.
 */
function switchTab(tabId) {
    const tabGroup = document.getElementById("tctCalc");
    if (!tabGroup) return;

    const btn = tabGroup.querySelector(`button[data-tab="${tabId}"]`);
    if (btn instanceof HTMLElement) {
        btn.click();
    }
}

/**
 * Sets up Tab 1: Make.
 */
function setupMakeTab() {
    const txtN1 = /** @type {HTMLInputElement | null} */ (document.getElementById("txtN1"));
    const txtN2 = /** @type {HTMLInputElement | null} */ (document.getElementById("txtN2"));
    const txtFormula = /** @type {HTMLInputElement | null} */ (document.getElementById("txtFormula"));
    const lblAnswer = document.getElementById("lblYourAnswer");
    const btnEvaluate = document.getElementById("btnEvaluate");
    const btnClearAll = document.getElementById("btnClearAll");
    const btnQuit = document.getElementById("btnQuit");
    const chkLock = /** @type {HTMLInputElement | null} */ (document.getElementById("chkLock"));

    chkLock?.addEventListener("change", () => {
        isLocked = chkLock.checked;
    });

    btnQuit?.addEventListener("click", () => {
        if (isLocked) {
            alert("Session is locked.");
            return;
        }
        window.location.href = "/";
    });

    // Operator buttons (auto-evaluates via RPC on click)
    const opButtons = document.querySelectorAll("[data-op]");
    for (const btn of opButtons) {
        btn.addEventListener("click", async () => {
            const op = btn.getAttribute("data-op");
            if (op === "AC") {
                if (txtN1) txtN1.value = "";
                if (txtN2) txtN2.value = "";
                if (txtFormula) txtFormula.value = "";
                if (lblAnswer) lblAnswer.textContent = "";
                return;
            }

            if (!op) return;
            currentOp = op;

            const n1Str = txtN1?.value.trim() || "";
            const n2Str = txtN2?.value.trim() || "";

            if (n1Str !== "" && n2Str !== "") {
                const n1 = parseFloat(n1Str);
                const n2 = parseFloat(n2Str);
                if (isNaN(n1) || isNaN(n2)) {
                    if (lblAnswer) lblAnswer.textContent = "Error: Invalid Number";
                    return;
                }
                try {
                    const result = await calcCall("evaluateBasicOp", [op, n1, n2]);
                    if (lblAnswer) {
                        lblAnswer.textContent = String(result);
                    }
                } catch (err) {
                    if (lblAnswer) {
                        lblAnswer.textContent = `Error: ${/** @type {Error} */ (err).message}`;
                    }
                }
            }
        });
    }

    // Evaluate button (evaluates formula via RPC)
    btnEvaluate?.addEventListener("click", async () => {
        const formula = txtFormula?.value.trim() || "";
        if (formula !== "") {
            try {
                const result = await calcCall("evaluateExpression", [formula]);
                if (lblAnswer) {
                    lblAnswer.textContent = String(result);
                }
            } catch (err) {
                if (lblAnswer) {
                    lblAnswer.textContent = `Error: ${/** @type {Error} */ (err).message}`;
                }
            }
            return;
        }

        const n1Str = txtN1?.value.trim() || "0";
        const n2Str = txtN2?.value.trim() || "0";
        const n1 = parseFloat(n1Str);
        const n2 = parseFloat(n2Str);

        if (isNaN(n1) || isNaN(n2)) {
            if (lblAnswer) lblAnswer.textContent = "Error: Invalid Number";
            return;
        }

        try {
            const result = await calcCall("evaluateBasicOp", [currentOp, n1, n2]);
            if (lblAnswer) {
                lblAnswer.textContent = String(result);
            }
        } catch (err) {
            if (lblAnswer) {
                lblAnswer.textContent = `Error: ${/** @type {Error} */ (err).message}`;
            }
        }
    });

    btnClearAll?.addEventListener("click", () => {
        if (isLocked) {
            alert("Session is locked.");
            return;
        }
        clearAllInputs();
    });
}

/**
 * Resets all inputs in the application.
 */
function clearAllInputs() {
    const inputs = document.querySelectorAll("input[type='text'], textarea");
    for (const input of inputs) {
        if (input instanceof HTMLInputElement || input instanceof HTMLTextAreaElement) {
            input.value = "";
        }
    }
    const lblAnswer = document.getElementById("lblYourAnswer");
    if (lblAnswer) lblAnswer.textContent = "";
}

/**
 * Sets up Tab 2: Prime verification.
 */
function setupPrimeTab() {
    const txtNum1 = /** @type {HTMLInputElement | null} */ (document.getElementById("txtNum1"));
    const btnTest = document.getElementById("btn1");
    const lbl1 = document.getElementById("lbl1");
    const lblFactors = document.getElementById("lblFactors");

    btnTest?.addEventListener("click", async () => {
        const numStr = txtNum1?.value.trim() || "";
        const num = parseInt(numStr, 10);

        if (isNaN(num)) {
            if (lbl1) lbl1.textContent = "Please enter a valid integer.";
            if (lblFactors) lblFactors.textContent = "";
            return;
        }

        try {
            const result = await calcCall("verifyPrimeAndFactors", [num]);
            if (result.is_prime) {
                if (lbl1) lbl1.textContent = "This number is prime.";
                if (lblFactors) lblFactors.textContent = "";
            } else if (result.factor_a && result.factor_b) {
                if (lbl1) lbl1.textContent = `This number is not prime. Two factors are ${result.factor_a} and ${result.factor_b}.`;
                if (lblFactors) lblFactors.textContent = `Factors: ${result.factor_a} × ${result.factor_b} = ${num}`;
            } else {
                if (lbl1) lbl1.textContent = "This number is not prime.";
                if (lblFactors) lblFactors.textContent = "";
            }
        } catch (err) {
            if (lbl1) lbl1.textContent = `Error: ${/** @type {Error} */ (err).message}`;
            if (lblFactors) lblFactors.textContent = "";
        }
    });
}

/**
 * Sets up Tab 3: Random numbers.
 */
function setupRandomTab() {
    const btnRand = document.getElementById("btnRand");

    const refreshRandoms = async () => {
        try {
            const numbers = await calcCall("getRandomScaleTable", []);
            for (let i = 0; i < numbers.length; i++) {
                const lbl = document.getElementById(`lblRan${i + 1}`);
                if (lbl) {
                    lbl.textContent = typeof numbers[i] === "number" ? numbers[i].toFixed(4) : String(numbers[i]);
                }
            }
        } catch (err) {
            console.error("Failed to load random scale table:", err);
        }
    };

    btnRand?.addEventListener("click", refreshRandoms);
    refreshRandoms();
}

/**
 * Sets up Tab 4: Square Root.
 */
function setupSqrtTab() {
    const txtSqRt = /** @type {HTMLInputElement | null} */ (document.getElementById("txtSqRt"));
    const btnSqRt = document.getElementById("btnSqRt");
    const lblAnswer = document.getElementById("lblSqRtAnswer");

    btnSqRt?.addEventListener("click", async () => {
        const numStr = txtSqRt?.value.trim() || "";
        const num = parseFloat(numStr);

        if (isNaN(num)) {
            if (lblAnswer) lblAnswer.textContent = "Error: Invalid Number";
            return;
        }

        try {
            const result = await calcCall("squareRoot", [num]);
            if (lblAnswer) lblAnswer.textContent = String(result);
        } catch (err) {
            if (lblAnswer) lblAnswer.textContent = `Error: ${/** @type {Error} */ (err).message}`;
        }
    });
}

/**
 * Sets up Tab 5: Temperature converter.
 */
function setupTemperatureTab() {
    const txtTemp = /** @type {HTMLInputElement | null} */ (document.getElementById("txtTemp"));
    const btnToFahrenheit = document.getElementById("btnToFahrenheit");
    const btnToCelsius = document.getElementById("btnToCelsius");
    const lblResult = document.getElementById("lblTempResult");

    btnToFahrenheit?.addEventListener("click", async () => {
        const valStr = txtTemp?.value.trim() || "";
        const val = parseFloat(valStr);
        if (isNaN(val)) {
            if (lblResult) lblResult.textContent = "Please enter a valid temperature.";
            return;
        }
        try {
            const res = await calcCall("celsiusToFahrenheit", [val]);
            if (lblResult) {
                lblResult.textContent = `The temperature in Fahrenheit is ${res.toFixed(2)} °F.`;
            }
        } catch (err) {
            if (lblResult) lblResult.textContent = `Error: ${/** @type {Error} */ (err).message}`;
        }
    });

    btnToCelsius?.addEventListener("click", async () => {
        const valStr = txtTemp?.value.trim() || "";
        const val = parseFloat(valStr);
        if (isNaN(val)) {
            if (lblResult) lblResult.textContent = "Please enter a valid temperature.";
            return;
        }
        try {
            const res = await calcCall("fahrenheitToCelsius", [val]);
            if (lblResult) {
                lblResult.textContent = `The temperature in Celsius is ${res.toFixed(2)} °C.`;
            }
        } catch (err) {
            if (lblResult) lblResult.textContent = `Error: ${/** @type {Error} */ (err).message}`;
        }
    });
}

/**
 * Sets up Tab 6: Perimeter.
 */
function setupPerimeterTab() {
    const txtBase = /** @type {HTMLInputElement | null} */ (document.getElementById("txtRectPeri1"));
    const txtHeight = /** @type {HTMLInputElement | null} */ (document.getElementById("txtRectPeri2"));
    const btnGetPeri = document.getElementById("btnGetperi");
    const lblAnswer = document.getElementById("lblPeriAnswer");

    btnGetPeri?.addEventListener("click", async () => {
        const b = parseFloat(txtBase?.value.trim() || "0");
        const h = parseFloat(txtHeight?.value.trim() || "0");

        if (isNaN(b) || isNaN(h)) {
            if (lblAnswer) lblAnswer.textContent = "Error: Invalid Dimensions";
            return;
        }

        try {
            const res = await calcCall("rectanglePerimeter", [b, h]);
            if (lblAnswer) lblAnswer.textContent = String(res);
        } catch (err) {
            if (lblAnswer) lblAnswer.textContent = `Error: ${/** @type {Error} */ (err).message}`;
        }
    });
}

/**
 * Sets up Tab 7: Constants.
 */
function setupConstantsTab() {
    const buttons = document.querySelectorAll("button[data-const]");
    const lblStatus = document.getElementById("lblConstantStatus");

    for (const btn of buttons) {
        btn.addEventListener("click", async () => {
            const constKey = btn.getAttribute("data-const");
            try {
                const constants = await calcCall("getConstants", []);
                let val = 0;
                if (constKey === "pi") val = constants.pi;
                else if (constKey === "e") val = constants.e;
                else if (constKey === "radical13") val = constants.radical13;

                if (navigator.clipboard) {
                    await navigator.clipboard.writeText(String(val));
                }

                const txtN1 = /** @type {HTMLInputElement | null} */ (document.getElementById("txtN1"));
                if (txtN1) {
                    txtN1.value = String(val);
                }

                if (lblStatus) {
                    lblStatus.textContent = `Copied ${val} to clipboard and set into Operand 1.`;
                }
            } catch (err) {
                if (lblStatus) {
                    lblStatus.textContent = `Error loading constant: ${/** @type {Error} */ (err).message}`;
                }
            }
        });
    }
}

/**
 * Sets up Tab 8: Area.
 */
function setupAreaTab() {
    const txtRadius = /** @type {HTMLInputElement | null} */ (document.getElementById("txtRadiusValue"));
    const btnCircleArea = document.getElementById("btnArea");
    const lblCircleAnswer = document.getElementById("lblCircleAreaAnswer");

    btnCircleArea?.addEventListener("click", async () => {
        const r = parseFloat(txtRadius?.value.trim() || "0");
        if (isNaN(r)) {
            if (lblCircleAnswer) lblCircleAnswer.textContent = "Error: Invalid Radius";
            return;
        }
        try {
            const res = await calcCall("circleArea", [r]);
            if (lblCircleAnswer) lblCircleAnswer.textContent = res.toFixed(4);
        } catch (err) {
            if (lblCircleAnswer) lblCircleAnswer.textContent = `Error: ${/** @type {Error} */ (err).message}`;
        }
    });

    const txtBase = /** @type {HTMLInputElement | null} */ (document.getElementById("txtBase"));
    const txtHeight = /** @type {HTMLInputElement | null} */ (document.getElementById("txtHeighth"));
    const btnRectArea = document.getElementById("btnRectArea");
    const lblRectAnswer = document.getElementById("lblRectAreaAnswer");

    btnRectArea?.addEventListener("click", async () => {
        const b = parseFloat(txtBase?.value.trim() || "0");
        const h = parseFloat(txtHeight?.value.trim() || "0");
        if (isNaN(b) || isNaN(h)) {
            if (lblRectAnswer) lblRectAnswer.textContent = "Error: Invalid Dimensions";
            return;
        }
        try {
            const res = await calcCall("rectangleArea", [b, h]);
            if (lblRectAnswer) lblRectAnswer.textContent = res.toFixed(4);
        } catch (err) {
            if (lblRectAnswer) lblRectAnswer.textContent = `Error: ${/** @type {Error} */ (err).message}`;
        }
    });
}

/**
 * Sets up Rock-Paper-Scissors Sidecar modal.
 */
function setupRpsModal() {
    const choiceButtons = document.querySelectorAll("button[data-choice]");
    const lblDecision = document.getElementById("lblRpsDecision");
    const lblOutcome = document.getElementById("lblRpsOutcome");
    const lblWins = document.getElementById("lblRpsWins");
    const lblDraws = document.getElementById("lblRpsDraws");
    const lblLosses = document.getElementById("lblRpsLosses");
    const btnNewSession = document.getElementById("btnRpsNewSession");

    const names = { 1: "Rock", 2: "Paper", 3: "Scissors" };

    for (const btn of choiceButtons) {
        btn.addEventListener("click", async () => {
            const choice = parseInt(btn.getAttribute("data-choice") || "1", 10);
            try {
                const res = await calcCall("playRps", [choice]);
                const userChoiceName = names[/** @type {keyof typeof names} */ (res.userChoice)] || "Unknown";
                const compChoiceName = names[/** @type {keyof typeof names} */ (res.computerChoice)] || "Unknown";

                if (lblDecision) {
                    lblDecision.textContent = `You played: ${userChoiceName} | Computer played: ${compChoiceName}`;
                }

                if (res.outcome === "Win") {
                    rpsScores.wins += 1;
                    if (lblOutcome) lblOutcome.textContent = "You Win! 🎉";
                } else if (res.outcome === "Loss") {
                    rpsScores.losses += 1;
                    if (lblOutcome) lblOutcome.textContent = "You Lose. 😔";
                } else {
                    rpsScores.draws += 1;
                    if (lblOutcome) lblOutcome.textContent = "It's a Draw! 🤝";
                }

                if (lblWins) lblWins.textContent = `${rpsScores.wins} Wins`;
                if (lblDraws) lblDraws.textContent = `${rpsScores.draws} Draws`;
                if (lblLosses) lblLosses.textContent = `${rpsScores.losses} Losses`;
            } catch (err) {
                if (lblDecision) lblDecision.textContent = `Error: ${/** @type {Error} */ (err).message}`;
            }
        });
    }

    btnNewSession?.addEventListener("click", () => {
        rpsScores = { wins: 0, draws: 0, losses: 0 };
        if (lblWins) lblWins.textContent = "0 Wins";
        if (lblDraws) lblDraws.textContent = "0 Draws";
        if (lblLosses) lblLosses.textContent = "0 Losses";
        if (lblDecision) lblDecision.textContent = "Select a play to challenge the computer.";
        if (lblOutcome) lblOutcome.textContent = "";
    });
}

/**
 * Sets up 6r2 Unique Random Number Generator Sidecar modal.
 */
function setupSixR2Modal() {
    const btnGenerate = document.getElementById("btnSixR2Generate");
    const lblNum1 = document.getElementById("lblSixR2Num1");
    const lblNum2 = document.getElementById("lblSixR2Num2");
    const lblNum3 = document.getElementById("lblSixR2Num3");
    const lblIteration = document.getElementById("lblSixR2Iteration");

    btnGenerate?.addEventListener("click", async () => {
        try {
            const triplet = await calcCall("generateUniqueRandomTriplet", [0, 5]);
            sixR2Iterations += 1;

            if (lblNum1) lblNum1.textContent = String(triplet[0]);
            if (lblNum2) lblNum2.textContent = String(triplet[1]);
            if (lblNum3) lblNum3.textContent = String(triplet[2]);
            if (lblIteration) lblIteration.textContent = String(sixR2Iterations);
        } catch (err) {
            alert(`Failed to generate unique random numbers: ${/** @type {Error} */ (err).message}`);
        }
    });
}

// Auto-initialize when loaded
if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initCalculator);
} else {
    initCalculator();
}
