/**
 * Notifications component logic for Collective Toolbox
 */

import { $ } from './utilities.js';

/**
 * Programmatically show a toast notification.
 * 
 * @param {string} message The message to show in the toast
 * @param {CtbNoticeLevel} [level='info'] The level ('debug', 'info', 'warning', 'error')
 * @param {number|null} timeoutMs Optional custom timeout. If null, uses defaults. 0 means persistent.
 * @returns {HTMLElement|null} The created toast element
 */
export function showToast(message, level = 'info', timeoutMs = null) {
    const template = /** @type {HTMLTemplateElement | null} */ (
        document.getElementById('toast-template')
    );
    const container = document.getElementById('toast-container');
    if (!template || !container) {
        console.warn('Toast templates or container not found in DOM.');
        return null;
    }

    const clone = template.content.cloneNode(true);
    const toast = clone instanceof DocumentFragment
        ? /** @type {HTMLElement | null} */ (clone.querySelector('.toast'))
        : null;
    if (!toast) return null;

    // Set level class
    toast.classList.add(`toast-${level}`);

    // Set dynamic ARIA attributes based on level
    if (level === 'error') {
        toast.setAttribute('role', 'alert');
        toast.setAttribute('aria-live', 'assertive');
    } else {
        toast.setAttribute('role', 'status');
        toast.setAttribute('aria-live', 'polite');
    }

    // Set icon src
    const iconImg = /** @type {HTMLImageElement | null} */ (
        toast.querySelector('.toast-icon img')
    );
    if (iconImg) {
        iconImg.src = `/resources/icons/${level}.svg`;
        iconImg.alt = level;
    }

    // Set message
    const content = toast.querySelector('.toast-content');
    if (content) {
        content.textContent = message;
    }

    // Append to container
    container.appendChild(toast);

    // Force reflow to ensure the transition animations trigger
    toast.offsetHeight;
    toast.classList.add('toast-show');

    // Determine timeout
    let timeout = timeoutMs;
    if (timeout === null) {
        if (level === 'debug') timeout = 2000;
        else if (level === 'info') timeout = 2000;
        else if (level === 'warning') timeout = 2000;
        else if (level === 'error') timeout = 4000;
        else timeout = 2000;
    }

    if (timeout > 0) {
        const totalTimeout = timeout;
        const existingProgress = /** @type {HTMLDivElement | null} */ (
            toast.querySelector('.toast-progress')
        );
        const progressLine = existingProgress || document.createElement('div');
        if (!existingProgress) {
            progressLine.className = 'toast-progress';
            toast.appendChild(progressLine);
        }

        let timeLeft = totalTimeout;
        let lastTime = performance.now();
        let isHovered = false;

        /**
         * @param {DOMHighResTimeStamp} timestamp
         * @returns {void}
         */
        const tick = (timestamp) => {
            if (toast.classList.contains('toast-hide') || !document.body.contains(toast)) {
                return;
            }
            const elapsed = timestamp - lastTime;
            lastTime = timestamp;

            if (!isHovered) {
                timeLeft -= elapsed;
                if (timeLeft <= 0) {
                    timeLeft = 0;
                    progressLine.style.transform = 'scaleX(0)';
                    dismissToast(toast);
                    return;
                }
                const percentage = Math.max(0, Math.min(1, timeLeft / totalTimeout));
                progressLine.style.transform = `scaleX(${percentage})`;
            }

            requestAnimationFrame(tick);
        };

        toast.addEventListener('mouseenter', () => {
            isHovered = true;
        });

        toast.addEventListener('mouseleave', () => {
            isHovered = false;
            lastTime = performance.now();
        });

        requestAnimationFrame((timestamp) => {
            lastTime = timestamp;
            tick(timestamp);
        });
    } else {
        const progressLine = toast.querySelector('.toast-progress');
        if (progressLine) {
            progressLine.remove();
        }
    }

    return toast;
}

/**
 * Close/dismiss a toast notification.
 * 
 * @param {HTMLElement} toast The toast element to dismiss
 * @returns {void}
 */
export function dismissToast(toast) {
    if (!toast || toast.classList.contains('toast-hide')) return;
    toast.classList.remove('toast-show');
    toast.classList.add('toast-hide');
    setTimeout(() => {
        toast.remove();
    }, 300);
}

/**
 * Programmatically create a dismissible alert banner.
 * 
 * @param {string} message Message content (HTML allowed)
 * @param {CtbNoticeLevel} [level='info'] Alert level ('debug', 'info', 'warning', 'error')
 * @param {string|HTMLElement|null} containerSelector Target container to prepend to
 * @param {boolean} dismissible Whether to include a close button
 * @returns {HTMLElement|null} The created alert banner element
 */
export function showAlertBanner(message, level = 'info', containerSelector = null, dismissible = true) {
    const template = /** @type {HTMLTemplateElement | null} */ (
        document.getElementById('alert-banner-template')
    );
    if (!template) {
        console.warn('Alert banner template not found in DOM.');
        return null;
    }

    let $container;
    if (containerSelector) {
        $container = $(/** @type {any} */ (containerSelector));
    } else {
        $container = $('[role="main"]');
        if (!$container.length) {
            $container = $(document.body);
        }
    }

    const clone = template.content.cloneNode(true);
    const alertEl = clone instanceof DocumentFragment
        ? /** @type {HTMLElement | null} */ (clone.querySelector('.alert-banner'))
        : null;
    if (!alertEl) return null;

    // Set level class
    alertEl.classList.add(`alert-banner-${level}`);

    // Set dynamic ARIA attributes based on level
    if (level === 'error') {
        alertEl.setAttribute('role', 'alert');
        alertEl.setAttribute('aria-live', 'assertive');
    } else {
        alertEl.setAttribute('role', 'status');
        alertEl.setAttribute('aria-live', 'polite');
    }

    // Set icon src
    const iconImg = /** @type {HTMLImageElement | null} */ (
        alertEl.querySelector('.alert-banner-icon img')
    );
    if (iconImg) {
        iconImg.src = `/resources/icons/${level}.svg`;
        iconImg.alt = level;
    }

    // Set content (supports HTML)
    const content = alertEl.querySelector('.alert-banner-content');
    if (content) {
        content.innerHTML = message;
    }

    // Handle dismissibility
    if (!dismissible) {
        const dismissBtn = alertEl.querySelector('.alert-banner-dismiss');
        if (dismissBtn) dismissBtn.remove();
    }

    // Prepend to target container
    $container.prepend(alertEl);

    if (window.ctb && typeof window.ctb.upgradeButtons === 'function') {
        window.ctb.upgradeButtons(alertEl);
    }

    return alertEl;
}

/**
 * Close/dismiss an alert banner.
 * 
 * @param {HTMLElement} alertEl The alert banner element to dismiss
 * @returns {void}
 */
export function dismissAlertBanner(alertEl) {
    if (!alertEl || alertEl.classList.contains('alert-banner-hide')) return;
    alertEl.classList.add('alert-banner-hide');
    setTimeout(() => {
        alertEl.remove();
    }, 300);
}

// Global delegated listener for close/dismiss clicks
document.addEventListener('click', (event) => {
    const target = event.target;
    if (!(target instanceof Element)) {
        return;
    }

    // Toast Close click
    const toastClose = target.closest('.toast-close');
    if (toastClose) {
        const toast = toastClose.closest('.toast');
        if (toast instanceof HTMLElement) {
            dismissToast(toast);
        }
        return;
    }

    // Alert Banner Dismiss click
    const alertDismiss = target.closest('.alert-banner-dismiss');
    if (alertDismiss) {
        const alertEl = alertDismiss.closest('.alert-banner');
        if (alertEl instanceof HTMLElement) {
            dismissAlertBanner(alertEl);
        }
        return;
    }
});
