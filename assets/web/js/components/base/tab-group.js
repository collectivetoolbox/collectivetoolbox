/**
 * Custom Element representing a tab group container (`<ctb-tab-group>`).
 * Connects a `<ctb-segmented-control>` tablist to `<ctb-layout>` tab panels.
 * Supports progressive enhancement, accessible ARIA roles/linking, keyboard navigation,
 * nested tab groups, and theme synchronization.
 */
export class CtbTabGroup extends HTMLElement {
    /** @type {MutationObserver | null} */
    #themeObserver = null;

    /** @type {string} */
    #instanceId = Math.random().toString(36).slice(2, 8);

    /**
     * Change listener on the tab group (bubbling from ctb-segmented-control or radio inputs).
     *
     * @param {Event} event
     * @returns {void}
     */
    #onChange = (event) => {
        const target = /** @type {HTMLElement | null} */ (event.target);
        if (target && this.#isImmediateChild(target)) {
            this.syncActiveTab();
        }
    };

    /**
     * Keydown handler for keyboard navigation across tabs in the segmented control.
     *
     * @param {KeyboardEvent} event
     * @returns {void}
     */
    #onKeyDown = (event) => {
        const tabs = this.getTabs();
        if (tabs.length === 0) return;

        const activeIndex = tabs.findIndex((tab) =>
            tab === document.activeElement || tab.contains(document.activeElement)
        );

        if (activeIndex < 0) return;

        let targetIndex = -1;
        if (event.key === "ArrowRight" || event.key === "ArrowDown") {
            event.preventDefault();
            targetIndex = (activeIndex + 1) % tabs.length;
        } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
            event.preventDefault();
            targetIndex = activeIndex <= 0 ? tabs.length - 1 : activeIndex - 1;
        } else if (event.key === "Home") {
            event.preventDefault();
            targetIndex = 0;
        } else if (event.key === "End") {
            event.preventDefault();
            targetIndex = tabs.length - 1;
        }

        if (targetIndex >= 0) {
            const targetTab = tabs[targetIndex];
            this.#focusAndSelectTab(targetTab);
        }
    };

    /**
     * Observed attributes list.
     *
     * @returns {string[]}
     */
    static get observedAttributes() {
        return ["theme"];
    }

    /**
     * Creates an instance of CtbTabGroup.
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
        this.addEventListener("change", this.#onChange);
        this.addEventListener("keydown", this.#onKeyDown);

        this.syncActiveTab();
        this.#syncTheme();
        this.#setupThemeObserver();
    }

    /**
     * Disconnected callback lifecycle hook.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        this.removeEventListener("change", this.#onChange);
        this.removeEventListener("keydown", this.#onKeyDown);

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
     * Checks if an element belongs directly to this tab group (and not to a nested tab group).
     *
     * @param {HTMLElement} element
     * @returns {boolean}
     */
    #isImmediateChild(element) {
        const closestGroup = element.closest("ctb-tab-group");
        return closestGroup === this;
    }

    /**
     * Returns the segmented control belonging directly to this tab group.
     *
     * @returns {HTMLElement | null}
     */
    getSegmentedControl() {
        const controls = this.querySelectorAll("ctb-segmented-control");
        for (const ctrl of controls) {
            if (ctrl instanceof HTMLElement && this.#isImmediateChild(ctrl)) {
                return ctrl;
            }
        }
        return null;
    }

    /**
     * Returns all tab button/input elements belonging to this tab group's segmented control.
     *
     * @returns {HTMLElement[]}
     */
    getTabs() {
        const control = this.getSegmentedControl();
        if (!control) return [];

        const candidates = control.querySelectorAll(
            "input[type='radio'], ctb-button, button:not(ctb-button button)"
        );
        /** @type {HTMLElement[]} */
        const tabs = [];
        for (const el of candidates) {
            if (el instanceof HTMLElement) {
                // If it's a ctb-button wrapping a radio, only include one representation
                if (el.tagName === "CTB-BUTTON" && el.querySelector("input[type='radio']")) {
                    continue;
                }
                tabs.push(el);
            }
        }
        return tabs;
    }

    /**
     * Returns all <ctb-layout> panels belonging directly to this tab group.
     *
     * @returns {HTMLElement[]}
     */
    getPanels() {
        const candidates = this.querySelectorAll("ctb-layout");
        /** @type {HTMLElement[]} */
        const panels = [];
        for (const el of candidates) {
            if (el instanceof HTMLElement && this.#isImmediateChild(el)) {
                panels.push(el);
            }
        }
        return panels;
    }

    /**
     * Focuses and selects a tab element.
     *
     * @param {HTMLElement} tab
     * @returns {void}
     */
    #focusAndSelectTab(tab) {
        const focusTarget = tab.querySelector("input, button, a") || tab;
        if (focusTarget instanceof HTMLElement) {
            focusTarget.focus();
        }

        const radio = tab instanceof HTMLInputElement && tab.type === "radio"
            ? tab
            : tab.querySelector("input[type='radio']");

        if (radio instanceof HTMLInputElement) {
            radio.checked = true;
            radio.dispatchEvent(new Event("change", { bubbles: true }));
        } else {
            const control = /** @type {any} */ (this.getSegmentedControl());
            if (control && typeof control.selectButton === "function") {
                control.selectButton(tab);
            } else {
                tab.setAttribute("selected", "");
                this.syncActiveTab();
            }
        }
    }

    /**
     * Synchronizes active tab, ARIA relationships, and visible layout panels.
     *
     * @returns {void}
     */
    syncActiveTab() {
        const tabs = this.getTabs();
        const panels = this.getPanels();
        if (tabs.length === 0 || panels.length === 0) return;

        // Ensure segmented control has role="tablist"
        const control = /** @type {any} */ (this.getSegmentedControl());
        if (control && !control.hasAttribute("role")) {
            control.setAttribute("role", "tablist");
        }

        let activeTab = null;
        let activeKey = null;
        let activeIndex = -1;

        for (let i = 0; i < tabs.length; i++) {
            const tab = tabs[i];
            const radio = tab instanceof HTMLInputElement && tab.type === "radio"
                ? tab
                : tab.querySelector("input[type='radio']");

            if (radio instanceof HTMLInputElement) {
                const buttonWrapper = tab.closest("ctb-button");
                if (radio.checked) {
                    activeTab = tab;
                    activeKey = radio.value || radio.id;
                    activeIndex = i;
                    if (buttonWrapper) {
                        buttonWrapper.setAttribute("selected", "");
                    }
                } else if (buttonWrapper) {
                    buttonWrapper.removeAttribute("selected");
                }
            } else if (tab.hasAttribute("selected") || tab.getAttribute("aria-selected") === "true") {
                activeTab = tab;
                activeKey =
                    tab.getAttribute("data-tab") ||
                    tab.getAttribute("aria-controls") ||
                    tab.getAttribute("data-target") ||
                    tab.id;
                activeIndex = i;
            }
        }

        // If no tab is selected, default to the first tab
        if (!activeTab && tabs.length > 0) {
            activeTab = tabs[0];
            activeIndex = 0;
            const radio = activeTab instanceof HTMLInputElement && activeTab.type === "radio"
                ? activeTab
                : activeTab.querySelector("input[type='radio']");

            if (radio instanceof HTMLInputElement) {
                radio.checked = true;
                activeKey = radio.value || radio.id;
                const buttonWrapper = activeTab.closest("ctb-button");
                if (buttonWrapper) {
                    buttonWrapper.setAttribute("selected", "");
                }
            } else {
                activeTab.setAttribute("selected", "");
                activeKey =
                    activeTab.getAttribute("data-tab") ||
                    activeTab.getAttribute("aria-controls") ||
                    activeTab.getAttribute("data-target") ||
                    activeTab.id;
            }
        }

        // Match and unhide active panel
        let matchedPanel = null;
        if (activeKey) {
            matchedPanel = panels.find((p) =>
                p.id === activeKey ||
                p.getAttribute("data-tab") === activeKey ||
                p.getAttribute("data-pane") === activeKey
            );
        }

        if (!matchedPanel && activeIndex >= 0 && activeIndex < panels.length) {
            matchedPanel = panels[activeIndex];
        }

        // Set up accessible ARIA links between tabs and panels
        for (let i = 0; i < panels.length; i++) {
            const panel = panels[i];
            const tab = tabs[i];

            if (!panel.id) {
                panel.id = `ctb-pane-${this.#instanceId}-${i}`;
            }

            if (tab) {
                const tabTarget = tab.querySelector("button") || tab;
                if (!tabTarget.id) {
                    tabTarget.id = `ctb-tab-${this.#instanceId}-${i}`;
                }
                tabTarget.setAttribute("aria-controls", panel.id);
                panel.setAttribute("aria-labelledby", tabTarget.id);
            }

            panel.setAttribute("role", "tabpanel");
            panel.setAttribute("tabindex", "0");

            if (panel === matchedPanel) {
                panel.removeAttribute("hidden");
                panel.setAttribute("aria-hidden", "false");
            } else {
                panel.setAttribute("hidden", "");
                panel.setAttribute("aria-hidden", "true");
            }
        }

        if (control && typeof control.syncSelectedState === "function") {
            control.syncSelectedState();
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
     * Applies theme attribute to child components.
     *
     * @param {string | null} theme
     * @returns {void}
     */
    #applyThemeToChildren(theme) {
        if (!theme) return;
        const themedElements = this.querySelectorAll("ctb-button, ctb-segmented-control");
        for (const el of themedElements) {
            if (el instanceof HTMLElement && this.#isImmediateChild(el)) {
                el.setAttribute("theme", theme);
            }
        }
    }
}

customElements.define("ctb-tab-group", CtbTabGroup);
