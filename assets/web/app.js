/**
 * Main application script module for Collective Toolbox
 */

import {
    copyToClipboard,
    $,
    $$,
    isModifiedClick,
    markLoading,
    isCrossOriginUrl,
    updateScrollbarSize,
    setLoading,
    unSetLoading,
    remToPx
} from './js/utilities.js';

import {
    startUpdatePolling
} from './js/update-polling.js';

import {
    modalOpen,
    modalContentClose,
    modalContentPopout,
    showModal,
    showAlert,
    showConfirm,
    showConfirmHtml,
    showPrompt
} from './js/modals.js';

import {
    setupSidebarToggle,
    sidebarSetOpen
} from './js/sidebar.js';

import {
    showToast,
    showAlertBanner
} from './js/notifications.js';

import './js/proxy-notice.js';
import './js/backdrop-filter-polyfill.js';
import { CtbButton } from './js/components/base/button.js';
import { CtbSegmentedControl as _CtbSegmentedControl } from './js/components/base/segmented-control.js';
import { CtbFileInput as _CtbFileInput } from './js/components/base/file-input.js';
import { CtbDropdown as _CtbDropdown } from './js/components/base/dropdown.js';
import { CtbMenu as _CtbMenu } from './js/components/base/menu.js';
import { CtbMenubar as _CtbMenubar } from './js/components/base/menubar.js';
import { CtbTabGroup as _CtbTabGroup } from './js/components/base/tab-group.js';
import { CtbLayout as _CtbLayout } from './js/components/base/layout.js';
CtbButton.defaultTheme = "glass";

const CLIENT_SETTINGS_KEY = "ctb-client-side-settings";
const colorThemeStorageKey = "color-theme-setting";
const btnThemeStorageKey = "btn-theme-setting";

/** @type {any} */
let activeDownloadTimerId = null;

// ---- Theme & Client Settings Helpers ----------------------------------------
/**
 * Safe wrapper around localStorage access.
 */
const safeLocalStorage = {
    /**
     * @param {string} key The storage key.
     * @returns {string | null} The item value or null.
     */
    getItem(key) {
        try {
            return localStorage.getItem(key);
        } catch (_err) {
            return null;
        }
    },
    /**
     * @param {string} key The storage key.
     * @param {string} value The storage value.
     * @returns {void}
     */
    setItem(key, value) {
        try {
            localStorage.setItem(key, value);
        } catch (_err) {
            // Ignore quota or disabled storage error
        }
    }
};

/**
 * Retrieves the full ctb-client-side-settings object from localStorage.
 *
 * @returns {Record<string, string>}
 */
const getClientSideSettings = () => {
    try {
        const raw = safeLocalStorage.getItem(CLIENT_SETTINGS_KEY);
        if (raw) {
            return JSON.parse(raw);
        }
    } catch (_err) {
        // Ignore JSON parse error
    }
    return {};
};

/**
 * Gets a client-side setting value, checking ctb-client-side-settings,
 * then localStorage, then sessionStorage.
 *
 * @param {string} key
 * @returns {string|null}
 */
const getClientSetting = (key) => {
    const settings = getClientSideSettings();
    if (settings && key in settings && settings[key] !== null && settings[key] !== undefined) {
        return settings[key];
    }
    const fromLocal = safeLocalStorage.getItem(key);
    if (fromLocal !== null && fromLocal !== undefined) return fromLocal;

    try {
        const fromSession = sessionStorage.getItem(key);
        if (fromSession !== null && fromSession !== undefined) return fromSession;
    } catch (_err) {
        // Ignore storage error
    }

    return null;
};

/**
 * Persists a setting into ctb-client-side-settings JSON object,
 * while maintaining backward compatibility with individual storage keys.
 *
 * @param {string} key
 * @param {string} value
 * @returns {void}
 */
const setClientSetting = (key, value) => {
    const settings = getClientSideSettings();
    settings[key] = value;
    try {
        safeLocalStorage.setItem(CLIENT_SETTINGS_KEY, JSON.stringify(settings));
    } catch (_err) {
        // Ignore storage error
    }
    safeLocalStorage.setItem(key, value);
    try {
        sessionStorage.setItem(key, value);
    } catch (_err) {
        // Ignore storage error
    }
};

/**
 * Applies saved color scheme and button theme attributes to documentElement immediately.
 *
 * @returns {void}
 */
const applySavedThemes = () => {
    const savedColor = getClientSetting(colorThemeStorageKey);
    if (savedColor) {
        document.documentElement.classList.remove("theme-auto", "theme-light", "theme-dark");
        document.documentElement.classList.add(`theme-${savedColor}`);
    }
    const savedBtn = getClientSetting(btnThemeStorageKey) || "glass";
    document.documentElement.setAttribute("data-ctb-ui-theme", savedBtn);
};

// Apply themes early on script load to prevent layout flicker
applySavedThemes();

/**
 * Restore the checked theme radio inputs and synchronize theme state.
 *
 * @returns {void}
 */
const rememberThemeRadioState = () => {
    const savedTheme = getClientSetting(colorThemeStorageKey);
    if (savedTheme) {
        const $radioToCheck = $(
            `input[name="color-theme-setting"][value="${savedTheme}"]`
        );
        if ($radioToCheck.length) {
            $radioToCheck.prop('checked', true);
            $radioToCheck[0]?.dispatchEvent(new Event('change', { bubbles: true }));
        }
    }

    const savedBtnTheme = getClientSetting(btnThemeStorageKey) || "glass";
    const $btnRadioToCheck = $(
        `input[name="btn-theme-setting"][value="${savedBtnTheme}"]`
    );
    if ($btnRadioToCheck.length) {
        $btnRadioToCheck.prop('checked', true);
        $btnRadioToCheck[0]?.dispatchEvent(new Event('change', { bubbles: true }));
    }

    applySavedThemes();
};

// ---- Error modal -----------------------------------------------------------
/**
 * @param {{ message: string, isJson: boolean, status?: number }} options
 * @returns {void}
 */
function showError({ message, isJson, status }) {
    // 4xx errors are considered recoverable
    const isRecoverable = status && status >= 400 && status < 500;
    const templateId = isRecoverable ? "recoverable-error-template" : "unrecoverable-error-template";
    const template = document.getElementById(templateId);
    if (!template || !(template instanceof HTMLTemplateElement)) {
        markLoading(false);
        return;
    }

    // Clone the template content
    const fragment = /** @type {DocumentFragment} */ (template.content.cloneNode(true));
    const modalBoxElement = fragment.querySelector(".modal-box");
    if (!modalBoxElement) {
        markLoading(false);
        return;
    }
    const $modalBox = $(modalBoxElement);
    const $errorText = $modalBox.find(".error-modal-content");
    const $errorTitle = $modalBox.find(".recoverable-error-title");

    let displayMessage = message ? message.trim() : "";
    let displayDetails = "";

    if (isJson && displayMessage) {
        try {
            const obj = JSON.parse(displayMessage);
            displayMessage = obj.message || displayMessage;
            displayDetails = JSON.stringify(obj, null, 2).replaceAll(
                "\\n",
                "\\n\n"
            );
        } catch {
            // Keep displayMessage as is
        }
    }

    if (!displayMessage) {
        if (status) {
            displayMessage = `HTTP Error ${status}: ${status === 405 ? "Method Not Allowed" : "An unexpected error occurred"}`;
        } else {
            displayMessage = "An unexpected error occurred";
        }
    }

    if (isRecoverable) {
        if ($errorTitle.length) {
            $errorTitle.text(displayMessage);
        }
        if ($errorText.length) {
            if (displayDetails) {
                $errorText.text(displayDetails).removeClass("hidden");
            } else {
                $errorText.addClass("hidden");
            }
        }
    } else {
        if ($errorText.length) {
            $errorText.text(displayDetails || displayMessage);
        }
    }

    // Append the cloned modal box to the body temporarily so we can pass it to showModal
    const modalBoxEl = /** @type {HTMLElement} */ ($modalBox[0]);
    document.body.appendChild(modalBoxEl);

    showModal(modalBoxEl, {
        closeOnEscape: false,
        closeOnOutsideClick: false,
        onClose: () => {
            modalBoxEl.remove();
        }
    });

    markLoading(false);
}

// ---- Default Toggles helper ------------------------------------------------
// TODO: Consider making it show the default value when disabled, but
// restore the user's value when unchecking Default checkbox?
/**
 * Toggle adjacent form controls when a default checkbox is checked.
 *
 * @returns {void}
 */
/**
 * Toggle adjacent form controls when a default checkbox is checked.
 *
 * @param {ParentNode} [root=document] The root node (defaults to document).
 * @returns {void}
 */
function setupDefaultAdjacentToggles(root = document) {
    const toggles = /** @type {HTMLInputElement[]} */ (
        Array.from(root.querySelectorAll(".form-default-adjacent"))
    );
    for (const toggle of toggles) {
        if (toggle.dataset.adjacentToggleAttached) continue;
        const apply = () => {
            const container = toggle.closest("td") || toggle.parentElement;
            if (!container) return;

            const controls = /** @type {(HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement)[]} */ (
                Array.from(container.querySelectorAll("input, textarea, select"))
            ).filter((el) => el !== toggle);
            const shouldDisable = !!toggle.checked;
            for (const el of controls) {
                el.disabled = shouldDisable;
            }
        };

        toggle.addEventListener("change", apply);
        toggle.dataset.adjacentToggleAttached = "true";
        apply();
    }
}

// ---- Update Nav Auth Buttons ------------------------------------------------
/**
 * Synchronize the visibility of the nav login/logout buttons.
 * If no session cookie exists in document.cookie, forces logged-out state.
 * If a new document is provided (during SPA navigation), syncs state from the server-rendered document.
 *
 * @param {Document} [doc=document] Optional newly fetched document.
 * @returns {void}
 */
function updateNavAuthButtons(doc = document) {
    const hasSession = document.cookie.split(';').some((item) => {
        const trimmed = item.trim();
        return trimmed.startsWith('session=') && trimmed.substring(8).trim() !== '';
    });
    const loginAnchor = document.getElementById('login-btn');
    const logoutAnchor = document.getElementById('logout-btn');

    if (!hasSession) {
        if (loginAnchor) {
            loginAnchor.classList.remove('hidden');
            if (loginAnchor.parentElement?.tagName === 'CTB-BUTTON') {
                loginAnchor.parentElement.classList.remove('hidden');
            }
        }
        if (logoutAnchor) {
            logoutAnchor.classList.add('hidden');
            if (logoutAnchor.parentElement?.tagName === 'CTB-BUTTON') {
                logoutAnchor.parentElement.classList.add('hidden');
            }
        }
        return;
    }

    if (doc && doc !== document) {
        const newLoginAnchor = doc.getElementById('login-btn');
        const newLogoutAnchor = doc.getElementById('logout-btn');
        const isNewLoginHidden = newLoginAnchor ? newLoginAnchor.classList.contains('hidden') : false;
        const isNewLogoutHidden = newLogoutAnchor ? newLogoutAnchor.classList.contains('hidden') : true;

        if (loginAnchor) {
            loginAnchor.classList.toggle('hidden', isNewLoginHidden);
            if (loginAnchor.parentElement?.tagName === 'CTB-BUTTON') {
                loginAnchor.parentElement.classList.toggle('hidden', isNewLoginHidden);
            }
        }
        if (logoutAnchor) {
            logoutAnchor.classList.toggle('hidden', isNewLogoutHidden);
            if (logoutAnchor.parentElement?.tagName === 'CTB-BUTTON') {
                logoutAnchor.parentElement.classList.toggle('hidden', isNewLogoutHidden);
            }
        }
    }
}

// ---- Body Swapper (SPA Nav) ------------------------------------------------
/**
 * Replace the current document body with server-rendered HTML.
 *
 * @param {string} htmlString
 * @returns {void}
 */
function updateBody(htmlString) {
    const parser = new DOMParser();
    const newDoc = parser.parseFromString(htmlString, "text/html");

    const activeMain = document.querySelector('div[role="main"]');
    const newMain = newDoc.querySelector('div[role="main"]');

    if (activeMain && newMain) {
        // Upgrade the new main content in memory before insertion
        upgradeButtons(newMain);

        // Sync body class and attributes to document.body
        if (newDoc.body) {
            document.body.className = newDoc.body.className;
            for (const attr of newDoc.body.attributes) {
                document.body.setAttribute(attr.name, attr.value);
            }
        }

        // Swap only the main content area
        activeMain.innerHTML = newMain.innerHTML;

        // Execute any script tags in activeMain so that dynamic scripts run
        activeMain.querySelectorAll("script").forEach((oldScript) => {
            const newScript = document.createElement("script");
            for (const attr of oldScript.attributes) {
                newScript.setAttribute(attr.name, attr.value);
            }
            newScript.textContent = oldScript.textContent;
            if (oldScript.parentNode) {
                oldScript.parentNode.replaceChild(newScript, oldScript);
            }
        });

        // Also update document title
        document.title = newDoc.title;
    } else {
        // Fallback to full body swap if main content container isn't found
        upgradeButtons(newDoc);
        document.body.innerHTML = newDoc.body.innerHTML;
        document.body.querySelectorAll("script").forEach((oldScript) => {
            const newScript = document.createElement("script");
            for (const attr of oldScript.attributes) {
                newScript.setAttribute(attr.name, attr.value);
            }
            newScript.textContent = oldScript.textContent;
            if (oldScript.parentNode) {
                oldScript.parentNode.replaceChild(newScript, oldScript);
            }
        });
        setupSidebarToggle();
    }

    rememberThemeRadioState();
    updateNavAuthButtons(newDoc);
    markLoading(false);

    if (window.location.hash) {
        history.replaceState(
            null,
            "",
            window.location.pathname + window.location.search
        );
    }

    document.body.scrollTo({ top: 0, behavior: "instant" });

    const targetRoot = activeMain && newMain ? activeMain : document.body;
    applyExternalLinkAttrs();
    setupDefaultAdjacentToggles(targetRoot);
}

/**
 * Upgrade native buttons and anchors with the button styling to use the custom ctb-button web component wrapper.
 *
 * @param {ParentNode} [root=document] The root node containing the buttons to upgrade (defaults to document).
 * @returns {void}
 */
function upgradeButtons(root = document) {
    const elements = /** @type {(HTMLButtonElement | HTMLAnchorElement)[]} */ (
        Array.from(root.querySelectorAll("button, a.btn"))
    );
    for (const el of elements) {
        // Skip elements inside templates, already wrapped, or with class to ignore
        if (
            el.closest("template") ||
            (el.parentElement && el.parentElement.tagName === "CTB-BUTTON") ||
            el.classList.contains("btn-no-upgrade") ||
            el.classList.contains("btn-link")
        ) {
            continue;
        }

        const wrapper = document.createElement("ctb-button");

        // Extract theme variant from element classes
        if (el.classList.contains("btn-primary")) {
            wrapper.setAttribute("variant", "primary");
        } else {
            wrapper.setAttribute("variant", "secondary");
        }

        if (el.hasAttribute("disabled") || el.classList.contains("disabled")) {
            wrapper.setAttribute("disabled", "");
        }

        if (el.classList.contains("hidden")) {
            wrapper.classList.add("hidden");
        }

        // Insert wrapper in DOM and wrap element
        el.parentNode?.insertBefore(wrapper, el);
        wrapper.appendChild(el);
    }
}

/**
 * Fetch a document fragment and swap it into the current page.
 *
 * @param {string} url
 * @param {RequestInit & { historyAction?: "push" | "replace" | null }} [options={}]
 * @returns {Promise<void>}
 */
async function fetchAndSwap(url, options = {}) {
    markLoading(true);

    let historyAction = options.historyAction;
    if (historyAction === undefined) {
        historyAction = "push";
    }

    try {
        const response = await fetch(url, {
            headers: {
                Accept: "text/html;q=0.9,application/json;q=0.8,*/*;q=0.7",
                "X-CollectiveToolbox-IsJsRequest": "true",
                "X-Back-Url": window.location.pathname + window.location.search,
                ...(options.headers || {}),
            },
            ...Object.fromEntries(Object.entries(options).filter(([k]) => k !== "historyAction")),
        });

        const contentType = response.headers.get("content-type") || "";
        const isJson = contentType.includes("application/json");
        const isJsRedirect = response.headers.get("X-CollectiveToolbox-IsJsRedirect");

        if (isJsRedirect) {
            const text = await response.text();
            try {
                const json = JSON.parse(text);
                const redirectUrl = json.url;

                if (isCrossOriginUrl(redirectUrl)) {
                    markLoading(false);
                    window.open(redirectUrl, "_blank");
                    return;
                }

                // Redirect should always replace the history entry rather than pushing a new one
                // Standard redirects (like 303 See Other) transition POST requests to GET requests and drop the body.
                const { method: _method, body: _body, ...redirectOptions } = options;
                await fetchAndSwap(redirectUrl, {
                    ...redirectOptions,
                    historyAction: "replace",
                });
                return;
            } catch (e) {
                const message = e instanceof Error ? e.message : String(e);
                showError({
                    message: "Invalid redirect response: " + message,
                    isJson: false,
                });
                return;
            }
        }

        const finalUrl = response.url || url;
        if (isCrossOriginUrl(finalUrl)) {
            markLoading(false);
            window.open(finalUrl, "_blank");
            return;
        }

        const text = await response.text();
        if (!response.ok) {
            showError({ message: text, isJson, status: response.status });
        } else {
            if (historyAction === "push") {
                history.pushState(null, "", finalUrl);
            } else if (historyAction === "replace") {
                history.replaceState(null, "", finalUrl);
            }
            updateBody(text);
        }
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        showError({ message, isJson: false });
    }
}

// ---- External links --------------------------------------------------------
/**
 * Apply safe target and rel attributes to external links.
 *
 * @returns {void}
 */
function applyExternalLinkAttrs() {
    $$(".link-external").forEach((link) => {
        link.setAttribute("target", "_blank");
        link.setAttribute("rel", "noopener noreferrer");
    });
}

/**
 * Add copy actions next to elements marked as copiable.
 *
 * @returns {void}
 */
function applyCopiableElements() {
    $$('.copiable').forEach((e) => {
        const copyLink = document.createElement('a');
        copyLink.href = '#';
        copyLink.classList.add('copiable-copy-link');
        copyLink.textContent = '(Copy)';
        copyLink.addEventListener('click', (event) => {
            event.preventDefault();
            event.stopPropagation();
            const textToCopy = e.textContent || '';
            copyToClipboard(textToCopy);
        });

        // Add the link after the copiable element
        e.insertAdjacentElement('afterend', copyLink);
    })
}

// ---- Autoload fragment handler ---------------------------------------------
/**
 * Follow a valid autoload fragment after the initial page render.
 *
 * @returns {void}
 */
function handleAutoloadFragment() {
    const hash = window.location.hash;
    if (!hash.startsWith("#autoload-")) return;

    const autoloadPath = decodeURI(hash.substring("#autoload-".length));
    if (
        !autoloadPath ||
        autoloadPath.startsWith("/") ||
        URL.canParse(autoloadPath) ||
        /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(autoloadPath)
    ) {
        history.replaceState(null, "", window.location.pathname + window.location.search);
        console.log("Ignoring invalid autoload path:", autoloadPath);
        return;
    }

    history.replaceState(null, "", window.location.pathname + window.location.search);
    fetchAndSwap(autoloadPath);
}

// ---- Public global API binding ---------------------------------------------
window.ctb = {
    copyToClipboard,
    setLoading,
    unSetLoading,
    modal: {
        close: modalContentClose,
        popout: modalContentPopout
    },
    toast: showToast,
    alertBanner: showAlertBanner,
    upgradeButtons: upgradeButtons,
    debug: (msg, timeout) => showToast(msg, 'debug', timeout),
    info: (msg, timeout) => showToast(msg, 'info', timeout),
    warn: (msg, timeout) => showToast(msg, 'warning', timeout),
    error: (msg, timeout) => showToast(msg, 'error', timeout),
    alert: showAlert,
    confirm: showConfirm,
    confirmHtml: showConfirmHtml,
    prompt: showPrompt,
    showModal: showModal,
    remToPx: remToPx,
    settings: {
        get: getClientSetting,
        set: setClientSetting,
        getAll: getClientSideSettings
    }
};

window.ctbClientSideSettings = {
    get: getClientSetting,
    set: setClientSetting,
    getAll: getClientSideSettings
};

// ---- DOM Initialization -----------------------------------------------------
document.addEventListener("DOMContentLoaded", () => {
    updateScrollbarSize();
    window.addEventListener("resize", () => {
        updateScrollbarSize();
    });

    window.addEventListener("storage", (event) => {
        if (
            event.key === CLIENT_SETTINGS_KEY ||
            event.key === colorThemeStorageKey ||
            event.key === btnThemeStorageKey
        ) {
            rememberThemeRadioState();
        }
    });

    // Delegated theme radio change listener
    document.addEventListener("change", (event) => {
        const radio = event.target;
        if (!(radio instanceof HTMLInputElement)) return;

        if (radio.name === "color-theme-setting") {
            if (radio.checked) {
                setClientSetting(colorThemeStorageKey, radio.value);
                document.documentElement.classList.remove("theme-auto", "theme-light", "theme-dark");
                document.documentElement.classList.add(`theme-${radio.value}`);
            }
        } else if (radio.name === "btn-theme-setting") {
            if (radio.checked) {
                setClientSetting(btnThemeStorageKey, radio.value);
                document.documentElement.setAttribute("data-ctb-ui-theme", radio.value);
            }
        }
    });

    // Delegated form submit listener
    document.addEventListener("submit", (event) => {
        const target = event.target;
        if (!(target instanceof Element)) return;

        const form = target.closest("form");
        if (!form) return;

        event.preventDefault();

        const formData = new FormData(form);
        const action = form.action || window.location.href;
        const method = (form.method || "GET").toUpperCase();

        fetchAndSwap(action, {
            method,
            body: method === "GET" ? null : formData,
        });
    });

    // Delegated click listener
    document.addEventListener("click", (event) => {
        const target = event.target;
        if (!(target instanceof Element)) return;

        const link = target.closest("a[href]");
        if (!(link instanceof HTMLAnchorElement)) return;
        if (!link) return;

        const hrefAttr = link.getAttribute("href") || "";
        if (hrefAttr.startsWith("#")) {
            if (hrefAttr === "#") {
                event.preventDefault();
                document.body.scrollTo({ top: 0, behavior: "smooth" });
                history.pushState(null, "", window.location.pathname + window.location.search);
                return;
            }
            const targetId = hrefAttr.substring(1);
            const targetElement = document.getElementById(targetId);
            if (targetElement) {
                event.preventDefault();
                targetElement.scrollIntoView({ behavior: "smooth" });
                history.pushState(null, "", window.location.pathname + window.location.search);
            }
            return;
        }

        if (link.classList.contains("link-download")) {
            const isLargeDownload =
                link.pathname === "/installer-linux-x64" ||
                link.pathname === "/installer-linux-x86" ||
                link.pathname === "/src.tar.gz" ||
                link.pathname === "/dependencies.tar.gz" ||
                link.pathname.startsWith("/releases/linux-x64/") ||
                link.pathname.startsWith("/releases/linux-x86/") ||
                link.pathname.startsWith("/releases/src/");

            if (isLargeDownload) {
                event.preventDefault();
                // Clear any old cookie first
                document.cookie = "ctb_download_started=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax";

                setLoading();

                // Trigger download
                window.location.href = link.href;

                // Poll for the download response cookie
                const checkInterval = 100;
                const timeout = 120000;
                let elapsed = 0;
                if (activeDownloadTimerId) {
                    clearInterval(activeDownloadTimerId);
                }
                activeDownloadTimerId = setInterval(() => {
                    elapsed += checkInterval;
                    if (document.cookie.includes("ctb_download_started=1")) {
                        clearInterval(activeDownloadTimerId);
                        activeDownloadTimerId = null;
                        document.cookie = "ctb_download_started=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax";
                        unSetLoading();
                    } else if (elapsed >= timeout) {
                        clearInterval(activeDownloadTimerId);
                        activeDownloadTimerId = null;
                        unSetLoading();
                        ctb.warn("Sorry, it looks like your download did not start successfully. You may try again if you like.");
                    }
                }, checkInterval);
            }
            return;
        }
        if (link.classList.contains("link-external")) return;

        try {
            const url = new URL(link.href, window.location.href);
            if (
                url.origin === window.location.origin &&
                (url.pathname.startsWith("/docs/lib") ||
                 url.pathname.startsWith("/docs/rust") ||
                 url.pathname === "/installer-linux-x64" ||
                 url.pathname === "/installer-linux-x86" ||
                 url.pathname === "/src.tar.gz" ||
                 url.pathname === "/dependencies.tar.gz" ||
                 url.pathname.startsWith("/releases/"))
            ) {
                return;
            }
        } catch (_e) {
            // Ignore parsing errors for invalid links
        }

        if (link.classList.contains("link-open-in-frame")) {
            if (isModifiedClick(event)) return;
            event.preventDefault();
            event.stopPropagation();
            modalOpen(link.href);
            return;
        }

        if (hrefAttr.startsWith("javascript")) return;

        if (isModifiedClick(event) || link.target === "_blank") return;

        event.preventDefault();
        fetchAndSwap(link.href);
    });

    // Escape closes sidebar (modals are handled in modals.js)
    document.addEventListener("keydown", (event) => {
        if (event.key !== "Escape") return;

        if (activeDownloadTimerId) {
            clearInterval(activeDownloadTimerId);
            activeDownloadTimerId = null;
            unSetLoading();
            ctb.warn("Sorry, it looks like your download did not start successfully. You may try again if you like.");
            return;
        }

        const sidebar = document.getElementById("sidebar");
        const isOpen = sidebar && sidebar.classList.contains("sidebar-open");
        if (isOpen) sidebarSetOpen(false);
    });

    // Sidebar click delegation
    document.addEventListener("click", (event) => {
        const target = event.target;
        if (!(target instanceof Element)) return;

        const toggle = target.closest('[href="#sidebar"]');
        const sidebar = document.getElementById("sidebar");
        if (!sidebar) return;

        const removeFragment = () => {
            if (window.location.hash === "#sidebar") {
                history.replaceState(
                    null,
                    "",
                    window.location.pathname + window.location.search
                );
            }
        };

        if (toggle) {
            removeFragment();
            event.preventDefault();
            const isOpen = sidebar.classList.contains("sidebar-open");
            sidebarSetOpen(!isOpen);
            return;
        }

        const isOpen = sidebar.classList.contains("sidebar-open");
        if (!isOpen) return;

        const hamburger = $$('[href="#sidebar"]')[0];
        if (
            hamburger &&
            !sidebar.contains(target) &&
            !hamburger.contains(target)
        ) {
            removeFragment();
            sidebarSetOpen(false);
        }
    });

    // Initial page setup calls
    rememberThemeRadioState();
    updateNavAuthButtons();
    applyExternalLinkAttrs();
    applyCopiableElements();
    setupSidebarToggle();
    setupDefaultAdjacentToggles();
    handleAutoloadFragment();
    upgradeButtons();

    // Auto-upgrade newly inserted buttons or anchors
    const buttonUpgradeObserver = new MutationObserver((mutations) => {
        let shouldUpgrade = false;
        for (const mutation of mutations) {
            for (const node of mutation.addedNodes) {
                if (node.nodeType === Node.ELEMENT_NODE) {
                    const el = /** @type {HTMLElement} */ (node);
                    const targets = el.matches("button, a.btn") ? [el] : Array.from(el.querySelectorAll("button, a.btn"));
                    for (const target of targets) {
                        if (
                            target.parentElement?.tagName !== "CTB-BUTTON" &&
                            !target.closest("template") &&
                            !target.classList.contains("btn-no-upgrade")
                        ) {
                            shouldUpgrade = true;
                            break;
                        }
                    }
                }
                if (shouldUpgrade) break;
            }
            if (shouldUpgrade) break;
        }
        if (shouldUpgrade) {
            upgradeButtons();
        }
    });
    buttonUpgradeObserver.observe(document.body, { childList: true, subtree: true });

    // Handle back/forward navigation
    window.addEventListener("popstate", () => {
        fetchAndSwap(window.location.href, { historyAction: null });
    });

    // Start checking for updates
    startUpdatePolling();
});
