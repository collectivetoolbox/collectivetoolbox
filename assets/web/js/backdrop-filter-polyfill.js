/**
 * Backdrop Filter Polyfill for Blink browsers.
 * Bypasses the strict W3C backdrop root boundary specification in Chromium.
 * Moves backdrop-filter and background styles to absolute-positioned child helper elements.
 */

// Detect Blink engine (Chrome, Edge, Opera, etc. excluding iOS Chrome which uses WebKit)
const isBlink = /Chrome|Chromium|Edg|OPR/.test(navigator.userAgent) && !/CriOS/.test(navigator.userAgent);

if (isBlink) {
    document.documentElement.setAttribute('data-is-blink', 'true');
    // At the moment the issues seem to just be cosmetic, so not bothering to warn visibly for now.
    // ctb.warn("There are some known issues with Blink browsers.")
}

// /**
//  * Synchronize backdrop-filter and background styling from element to a child helper element.
//  *
//  * @param {HTMLElement} el
//  * @returns {void}
//  */
// export function syncElement(el) {
//     /** @type {HTMLDivElement | null} */
//     const helper = el.querySelector(':scope > .backdrop-filter-helper');
//     const elStyle = /** @type {any} */ (el.style);

//     // Temporarily remove the inline overrides if they exist so we can read the stylesheet computed style
//     const hasInlineOverride = elStyle.backdropFilter === 'none';
//     if (hasInlineOverride) {
//         elStyle.backdropFilter = '';
//         elStyle.webkitBackdropFilter = '';
//         elStyle.background = '';
//     }

//     const style = /** @type {any} */ (window.getComputedStyle(el));
//     const backdropFilter = style.backdropFilter || style.webkitBackdropFilter;

//     if (backdropFilter && backdropFilter !== 'none') {
//         let activeHelper = helper;
//         if (!activeHelper) {
//             activeHelper = document.createElement('div');
//             activeHelper.className = 'backdrop-filter-helper';

//             const helperStyle = /** @type {any} */ (activeHelper.style);

//             // Set styles to absolutely position helper to cover parent
//             helperStyle.position = 'absolute';
//             helperStyle.top = '0';
//             helperStyle.left = '0';
//             helperStyle.right = '0';
//             helperStyle.bottom = '0';
//             helperStyle.zIndex = '-1';
//             helperStyle.pointerEvents = 'none';

//             // Ensure parent has a positioning context and creates a stacking context (without isolation: isolate)
//             if (style.position === 'static') {
//                 el.dataset.originalPosition = 'static';
//                 elStyle.position = 'relative';
//             }
//             if (style.zIndex === 'auto') {
//                 el.dataset.originalZIndex = 'auto';
//                 elStyle.zIndex = '0';
//             }

//             el.insertBefore(activeHelper, el.firstChild);
//         }

//         const activeHelperStyle = /** @type {any} */ (activeHelper.style);

//         // Sync backdrop filter styles
//         activeHelperStyle.backdropFilter = style.backdropFilter;
//         activeHelperStyle.webkitBackdropFilter = style.webkitBackdropFilter;

//         // Sync parent background to helper so it is layered ON TOP of the filter
//         activeHelperStyle.backgroundColor = style.backgroundColor;
//         activeHelperStyle.backgroundImage = style.backgroundImage;
//         activeHelperStyle.backgroundPosition = style.backgroundPosition;
//         activeHelperStyle.backgroundSize = style.backgroundSize;
//         activeHelperStyle.backgroundRepeat = style.backgroundRepeat;
//         activeHelperStyle.backgroundOrigin = style.backgroundOrigin;
//         activeHelperStyle.backgroundClip = style.backgroundClip;
//         activeHelperStyle.backgroundAttachment = style.backgroundAttachment;

//         // Sync border radius and overflow to clip correctly
//         activeHelperStyle.borderTopLeftRadius = style.borderTopLeftRadius;
//         activeHelperStyle.borderTopRightRadius = style.borderTopRightRadius;
//         activeHelperStyle.borderBottomLeftRadius = style.borderBottomLeftRadius;
//         activeHelperStyle.borderBottomRightRadius = style.borderBottomRightRadius;
//         activeHelperStyle.overflow = style.overflow;

//         // Clean up legacy isolation attribute from previous polyfill versions if present
//         if (el.dataset.originalIsolation) {
//             elStyle.isolation = el.dataset.originalIsolation === 'auto' ? '' : el.dataset.originalIsolation;
//             el.removeAttribute('data-original-isolation');
//         }

//         // Set inline overrides on the parent to remove its backdrop root status and background
//         elStyle.backdropFilter = 'none';
//         elStyle.webkitBackdropFilter = 'none';
//         elStyle.background = 'transparent';
//         el.dataset.backdropFilterPolyfillApplied = 'true';
//     } else {
//         // No longer has backdrop filter, clean up helper if it exists
//         if (helper) {
//             helper.remove();
//         }
//         if (el.dataset.originalPosition === 'static') {
//             elStyle.position = '';
//             el.removeAttribute('data-original-position');
//         }
//         if (el.dataset.originalZIndex === 'auto') {
//             elStyle.zIndex = '';
//             el.removeAttribute('data-original-z-index');
//         }
//         if (el.dataset.originalIsolation) {
//             elStyle.isolation = el.dataset.originalIsolation === 'auto' ? '' : el.dataset.originalIsolation;
//             el.removeAttribute('data-original-isolation');
//         }
//         el.removeAttribute('data-backdrop-filter-polyfill-applied');
//     }
// }

// /**
//  * Scan all descendants of a root element and sync their polyfill state.
//  *
//  * @param {HTMLElement} root
//  * @returns {void}
//  */
// export function scanAndSync(root) {
//     if (!isBlink) return;

//     const elements = root.querySelectorAll('*');
//     const allElements = [root, ...elements];

//     for (const el of allElements) {
//         if (!(el instanceof HTMLElement)) continue;
//         if (el.classList.contains('backdrop-filter-helper')) continue;
//         if (el.tagName.toLowerCase().includes('-')) continue;
//         if (el.closest('ctb-button')) continue;

//         /** @type {HTMLDivElement | null} */
//         const hasHelper = el.querySelector(':scope > .backdrop-filter-helper');

//         const elStyle = /** @type {any} */ (el.style);
//         const hasInlineOverride = elStyle.backdropFilter === 'none';
//         if (hasInlineOverride) {
//             elStyle.backdropFilter = '';
//             elStyle.webkitBackdropFilter = '';
//             elStyle.background = '';
//         }

//         const style = /** @type {any} */ (window.getComputedStyle(el));
//         const hasFilterVal = (style.backdropFilter && style.backdropFilter !== 'none') ||
//                              (style.webkitBackdropFilter && style.webkitBackdropFilter !== 'none');

//         if (hasFilterVal || hasHelper) {
//             syncElement(el);
//         } else if (hasInlineOverride) {
//             elStyle.backdropFilter = '';
//             elStyle.webkitBackdropFilter = '';
//             elStyle.background = '';
//         }
//     }
// }

// // Set up MutationObserver to sync elements dynamically as the page changes (SPA navigation/modals)
// const observerConfig = {
//     childList: true,
//     subtree: true,
//     attributes: true,
//     attributeFilter: ['class', 'style']
// };

// const observer = new MutationObserver((mutations) => {
//     if (!isBlink) return;

//     observer.disconnect();
//     try {
//         /** @type {Set<HTMLElement>} */
//         const elementsToCheck = new Set();

//         for (const mutation of mutations) {
//             if (mutation.type === 'childList') {
//                 for (const node of mutation.addedNodes) {
//                     if (node.nodeType === Node.ELEMENT_NODE) {
//                         const el = /** @type {HTMLElement} */ (node);
//                         elementsToCheck.add(el);
//                         const descendants = el.querySelectorAll('*');
//                         for (const desc of descendants) {
//                             if (desc instanceof HTMLElement) {
//                                 elementsToCheck.add(desc);
//                             }
//                         }
//                     }
//                 }
//             } else if (mutation.type === 'attributes' && mutation.target instanceof HTMLElement) {
//                 elementsToCheck.add(mutation.target);
//             }
//         }

//         for (const el of elementsToCheck) {
//             if (el.classList.contains('backdrop-filter-helper')) continue;
//             if (el.tagName.toLowerCase().includes('-')) continue;
//             if (el.closest('ctb-button')) continue;

//             /** @type {HTMLDivElement | null} */
//             const hasHelper = el.querySelector(':scope > .backdrop-filter-helper');

//             const elStyle = /** @type {any} */ (el.style);
//             const hasInlineOverride = elStyle.backdropFilter === 'none';
//             if (hasInlineOverride) {
//                 elStyle.backdropFilter = '';
//                 elStyle.webkitBackdropFilter = '';
//                 elStyle.background = '';
//             }

//             const style = /** @type {any} */ (window.getComputedStyle(el));
//             const hasFilterVal = (style.backdropFilter && style.backdropFilter !== 'none') ||
//                                  (style.webkitBackdropFilter && style.webkitBackdropFilter !== 'none');

//             if (hasFilterVal || hasHelper) {
//                 syncElement(el);
//             } else if (hasInlineOverride) {
//                 elStyle.backdropFilter = '';
//                 elStyle.webkitBackdropFilter = '';
//                 elStyle.background = '';
//             }
//         }
//     } finally {
//         observer.observe(document.body, observerConfig);
//     }
// });

// // Initialize polyfill when DOM is ready
// if (isBlink) {
//     if (document.readyState === 'loading') {
//         document.addEventListener('DOMContentLoaded', () => {
//             scanAndSync(document.body);
//             observer.observe(document.body, observerConfig);
//         });
//     } else {
//         scanAndSync(document.body);
//         observer.observe(document.body, observerConfig);
//     }
// }
