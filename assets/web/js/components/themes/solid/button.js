/**
 * Configuration and behavior definition for the Solid theme.
 *
 * @type {{
 *   name: string,
 *   cssPath: string,
 *   init: (container: HTMLElement) => void
 * }}
 */
export const themeConfig = {
    name: "solid",
    cssPath: "/js/components/themes/solid/button.css",

    /**
     * Initializes any theme-specific DOM overlay elements.
     *
     * @param {HTMLElement} _container The button container element inside shadow DOM.
     * @returns {void}
     */
    init(_container) {
        // Solid theme has no extra DOM overlays
    },
};
