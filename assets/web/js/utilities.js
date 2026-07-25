/**
 * Utility functions for Collective Toolbox
 */

import { $ as jQueryInstance } from '../vendor/jquery/src/jquery.js';

/**
 * Copy text to the clipboard and surface a toast for the result.
 *
 * @param {string} text
 * @returns {void}
 */
export function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(
        () => {
            ctb.info("Copied to clipboard!");
        },
        (err) => {
            ctb.error("Failed to copy to clipboard: " + err);
        }
    );
}

/**
 * The jQuery helper function.
 */
export const $ = jQueryInstance;

/**
 * Return all matching elements within the given root.
 *
 * @template {Element} T
 * @param {string} sel
 * @param {ParentNode} [root=document]
 * @returns {T[]}
 */
export const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

/**
 * Check whether a mouse or keyboard activation uses a modifier.
 *
 * @param {MouseEvent | KeyboardEvent | { ctrlKey?: boolean; metaKey?: boolean; shiftKey?: boolean; altKey?: boolean; }} event
 * @returns {boolean}
 */
export function isModifiedClick(event) {
    return !!(event.ctrlKey || event.metaKey || event.shiftKey || event.altKey);
}

/**
 * Toggle the page loading state class.
 *
 * @param {boolean} v
 * @returns {void}
 */
export function markLoading(v) {
    if (v) document.body.classList.add("content-loading");
    else document.body.classList.remove("content-loading");
}

/** @type {HTMLDivElement | null} */
let loadingOverlayElement = null;

/**
 * Show a full-screen loading overlay and set the busy cursor.
 *
 * @returns {void}
 */
export function setLoading() {
    if (!loadingOverlayElement) {
        loadingOverlayElement = document.createElement("div");
        loadingOverlayElement.id = "loading-overlay";
        loadingOverlayElement.className = "loading-overlay";
        loadingOverlayElement.innerHTML = `
            <div class="loading-spinner-container">
                <div class="loading-spinner"></div>
                <div class="loading-text">Preparing download...</div>
            </div>
        `;
        document.body.appendChild(loadingOverlayElement);
    }
    // Force a reflow
    void loadingOverlayElement.offsetWidth;
    loadingOverlayElement.classList.add("active");
    document.body.classList.add("download-loading");
}

/**
 * Hide the full-screen loading overlay and restore the cursor.
 *
 * @returns {void}
 */
export function unSetLoading() {
    if (loadingOverlayElement) {
        loadingOverlayElement.classList.remove("active");
    }
    document.body.classList.remove("download-loading");
}

/**
 * Determine whether a URL resolves to a different origin.
 *
 * @param {string} url
 * @returns {boolean}
 */
export function isCrossOriginUrl(url) {
    try {
        return new URL(url, window.location.href).origin !== window.location.origin;
    } catch {
        return false;
    }
}

/**
 * Measure the scrollbar width and expose it as a CSS custom property.
 *
 * @returns {void}
 */
export function updateScrollbarSize() {
    const html = document.documentElement;
    const body = document.body;
    // FIXME replace this with JS once CSS container style queries are widely supported
    html.classList.add("calculating-scrollbar-size");
    const scrollbarWidth = window.innerWidth - body.clientWidth;
    document.body.style.setProperty('--scrollbar', `${scrollbarWidth}px`);
    html.classList.remove("calculating-scrollbar-size");
}

/**
 * Build an icon asset path for the given icon name.
 *
 * @param {string} iconName
 * @returns {string}
 */
export function icon(iconName) {
    return `/resources/icons/${iconName}.svg`;
}

/**
 * Helper to escape HTML to prevent XSS in modal content.
 *
 * @param {string | number | boolean | null | undefined} str
 * @returns {string}
 */
export function escapeHtml(str) {
    if (!str) return '';
    return str.toString()
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}

/**
 * Convert a value in rem units to pixels.
 *
 * @param {number} rem
 * @returns {number}
 */
export function remToPx(rem) {
    return rem * parseFloat(getComputedStyle(document.documentElement).fontSize);
}