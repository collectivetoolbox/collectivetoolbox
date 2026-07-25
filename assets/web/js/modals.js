/**
 * Modal Dialog component for Collective Toolbox.
 *
 * Create modals through the ctb.* APIs rather than calling this directly.
 */

import { escapeHtml } from './utilities.js';

/**
 * @typedef {{
 *   backdrop?: HTMLDivElement | HTMLElement,
 *   onClose?: () => void,
 *   closeOnEscape?: boolean,
 *   closeOnOutsideClick?: boolean,
 * }} ModalOptions
 */

/**
 * @typedef {{ destroy: () => void }} FocusTrapHandle
 */

export const modalState = {
    open: false,
};

/** @type {CtbModalHandle[]} */
const modalStack = [];

/**
 * Returns true if any modal is currently open.
 *
 * @returns {boolean}
 */
export function isModalOpen() {
    return modalStack.length > 0;
}

/**
 * Focus trap function to capture keyboard focus within a modal.
 *
 * @param {HTMLElement} element
 * @returns {{ destroy: () => void }}
 */
function trapFocus(element) {
    const focusableSelectors = 'a[href], area[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), iframe, object, embed, [tabindex="0"], [contenteditable]';

    /**
     * @returns {HTMLElement[]}
     */
    const getFocusableElements = () => {
        return /** @type {HTMLElement[]} */ (
            Array.from(element.querySelectorAll(focusableSelectors))
        )
            .filter(el => el.tabIndex !== -1 && el.offsetWidth > 0 && el.offsetHeight > 0);
    };

    /**
     * @param {KeyboardEvent} e
     * @returns {void}
     */
    const handleKeyDown = (e) => {
        if (e.key !== 'Tab') return;

        const focusable = getFocusableElements();
        if (focusable.length === 0) {
            e.preventDefault();
            return;
        }

        const first = focusable[0];
        const last = focusable[focusable.length - 1];

        if (e.shiftKey) {
            if (document.activeElement === first) {
                last.focus();
                e.preventDefault();
            }
        } else {
            if (document.activeElement === last) {
                first.focus();
                e.preventDefault();
            }
        }
    };

    element.addEventListener('keydown', handleKeyDown);

    // Set initial focus
    const focusable = getFocusableElements();
    if (focusable.length > 0) {
        // Focus OK or Cancel button if present, otherwise first focusable element
        const preferredFocus = focusable.find(el => el.hasAttribute('data-modal-ok') || el.hasAttribute('data-modal-cancel')) || focusable[0];
        preferredFocus.focus();
    } else {
        element.setAttribute('tabindex', '-1');
        element.focus();
    }

    return {
        destroy() {
            element.removeEventListener('keydown', handleKeyDown);
        }
    };
}

// Global listeners for closing/interaction
let globalListenersInitialized = false;
/**
 * Initialize global document-level event listeners for modals (e.g. Escape key).
 *
 * @returns {void}
 */
function initGlobalListeners() {
    if (globalListenersInitialized) return;
    globalListenersInitialized = true;

    // Close on Escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && modalStack.length > 0) {
            const topModal = modalStack[modalStack.length - 1];
            if (topModal.closeOnEscape !== false) {
                topModal.close();
                e.preventDefault();
                e.stopPropagation();
            }
        }
    }, true); // Use capture phase
}

/**
 * Core function to open any content element inside a modal backdrop overlay.
 *
 * @param {HTMLElement} contentElement
 * @param {ModalOptions} [options={}]
 * @returns {CtbModalHandle}
 */
export function openModal(contentElement, options = {}) {
    initGlobalListeners();

    const backdrop = options.backdrop || document.createElement('div');
    const isExistingContainer = !!options.backdrop;

    if (!options.backdrop) {
        backdrop.className = 'modal-backdrop';
        document.body.appendChild(backdrop);
    }

    const previousActiveElement = document.activeElement instanceof HTMLElement || document.activeElement instanceof SVGElement
        ? document.activeElement
        : null;

    // Center contentElement within the backdrop as flex container
    backdrop.style.display = 'flex';

    /** @type {Comment | null} */
    let placeholder = null;
    // Append element into backdrop if not already there
    if (contentElement.parentNode !== backdrop) {
        const originalParent = contentElement.parentNode;
        if (originalParent) {
            placeholder = document.createComment('modal-placeholder');
            originalParent.insertBefore(placeholder, contentElement);
        }
        backdrop.appendChild(contentElement);
    }

    // Force show/unhide elements
    const originalDisplay = contentElement.style.display;
    const originalHidden = contentElement.classList.contains('hidden');
    contentElement.classList.remove('hidden');
    contentElement.style.display = '';

    // Transition styles
    backdrop.offsetHeight; // force reflow
    backdrop.classList.add('modal-show');
    if (isExistingContainer) {
        backdrop.classList.remove('hidden');
    }

    // Set standard ARIA attributes
    if (!contentElement.getAttribute('role')) {
        contentElement.setAttribute('role', 'dialog');
    }
    contentElement.setAttribute('aria-modal', 'true');

    const closeOnEscape = options.closeOnEscape !== false;
    const closeOnOutsideClick = options.closeOnOutsideClick !== false;

    /** @type {CtbModalHandle} */
    const modalObj = {
        backdrop,
        content: contentElement,
        previousActiveElement,
        placeholder,
        originalDisplay,
        originalHidden,
        /** @type {FocusTrapHandle | null} */
        focusTrap: null,
        isExistingContainer,
        close: () => {},
        enableFocusTrap: () => {},
        closeOnEscape,
        closeOnOutsideClick,
    };

    const close = () => {
        const index = modalStack.indexOf(modalObj);
        if (index === -1) return;

        modalStack.splice(index, 1);
        modalState.open = modalStack.length > 0;

        backdrop.classList.remove('modal-show');

        if (modalObj.focusTrap) {
            modalObj.focusTrap.destroy();
        }

        const cleanup = () => {
            if (isExistingContainer) {
                backdrop.classList.add('hidden');
                backdrop.style.display = 'none';
                contentElement.style.display = originalDisplay;
                if (originalHidden) {
                    contentElement.classList.add('hidden');
                }
            } else {
                if (placeholder && placeholder.parentNode) {
                    placeholder.parentNode.insertBefore(contentElement, placeholder);
                    placeholder.remove();
                }
                contentElement.style.display = originalDisplay;
                if (originalHidden) {
                    contentElement.classList.add('hidden');
                }
                backdrop.remove();
            }

            if (previousActiveElement && typeof previousActiveElement.focus === 'function') {
                previousActiveElement.focus();
            }

            // Restore focus trap of the next modal in the stack
            if (modalStack.length > 0) {
                modalStack[modalStack.length - 1].enableFocusTrap();
            }

            if (options.onClose) {
                options.onClose();
            }
        };

        // Wait for CSS transition (0.2s = 200ms)
        setTimeout(cleanup, 200);
    };

    modalObj.close = close;

    const enableFocusTrap = () => {
        if (modalObj.focusTrap) modalObj.focusTrap.destroy();
        modalObj.focusTrap = trapFocus(contentElement);
    };
    modalObj.enableFocusTrap = enableFocusTrap;

    // Pause focus trap of the previous top modal
    if (modalStack.length > 0) {
        const prevTop = modalStack[modalStack.length - 1];
        if (prevTop.focusTrap) {
            prevTop.focusTrap.destroy();
            prevTop.focusTrap = null;
        }
    }

    modalStack.push(modalObj);
    modalState.open = true;
    enableFocusTrap();

    // Event listener for elements inside that are meant to close the modal
    const closeButtons = /** @type {HTMLElement[]} */ (
        Array.from(
            contentElement.querySelectorAll('[data-modal-close], .modal-close-btn, .btn-close')
        )
    );
    closeButtons.forEach(btn => {
        // Prevent registering multiple click listeners if called repeatedly
        /** @param {MouseEvent} e */
        btn.onclick = (e) => {
            e.preventDefault();
            close();
        };
    });

    // Close on click outside (only when clicking the backdrop itself)
    backdrop.onclick = (e) => {
        if (e.target === backdrop && modalObj.closeOnOutsideClick !== false) {
            close();
        }
    };

    return modalObj;
}

/**
 * Show a specified element (or element matching selector) inside a modal overlay.
 *
 * @param {string | HTMLElement} param
 * @param {ModalOptions} [options={}]
 * @returns {CtbModalHandle | null}
 */
export function showModal(param, options = {}) {
    const candidate = typeof param === 'string' ? document.querySelector(param) : param;
    if (!(candidate instanceof HTMLElement)) {
        console.warn('showModal: Target element not found.', param);
        return null;
    }
    return openModal(candidate, options);
}

/**
 * Render a modal alert.
 *
 * @param {string} message
 * @param {string} [title='Alert']
 * @returns {Promise<void>}
 */
export function showAlert(message, title = 'Alert') {
    return new Promise((resolve) => {
        const modalId = 'ctb-alert-' + Date.now();
        const modalEl = document.createElement('div');
        modalEl.className = 'modal-box modal-box-glass';
        modalEl.setAttribute('role', 'alertdialog');
        modalEl.setAttribute('aria-labelledby', `${modalId}-title`);
        modalEl.setAttribute('aria-describedby', `${modalId}-body`);

        modalEl.innerHTML = `
            <div class="modal-header">
                <h2 class="modal-title" id="${modalId}-title">${escapeHtml(title)}</h2>
                <button class="modal-close-btn" aria-label="Close" data-modal-close>
                    <img class="icon" src="/resources/icons/close.svg" alt="Close">
                </button>
            </div>
            <div class="modal-body" id="${modalId}-body">
                <p>${escapeHtml(message)}</p>
            </div>
            <div class="modal-footer">
                <button class="btn-primary" data-modal-ok>OK</button>
            </div>
        `;

        const modalObj = openModal(modalEl, {
            onClose: () => resolve(),
        });

        const okBtn = /** @type {HTMLButtonElement | null} */ (
            modalEl.querySelector('[data-modal-ok]')
        );
        if (okBtn) {
            okBtn.onclick = () => {
                modalObj.close();
            };
        }
    });
}

/**
 * Render a modal confirm dialog.
 *
 * @param {string} message
 * @param {string} [title='Confirm']
 * @returns {Promise<boolean>}
 */
export function showConfirm(message, title = 'Confirm') {
    return new Promise((resolve) => {
        const modalId = 'ctb-confirm-' + Date.now();
        const modalEl = document.createElement('div');
        modalEl.className = 'modal-box modal-box-glass';
        modalEl.setAttribute('role', 'alertdialog');
        modalEl.setAttribute('aria-labelledby', `${modalId}-title`);
        modalEl.setAttribute('aria-describedby', `${modalId}-body`);

        modalEl.innerHTML = `
            <div class="modal-header">
                <h2 class="modal-title" id="${modalId}-title">${escapeHtml(title)}</h2>
                <button class="modal-close-btn" aria-label="Close" data-modal-close>
                    <img class="icon" src="/resources/icons/close.svg" alt="Close">
                </button>
            </div>
            <div class="modal-body" id="${modalId}-body">
                <p>${escapeHtml(message)}</p>
            </div>
            <div class="modal-footer">
                <button class="btn-secondary" data-modal-cancel>Cancel</button>
                <button class="btn-primary" data-modal-ok>OK</button>
            </div>
        `;

        let confirmed = false;

        const modalObj = openModal(modalEl, {
            onClose: () => resolve(confirmed),
        });

        const okBtn = /** @type {HTMLButtonElement | null} */ (
            modalEl.querySelector('[data-modal-ok]')
        );
        if (okBtn) {
            okBtn.onclick = () => {
                confirmed = true;
                modalObj.close();
            };
        }

        const cancelBtn = /** @type {HTMLButtonElement | null} */ (
            modalEl.querySelector('[data-modal-cancel]')
        );
        if (cancelBtn) {
            cancelBtn.onclick = () => {
                confirmed = false;
                modalObj.close();
            };
        }
    });
}

/**
 * Render a modal confirm dialog with unescaped HTML content.
 *
 * @param {string} htmlMessage
 * @param {string} [title='Confirm']
 * @returns {Promise<boolean>}
 */
export function showConfirmHtml(htmlMessage, title = 'Confirm') {
    return new Promise((resolve) => {
        const modalId = 'ctb-confirm-' + Date.now();
        const modalEl = document.createElement('div');
        modalEl.className = 'modal-box modal-box-glass';
        modalEl.setAttribute('role', 'alertdialog');
        modalEl.setAttribute('aria-labelledby', `${modalId}-title`);
        modalEl.setAttribute('aria-describedby', `${modalId}-body`);

        modalEl.innerHTML = `
            <div class="modal-header">
                <h2 class="modal-title" id="${modalId}-title">${escapeHtml(title)}</h2>
                <button class="modal-close-btn" aria-label="Close" data-modal-close>
                    <img class="icon" src="/resources/icons/close.svg" alt="Close">
                </button>
            </div>
            <div class="modal-body" id="${modalId}-body">
                <p>${htmlMessage}</p>
            </div>
            <div class="modal-footer">
                <button class="btn-secondary" data-modal-cancel>Cancel</button>
                <button class="btn-primary" data-modal-ok>OK</button>
            </div>
        `;

        let confirmed = false;

        const modalObj = openModal(modalEl, {
            onClose: () => resolve(confirmed),
        });

        const okBtn = /** @type {HTMLButtonElement | null} */ (
            modalEl.querySelector('[data-modal-ok]')
        );
        if (okBtn) {
            okBtn.onclick = () => {
                confirmed = true;
                modalObj.close();
            };
        }

        const cancelBtn = /** @type {HTMLButtonElement | null} */ (
            modalEl.querySelector('[data-modal-cancel]')
        );
        if (cancelBtn) {
            cancelBtn.onclick = () => {
                confirmed = false;
                modalObj.close();
            };
        }
    });
}

/**
 * Render a modal prompt dialog requesting text input.
 *
 * @param {string} message
 * @param {string} [title='Prompt']
 * @param {string} [defaultValue='']
 * @returns {Promise<string | null>}
 */
export function showPrompt(message, title = 'Prompt', defaultValue = '') {
    return new Promise((resolve) => {
        const modalId = 'ctb-prompt-' + Date.now();
        const modalEl = document.createElement('div');
        modalEl.className = 'modal-box modal-box-glass';
        modalEl.setAttribute('role', 'dialog');
        modalEl.setAttribute('aria-labelledby', `${modalId}-title`);
        modalEl.setAttribute('aria-describedby', `${modalId}-body`);

        modalEl.innerHTML = `
            <div class="modal-header">
                <h2 class="modal-title" id="${modalId}-title">${escapeHtml(title)}</h2>
                <button class="modal-close-btn" aria-label="Close" data-modal-close>
                    <img class="icon" src="/resources/icons/close.svg" alt="Close">
                </button>
            </div>
            <div class="modal-body" id="${modalId}-body">
                <p class="mb-2">${escapeHtml(message)}</p>
                <input type="text" class="input-text w-full" id="${modalId}-input" value="${escapeHtml(defaultValue)}">
            </div>
            <div class="modal-footer">
                <button class="btn-secondary" data-modal-cancel>Cancel</button>
                <button class="btn-primary" data-modal-ok>OK</button>
            </div>
        `;

        const inputEl = /** @type {HTMLInputElement} */ (modalEl.querySelector(`#${modalId}-input`));
        /** @type {string | null} */
        let result = null;

        const modalObj = openModal(modalEl, {
            onClose: () => resolve(result),
        });

        const okBtn = /** @type {HTMLButtonElement | null} */ (
            modalEl.querySelector('[data-modal-ok]')
        );
        if (okBtn) {
            okBtn.onclick = () => {
                result = inputEl.value;
                modalObj.close();
            };
        }

        const cancelBtn = /** @type {HTMLButtonElement | null} */ (
            modalEl.querySelector('[data-modal-cancel]')
        );
        if (cancelBtn) {
            cancelBtn.onclick = () => {
                result = null;
                modalObj.close();
            };
        }
    });
}

// ---- Extracted Iframe Modal Controls (Backward Compatibility) ----------------

/**
 * Resolve the legacy iframe modal elements.
 *
 * @returns {{
 *   iframe: HTMLIFrameElement | null,
 *   wrapper: HTMLElement | null,
 *   root: HTMLElement | null,
 * }}
 */
function getModalElements() {
    const iframe = /** @type {HTMLIFrameElement | null} */ (
        document.getElementById("modal-content-frame")
    );
    const wrapper = iframe ? iframe.parentElement : null;
    const root = document.getElementById("root");
    return { iframe, wrapper, root };
}

/**
 * Add embed framing parameters to same-origin docs URLs.
 *
 * @param {string} url
 * @returns {string}
 */
function buildEmbeddedFrameUrl(url) {
    const finalUrl = new URL(url, window.location.href);
    if (
        finalUrl.origin === window.location.origin &&
        finalUrl.pathname.startsWith("/docs")
    ) {
        finalUrl.searchParams.set("embed", "1");
    }
    return finalUrl.toString();
}

/**
 * Remove the embed framing parameter from an iframe URL.
 *
 * @param {string} url
 * @returns {string}
 */
function stripEmbeddedFrameParam(url) {
    const finalUrl = new URL(url, window.location.href);
    finalUrl.searchParams.delete("embed");
    return finalUrl.toString();
}

/**
 * Open a legacy iframe modal for the given URL.
 *
 * @param {string} url
 * @returns {void}
 */
export function modalOpen(url) {
    const { iframe, wrapper, root } = getModalElements();
    const backdrop = document.getElementById("modal-content-frame-backdrop");
    if (!iframe || !wrapper || !root || !backdrop) return;

    iframe.src = buildEmbeddedFrameUrl(url);
    iframe.scrollIntoView({ behavior: "smooth" });
    root.style.overflow = "hidden";

    // Open existing container using robust manager
    openModal(wrapper, {
        backdrop: backdrop,
        onClose: () => {
            iframe.src = "about:blank";
            root.style.overflow = "auto";
            modalState.open = isModalOpen();
        }
    });
}

/**
 * Close the currently active legacy iframe modal, if present.
 *
 * @returns {void}
 */
export function modalContentClose() {
    const { wrapper } = getModalElements();
    if (!wrapper) return;
    const active = modalStack.find(m => m.content === wrapper);
    if (active) {
        active.close();
    }
}

/**
 * Open the current iframe modal content in a separate tab.
 *
 * @returns {void}
 */
export function modalContentPopout() {
    const { iframe } = getModalElements();
    if (iframe && iframe.src && iframe.src !== "about:blank") {
        let currentUrl = iframe.src;
        try {
            if (iframe.contentWindow && iframe.contentWindow.location) {
                currentUrl = iframe.contentWindow.location.href;
            }
        } catch (_e) {
            // Ignore cross-origin errors
        }
        window.open(stripEmbeddedFrameParam(currentUrl), "_blank");
    }
    modalContentClose();
}
