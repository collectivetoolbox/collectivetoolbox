# Design System: Buttons & Actions

Collective Toolbox uses a unified Web Component wrapper (`<ctb-button>`) to render buttons and actions across the application.

This component uses **Progressive Enhancement**: templates author standard, semantic HTML elements (such as `<button>` or `<a>`), and client-side JavaScript automatically upgrades them into custom components at runtime.

---

## 1. How to Author Buttons

Developers should write standard HTML elements. Do not include nested highlight spans or wrapper divs manually.

### Standard Clickable Buttons (Actions)
Use standard `<button>` elements:
```html
<button type="submit" class="btn btn-primary">Save Changes</button>
<button type="button" class="btn">Cancel</button>
```

### Navigation Links
Use standard `<a>` elements with the `btn` class:
```html
<a href="/subscribe" class="btn btn-primary" target="_blank">Subscribe</a>
<a href="/docs" class="btn">Read Documentation</a>
```

### Opting Out
To prevent a button or link from being automatically upgraded to a styled custom component, add the `btn-no-upgrade` class:
```html
<a href="#" class="btn-no-upgrade">Ignore</a>
```

---

## 2. Variations & States

Buttons support the following classes and attributes:

| Type / State | Class / Attribute | Description |
| :--- | :--- | :--- |
| **Primary Variant** | `.btn-primary` (CSS class) | Filled visual style, usually highlighted with a brand color gradient. |
| **Secondary Variant** | Default / `.btn` (CSS class) | Semi-translucent outline layout. |
| **Selected / Active** | `[selected]` (HTML attribute) | Indicates the button is currently in a toggled-on or active state. |
| **Disabled** | `[disabled]` / `.disabled` (HTML/CSS) | Disables user interaction, dims opacity, and removes focus. |
| **Link-like** | `.btn-link` | Button styled visually as a link. |

---

## 3. Themes

The system supports theming, and separately from the theme, dark-on-light and light-on-dark color schemes. Icon themes are planned but not yet implemented.

### How to Toggle Themes Globally
The theme is controlled by the `data-ctb-ui-theme` attribute on the `<html>` root element. It defaults to `glass` if omitted.
- To set the solid theme globally, for example:
  ```html
  <html data-ctb-ui-theme="solid">
  ```
The custom components dynamically listen for changes on this attribute and switch styles instantly.

---

## 4. Under the Hood (For Developers and Agents)

### JavaScript Auto-wrapping (`app.js`)
At page load and after SPA content changes, the client-side JavaScript scans the DOM for elements matching:
```css
button, a.btn
```
It wraps each matching element in a `<ctb-button>` custom component and copies style variations (like `variant="primary"`) to the wrapper.

### Modular Component Structure (`base/button.js`)
When `<ctb-button>` connects, it attaches a Shadow DOM containing:
1. A `<slot>` that receives the slotted native `<button>` or `<a>`.
2. Dynamic stylesheet references loaded only when a theme is active.

The custom component behavior, resets, and theme styling are split across the following modular files:
- `js/components/base/button.js`: Defines the core `<ctb-button>` custom element, monitors global theme adjustments on the `html` tag, and dynamically imports theme modules.
- `js/components/base/button-theme-resets.css`: Declares the theme-neutral structural resets, layout, and slot sizing constraints inside Shadow DOM, alongside generic fallback rules. This is conditionally loaded only when an active theme is applied.
- Example for the Glass theme: `js/components/themes/glass/common.css`, `js/components/themes/glass/button.js`, and `js/components/themes/glass/button.css`: Theme-specific styling and logic.

When no theme is applied (e.g. data-ctb-ui-theme is empty/absent), the custom component skips loading the resets and theme links, allowing the slotted element to inherit standard browser user-agent styles.

---

## 5. Menu Bar (`<ctb-menubar>`)

A desktop-style application menu bar component for hosting top-level actions and dropdown menus.

### Features
- **Desktop Hover Activation**: When a menu dropdown is clicked open, hovering over sibling top-level menus automatically opens them.
- **Keyboard Navigation**:
  - `ArrowLeft` / `ArrowRight`: Navigate between top-level items (switches open dropdowns if active).
  - `ArrowDown`: Opens the dropdown and focuses its first menu item.
  - `Escape`: Closes open dropdown and returns focus to the menu trigger.
- **Theming**: Child `<ctb-button>` and `<ctb-dropdown>` components automatically inherit and respond to global theme changes.

### Authoring Example
```html
<ctb-menubar>
    <ctb-dropdown>
        <button type="button" class="btn">File</button>
        <ctb-menu>
            <button type="button">New</button>
            <button type="button">Open...</button>
            <button type="button">Exit</button>
        </ctb-menu>
    </ctb-dropdown>
    <ctb-dropdown>
        <button type="button" class="btn">Help</button>
        <ctb-menu>
            <button type="button">Documentation</button>
            <button type="button">About</button>
        </ctb-menu>
    </ctb-dropdown>
</ctb-menubar>
```

---

## 6. Tab Group (`<ctb-tab-group>`)

A flexible tab container connecting a `<ctb-segmented-control>` tablist to `<ctb-layout>` tab panels.

### Features
- **Clean Component Structure**: Uses `<ctb-segmented-control>` for tab buttons and `<ctb-layout>` for tab content panes.
- **Button or Radio Input Support**: Supports simple `<button>` elements (or upgraded `<ctb-button>`s) or `<input type="radio">` options in the segmented control.
- **Automatic Accessibility (WAI-ARIA)**:
  - Inside a tab group, `<ctb-segmented-control>` automatically receives `role="tablist"`, and button items receive `role="tab"`, `aria-selected`, `aria-controls`, and roving `tabindex="0"|"-1"`.
  - In a standalone segmented control with buttons, it receives `role="radiogroup"` with items having `role="radio"`, `aria-checked`, and roving `tabindex`.
  - Panes (`<ctb-layout>`) automatically receive `role="tabpanel"`, `tabindex="0"`, and `aria-labelledby`.
- **Nested Tab Groups**: Safely isolates child tab groups (e.g. sub-tabs in an assistance pane).
- **Keyboard Navigation**: `ArrowLeft`, `ArrowRight`, `Home`, and `End` keys switch active tabs and panes with roving focus.
- **State Synchronization**: Automatically marks active tab button with `[selected]` and toggles `hidden` on non-active `<ctb-layout>` panes.

### Authoring Example (Simple Buttons)
```html
<ctb-tab-group>
    <ctb-segmented-control>
        <button type="button" selected>Make</button>
        <button type="button">Prime verification</button>
        <button type="button">Random Numbers</button>
    </ctb-segmented-control>
    <ctb-layout id="tab-make">
        <!-- Content for Make tab -->
    </ctb-layout>
    <ctb-layout id="tab-prime" hidden>
        <!-- Content for Prime tab -->
    </ctb-layout>
    <ctb-layout id="tab-rand" hidden>
        <!-- Content for Random Numbers tab -->
    </ctb-layout>
</ctb-tab-group>
```

### Authoring Example (Radio-based)
```html
<ctb-tab-group>
    <ctb-segmented-control>
        <fieldset name="calc-tabs">
            <ctb-button>
                <input type="radio" id="tab-calc" name="calc-tabs" value="pane-calc" checked />
                <label for="tab-calc">Calculator</label>
            </ctb-button>
            <ctb-button>
                <input type="radio" id="tab-help" name="calc-tabs" value="pane-help" />
                <label for="tab-help">Help</label>
            </ctb-button>
        </fieldset>
    </ctb-segmented-control>
    <ctb-layout id="pane-calc">
        <!-- Calculator pane content -->
    </ctb-layout>
    <ctb-layout id="pane-help" hidden>
        <!-- Help pane content -->
    </ctb-layout>
</ctb-tab-group>
```
