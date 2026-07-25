import re
import os

def parse_args(arg_str):
    args = []
    current = []
    paren_depth = 0
    in_quote = False
    i = 0
    while i < len(arg_str):
        c = arg_str[i]
        if c == "'" and (i == 0 or arg_str[i-1] != '\\'):
            in_quote = not in_quote
            current.append(c)
        elif not in_quote:
            if c == '(':
                paren_depth += 1
                current.append(c)
            elif c == ')':
                paren_depth -= 1
                current.append(c)
            elif c == ',' and paren_depth == 0:
                args.append("".join(current).strip())
                current = []
            else:
                current.append(c)
        else:
            current.append(c)
        i += 1
    if current:
        args.append("".join(current).strip())
    return args

def to_rust_bytes_literal(s):
    s_escaped = s.replace('\\', '\\\\').replace('"', '\\"')
    if all(ord(c) < 128 for c in s):
        return f'b"{s_escaped}"'
    else:
        return f'"{s_escaped}".as_bytes()'

def to_rust_expr(expr):
    expr = expr.strip()

    # Match hex2bin
    m_hex = re.match(r"^hex2bin\((?P<arg>'.*?'|\".*?\")\)$", expr)
    if m_hex:
        hex_str = m_hex.group('arg')[1:-1]
        return f'&ctb_formats_hexdump::hex2bin("{hex_str}").unwrap()'

    # Match base64_encode
    m_b64e = re.match(r"^base64_encode\((?P<arg>'.*?'|\".*?\")\)$", expr)
    if m_b64e:
        s = m_b64e.group('arg')[1:-1]
        return f'ctb_formats_base64::base64_encode({to_rust_bytes_literal(s)}).into_bytes()'

    # Match base64_decode
    m_b64d = re.match(r"^base64_decode\((?P<arg>'.*?'|\".*?\")\)$", expr)
    if m_b64d:
        s = m_b64d.group('arg')[1:-1]
        return f'ctb_formats_base64::base64_decode("{s}").unwrap()'

    # Match iconv('UTF-8', 'UTF-32BE', '...')
    m_iconv = re.match(r"^iconv\('UTF-8',\s*'UTF-32BE',\s*(?P<arg>'.*?'|\".*?\")\)$", expr)
    if m_iconv:
        s = m_iconv.group('arg')[1:-1]
        return f'ctb_formats_encoding::unicode::utf8_to_utf32be({to_rust_bytes_literal(s)}).unwrap()'

    # Match string literals
    if expr.startswith("'") and expr.endswith("'"):
        s = expr[1:-1]
        return to_rust_bytes_literal(s)

    # Match dce_convert
    m_conv = re.match(r"^dce_convert\((?P<args>.*)\)$", expr)
    if m_conv:
        sub_args = parse_args(m_conv.group('args'))
        rust_sub_args = []
        for arg in sub_args:
            rust_sub_args.append(to_rust_expr(arg))

        data_arg = rust_sub_args[0]
        if not data_arg.startswith('&') and not data_arg.endswith(').unwrap()') and not data_arg.endswith('.unwrap()') and not data_arg.endswith('.into_bytes()'):
            data_arg = f'&{data_arg}[..]'
        elif data_arg.startswith('&'):
            pass
        else:
            data_arg = f'&{data_arg}'

        in_fmt = rust_sub_args[1].replace('b"', '"')
        if len(rust_sub_args) > 2:
            out_fmt = rust_sub_args[2].replace('b"', '"')
        else:
            out_fmt = '"none"'

        return f'crate::dce_convert({data_arg}, {in_fmt}, {out_fmt})'

    # Match get_dce_version
    m_ver = re.match(r"^get_dce_version\((?P<args>.*)\)$", expr)
    if m_ver:
        sub_args = parse_args(m_ver.group('args'))
        data_arg = to_rust_expr(sub_args[0])
        if not data_arg.startswith('&') and not data_arg.endswith(').unwrap()') and not data_arg.endswith('.unwrap()') and not data_arg.endswith('.into_bytes()'):
            data_arg = f'&{data_arg}[..]'
        elif data_arg.startswith('&'):
            pass
        else:
            data_arg = f'&{data_arg}'
        return f'crate::get_dce_version({data_arg})'

    return expr

def main():
    php_path = "~/ctoolbox/src/formats/dceutils/data/libdce-2.51/dceutils_tests.php"
    with open(php_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    rust_tests = []
    rust_tests.append('// Translated from PHP by test_converter.py.\n')
    rust_tests.append('#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]\n')
    rust_tests.append('use crate::utilities::*;\n\n')

    rust_tests.append('fn run_test(name: &str, result: Result<Vec<u8>>, expected: &[u8]) {\n')
    rust_tests.append('    match result {\n')
    rust_tests.append('        Ok(bytes) => {\n')
    rust_tests.append('            assert_eq!(bytes, expected, "Test \'{}\' failed: expected {:?}, got {:?}", name, String::from_utf8_lossy(expected), String::from_utf8_lossy(&bytes));\n')
    rust_tests.append('        }\n')
    rust_tests.append('        Err(e) => {\n')
    rust_tests.append('            let err_str = e.to_string();\n')
    rust_tests.append('            assert_eq!(err_str.as_bytes(), expected, "Test \'{}\' failed: expected error {:?}, got {:?}", name, String::from_utf8_lossy(expected), err_str);\n')
    rust_tests.append('        }\n')
    rust_tests.append('    }\n')
    rust_tests.append('}\n\n')

    rust_tests.append('fn run_test_ver(name: &str, result: String, expected: &str) {\n')
    rust_tests.append('    assert_eq!(result, expected, "Test \'{}\' failed: expected \'{}\', got \'{}\'", name, expected, result);\n')
    rust_tests.append('}\n\n')

    rust_tests.append("#[crate::ctb_test]\nfn test_libdce_compatibility() {\n")

    test_counter = 0
    for line in lines:
        line_stripped = line.strip()
        # Find lines like: test(name, expr, expected);
        # We can also capture comments or test_add titles as headings (for code readability/separators).
        if line_stripped.startswith("test_add("):
            m = re.match(r"^test_add\('(?P<title>.*?)'\);$", line_stripped)
            if m:
                clean_title = re.sub(r"<.*?>", "", m.group('title'))
                if clean_title:
                    rust_tests.append(f"    // --- {clean_title} ---\n")
            continue

        if line_stripped.startswith("test(") or line_stripped.startswith("//test("):
            is_commented = line_stripped.startswith("//")
            if is_commented:
                content = line_stripped[2:].strip()
            else:
                content = line_stripped

            # Match: test(name, expr, expected);
            m = re.match(r"^test\((?P<args>.*)\);$", content)
            if m:
                args = parse_args(m.group('args'))
                if len(args) == 3:
                    name_raw = args[0]
                    expr_raw = args[1]
                    expected_raw = args[2]

                    rust_name = name_raw[1:-1].replace('"', '\\"')
                    rust_expr = to_rust_expr(expr_raw)
                    rust_expected = to_rust_expr(expected_raw)

                    is_ver = "get_dce_version" in expr_raw
                    test_counter += 1

                    test_code = ""
                    if is_ver:
                        expected_str = expected_raw.replace("'", '"')
                        test_code = f'    run_test_ver("{rust_name}", {rust_expr}, {expected_str});'
                    else:
                        if not rust_expected.startswith('&') and not rust_expected.endswith(').unwrap()') and not rust_expected.endswith('.unwrap()') and not rust_expected.endswith('.into_bytes()') and not rust_expected.endswith('.as_bytes()'):
                            rust_expected = f'&{rust_expected}[..]'
                        elif rust_expected.startswith('&'):
                            pass
                        elif rust_expected.endswith('.as_bytes()'):
                            pass
                        else:
                            rust_expected = f'&{rust_expected}'
                        test_code = f'    run_test("{rust_name}", {rust_expr}, {rust_expected});'

                    if is_commented:
                        rust_tests.append(f"    // {test_code}\n")
                    else:
                        rust_tests.append(f"    {test_code}\n")

    rust_tests.append("}\n")

    out_path = "~/ctoolbox/src/formats/dceutils/tests.rs"
    with open(out_path, "w", encoding="utf-8") as f:
        f.writelines(rust_tests)
    print(f"Generated {test_counter} test assertions in {out_path}")

if __name__ == "__main__":
    main()
