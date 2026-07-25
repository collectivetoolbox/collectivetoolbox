# Frontend Styling and Semantic Markup Guidelines

## Semantic HTML
- Do not use generic elements (like `<span>` or `<div>`) styled as headings. Use proper semantic heading tags (e.g. `<h2>`, `<h3>`).
- Never use `<span>` tags with `.btn` class for placeholder or disabled buttons. Use `<button disabled>` or `<button type="button" disabled>` for standard semantic disabled buttons.

## Contrast and Accessibility
- Avoid low-contrast text colors like gray (`text-gray-500` or `#6B7280`).
- Ensure all text elements are readable with sufficient contrast against their background.

## Styling Placement
- Do not mix non-layout utility classes (such as text size, font weight, background colors, custom colors) in HTML templates.
- Rely on semantic markup and the project-wide stylesheet for consistent, high-contrast, and premium styling.
