#### 3-Rule Specification for `Dc 255`
Because code, markup, and literal data all share the same character space in Dc documents, `Dc 255` acts as a single-character syntactic-role stripper (analogous to a backslash in Bash):

1. **Single-token scope**:
   `255` applies strictly to the single immediately following Dc `X`. A trailing `255` at EOF (or end of an enclosed span) is a dangling-escape syntax error.
2. **Strips one layer of syntactic role in the current pass**:
   In the current framing/parsing/evaluation pass, `255 X` suppresses `X`'s syntactic role—it cannot act as a container opener or closer, list/kv separator (`312`), field/statement tag, prefix operator, EL routine, long-Dc prefix (`308`), or escape (`255`). Instead, `255 X` is treated as the **inert literal character `X`**:
   - It matches any negated character class `[^...]` (including comments `246..248` / `249..251`, literals `260..261`, identifiers `270..271`, key-value operands `514..515` / `313..314`, and open container bodies `317..318` / `321..322`) without closing the enclosing frame or splitting elements.
   - It produces an escaped `DcSyntaxKind::Atom(X)` in the CST (`DcFrameHeaderInfo::Escaped` / `DcSyntaxNode::is_escaped_atom()`) so downstream passes (`split_operands_with_resolver`, `extract_kv_map_entries`, `normalize_field_nodes_with_resolver`) never mistake an escaped `312` or tag for an unescaped separator or field tag.
   - It does **not** satisfy a positive grammar requirement for a structural tag (`376`) or typed construct (`[number]`), and is **never** silently deleted.
3. **Peels one `255` per unescaping/decoding pass (clean multi-layer nesting)**:
   Whenever a payload region is decoded into its inner content (e.g., decoding a `260..261` literal or `270..271` identifier, or unescaping `DcArray -> DcList` in EITE), **one** `255` is removed from each `255 X` $\to$ `X`. Embedding any Dc fragment inside an outer `260..261` literal therefore only requires prefixing each `255` and `261` with `255` (`261` $\to$ `255 261`, `255 261` $\to$ `255 255 255 261`), which unescapes losslessly back to the inner document when the outer literal is decoded.
