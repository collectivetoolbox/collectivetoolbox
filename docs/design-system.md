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
