/**
 * @returns {Number}
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