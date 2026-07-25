export class CtbSegmentedControl extends HTMLElement {
    /**
     * Change handler for radio inputs.
     *
     * @param {Event} event The change event.
     * @returns {void}
     */
    #changeHandler = (event) => {
        if (event.target instanceof HTMLInputElement && event.target.type === "radio") {
            this.#syncSelectedState();
        }
    };

    /**
     * Creates an instance of CtbSegmentedControl.
     */
    constructor() {
        super();
        // Lightweight layout container using Light DOM
    }

    /**
     * Connected callback lifecycle hook.
     * Sets up initial attributes and listens for change events on radio inputs.
     *
     * @returns {void}
     */
    connectedCallback() {
        // Setup initial selected attributes
        this.#syncSelectedState();

        // Listen for change events from radio inputs bubbling up
        this.addEventListener("change", this.#changeHandler);
    }

    /**
     * Disconnected callback lifecycle hook.
     * Cleans up event listeners.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        this.removeEventListener("change", this.#changeHandler);
    }

    /**
     * Synchronizes selected states to the child buttons based on their checked state.
     *
     * @returns {void}
     */
    #syncSelectedState() {
        const buttons = this.querySelectorAll("ctb-button");
        for (const button of buttons) {
            const input = button.querySelector("input[type='radio']");
            if (input instanceof HTMLInputElement) {
                if (input.checked) {
                    button.setAttribute("selected", "");
                } else {
                    button.removeAttribute("selected");
                }
            }
        }
    }
}

customElements.define("ctb-segmented-control", CtbSegmentedControl);
