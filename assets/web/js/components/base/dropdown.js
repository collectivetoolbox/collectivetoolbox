/**
 * Custom Element representing a reusable, themeable dropdown menu composite widget.
 * Wraps a trigger button and a collapsible menu containing multiple ctb-buttons.
 * Automatically closes on click outside or when pressing Escape.
 */
export class CtbDropdown extends HTMLElement {
    /** @type {HTMLElement | null} */
    #triggerEl = null;

    /** @type {HTMLElement | null} */
    #menuEl = null;

    /** @type {MutationObserver | null} */
    #themeObserver = null;

    /**
     * Document pointerdown listener for closing on outside click.
     *
     * @param {PointerEvent | MouseEvent} event
     * @returns {void}
     */
    #onDocumentPointerDown = (event) => {
        if (!this.isOpen) return;
        const target = /** @type {Node | null} */ (event.target);
        if (target && !this.contains(target)) {
            this.close();
        }
    };

    /**
     * Document keydown listener for closing on Escape.
     *
     * @param {KeyboardEvent} event
     * @returns {void}
     */
    #onDocumentKeyDown = (event) => {
        if (event.key === "Escape" && this.isOpen) {
            this.close();
        }
    };

    /**
     * Trigger click handler.
     *
     * @param {Event} event
     * @returns {void}
     */
    #onTriggerClick = (event) => {
        event.stopPropagation();
        this.toggle();
    };

    /**
     * Menu click handler to close menu after selecting an option.
     *
     * @param {Event} event
     * @returns {void}
     */
    #onMenuClick = (event) => {
        const target = /** @type {HTMLElement | null} */ (event.target);
        if (target && target.closest("a, button, ctb-button")) {
            this.close();
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
     * Creates an instance of CtbDropdown.
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
        this.#initStructure();
        this.#setupEvents();
        this.#syncTheme();
        this.#setupThemeObserver();
    }

    /**
     * Disconnected callback lifecycle hook.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        document.removeEventListener("pointerdown", this.#onDocumentPointerDown);
        document.removeEventListener("keydown", this.#onDocumentKeyDown);

        if (this.#triggerEl) {
            this.#triggerEl.removeEventListener("click", this.#onTriggerClick);
        }
        if (this.#menuEl) {
            this.#menuEl.removeEventListener("click", this.#onMenuClick);
        }
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
            if (name === "open") {
                this.#updateState();
            } else if (name === "theme") {
                this.#applyThemeToChildren(newValue);
            }
        }
    }

    /**
     * Returns true if dropdown is open.
     *
     * @returns {boolean}
     */
    get isOpen() {
        return this.hasAttribute("open");
    }

    /**
     * Opens the dropdown menu.
     *
     * @returns {void}
     */
    open() {
        if (!this.isOpen) {
            this.setAttribute("open", "");
        }
    }

    /**
     * Closes the dropdown menu.
     *
     * @returns {void}
     */
    close() {
        if (this.isOpen) {
            this.removeAttribute("open");
        }
    }

    /**
     * Toggles the dropdown menu open/closed state.
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
     * Initializes component layout structure and trigger/menu elements.
     *
     * @returns {void}
     */
    #initStructure() {
        // Locate trigger element (explicit or first button/anchor/ctb-button)
        this.#triggerEl = /** @type {HTMLElement | null} */ (
            this.querySelector(".ctb-dropdown-trigger, [data-ctb-dropdown-trigger]") ||
            this.firstElementChild
        );

        if (this.#triggerEl) {
            this.#triggerEl.setAttribute("aria-haspopup", "true");
            this.#triggerEl.setAttribute("aria-expanded", this.isOpen ? "true" : "false");
        }

        // Locate menu container
        this.#menuEl = /** @type {HTMLElement | null} */ (
            this.querySelector("ctb-menu, .ctb-dropdown-menu, [data-ctb-dropdown-menu]")
        );

        if (this.#menuEl) {
            this.#menuEl.setAttribute("role", "menu");
        }
    }

    /**
     * Binds event listeners.
     *
     * @returns {void}
     */
    #setupEvents() {
        if (this.#triggerEl) {
            this.#triggerEl.addEventListener("click", this.#onTriggerClick);
        }
        if (this.#menuEl) {
            this.#menuEl.addEventListener("click", this.#onMenuClick);
        }
        document.addEventListener("pointerdown", this.#onDocumentPointerDown);
        document.addEventListener("keydown", this.#onDocumentKeyDown);
    }

    /**
     * Updates internal state and aria attributes.
     *
     * @returns {void}
     */
    #updateState() {
        const isExpanded = this.isOpen;
        if (this.#triggerEl) {
            this.#triggerEl.setAttribute("aria-expanded", isExpanded ? "true" : "false");
        }
        if (this.#menuEl) {
            if (isExpanded) {
                this.#menuEl.setAttribute("open", "");
            } else {
                this.#menuEl.removeAttribute("open");
            }
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

customElements.define("ctb-dropdown", CtbDropdown);
