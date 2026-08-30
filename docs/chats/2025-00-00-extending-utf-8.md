# Extending UTF‑8 to 128‑bit integers

You can add a backward-compatible “escape” form that leaves all code points up to U+10FFFF encoded exactly as in RFC 3629, while introducing an extended, self‑synchronizing, byte‑aligned form for larger integers. Below is a concrete scheme that preserves UTF‑8’s critical properties and avoids overlap with existing well‑formed UTF‑8.

---

## Design goals and invariants

- **Transparency:** All scalar values (\le) U+10FFFF use standard UTF‑8 (1–4 bytes) unchanged. Extended sequences are used only for values above U+10FFFF.
- **Self‑synchronization:** Only the first byte of an extended sequence is a “start”; all following bytes are continuation‑pattern bytes (10xxxxxx). A decoder can resync by scanning for a non‑continuation byte, exactly like UTF‑8.
- **Byte alignment:** All units are octets; no partial bytes.
- **Canonical form:** No overlong encodings. Each value has exactly one encoding.
- **Local error detection:** Impossible byte patterns and length constraints make errors detectable without global context.

---

## Encoding format (UTF‑8e‑128)

Use a single “escape” lead byte that never appears in well‑formed UTF‑8, then count‑prefixed continuation bytes holding the integer in 6‑bit chunks.

- **Standard range (unchanged):** Values (x \in [0, \text{U+10FFFF}]) use normal UTF‑8 (1–4 bytes).
- **Extended range (new):**
    - **Lead byte:** 0xFF (binary 11111111).
    - **Length byte:** One continuation byte `H = 10 llllll`. Let (L = \text{value}(llllll)) with constraint (1 \le L \le 22).
        - (L) is the number of following payload continuation bytes.
        - With (L \le 22), capacity is (6L \le 132) bits, sufficient for up to (2^{128}-1).
    - **Payload:** Exactly (L) continuation bytes `Pi = 10 b_i5 b_i4 b_i3 b_i2 b_i1 b_i0`.
        - Concatenate the 6‑bit groups big‑endian: ((P_1 | P_2 | \dots | P_L)) to form the bitstring value.
        - For canonical form: if the integer is nonzero, (P_1 \ne 0). (No leading all‑zero 6‑bit group.)
        - For 128‑bit unsigned integers, choose the smallest (L) with (6L \ge 128) and ensure the top unused bits of (P_1) are zero.
- **Signed vs. unsigned:**
    - **Unsigned:** Interpret the concatenated bitstring as a number in ([0, 2^{6L}-1]). For 128‑bit, use (L=22) and ensure the top 4 bits of (P_1) are zero to cap at (2^{128}-1).
    - **Signed:** Encode the two’s‑complement 128‑bit bitstring of (x \bmod 2^{128}). Canonical form: choose the smallest (L) whose top group reproduces the sign extension without redundant leading groups (i.e., leading group not all zeros for nonnegative values and not all ones for negative values), while keeping (L \le 22).
- **Forbidden overlaps:**
    - 0xFF as a leading byte is reserved for the extended form only.
    - Values (\le) U+10FFFF must not use the extended form.

---

## Decoding and validation

- **Identify start:** If a byte is not of the form 10xxxxxx, it begins a code unit. For 0xFF, enter extended mode.
- **Extended mode:**
    1. Read `H`. If `H` is not 10xxxxxx, error.
    2. Let (L = \text{value}(H & 0x3F)). If (L = 0) or (L > 22), error.
    3. Read exactly (L) continuation bytes. If any is not 10xxxxxx, error.
    4. Concatenate their 6‑bit payloads to obtain the bitstring. Apply unsigned or signed interpretation per context.
    5. Canonicality checks:
        - If value (\le) U+10FFFF, error (must have used standard UTF‑8).
        - If value (\ne 0) and (P_1 = 0), error (overlong).
        - For unsigned 128‑bit canonical form: if (L=22), require the top 4 bits of (P_1) be zero.
- **Self‑synchronization:** Inside an extended unit, all non‑first bytes are 10xxxxxx. A scanner can skip forward past any run of 10xxxxxx to find the next start, just like UTF‑8.

---

## Bit budgets and limits

- **Per continuation byte:** 6 payload bits.
- **Header cost:** 1 continuation byte (6 bits) to encode (L) (1–22).
- **Capacity:** (6L) bits. With (L=22), capacity is 132 bits, enough for the full unsigned range ([0, 2^{128}-1]) and any 128‑bit two’s‑complement integer.
- **Total length for 128‑bit max:** 1 lead + 1 length + 22 payload = 24 bytes.

---

## Examples

- **First value above Unicode scalar range:** (x = \text{0x110000} = 1{,}114{,}112).
    - Binary groups (6‑bit, big‑endian): 010001, 000000, 000000. So (L=3), (P_1=0x11), (P_2=0x00), (P_3=0x00).
    - Bytes: [FF], [83], [91], [80], [80].
        - 0xFF (lead)
        - 0x83 = 10 000011 (L=3)
        - 0x91 = 10 010001 (P1)
        - 0x80 = 10 000000 (P2)
        - 0x80 = 10 000000 (P3)
- **Maximum 128‑bit unsigned:** (x = 2^{128} - 1).
    - Needs (L=22) groups. First group is 000011 (top 4 zeros then two ones), remaining 21 groups are 111111.
    - Bytes: [FF], [96], [83], [BF] × 21.
        - 0xFF (lead)
        - 0x96 = 10 010110 (L=22)
        - 0x83 = 10 000011 (P1)
        - 0xBF = 10 111111 repeated 21 times

---

## Notes and trade‑offs

- **Why 0xFF?** It never appears in well‑formed UTF‑8, so it’s a clean, unambiguous escape. Standard decoders will reject extended sequences rather than misinterpret them, preserving safety.
- **Length in one byte:** Because (L \le 22), one 6‑bit length byte suffices, minimizing overhead and keeping all non‑first bytes as continuation bytes for synchronization.
- **Canonical minimality:** Prohibiting leading zero (or all‑one for signed) groups prevents overlong encodings and keeps a one‑to‑one mapping between integers and encodings.
- **Ordering:** Like UTF‑8, this scheme is not magnitude‑order preserving for integers; if needed, add an application‑level framing for sortable encodings.
- **Interoperability:** Old UTF‑8 parsers will treat 0xFF as invalid and can safely reject or replace these sequences. New parsers can support both seamlessly while remaining strict on legacy UTF‑8 rules.

If you want, I can also outline an alternative with a fixed 23‑byte block for 128‑bit values (slightly smaller header, simpler decoder), or show reference pseudocode for encode/decode.