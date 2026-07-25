/**
 * Configuration and behavior definition for the Glass theme.
 * Based on https://github.com/reimar/glass-button

MIT License

Copyright (c) 2023 David Darnes

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 *
 * @type {{
 *   name: string,
 *   cssPath: string,
 *   init: (container: HTMLElement) => void
 * }}
 */
export const themeConfig = {
    name: "glass",
    cssPath: "/js/components/themes/glass/button.css",

    /**
     * Initializes any theme-specific DOM overlay elements.
     *
     * @param {HTMLElement} container The button container element inside shadow DOM.
     * @returns {void}
     */
    init(container) {
        // Render the 5 glass highlight spans inside a wrapper div
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
    },
};
