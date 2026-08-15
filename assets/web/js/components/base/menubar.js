/**
 * Custom Element representing a desktop-style application menu bar (`<ctb-menubar>`).
 * Contains top-level buttons or dropdown menus (`<ctb-dropdown>`).
 * Supports desktop-style hover-switching when active and keyboard navigation.
 */
export class CtbMenubar extends HTMLElement {
    /** @type {MutationObserver | null} */
    #themeObserver = null;

    /** @type {boolean} */
    #isMenuActivated = false;

    /**
     * Document pointerdown listener to deactivate menu bar when clicking outside.
     *
     * @param {PointerEvent | MouseEvent} event
     * @returns {void}
     */
    #onDocumentPointerDown = (event) => {
        const target = /** @type {Node | null} */ (event.target);
        if (target && !this.contains(target)) {
            this.#isMenuActivated = false;
        }
    };

    /**
     * Mouseover listener on the menubar.
     * When one dropdown is open, hovering over another top-level dropdown switches to it.
     *
     * @param {MouseEvent} event
     * @returns {void}
     */
    #onMouseOver = (event) => {
        if (!this.#isMenuActivated) return;

        const target = /** @type {HTMLElement | null} */ (event.target);
        if (!target) return;

        const targetDropdown = /** @type {any} */ (target.closest("ctb-dropdown"));
        if (targetDropdown && targetDropdown.parentElement === this) {
            if (!targetDropdown.isOpen) {
                this.#closeAllDropdowns(targetDropdown);
                if (typeof targetDropdown.open === "function") {
                    targetDropdown.open();
                }
            }
        }
    };

    /**
     * Click listener on the menubar to track activation state.
     *
     * @param {MouseEvent} event
     * @returns {void}
     */
    #onClick = (event) => {
        const target = /** @type {HTMLElement | null} */ (event.target);
        if (!target) return;

        const targetDropdown = /** @type {any} */ (target.closest("ctb-dropdown"));
        if (targetDropdown && targetDropdown.parentElement === this) {
            // Check state after click event cycle
            setTimeout(() => {
                this.#isMenuActivated = this.hasOpenDropdown;
            }, 0);
        }
    };

    /**
     * Keydown listener for keyboard navigation across the menu bar.
     *
     * @param {KeyboardEvent} event
     * @returns {void}
     */
    #onKeyDown = (event) => {
        const items = this.#getTopLevelItems();
        if (items.length === 0) return;

        const activeIndex = items.findIndex((item) =>
            item === document.activeElement || item.contains(document.activeElement)
        );

        if (event.key === "ArrowRight") {
            event.preventDefault();
            const nextIndex = activeIndex < 0 ? 0 : (activeIndex + 1) % items.length;
            this.#focusAndSwitchItem(items, nextIndex);
        } else if (event.key === "ArrowLeft") {
            event.preventDefault();
            const prevIndex = activeIndex <= 0 ? items.length - 1 : activeIndex - 1;
            this.#focusAndSwitchItem(items, prevIndex);
        } else if (event.key === "ArrowDown") {
            if (activeIndex >= 0) {
                const item = items[activeIndex];
                /** @type {any} */
                const dropdown = item.closest("ctb-dropdown");
                if (dropdown && typeof dropdown.open === "function") {
                    event.preventDefault();
                    dropdown.open();
                    this.#isMenuActivated = true;
                    const firstMenuBtn = dropdown.querySelector("ctb-menu button, ctb-menu a");
                    if (firstMenuBtn instanceof HTMLElement) {
                        firstMenuBtn.focus();
                    }
                }
            }
        } else if (event.key === "Escape") {
            if (this.hasOpenDropdown) {
                event.preventDefault();
                this.#closeAllDropdowns();
                this.#isMenuActivated = false;
                if (activeIndex >= 0) {
                    const trigger = items[activeIndex].querySelector("button, a") || items[activeIndex];
                    if (trigger instanceof HTMLElement) {
                        trigger.focus();
                    }
                }
            }
        }
    };

    /**
     * Closes all dropdowns in the menubar except optionally an excluded one.
     *
     * @param {HTMLElement | null} [excludeDropdown=null]
     * @returns {void}
     */
    #closeAllDropdowns(excludeDropdown = null) {
        const dropdowns = this.querySelectorAll("ctb-dropdown");
        for (const dd of dropdowns) {
            /** @type {any} */
            const dropdown = dd;
            if (dropdown !== excludeDropdown && typeof dropdown.close === "function") {
                dropdown.close();
            }
        }
    }

    /**
     * Gets all top-level interactive items directly in the menubar.
     *
     * @returns {HTMLElement[]}
     */
    #getTopLevelItems() {
        /** @type {HTMLElement[]} */
        const items = [];
        for (const child of this.children) {
            if (child.tagName === "CTB-DROPDOWN" || child.tagName === "CTB-BUTTON" || child.tagName === "BUTTON" || child.tagName === "A") {
                items.push(/** @type {HTMLElement} */ (child));
            }
        }
        return items;
    }

    /**
     * Focuses and optionally opens an item at the target index.
     *
     * @param {HTMLElement[]} items
     * @param {number} index
     * @returns {void}
     */
    #focusAndSwitchItem(items, index) {
        const item = items[index];
        if (!item) return;

        const trigger = item.querySelector("button, a") || item;
        if (trigger instanceof HTMLElement) {
            trigger.focus();
        }

        if (this.#isMenuActivated) {
            this.#closeAllDropdowns();
            /** @type {any} */
            const dropdown = item.closest("ctb-dropdown");
            if (dropdown && typeof dropdown.open === "function") {
                dropdown.open();
            }
        }
    }

    /**
     * Observed attributes list.
     *
     * @returns {string[]}
     */
    static get observedAttributes() {
        return ["theme"];
    }

    /**
     * Creates an instance of CtbMenubar.
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
        if (!this.hasAttribute("role")) {
            this.setAttribute("role", "menubar");
        }

        this.addEventListener("mouseover", this.#onMouseOver);
        this.addEventListener("click", this.#onClick);
        this.addEventListener("keydown", this.#onKeyDown);
        document.addEventListener("pointerdown", this.#onDocumentPointerDown);

        this.#syncTheme();
        this.#setupThemeObserver();
    }

    /**
     * Disconnected callback lifecycle hook.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        this.removeEventListener("mouseover", this.#onMouseOver);
        this.removeEventListener("click", this.#onClick);
        this.removeEventListener("keydown", this.#onKeyDown);
        document.removeEventListener("pointerdown", this.#onDocumentPointerDown);

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
        if (oldValue !== newValue && name === "theme") {
            this.#applyThemeToChildren(newValue);
        }
    }

    /**
     * Returns true if any dropdown inside the menubar is open.
     *
     * @returns {boolean}
     */
    get hasOpenDropdown() {
        return this.querySelector("ctb-dropdown[open], ctb-dropdown.open") !== null;
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
     * Applies theme attribute to child components.
     *
     * @param {string | null} theme
     * @returns {void}
     */
    #applyThemeToChildren(theme) {
        if (!theme) return;
        const themedElements = this.querySelectorAll("ctb-button, ctb-dropdown, ctb-menu");
        for (const el of themedElements) {
            el.setAttribute("theme", theme);
        }
    }
}

customElements.define("ctb-menubar", CtbMenubar);
