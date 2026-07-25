/**
 * Configuration and behavior definition for the Simple theme.
 *
 * @type {{
 *   name: string,
 *   cssPath: string,
 *   init: (container: HTMLElement) => void
 * }}
 */
export const themeConfig = {
    name: "simple",
    cssPath: "/js/components/themes/simple/button.css",

    /**
     * Initializes any theme-specific DOM overlay elements.
     *
     * @param {HTMLElement} _container The button container element inside shadow DOM.
     * @returns {void}
     */
    init(_container) {
        // Simple theme has no extra DOM overlays
    },
};
