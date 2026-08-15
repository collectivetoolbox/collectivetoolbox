/**
 * Custom Element representing a segmented control or button group (`<ctb-segmented-control>`).
 * Can contain a group of radio inputs or buttons, maintaining the active selected state
 * and providing accessible WAI-ARIA roles (radiogroup/tablist) and roving tabindex navigation.
 */
export class CtbSegmentedControl extends HTMLElement {
    /**
     * Change handler for radio inputs.
     *
     * @param {Event} event The change event.
     * @returns {void}
     */
    #changeHandler = (event) => {
        if (event.target instanceof HTMLInputElement && event.target.type === "radio") {
            this.syncSelectedState();
        }
    };

    /**
     * Click handler for button clicks.
     *
     * @param {MouseEvent} event
     * @returns {void}
     */
    #clickHandler = (event) => {
        const target = /** @type {HTMLElement | null} */ (event.target);
        if (!target) return;

        const button = target.closest("button, ctb-button");
        if (button instanceof HTMLElement) {
            const hasRadio = button.querySelector("input[type='radio']");
            if (!hasRadio) {
                this.selectButton(button);
            }
        }
    };

    /**
     * Keydown handler for accessible roving tabindex and keyboard navigation.
     *
     * @param {KeyboardEvent} event
     * @returns {void}
     */
    #keyDownHandler = (event) => {
        // If native radio inputs exist, browser handles radio arrow keys
        if (this.querySelector("input[type='radio']")) return;

        const items = this.getItems();
        if (items.length === 0) return;

        const activeIndex = items.findIndex((item) =>
            item === document.activeElement || item.contains(document.activeElement)
        );

        if (activeIndex < 0) return;

        let targetIndex = -1;
        if (event.key === "ArrowRight" || event.key === "ArrowDown") {
            event.preventDefault();
            targetIndex = (activeIndex + 1) % items.length;
        } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
            event.preventDefault();
            targetIndex = activeIndex <= 0 ? items.length - 1 : activeIndex - 1;
        } else if (event.key === "Home") {
            event.preventDefault();
            targetIndex = 0;
        } else if (event.key === "End") {
            event.preventDefault();
            targetIndex = items.length - 1;
        }

        if (targetIndex >= 0) {
            const targetItem = items[targetIndex];
            this.selectButton(targetItem);
            const focusTarget = targetItem.querySelector("button, a") || targetItem;
            if (focusTarget instanceof HTMLElement) {
                focusTarget.focus();
            }
        }
    };

    /**
     * Creates an instance of CtbSegmentedControl.
     */
    constructor() {
        super();
    }

    /**
     * Connected callback lifecycle hook.
     * Sets up initial attributes and listens for events.
     *
     * @returns {void}
     */
    connectedCallback() {
        this.#setupAriaRoles();
        this.syncSelectedState();
        this.addEventListener("change", this.#changeHandler);
        this.addEventListener("click", this.#clickHandler);
        this.addEventListener("keydown", this.#keyDownHandler);
    }

    /**
     * Disconnected callback lifecycle hook.
     * Cleans up event listeners.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        this.removeEventListener("change", this.#changeHandler);
        this.removeEventListener("click", this.#clickHandler);
        this.removeEventListener("keydown", this.#keyDownHandler);
    }

    /**
     * Sets up ARIA roles depending on context (tablist inside tab-group, or radiogroup for standalone buttons).
     *
     * @returns {void}
     */
    #setupAriaRoles() {
        const isTabList = this.closest("ctb-tab-group") !== null;
        const hasRadios = this.querySelector("input[type='radio']") !== null;

        if (!this.hasAttribute("role")) {
            if (isTabList) {
                this.setAttribute("role", "tablist");
            } else if (!hasRadios) {
                this.setAttribute("role", "radiogroup");
            }
        }
    }

    /**
     * Gets all segmented item elements (buttons or radio inputs).
     *
     * @returns {HTMLElement[]}
     */
    getItems() {
        /** @type {HTMLElement[]} */
        const items = [];
        const children = this.querySelectorAll("ctb-button, button:not(ctb-button button)");
        for (const child of children) {
            if (child instanceof HTMLElement) {
                items.push(child);
            }
        }
        return items;
    }

    /**
     * Selects a specific button within the segmented control.
     *
     * @param {HTMLElement} button
     * @returns {void}
     */
    selectButton(button) {
        const items = this.getItems();
        const targetWrapper = button.tagName === "CTB-BUTTON" ? button : (button.closest("ctb-button") || button);

        for (const item of items) {
            const isTarget = item === targetWrapper || item === button;
            if (isTarget) {
                item.setAttribute("selected", "");
            } else {
                item.removeAttribute("selected");
            }
        }

        this.syncSelectedState();
        this.dispatchEvent(new Event("change", { bubbles: true }));
    }

    /**
     * Synchronizes selected states, ARIA attributes, and roving tabindex to the child buttons.
     *
     * @returns {void}
     */
    syncSelectedState() {
        const isTabList = this.closest("ctb-tab-group") !== null;
        const hasRadios = this.querySelector("input[type='radio']") !== null;
        const items = this.getItems();
        let selectedFound = false;

        for (const item of items) {
            const radio = item.querySelector("input[type='radio']");
            const isSelected = radio instanceof HTMLInputElement
                ? radio.checked
                : item.hasAttribute("selected") || item.getAttribute("aria-selected") === "true";

            if (isSelected) {
                item.setAttribute("selected", "");
                selectedFound = true;
            } else {
                item.removeAttribute("selected");
            }

            // Progressive enhancement of accessible roles & states for button items
            if (!hasRadios) {
                const innerBtn = item.querySelector("button") || (item instanceof HTMLButtonElement ? item : null);
                const targetElement = innerBtn || item;

                if (isTabList) {
                    targetElement.setAttribute("role", "tab");
                    targetElement.setAttribute("aria-selected", isSelected ? "true" : "false");
                } else {
                    targetElement.setAttribute("role", "radio");
                    targetElement.setAttribute("aria-checked", isSelected ? "true" : "false");
                }
                targetElement.setAttribute("tabindex", isSelected ? "0" : "-1");
            }
        }

        // Default to first item if none is selected
        if (!selectedFound && items.length > 0) {
            const firstItem = items[0];
            const radio = firstItem.querySelector("input[type='radio']");
            if (radio instanceof HTMLInputElement) {
                radio.checked = true;
            }
            firstItem.setAttribute("selected", "");
            if (!hasRadios) {
                const innerBtn = firstItem.querySelector("button") || (firstItem instanceof HTMLButtonElement ? firstItem : null);
                const targetElement = innerBtn || firstItem;
                if (isTabList) {
                    targetElement.setAttribute("aria-selected", "true");
                } else {
                    targetElement.setAttribute("aria-checked", "true");
                }
                targetElement.setAttribute("tabindex", "0");
            }
        }
    }
}

customElements.define("ctb-segmented-control", CtbSegmentedControl);
