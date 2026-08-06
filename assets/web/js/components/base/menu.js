/**
 * Custom Element representing a reusable, themeable popup/context/collapsible menu container.
 * Contains multiple buttons/links (upgraded to ctb-button).
 */
export class CtbMenu extends HTMLElement {
    /** @type {MutationObserver | null} */
    #themeObserver = null;

    /**
     * Menu click handler to close menu after selecting an option.
     *
     * @param {Event} event
     * @returns {void}
     */
    #onMenuClick = (event) => {
        const target = /** @type {HTMLElement | null} */ (event.target);
        if (target && target.closest("a, button, ctb-button")) {
            /** @type {any} */
            const dropdownParent = this.closest("ctb-dropdown");
            if (dropdownParent && typeof dropdownParent.close === "function") {
                dropdownParent.close();
            } else if (this.isOpen) {
                this.close();
            }
        }
    };

    /**
     * Observed attributes list.
     *
     * @returns {string[]}
     */
    static get observedAttributes() {
        return ["open", "theme"];
    }

    /**
     * Creates an instance of CtbMenu.
     */
    constructor() {
        super();
    }

    /**
     * Connected callback lifecycle hook.
     *
     * @returns {void}
     */
    connectedCallback() {
        this.setAttribute("role", "menu");
        this.addEventListener("click", this.#onMenuClick);
        this.#syncTheme();
        this.#setupThemeObserver();
    }

    /**
     * Disconnected callback lifecycle hook.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        this.removeEventListener("click", this.#onMenuClick);
        if (this.#themeObserver) {
            this.#themeObserver.disconnect();
            this.#themeObserver = null;
        }
    }

    /**
     * Attribute changed callback.
     *
     * @param {string} name
     * @param {string | null} oldValue
     * @param {string | null} newValue
     * @returns {void}
     */
    attributeChangedCallback(name, oldValue, newValue) {
        if (oldValue !== newValue) {
            if (name === "theme") {
                this.#applyThemeToChildren(newValue);
            }
        }
    }

    /**
     * Returns true if menu is open.
     *
     * @returns {boolean}
     */
    get isOpen() {
        return this.hasAttribute("open");
    }

    /**
     * Opens the menu.
     *
     * @returns {void}
     */
    open() {
        if (!this.isOpen) {
            this.setAttribute("open", "");
        }
    }

    /**
     * Closes the menu.
     *
     * @returns {void}
     */
    close() {
        if (this.isOpen) {
            this.removeAttribute("open");
        }
    }

    /**
     * Toggles the menu state.
     *
     * @returns {void}
     */
    toggle() {
        if (this.isOpen) {
            this.close();
        } else {
            this.open();
        }
    }

    /**
     * Syncs theme from document.
     *
     * @returns {void}
     */
    #syncTheme() {
        const theme = document.documentElement.getAttribute("data-ctb-ui-theme") || "none";
        this.setAttribute("theme", theme);
        this.#applyThemeToChildren(theme);
    }

    /**
     * Sets up observer for document theme changes.
     *
     * @returns {void}
     */
    #setupThemeObserver() {
        if (this.#themeObserver) return;
        this.#themeObserver = new MutationObserver((mutations) => {
            for (const mutation of mutations) {
                if (
                    mutation.type === "attributes" &&
                    mutation.attributeName === "data-ctb-ui-theme"
                ) {
                    this.#syncTheme();
                }
            }
        });
        this.#themeObserver.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["data-ctb-ui-theme"]
        });
    }

    /**
     * Applies theme attribute to child ctb-button elements.
     *
     * @param {string | null} theme
     * @returns {void}
     */
    #applyThemeToChildren(theme) {
        if (!theme) return;
        const buttons = this.querySelectorAll("ctb-button");
        for (const btn of buttons) {
            btn.setAttribute("theme", theme);
        }
    }
}

customElements.define("ctb-menu", CtbMenu);
