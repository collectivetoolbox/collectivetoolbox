Listed directory tools
Viewed center-of-gravity.js:1-100
Viewed calculator.js:1-100
Viewed eite-edit-tool.js:1-100
Searched for "center_of_gravity"
Viewed routes.rs:1-80
Listed directory tools
Listed directory calculator
Viewed calculator.rs:1-41
Viewed eite.rs:1-100
Listed directory rpc
Viewed center_of_gravity.rs:1-100
Viewed calculator.rs:1-70
Viewed eite.rs:1-70
Listed directory js
Viewed sidebar.js:1-68
Listed directory views
Viewed subscribe.hbs:1-60
Viewed center-of-gravity.hbs:1-100
Viewed eite-edit-tool.hbs:1-90
Listed directory src
Listed directory cli
Listed directory renderer
Viewed renderer.rs:1-60

## Architectural Review & Assessment

Eliminating direct JavaScript dependencies from these tools by adopting **Progressive Enhancement (Server-Rendered Handlebars + Form Controllers)** is not only feasible, but it is also the **optimal architectural pattern** for multi-modal rendering across Web, CLI, TUI, and Teletype interfaces.

The backend Rust crates (`ctb_formats_math` and `ctb_formats_eite`) already contain the business logic, math, and conversions. Currently, JavaScript acts as an ad-hoc intermediary: reading DOM elements, formatting JSON-RPC requests, calling `/api/rpc/{service}`, and writing values back to DOM elements.

By replacing that ad-hoc client-side glue with standard **HTTP Form Handlers + Handlebars Templates**, paired with an **Action/State model**, the same core functions serve no-JS browsers, full-JS browsers, TUIs, teletypes, and CLI subcommands seamlessly.

---

## 1. Deep Dive: The Three Legacy Tools

### A. Center of Gravity Calculator
* **Current Implementation (`center-of-gravity.js`, `center-of-gravity.hbs`)**:
  - The template contains an unconfigured form (`#cgForm`) without `action` or `method`.
  - JS intercepts `#Calculate` clicks, extracts inputs, calls `calculateCenterOfGravity` over RPC, and writes `TotalMoment`, `LoadedCg`, etc., into readonly inputs.
* **Controller + Handlebars Transition**:
  - Convert to `<form action="/tools/center-of-gravity" method="POST">`.
  - In `src/io/webui/controllers/calculator.rs`, implement `post_center_of_gravity(Form(input): Form<CenterOfGravityInputForm>) -> Response`.
  - Controller invokes `ctb_formats_math::center_of_gravity::calculate_center_of_gravity(&input)` and passes the inputs + calculated outputs into `tools.calculator.center-of-gravity` template.
  - Template populates `<input id="LoadedCg" value="{{output.loaded_cg}}" />`.
* **Progressive JS Layer (Zero Browser Regression)**:
  - A small generic handler can intercept the `submit` event via `fetch(form.action, { method: 'POST', body: new FormData(form) })` to update the DOM without full page refresh if JS is present.

---

### B. Classic Calculator
* **Current Implementation (`calculator.js`, `assets/views/tools/calculator/*.hbs`)**:
  - 8 separate tab panels (Make, Prime, Random, Sqrt, Temperature, Perimeter, Constants, Area) and 2 modals (RPS, 6R2).
  - JS maintains tab switching state (`switchTab`), intercepts clicks, and invokes individual RPC methods (`evaluateBasicOp`, `isPrime`, `convertTemperature`, `getRandomScaleTable`, etc.).
* **Controller + Handlebars Transition**:
  - Each tab is a self-contained sub-form posting to `/tools/calculator/{tab_name}` (e.g., `POST /tools/calculator/prime`, `POST /tools/calculator/temperature`).
  - Active tab selection can be driven by a query/form parameter (e.g., `?tab=prime`) and rendered with CSS/Handlebars conditional `{{#if (eq active_tab "prime")}}`.
  - Sidecars/modals (RPS game, 6R2): Can be full sub-pages or sub-views (`/tools/calculator/rps`, `/tools/calculator/6r2`) posting turn state in hidden fields or session state.

---

### C. EITE Document Edit Tool
* **Current Implementation (`eite-edit-tool.js`, `eite-edit-tool.hbs`)**:
  - A heavier interactive tool with 1,000+ lines of JS managing document state, DC (Data Compression) lists, import/export format selection, and text editor buffer.
* **Controller + Handlebars Transition**:
  - EITE functions operate on `EiteState` (`ctb_formats_eite::eite_state::EiteState`).
  - The web UI can be represented as an editing session:
    - `POST /tools/eite/import`: Takes raw file/text + format ID, stores into session/state, re-renders page with decoded text in `<textarea name="document_text">{{document_text}}</textarea>`.
    - `POST /tools/eite/convert` or `POST /tools/eite/export`: Takes `<textarea>` contents + target format ID and downloads or displays the converted result.
    - DC insertion: A standard `<select>` or search form that inserts selected tokens upon submission or preview.
  - While EITE has more interactive surface area, breaking it down into distinct stateful actions (**Load**, **Edit**, **Convert**, **Export**) makes it 100% usable without JS.

---

### D. Sidebar & Application Navigation
* **Current Implementation (`sidebar.js`)**:
  - Slides in/out using JS event listeners on `[href="#sidebar"]` and CSS transforms.
* **Pure CSS / Semantic HTML Fallback**:
  - Pure CSS drawer using `:target` selector:
    ```css
    #sidebar { transform: translateX(110%); }
    #sidebar:target { transform: translateX(0); display: block; }
    ```
  - When JS is enabled, `sidebar.js` can still enhance it with smooth touch swiping and focus-trapping, but basic open/close requires zero JS.

---

## 2. Multi-Modal Interface Support Spectrum

To support terminals with varying capabilities and document runtime engines, application components can follow the **Universal Action/State Pattern**:

```
 ┌────────────────────────────────────────────────────────┐
 │            Core Logic (Rust Format/Math Crates)        │
 └───────────────────────────┬────────────────────────────┘
                             │
 ┌───────────────────────────┴────────────────────────────┐
 │             Action / State Controller Layer            │
 │   - parse_args_or_form() -> ToolAction                 │
 │   - execute_action(State, ToolAction) -> Result<State> │
 └──────┬────────────────────┬────────────────────┬───────┘
        │                    │                    │
 ┌──────▼──────┐      ┌──────▼──────┐      ┌──────▼──────┐
 │ RenderTarget│      │ RenderTarget│      │ RenderTarget│
 │     Web     │      │ Videoterm   │      │  Teletype   │
 ├─────────────┤      ├─────────────┤      ├─────────────┤
 │ Handlebars  │      │ TUI/Curses  │      │ CLI Stream  │
 │ HTML + Form │      │ Grid/Keys   │      │ Prompt/Line │
 └─────────────┘      └─────────────┘      └─────────────┘
```

### Output Modalities & Handling:

| Modality / Target | Interaction Model | Implementation Strategy |
| :--- | :--- | :--- |
| **`RenderTarget::Teletype`** (Line printer, dumb pipe, CLI pipes) | Non-interactive or sequential question-and-answer prompt stream. | Standard CLI positional/flag arguments (e.g., `ctb math cg --empty-wt 1200 ...`) or linear line-by-line stdin prompts. No cursor jumps or screen clears. |
| **`RenderTarget::Videoterminal`** (VT-100, xterm, Curses) | Full-screen terminal UI (TUI) with cursor movement, form fields, and status bar. | Curses / TUI terminal renderer drawing tabs and input boxes mapping directly to the tool's field schemas. |
| **Refresh-a-page Terminal** | Page-oriented terminal (e.g., IBM 3270, VT-52 page mode). | Renders a full text screen with fields, waits for `Enter`/Submit key, re-draws the entire screen with updated state. |
| **`RenderTarget::Web` (No-JS)** | Standard HTTP GET/POST form submission. | Handlebars templates with standard HTML `<form>`, `<input>`, `<select>`, `<button type="submit">`. Fast server round-trips. |
| **`RenderTarget::Web` (With JS)** | In-place dynamic updates without page reload. | Progressive enhancement: JS intercepts `<form submit>`, sends request via `fetch`, and updates target DOM elements seamlessly. |

---

## 3. Integrating with the Custom Document Format Runtime

For the upcoming custom document format with runtime:
1. **Declarative Component Schema**: Define tools and form widgets in a format-agnostic way (e.g., input field types, validation rules, formulas/actions).
2. **Pluggable Renderers**: The document runtime evaluates the document tree and delegates rendering to the active `RenderTarget` driver:
   - For `RenderTarget::Web`, output HTML `<form>` + Handlebars.
   - For `RenderTarget::Videoterminal`, output character cells and interactive widgets.
   - For `RenderTarget::Teletype`, output linear prompts or dump rendered text.
3. **Execution Runtime**: By keeping tool computations in native Rust functions rather than JS scripts, the document runtime can execute expressions directly in Rust across all target modalities without requiring a JS runtime (like Boa) for basic tool execution.

---

## 4. Recommended Next Steps

1. **Refactor Center of Gravity First (Proof of Concept)**:
   - Add `POST /tools/center-of-gravity` route and form struct in `calculator.rs`.
   - Update `center-of-gravity.hbs` with form action and server-bound value tags (`{{output.loaded_cg}}`).
   - Add progressive JS form interceptor in `center-of-gravity.js` (or a generic `ajax-form.js`).
   - Add a CLI command (e.g. `ctoolbox math cg`) calling the same underlying function.
2. **Refactor Calculator Tabs**:
   - Migrate calculator tabs to sub-routes/forms with server-side evaluation.
3. **Refactor EITE Document Tool**:
   - Formalize the EITE session state and split import/edit/export workflows into standard form actions.
4. **CSS-First Navigation**:
   - Transition sidebar drawer to CSS target/checkbox patterns with optional JS enhancement.
