/**
 * Custom Element representing a styled button or anchor.
 * Supports progressive enhancement by wrapping standard <a> or <button> elements.
 */
/** @type {Record<string, { cssPath: string, init: (container: HTMLElement) => void }>} */
const BUILTIN_THEMES = {
    glass: {
        cssPath: "/js/components/themes/glass/button.css",
        /**
         * Initializes the glass theme overlays in the container.
         *
         * @param {HTMLElement} container The button container element.
         * @returns {void}
         */
        init(container) {
            if (container.querySelector(".ctb-glass-overlay")) return;
            const glassOverlay = document.createElement("div");
            glassOverlay.className = "ctb-glass-overlay ctb-theme-overlay";

            const h1b = document.createElement("span");
            h1b.className = "highlight-1-base";
            glassOverlay.appendChild(h1b);

            const h1s = document.createElement("span");
            h1s.className = "highlight-1-spot";
            glassOverlay.appendChild(h1s);

            const h2b = document.createElement("span");
            h2b.className = "highlight-2-base";
            glassOverlay.appendChild(h2b);

            const h2s = document.createElement("span");
            h2s.className = "highlight-2-spot";
            glassOverlay.appendChild(h2s);

            const gloss = document.createElement("span");
            gloss.className = "gloss-1";
            glassOverlay.appendChild(gloss);

            container.appendChild(glassOverlay);
        }
    },
    solid: {
        cssPath: "/js/components/themes/solid/button.css",
        /**
         * Initializes the solid theme overlays (none needed).
         *
         * @param {HTMLElement} _container The button container element.
         * @returns {void}
         */
        init(_container) {
            // solid theme needs no overlays
        }
    },
    simple: {
        cssPath: "/js/components/themes/simple/button.css",
        /**
         * Initializes the simple theme overlays (none needed).
         *
         * @param {HTMLElement} _container The button container element.
         * @returns {void}
         */
        init(_container) {
            // simple theme needs no overlays
        }
    }
};

export class CtbButton extends HTMLElement {
    /** @type {string} */
    static defaultTheme = "none";
    /** @type {ShadowRoot} */
    #shadowRoot;

    /** @type {HTMLElement | null} */
    #container = null;

    /** @type {HTMLLinkElement | null} */
    #resetsLink = null;

    /** @type {HTMLLinkElement | null} */
    #themeLink = null;

    /** @type {MutationObserver | null} */
    #themeObserver = null;

    /** @type {ResizeObserver | null} */
    #resizeObserver = null;

    /** @type {MutationObserver | null} */
    #childObserver = null;

    /**
     * Returns the list of attributes to observe for changes.
     *
     * @returns {string[]} The list of observed attributes.
     */
    static get observedAttributes() {
        return ["variant", "selected", "disabled", "theme"];
    }

    /**
     * Creates an instance of CtbButton and initializes the Shadow DOM.
     */
    constructor() {
        super();
        this.#shadowRoot = this.attachShadow({ mode: "open" });
        this.#setupShadowDom();
    }

    /**
     * Lifecycle callback invoked when the element is connected to the DOM.
     * Updates the theme, syncs attributes to the child element, and sets up observers.
     *
     * @returns {void}
     */
    connectedCallback() {
        this.classList.add("no-transition");
        this.#updateTheme();
        this.#syncChildAttributes();
        this.#setupThemeObserver();

        this.#resizeObserver = new ResizeObserver(() => {
            this.#checkWrapping();
        });
        this.#resizeObserver.observe(this);

        this.addEventListener("click", (e) => {
            if (e.target === this) {
                const slotted = this.querySelector("button, a, label");
                if (slotted instanceof HTMLElement) {
                    slotted.click();
                }
            }
        });
    }

    /**
     * Lifecycle callback invoked when the element is disconnected from the DOM.
     * Cleans up observers.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        if (this.#themeObserver) {
            this.#themeObserver.disconnect();
            this.#themeObserver = null;
        }
        if (this.#resizeObserver) {
            this.#resizeObserver.disconnect();
            this.#resizeObserver = null;
        }
        if (this.#childObserver) {
            this.#childObserver.disconnect();
            this.#childObserver = null;
        }
    }

    /**
     * Callback when one of the observed attributes changes.
     *
     * @param {string} name The name of the attribute.
     * @param {string | null} oldValue The old value.
     * @param {string | null} newValue The new value.
     * @returns {void}
     */
    attributeChangedCallback(name, oldValue, newValue) {
        if (oldValue !== newValue) {
            if (name === "theme") {
                this.#applyTheme(newValue || CtbButton.defaultTheme || "none");
            } else {
                this.#syncChildAttributes();
            }
        }
    }

    /**
     * Sets up the Shadow DOM internal HTML and stylesheets.
     *
     * @returns {void}
     */
    #setupShadowDom() {
        // Resets link element
        const resetsLink = document.createElement("link");
        resetsLink.rel = "stylesheet";
        resetsLink.className = "resets-stylesheet";
        this.#resetsLink = resetsLink;

        // Theme link element
        const themeLink = document.createElement("link");
        themeLink.rel = "stylesheet";
        themeLink.className = "theme-stylesheet";
        this.#themeLink = themeLink;

        const container = document.createElement("div");
        container.className = "ctb-btn-container";
        this.#container = container;

        const slot = document.createElement("slot");
        slot.addEventListener("slotchange", () => {
            this.#syncChildAttributes();
            this.#checkWrapping();
            this.#observeChild();
        });
        container.appendChild(slot);

        this.#shadowRoot.appendChild(resetsLink);
        this.#shadowRoot.appendChild(themeLink);
        this.#shadowRoot.appendChild(container);
    }

    /**
     * Synchronizes the global theme setting from documentElement to this element.
     *
     * @returns {void}
     */
    #updateTheme() {
        const theme = document.documentElement.getAttribute("data-ctb-ui-theme") || CtbButton.defaultTheme || "none";
        if (this.getAttribute("theme") !== theme) {
            this.setAttribute("theme", theme);
        } else {
            // Force apply on initial load
            this.#applyTheme(theme);
        }
    }

    /**
     * Apply the theme configurations and stylesheets dynamically.
     *
     * @param {string} themeName The name of the theme.
     * @returns {Promise<void>}
     */
    async #applyTheme(themeName) {
        if (!this.#themeLink || !this.#resetsLink || !this.#container) return;

        this.classList.add("no-transition");

        if (themeName && themeName !== "none") {
            const builtin = BUILTIN_THEMES[themeName];
            if (builtin) {
                this.#applyThemeConfig(builtin);
            } else {
                try {
                    this.removeAttribute("ready");
                    // Dynamically import the theme configuration module
                    const modulePath = `../themes/${themeName}/button.js`;
                    const module = await import(modulePath);
                    const theme = module.themeConfig;

                    if (theme) {
                        this.#applyThemeConfig(theme);
                    } else {
                        this.setAttribute("ready", "");
                        this.classList.remove("no-transition");
                    }
                } catch (err) {
                    console.error(`Failed to load theme "${themeName}":`, err);
                    this.setAttribute("ready", "");
                    this.classList.remove("no-transition");
                }
            }
        } else {
            // Remove href attributes to fallback cleanly to default user-agent styles
            this.#resetsLink.removeAttribute("href");
            this.#themeLink.removeAttribute("href");

            // Clean up old theme elements from the container
            const overlays = this.#container.querySelectorAll(".ctb-theme-overlay");
            for (const overlay of overlays) {
                overlay.remove();
            }

            this.setAttribute("ready", "");
            this.classList.remove("no-transition");
        }
    }

    /**
     * Apply the theme configurations synchronously.
     *
     * @param {{ cssPath: string, init: (container: HTMLElement) => void }} theme
     * @returns {void}
     */
    #applyThemeConfig(theme) {
        if (!this.#resetsLink || !this.#themeLink || !this.#container) return;

        const resetsHref = "/js/components/base/button-theme-resets.css";

        let resetsPending = false;
        let themePending = false;

        const checkReady = () => {
            if (!resetsPending && !themePending) {
                this.setAttribute("ready", "");
                this.#checkWrapping();
                requestAnimationFrame(() => {
                    requestAnimationFrame(() => {
                        this.classList.remove("no-transition");
                    });
                });
            }
        };

        if (this.#resetsLink.getAttribute("href") !== resetsHref) {
            resetsPending = true;
            this.#resetsLink.onload = () => {
                resetsPending = false;
                checkReady();
            };
            this.#resetsLink.onerror = () => {
                resetsPending = false;
                checkReady();
            };
            this.#resetsLink.setAttribute("href", resetsHref);
        }

        if (this.#themeLink.getAttribute("href") !== theme.cssPath) {
            themePending = true;
            this.#themeLink.onload = () => {
                themePending = false;
                checkReady();
            };
            this.#themeLink.onerror = () => {
                themePending = false;
                checkReady();
            };
            this.#themeLink.setAttribute("href", theme.cssPath);
        }

        // Clean up old theme elements from the container
        const overlays = this.#container.querySelectorAll(".ctb-theme-overlay");
        for (const overlay of overlays) {
            overlay.remove();
        }

        // Run theme specific DOM init
        theme.init(this.#container);

        checkReady();
    }

    /**
     * Sets up a MutationObserver to listen for changes on documentElement data-ctb-ui-theme attribute.
     *
     * @returns {void}
     */
    #setupThemeObserver() {
        if (this.#themeObserver) return;
        this.#themeObserver = new MutationObserver(() => {
            this.#updateTheme();
        });
        this.#themeObserver.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["data-ctb-ui-theme"],
        });
    }

    /**
     * Synchronizes host-level attributes like disabled to the slotted child element.
     *
     * @returns {void}
     */
    #syncChildAttributes() {
        const disabled = this.hasAttribute("disabled");
        const slotted = this.querySelector("button, a, input[type='radio']");
        if (slotted) {
            // Detect if this is an icon-only button
            const textSource = (slotted instanceof HTMLInputElement && slotted.type === "radio")
                ? this.querySelector("label")
                : slotted;

            const hasText = textSource && (Array.from(textSource.childNodes).some((node) => {
                return (
                    node.nodeType === Node.TEXT_NODE &&
                    node.textContent &&
                    node.textContent.trim().length > 0
                );
            }) || textSource.querySelector("span"));

            if (!hasText && slotted.querySelector("img, svg, .icon")) {
                this.setAttribute("icon-only", "");
            } else {
                this.removeAttribute("icon-only");
            }
            this.classList.toggle("hidden", slotted.classList.contains("hidden"));

            if (disabled) {
                slotted.setAttribute("tabindex", "-1");
                if (slotted instanceof HTMLButtonElement || slotted instanceof HTMLInputElement) {
                    slotted.disabled = true;
                }
            } else {
                slotted.removeAttribute("tabindex");
                if (slotted instanceof HTMLButtonElement || slotted instanceof HTMLInputElement) {
                    slotted.disabled = false;
                }
            }

            if (slotted instanceof HTMLAnchorElement) {
                if (!slotted.hasAttribute("role")) {
                    slotted.setAttribute("role", "button");
                }
                if (!slotted.dataset.spaceHandlerAttached) {
                    slotted.addEventListener("keydown", (e) => {
                        if (e.key === " " || e.key === "Spacebar") {
                            if (this.hasAttribute("disabled")) {
                                e.preventDefault();
                                return;
                            }
                            e.preventDefault();
                            slotted.click();
                        }
                    });
                    slotted.dataset.spaceHandlerAttached = "true";
                }
            }
        }
    }

    /**
     * Checks if the button text content wraps onto multiple lines.
     * Sets the multiline attribute if the measured range height exceeds a single line height.
     *
     * @returns {void}
     */
    #checkWrapping() {
        const slotted = this.querySelector("button, a, input[type='radio']");
        if (!slotted) return;

        const rangeSource = (slotted instanceof HTMLInputElement && slotted.type === "radio")
            ? this.querySelector("label")
            : slotted;
        if (!rangeSource) return;

        const range = document.createRange();
        range.selectNodeContents(rangeSource);
        const rect = range.getBoundingClientRect();

        // 16px text with standard line-height has a single line height of ~20-24px.
        // If it is > 28px, it has wrapped onto multiple lines.
        if (rect.height > 28) {
            if (!this.hasAttribute("multiline")) {
                this.setAttribute("multiline", "");
            }
        } else {
            if (this.hasAttribute("multiline")) {
                this.removeAttribute("multiline");
            }
        }
    }

    /**
     * Set up a MutationObserver to listen for attribute changes on the slotted child.
     * Propagates changes (disabled, style, class) from the child back to the host container.
     *
     * @returns {void}
     */
    #observeChild() {
        if (this.#childObserver) {
            this.#childObserver.disconnect();
            this.#childObserver = null;
        }

        const slotted = this.querySelector("button, a, input[type='radio']");
        if (!slotted) return;

        this.#childObserver = new MutationObserver((mutations) => {
            for (const mutation of mutations) {
                if (mutation.type === "attributes") {
                    const name = mutation.attributeName;
                    const target = /** @type {HTMLElement} */ (mutation.target);
                    if (name === "disabled") {
                        const childDisabled = target.hasAttribute("disabled") || (target instanceof HTMLButtonElement && target.disabled);
                        const hostDisabled = this.hasAttribute("disabled");
                        if (childDisabled !== hostDisabled) {
                            if (childDisabled) {
                                this.setAttribute("disabled", "");
                            } else {
                                this.removeAttribute("disabled");
                            }
                        }
                    } else if (name === "style") {
                        const childStyle = target.getAttribute("style");
                        const hostStyle = this.getAttribute("style");
                        if (childStyle !== hostStyle) {
                            if (childStyle) {
                                this.setAttribute("style", childStyle);
                            } else {
                                this.removeAttribute("style");
                            }
                        }
                    } else if (name === "class") {
                        this.classList.toggle("hidden", target.classList.contains("hidden"));
                    }
                }
            }
        });

        this.#childObserver.observe(slotted, {
            attributes: true,
            attributeFilter: ["disabled", "style", "class"]
        });
    }
}

customElements.define("ctb-button", CtbButton);
