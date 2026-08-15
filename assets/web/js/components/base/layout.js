/**
 * Custom Element representing a structured layout container or panel (`<ctb-layout>`).
 * Used for organizing sub-views, tool panels, tab panes, or structured content.
 */
export class CtbLayout extends HTMLElement {
    /**
     * Observed attributes list.
     *
     * @returns {string[]}
     */
    static get observedAttributes() {
        return ["hidden"];
    }

    /**
     * Creates an instance of CtbLayout.
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
        // Light DOM component
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
        if (name === "hidden" && oldValue !== newValue) {
            this.setAttribute("aria-hidden", this.hasAttribute("hidden") ? "true" : "false");
        }
    }
}

customElements.define("ctb-layout", CtbLayout);
