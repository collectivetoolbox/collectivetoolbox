/**
 * Update notification polling and banners
 */

/**
 * @typedef {{
 *   version: string,
 *   buildDate: string,
 * }} LoadedBuildInfo
 */

/**
 * @typedef {{
 *   is_newer?: boolean,
 *   server_version?: string,
 *   available?: boolean,
 *   version?: string,
 *   release_notes_url?: string,
 * }} UpdateStatusResponse
 */

const updateCheckIntervalMs = 60000; // 60 seconds
/** @type {number | null} */
let updateCheckTimer = null;

/**
 * Read the current build metadata embedded on the page.
 *
 * @returns {LoadedBuildInfo}
 */
function getLoadedBuildInfo() {
    const root = document.body;
    return {
        version: root?.dataset.ctbBuildVersion || "",
        buildDate: root?.dataset.ctbBuildDate || "",
    };
}

/**
 * Stop the background update polling timer.
 *
 * @returns {void}
 */
export function stopUpdatePolling() {
    if (updateCheckTimer) {
        clearInterval(updateCheckTimer);
        updateCheckTimer = null;
    }
}

/**
 * Render the restart-required update banner.
 *
 * @param {string} version
 * @param {string | undefined} releaseNotesUrl
 * @returns {void}
 */
function createUpdateBanner(version, releaseNotesUrl) {
    const existing = document.getElementById("update-notification-banner");
    if (existing) {
        existing.remove();
        document.body.style.paddingTop = "";
    }

    const message = `
        <span>
            <strong>Update available</strong> (v${version}) — Restart to upgrade
            ${releaseNotesUrl ? `<a href="${releaseNotesUrl}" target="_blank" class="underline ml-2">Release notes</a>` : ""}
        </span>
        <div class="flex items-center gap-2 ml-auto">
            <button type="button" id="update-restart-btn" class="btn btn-primary">
                Restart Now
            </button>
            <button type="button" id="update-later-btn" class="btn">
                Later
            </button>
        </div>
    `;

    const banner = ctb.alertBanner(message, "info", document.body, false);
    if (banner) {
        banner.id = "update-notification-banner";
        // Force reflow and set padding
        document.body.style.paddingTop = banner.offsetHeight + "px";

        document.getElementById("update-restart-btn")?.addEventListener("click", () => {
            ctb.alertBanner("Application restart would be triggered here. For now, please restart the application manually.");
        });

        document.getElementById("update-later-btn")?.addEventListener("click", () => {
            banner.remove();
            document.body.style.paddingTop = "";
            stopUpdatePolling();
            sessionStorage.setItem("update-dismissed", "true");
        });
    }
}

/**
 * Render the reload-required banner when the server is newer.
 *
 * @param {string} version
 * @returns {void}
 */
function createReloadBanner(version) {
    const existing = document.getElementById("update-notification-banner");
    if (existing) {
        existing.remove();
        document.body.style.paddingTop = "";
    }

    const message = `
        <span>
            <strong>Reload required</strong> ${version ? `(v${version})` : ""} — A newer build is running on the server
        </span>
        <div class="flex items-center gap-2 ml-auto">
            <button type="button" id="update-restart-btn" class="btn btn-primary">
                Reload Now
            </button>
            <button type="button" id="update-later-btn" class="btn">
                Later
            </button>
        </div>
    `;

    const banner = ctb.alertBanner(message, "warning", document.body, false);
    if (banner) {
        banner.id = "update-notification-banner";
        
        // Mutate the icon to use the reload SVG specifically
        const iconImg = /** @type {HTMLImageElement | null} */ (
            banner.querySelector(".alert-banner-icon img")
        );
        if (iconImg) {
            iconImg.src = "/resources/icons/reload.svg";
            iconImg.alt = "Reload";
        }

        // Force reflow and set padding
        document.body.style.paddingTop = banner.offsetHeight + "px";

        document.getElementById("update-restart-btn")?.addEventListener("click", () => {
            window.location.reload();
        });

        document.getElementById("update-later-btn")?.addEventListener("click", () => {
            banner.remove();
            document.body.style.paddingTop = "";
            stopUpdatePolling();
            sessionStorage.setItem("update-dismissed", "true");
        });
    }
}

/**
 * Check the server for update availability and show the matching banner.
 *
 * @returns {Promise<void>}
 */
async function checkForUpdates() {
    if (sessionStorage.getItem("update-dismissed") === "true") {
        return;
    }

    try {
        const buildInfo = getLoadedBuildInfo();
        const params = new URLSearchParams();
        if (buildInfo.version) {
            params.set("version", buildInfo.version);
        }
        if (buildInfo.buildDate) {
            params.set("build_date", buildInfo.buildDate);
        }

        const endpoint = params.size > 0
            ? `/api/update-status?${params.toString()}`
            : "/api/update-status";
        const response = await fetch(endpoint);
        if (!response.ok) return;

        /** @type {UpdateStatusResponse} */
        const data = await response.json();
        if (data.is_newer) {
            createReloadBanner(data.server_version || "");
            stopUpdatePolling();
            return;
        }

        if (data.available && data.version) {
            createUpdateBanner(data.version, data.release_notes_url);
            stopUpdatePolling();
        }
    } catch (e) {
        console.debug("Update check failed:", e);
    }
}

/**
 * Start periodic update polling for the current page.
 *
 * @returns {void}
 */
export function startUpdatePolling() {
    if (sessionStorage.getItem("update-dismissed") === "true") {
        return;
    }

    checkForUpdates();

    updateCheckTimer = setInterval(() => {
        if (!document.hidden) {
            checkForUpdates();
        }
    }, updateCheckIntervalMs);

    document.addEventListener("visibilitychange", () => {
        if (!document.hidden) {
            checkForUpdates();
        }
    });
}
