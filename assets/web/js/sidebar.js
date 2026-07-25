/**
 * Sidebar interaction logic
 */
import { $$ } from './utilities.js';

/**
 * Initialize the sidebar toggle button and hidden state.
 *
 * @returns {void}
 */
export function setupSidebarToggle() {
    const toggle = $$('[href="#sidebar"]')[0];
    const sidebar = document.getElementById("sidebar");
    if (!toggle || !sidebar) return;

    const removeFragment = () => {
        if (window.location.hash === "#sidebar") {
            history.replaceState(
                null,
                "",
                window.location.pathname + window.location.search
            );
        }
    };

    sidebar.classList.add("sidebar-js-enabled");
    sidebar.style.display = "none";
    sidebar.style.transform = "translateX(110%)";
    sidebar.style.transition = sidebar.style.transition || "transform 200ms ease-in-out";
    sidebar.style.willChange = "transform";
    sidebar.setAttribute("aria-hidden", "true");
    toggle.setAttribute("aria-expanded", "false");
    sidebar.classList.remove("sidebar-open");

    removeFragment();
}

/**
 * Open or close the sidebar drawer.
 *
 * @param {boolean} v
 * @returns {void}
 */
export function sidebarSetOpen(v) {
    const toggle = $$('[href="#sidebar"]')[0];
    const sidebar = document.getElementById("sidebar");
    if (!toggle || !sidebar) return;

    const open = !!v;
    if (open) {
        sidebar.style.display = "block";
        window.setTimeout(() => {
            sidebar.style.transform = "translateX(0)";
            sidebar.setAttribute("aria-hidden", "false");
            toggle.setAttribute("aria-expanded", "true");
            sidebar.classList.add("sidebar-open");
        }, 10);
    } else {
        sidebar.style.transform = "translateX(110%)";
        sidebar.setAttribute("aria-hidden", "true");
        toggle.setAttribute("aria-expanded", "false");
        sidebar.classList.remove("sidebar-open");
        window.setTimeout(() => {
            sidebar.style.display = "none";
        }, 200);
    }
}
