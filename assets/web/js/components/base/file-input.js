/**
 * Custom Element representing a styled file input.
 * Supports progressive enhancement by wrapping a standard <input type="file">.
 */
export class CtbFileInput extends HTMLElement {
    /** @type {HTMLInputElement | null} */
    #input = null;

    /** @type {HTMLButtonElement | null} */
    #button = null;

    /** @type {HTMLElement | null} */
    #fileNameDisplay = null;

    /**
     * Change handler for the file input.
     *
     * @returns {void}
     */
    #changeHandler = () => {
        this.#updateFileName();
    };

    /**
     * Click handler for the custom button.
     *
     * @returns {void}
     */
    #clickHandler = () => {
        if (this.#input) {
            this.#input.click();
        }
    };

    /**
     * Creates an instance of CtbFileInput.
     */
    constructor() {
        super();
    }

    /**
     * Connected callback lifecycle hook.
     * Sets up the custom elements, hides the native input, and attaches events.
     *
     * @returns {void}
     */
    connectedCallback() {
        // Prevent duplicate initialization
        if (this.querySelector("ctb-button")) return;

        // Locate or create the native file input element
        this.#input = this.querySelector("input[type='file']");
        if (!this.#input) {
            this.#input = document.createElement("input");
            this.#input.type = "file";
            this.appendChild(this.#input);
        }

        // Hide the native input using inline style to avoid external dependency
        this.#input.style.display = "none";

        // Create a layout container
        const container = document.createElement("div");
        container.className = "flex items-center gap-4";

        // Create the custom button wrapper and inner button
        const btnWrapper = document.createElement("ctb-button");
        btnWrapper.setAttribute("variant", "secondary");

        const button = document.createElement("button");
        button.type = "button";
        button.className = "btn";

        const iconSrc = this.getAttribute("icon-src");
        if (iconSrc) {
            const iconImg = document.createElement("img");
            iconImg.className = "icon";
            iconImg.src = iconSrc;
            iconImg.alt = "";
            button.appendChild(iconImg);
        }

        const btnText = this.getAttribute("button-text") || "Choose File";
        const span = document.createElement("span");
        span.textContent = btnText;
        button.appendChild(span);

        btnWrapper.appendChild(button);
        this.#button = button;

        // Create file name display
        const nameDisplay = document.createElement("span");
        nameDisplay.id = "file-name";
        nameDisplay.className = "muted";
        nameDisplay.textContent = "No file chosen";
        this.#fileNameDisplay = nameDisplay;

        // Append to container, and container to component
        container.appendChild(btnWrapper);
        container.appendChild(nameDisplay);
        this.appendChild(container);

        // Bind events
        this.#button.addEventListener("click", this.#clickHandler);
        this.#input.addEventListener("change", this.#changeHandler);
    }

    /**
     * Disconnected callback lifecycle hook.
     * Cleans up event listeners.
     *
     * @returns {void}
     */
    disconnectedCallback() {
        if (this.#button) {
            this.#button.removeEventListener("click", this.#clickHandler);
        }
        if (this.#input) {
            this.#input.removeEventListener("change", this.#changeHandler);
        }
    }

    /**
     * Updates the filename text display based on the selected file.
     *
     * @returns {void}
     */
    #updateFileName() {
        if (!this.#input || !this.#fileNameDisplay) return;
        if (this.#input.files && this.#input.files.length > 0) {
            this.#fileNameDisplay.textContent = this.#input.files[0].name;
            this.#fileNameDisplay.classList.remove("muted");
        } else {
            this.#fileNameDisplay.textContent = "No file chosen";
            this.#fileNameDisplay.classList.add("muted");
        }
    }
}

customElements.define("ctb-file-input", CtbFileInput);
