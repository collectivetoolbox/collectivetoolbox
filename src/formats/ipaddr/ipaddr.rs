// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from ip-regex: MIT
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

/*
Based on https://github.com/sindresorhus/ip-regex
MIT License

Copyright (c) Sindre Sorhus <sindresorhus@gmail.com> (https://sindresorhus.com)

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;
use fancy_regex::Regex;

#[derive(Default, Clone)]
struct Options {
    /// Only match an exact string. Useful with `RegExp#test()` to check if a string is an IP address.
    exact: bool,
    /// Include boundaries in the regex. When true, 192.168.0.2000000000 will report as an invalid IPv4 address. If this option is not set, the mentioned IPv4 address would report as valid (ignoring the trailing zeros).
    include_boundaries: bool,
}

enum RegexType {
    V4,
    V6,
    V4V6,
}

fn get_regex(regex_type: &RegexType, options: Option<&Options>) -> String {
    // Reason for fallback: when optional Options struct reference is None, default options are used.
    let options = options.cloned().unwrap_or_default();

    // In the JavaScript version, include_boundaries has no effect when exact is true
    assert!(
        !(options.exact && options.include_boundaries),
        "Cannot set both exact and include_boundaries options to true"
    );

    let word = "[a-fA-F\\d:]";

    let boundary = if options.include_boundaries {
        format!("(?:(?<=\\s|^)(?={word})|(?<={word})(?=\\s|$))")
    } else {
        String::new()
    };

    let v4 = "(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}";

    let v6segment = "[a-fA-F\\d]{1,4}";

    let v6 = format!(
        r"
(?:
(?:{v6segment}:){{7}}(?:{v6segment}|:)|
    // 1:2:3:4:5:6:7::  1:2:3:4:5:6:7:8
(?:{v6segment}:){{6}}(?:{v4}|:{v6segment}|:)|
    // 1:2:3:4:5:6::    1:2:3:4:5:6::8   1:2:3:4:5:6::8  1:2:3:4:5:6::1.2.3.4
(?:{v6segment}:){{5}}(?::{v4}|(?::{v6segment}){{1,2}}|:)|
    // 1:2:3:4:5::      1:2:3:4:5::7:8   1:2:3:4:5::8    1:2:3:4:5::7:1.2.3.4
(?:{v6segment}:){{4}}(?:(?::{v6segment}){{0,1}}:{v4}|(?::{v6segment}){{1,3}}|:)|
    // 1:2:3:4::        1:2:3:4::6:7:8   1:2:3:4::8      1:2:3:4::6:7:1.2.3.4
(?:{v6segment}:){{3}}(?:(?::{v6segment}){{0,2}}:{v4}|(?::{v6segment}){{1,4}}|:)|
    // 1:2:3::          1:2:3::5:6:7:8   1:2:3::8        1:2:3::5:6:7:1.2.3.4
(?:{v6segment}:){{2}}(?:(?::{v6segment}){{0,3}}:{v4}|(?::{v6segment}){{1,5}}|:)|
    // 1:2::            1:2::4:5:6:7:8   1:2::8          1:2::4:5:6:7:1.2.3.4
(?:{v6segment}:){{1}}(?:(?::{v6segment}){{0,4}}:{v4}|(?::{v6segment}){{1,6}}|:)|
    // 1::              1::3:4:5:6:7:8   1::8            1::3:4:5:6:7:1.2.3.4
(?::(?:(?::{v6segment}){{0,5}}:{v4}|(?::{v6segment}){{1,7}}|:))
    // ::2:3:4:5:6:7:8  ::2:3:4:5:6:7:8  ::8             ::1.2.3.4
)(?:%[0-9a-zA-Z]{{1,}})?
    // %eth0            %1
"
    );
    let v6 = normalize_v6(&v6);

    // Pre-compile only the exact regexes because adding a global flag make regexes stateful
    let v46_exact = format!(r"(?:^{v4}$)|(?:^{v6}$)");
    let v4_exact = format!(r"^{v4}$");
    let v6_exact = format!(r"^{v6}$");

    let is_exact = options.exact;

    let ip_regex = if is_exact {
        v46_exact
    } else {
        format!(r"(?:{boundary}{v4}{boundary})|(?:{boundary}{v6}{boundary})")
    };

    let ip_regex_v4 = if is_exact {
        v4_exact
    } else {
        format!(r"{boundary}{v4}{boundary}")
    };
    let ip_regex_v6 = if is_exact {
        v6_exact
    } else {
        format!(r"{boundary}{v6}{boundary}")
    };

    match regex_type {
        RegexType::V4 => ip_regex_v4,
        RegexType::V6 => ip_regex_v6,
        RegexType::V4V6 => ip_regex,
    }
}

fn normalize_v6(v6: &str) -> String {
    // remove //... to end-of-line (enable multiline mode so $ matches end of line)
    static RE_COMMENTS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        // Reason for fallback: static regex literal r"(?m)\s*//.*$" is provably valid, fallback to "$" matches end of string if compilation fails.
        Regex::new(r"(?m)\s*//.*$").unwrap_or_else(|_| match Regex::new("$") {
            Ok(r) => r,
            Err(_) => unreachable!(),
        })
    });
    let v6 = RE_COMMENTS.replace_all(v6, "").to_string();
    // remove newlines
    let v6 = v6.replace('\n', "");
    // trim
    v6.trim().to_string()
}

/// Get a regex string that matches both IPv4 and IPv6 addresses exactly.
/// If you run the regex against untrusted user input in a server context, you
/// should give it a timeout.
pub fn get_regex_ipv4_ipv6_exact() -> String {
    get_regex(
        &RegexType::V4V6,
        Some(&Options {
            exact: true,
            include_boundaries: false,
        }),
    )
}

/// Check if this looks like a "wildcard" IPv4 or IPv6 address (e.g.
/// `0.0.0.0`; `::`; `::0`; `0:0:0:0:0:0:0:0`).
pub fn is_unspecified(addr: &str) -> Result<bool> {
    let v4_result = addr
        .parse::<std::net::Ipv4Addr>()
        .map(|ip| ip.is_unspecified());

    let v6_result = addr
        .parse::<std::net::Ipv6Addr>()
        .map(|ip| ip.is_unspecified());

    if (v4_result.as_ref().is_ok_and(|r| *r))
        || (v6_result.as_ref().is_ok_and(|r| *r))
    {
        return Ok(true);
    } else if v4_result.is_err() && v6_result.is_err() {
        bail!("Invalid IP address: {addr}");
    }

    Ok(false)
}

// NOTE: I haven't checked over these tests to make sure they match the original at https://raw.githubusercontent.com/sindresorhus/ip-regex/refs/heads/main/test.js

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::sync::RwLock;

    fn build_regex_rt(rt: &RegexType, options: Option<&Options>) -> Regex {
        let pat = get_regex(rt, options);

        // Fast path: try read lock and return cloned Regex if present
        {
            let cache = REGEX_CACHE.read().unwrap();
            if let Some(re) = cache.get(&pat) {
                return re.clone();
            }
        }

        // Compile outside of write lock (may be done redundantly by multiple threads,
        // but avoids blocking readers during compilation)
        let invalid_msg = format!("invalid regex {pat}");
        let compiled = Regex::new(&pat).expect(&invalid_msg);

        // Insert into cache if absent (double-check to avoid overwriting another thread's insert)
        let mut cache = REGEX_CACHE.write().unwrap();
        let entry = cache.entry(pat).or_insert_with(|| compiled.clone());
        entry.clone()
    }

    #[crate::ctb_test]
    fn test_get_regex_ipv4_ipv6() {
        let regex_str = get_regex_ipv4_ipv6_exact();
        let regex = fancy_regex::Regex::new(&regex_str).unwrap();

        // Valid IPv4 addresses
        let valid_ipv4 = vec![
            "127.0.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "192.168.1.1",
            "1.2.3.4",
            "10.0.0.1",
        ];
        for ip in valid_ipv4 {
            assert!(
                regex.is_match(ip).expect("Error"),
                "expected '{}' to match IPv4 with regex '{}'",
                ip,
                regex.as_str()
            );
        }

        // Valid IPv6 addresses (including IPv4-mapped)
        let valid_ipv6 = vec![
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "2001:db8::1",
            "::1",
            "fe80::1ff:fe23:4567:890a",
            "2001:db8:85a3::8a2e:370:7334",
            "2001:0db8:0000:0000:0000:0000:1428:57ab",
            "::ffff:192.0.2.128",
        ];
        for ip in valid_ipv6 {
            assert!(
                regex.is_match(ip).expect("Error"),
                "expected '{}' to match IPv6 with regex '{}'",
                ip,
                regex.as_str()
            );
        }

        // Invalid addresses
        let invalid = vec![
            "256.256.256.256",
            "256.1.1.1",
            "123.456.78.90",
            "1.2.3",
            "1.2.3.4.5",
            "gibberish",
            "2001::85a3::7334",
            "2001:db8:85a3:8a2e:370:7334:1",
            "",
            ":",
            ":::",
        ];
        for ip in invalid {
            assert!(
                !regex.is_match(ip).expect("Error"),
                "expected '{}' to NOT match with regex '{}'",
                ip,
                regex.as_str()
            );
        }
    }

    // Global cache mapping pattern -> compiled Regex
    static REGEX_CACHE: std::sync::LazyLock<RwLock<HashMap<String, Regex>>> =
        std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

    #[crate::ctb_test]
    #[expect(
        clippy::too_many_lines,
        reason = "comprehensive test fixture matrix"
    )]
    fn test_ip() {
        let v4 = v4();
        let v6 = v6();
        let v4not = v4not();
        let v6not = v6not();
        // v4, v6, v4not, v6not must be declared in this module
        // Build boundary & extract fixtures equivalent to the JS objects:
        let mut v6boundaries: HashMap<&str, Vec<&str>> = HashMap::new();
        v6boundaries.insert(
            "02001:0000:1234:0000:0000:C1C0:ABCD:0876",
            vec!["2001:0000:1234:0000:0000:C1C0:ABCD:0876"],
        );
        v6boundaries.insert(
            "fe80:0000:0000:0000:0204:61ff:fe9d:f156245",
            vec!["fe80:0000:0000:0000:0204:61ff:fe9d:f156"],
        );

        let mut v6extract: HashMap<&str, Vec<&str>> = HashMap::new();
        v6extract.insert("::1, ::2, ::3-::5", vec!["::1", "::2", "::3", "::5"]);
        v6extract
            .insert("::1  ::2 ::3 - ::5", vec!["::1", "::2", "::3", "::5"]);
        v6extract.insert(
            "::ffff:192.168.1.1 1::1.2.3.4",
            vec!["::ffff:192.168.1.1", "1::1.2.3.4"],
        );
        v6extract.insert(
            "02001:0000:1234:0000:0000:C1C0:ABCD:0876 a::xyz",
            vec!["2001:0000:1234:0000:0000:C1C0:ABCD:0876", "a::"],
        );

        let mut v4boundaries: HashMap<&str, Vec<&str>> = HashMap::new();
        v4boundaries.insert("0000000192.168.0.200", vec!["192.168.0.200"]);
        v4boundaries.insert("192.168.0.2000000000", vec!["192.168.0.200"]);

        let mut v4extract: HashMap<&str, Vec<&str>> = HashMap::new();
        v4extract.insert(
            "255.255.255.255 0.0.0.0",
            vec!["255.255.255.255", "0.0.0.0"],
        );
        v4extract.insert(
            "1.2.3.4, 6.7.8.9, 1.2.3.4-5.6.7.8",
            vec!["1.2.3.4", "6.7.8.9", "1.2.3.4", "5.6.7.8"],
        );
        v4extract.insert(
            "1.2.3.4 6.7.8.9 1.2.3.4 - 5.6.7.8",
            vec!["1.2.3.4", "6.7.8.9", "1.2.3.4", "5.6.7.8"],
        );
        v4extract.insert("192.168.0.2000000000", vec!["192.168.0.200"]);

        // 1) v4 exact matches
        for fixture in &v4 {
            let re = build_regex_rt(
                &RegexType::V4V6,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(
                re.is_match(fixture).expect("Error"),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 2) v4 extract from surrounding text
        for fixture in &v4 {
            let re = build_regex_rt(&RegexType::V4V6, None);
            let haystack = format!("foo {fixture} bar");
            let caps = re.find(&haystack).expect("Error");
            assert_eq!(
                caps.map(|m| m.as_str()),
                Some(fixture.as_str()),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 3) v4 inside joined text and includeBoundaries behavior
        for fixture in &v4 {
            let re = build_regex_rt(&RegexType::V4V6, None);
            assert!(re.is_match(&format!("foo{fixture}bar")).expect("Error"));

            let re_bound = build_regex_rt(
                &RegexType::V4V6,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(
                !re_bound
                    .is_match(&format!("foo{fixture}bar"))
                    .expect("Error"),
                "Expected to NOT match '{}' with regex '{}'",
                fixture,
                re_bound.as_str()
            );
        }

        // 4) v4not exact should not match
        for fixture in &v4not {
            let re = build_regex_rt(
                &RegexType::V4V6,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(
                !re.is_match(fixture).expect("Error"),
                "Expected to NOT match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 5) v6 exact matches
        for fixture in &v6 {
            let re = build_regex_rt(
                &RegexType::V4V6,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(
                re.is_match(fixture).expect("Error"),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 6) v6 extract from surrounding text
        for fixture in &v6 {
            let re = build_regex_rt(&RegexType::V4V6, None);
            let haystack = format!("foo {fixture} bar");
            let caps = re.find(&haystack).expect("Error");
            assert_eq!(
                caps.map(|m| m.as_str()),
                Some(fixture.as_str()),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 7) v6 inside joined text and includeBoundaries behavior
        for fixture in &v6 {
            let re = build_regex_rt(&RegexType::V4V6, None);
            assert!(
                re.is_match(&format!("foo{fixture}bar")).expect("Error"),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );

            let re_bound = build_regex_rt(
                &RegexType::V4V6,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(
                !re_bound
                    .is_match(&format!("foo{fixture}bar"))
                    .expect("Error"),
                "Expected to NOT match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 8) v6not exact should not match
        for fixture in &v6not {
            let re = build_regex_rt(
                &RegexType::V4V6,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(
                !re.is_match(fixture).expect("Error"),
                "Expected to NOT match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 9) v4 boundaries/extraction behavior
        for (fixture, expected) in &v4boundaries {
            let re = build_regex_rt(&RegexType::V4, None);
            assert!(
                re.is_match(fixture).expect("Error"),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );

            // collect all matches
            let matches: Vec<&str> = re
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert_eq!(
                &matches,
                expected,
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );

            let re_bound = build_regex_rt(
                &RegexType::V4,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(!re_bound.is_match(fixture).expect("Error"));
            let matches2: Vec<&str> = re_bound
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert!(
                matches2.is_empty(),
                "Expected to find no matches for '{}' with regex '{}'",
                fixture,
                re_bound.as_str()
            );
        }

        for (fixture, expected) in &v4extract {
            let re = build_regex_rt(&RegexType::V4, None);
            let matches: Vec<&str> = re
                .find_iter(fixture)
                .map(|m| m.expect("Error").as_str())
                .collect();
            assert_eq!(
                &matches,
                expected,
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        // 10) v6 boundaries/extraction behavior
        for (fixture, expected) in &v6boundaries {
            let re = build_regex_rt(&RegexType::V6, None);
            assert!(re.is_match(fixture).expect("Error"));
            let matches: Vec<&str> = re
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert_eq!(
                &matches,
                expected,
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );

            let re_bound = build_regex_rt(
                &RegexType::V6,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(
                !re_bound.is_match(fixture).expect("Error"),
                "Expected to NOT match '{}' with regex '{}'",
                fixture,
                re_bound.as_str()
            );
            let matches2: Vec<&str> = re_bound
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert!(
                matches2.is_empty(),
                "Expected to find no matches for '{}' with regex '{}'",
                fixture,
                re_bound.as_str()
            );
        }

        for (fixture, expected) in &v6extract {
            let re = build_regex_rt(&RegexType::V6, None);
            let matches: Vec<&str> = re
                .find_iter(fixture)
                .map(|m| m.expect("Error").as_str())
                .collect();
            assert_eq!(
                &matches,
                expected,
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }
    }

    #[crate::ctb_test]
    #[expect(
        clippy::too_many_lines,
        reason = "comprehensive test fixture matrix"
    )]
    fn test_ip_v4() {
        let v4 = v4();
        let v4not = v4not();
        // v4 exact
        for fixture in &v4 {
            let re = build_regex_rt(
                &RegexType::V4,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(re.is_match(fixture).expect("Error"));
        }

        for fixture in &v4 {
            let re = build_regex_rt(&RegexType::V4, None);
            let haystack = format!("foo {fixture} bar");
            let caps = re.find(&haystack).expect("Error");
            assert_eq!(
                caps.map(|m| m.as_str()),
                Some(fixture.as_str()),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        for fixture in &v4 {
            let re = build_regex_rt(&RegexType::V4, None);
            assert!(
                re.is_match(&format!("foo{fixture}bar")).expect("Error"),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );

            let re_bound = build_regex_rt(
                &RegexType::V4,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(
                !re_bound
                    .is_match(&format!("foo{fixture}bar"))
                    .expect("Error"),
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }

        for fixture in &v4not {
            let re = build_regex_rt(
                &RegexType::V4,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(!re.is_match(fixture).expect("Error"));
        }

        // v4 boundaries/extract
        let mut v4boundaries: HashMap<&str, Vec<&str>> = HashMap::new();
        v4boundaries.insert("0000000192.168.0.200", vec!["192.168.0.200"]);
        v4boundaries.insert("192.168.0.2000000000", vec!["192.168.0.200"]);

        let mut v4extract: HashMap<&str, Vec<&str>> = HashMap::new();
        v4extract.insert(
            "255.255.255.255 0.0.0.0",
            vec!["255.255.255.255", "0.0.0.0"],
        );
        v4extract.insert(
            "1.2.3.4, 6.7.8.9, 1.2.3.4-5.6.7.8",
            vec!["1.2.3.4", "6.7.8.9", "1.2.3.4", "5.6.7.8"],
        );
        v4extract.insert(
            "1.2.3.4 6.7.8.9 1.2.3.4 - 5.6.7.8",
            vec!["1.2.3.4", "6.7.8.9", "1.2.3.4", "5.6.7.8"],
        );
        v4extract.insert("192.168.0.2000000000", vec!["192.168.0.200"]);

        for (fixture, expected) in &v4boundaries {
            let re = build_regex_rt(&RegexType::V4, None);
            assert!(re.is_match(fixture).expect("Error"));
            let matches: Vec<&str> = re
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert_eq!(&matches, expected);

            let re_bound = build_regex_rt(
                &RegexType::V4,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(!re_bound.is_match(fixture).expect("Error"));
            let matches2: Vec<&str> = re_bound
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert!(matches2.is_empty());
        }

        for (fixture, expected) in &v4extract {
            let re = build_regex_rt(&RegexType::V4, None);
            let matches: Vec<&str> = re
                .find_iter(fixture)
                .map(|m| m.expect("Error").as_str())
                .collect();
            assert_eq!(&matches, expected);
        }
    }

    #[crate::ctb_test]
    #[expect(
        clippy::too_many_lines,
        reason = "comprehensive test fixture matrix"
    )]
    fn test_ip_v6() {
        let v6 = v6();
        let v6not = v6not();
        // v6 exact
        for fixture in &v6 {
            let re = build_regex_rt(
                &RegexType::V6,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(re.is_match(fixture).expect("Error"));
        }

        for fixture in &v6 {
            let re = build_regex_rt(&RegexType::V6, None);
            let haystack = format!("foo {fixture} bar");
            let caps = re.find(&haystack).expect("Error");
            assert_eq!(caps.map(|m| m.as_str()), Some(fixture.as_str()));
        }

        for fixture in &v6 {
            let re = build_regex_rt(&RegexType::V6, None);
            assert!(re.is_match(&format!("foo{fixture}bar")).expect("Error"));

            let re_bound = build_regex_rt(
                &RegexType::V6,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(
                !re_bound
                    .is_match(&format!("foo{fixture}bar"))
                    .expect("Error")
            );
        }

        for fixture in &v6not {
            let re = build_regex_rt(
                &RegexType::V6,
                Some(&Options {
                    exact: true,
                    include_boundaries: false,
                }),
            );
            assert!(!re.is_match(fixture).expect("Error"));
        }

        // v6 boundaries/extract
        let mut v6boundaries: HashMap<&str, Vec<&str>> = HashMap::new();
        v6boundaries.insert(
            "02001:0000:1234:0000:0000:C1C0:ABCD:0876",
            vec!["2001:0000:1234:0000:0000:C1C0:ABCD:0876"],
        );
        v6boundaries.insert(
            "fe80:0000:0000:0000:0204:61ff:fe9d:f156245",
            vec!["fe80:0000:0000:0000:0204:61ff:fe9d:f156"],
        );

        let mut v6extract: HashMap<&str, Vec<&str>> = HashMap::new();
        v6extract.insert("::1, ::2, ::3-::5", vec!["::1", "::2", "::3", "::5"]);
        v6extract
            .insert("::1  ::2 ::3 - ::5", vec!["::1", "::2", "::3", "::5"]);
        v6extract.insert(
            "::ffff:192.168.1.1 1::1.2.3.4",
            vec!["::ffff:192.168.1.1", "1::1.2.3.4"],
        );
        v6extract.insert(
            "02001:0000:1234:0000:0000:C1C0:ABCD:0876 a::xyz",
            vec!["2001:0000:1234:0000:0000:C1C0:ABCD:0876", "a::"],
        );

        for (fixture, expected) in &v6boundaries {
            let re = build_regex_rt(&RegexType::V6, None);
            assert!(re.is_match(fixture).expect("Error"));
            let matches: Vec<&str> = re
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert_eq!(
                &matches,
                expected,
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );

            let re_bound = build_regex_rt(
                &RegexType::V6,
                Some(&Options {
                    exact: false,
                    include_boundaries: true,
                }),
            );
            assert!(
                !re_bound.is_match(fixture).expect("Error"),
                "Expected to NOT match '{}' with regex '{}'",
                fixture,
                re_bound.as_str()
            );
            let matches2: Vec<&str> = re_bound
                .find(fixture)
                .expect("Error")
                .map(|m| m.as_str())
                .into_iter()
                .collect();
            assert!(
                matches2.is_empty(),
                "Expected to find no matches for '{}' with regex '{}'",
                fixture,
                re_bound.as_str()
            );
        }

        for (fixture, expected) in &v6extract {
            let re = build_regex_rt(&RegexType::V6, None);
            let matches: Vec<&str> = re
                .find_iter(fixture)
                .map(|m| m.expect("Error").as_str())
                .collect();
            assert_eq!(
                &matches,
                expected,
                "Expected to match '{}' with regex '{}'",
                fixture,
                re.as_str()
            );
        }
    }

    #[crate::ctb_test]
    #[should_panic(
        expected = "Cannot set both exact and include_boundaries options to true"
    )]
    fn test_disallows_both_options() {
        get_regex(
            &RegexType::V4V6,
            Some(&Options {
                exact: true,
                include_boundaries: true,
            }),
        );
    }

    #[crate::ctb_test]
    fn test_compatible() {
        // Test that the get_regex function produces the same regex strings as the original ip-regex library for various options.
        let inexact_no_boundaries = Some(&Options {
            exact: false,
            include_boundaries: false,
        });
        let inexact_boundaries = Some(&Options {
            exact: false,
            include_boundaries: true,
        });
        let exact_no_boundaries = Some(&Options {
            exact: true,
            include_boundaries: false,
        });

        let both_inexact_no_boundaries =
            get_regex(&RegexType::V4V6, inexact_no_boundaries);
        let expected = "(?:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3})|(?:(?:(?:[a-fA-F\\d]{1,4}:){7}(?:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){6}(?:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){5}(?::(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,2}|:)|(?:[a-fA-F\\d]{1,4}:){4}(?:(?::[a-fA-F\\d]{1,4}){0,1}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,3}|:)|(?:[a-fA-F\\d]{1,4}:){3}(?:(?::[a-fA-F\\d]{1,4}){0,2}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,4}|:)|(?:[a-fA-F\\d]{1,4}:){2}(?:(?::[a-fA-F\\d]{1,4}){0,3}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,5}|:)|(?:[a-fA-F\\d]{1,4}:){1}(?:(?::[a-fA-F\\d]{1,4}){0,4}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,6}|:)|(?::(?:(?::[a-fA-F\\d]{1,4}){0,5}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,7}|:)))(?:%[0-9a-zA-Z]{1,})?)";
        assert_eq!(both_inexact_no_boundaries, expected);

        let both_inexact_boundaries =
            get_regex(&RegexType::V4V6, inexact_boundaries);
        let expected = "(?:(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$))(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$)))|(?:(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$))(?:(?:[a-fA-F\\d]{1,4}:){7}(?:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){6}(?:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){5}(?::(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,2}|:)|(?:[a-fA-F\\d]{1,4}:){4}(?:(?::[a-fA-F\\d]{1,4}){0,1}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,3}|:)|(?:[a-fA-F\\d]{1,4}:){3}(?:(?::[a-fA-F\\d]{1,4}){0,2}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,4}|:)|(?:[a-fA-F\\d]{1,4}:){2}(?:(?::[a-fA-F\\d]{1,4}){0,3}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,5}|:)|(?:[a-fA-F\\d]{1,4}:){1}(?:(?::[a-fA-F\\d]{1,4}){0,4}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,6}|:)|(?::(?:(?::[a-fA-F\\d]{1,4}){0,5}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,7}|:)))(?:%[0-9a-zA-Z]{1,})?(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$)))";
        assert_eq!(both_inexact_boundaries, expected);

        let both_exact_no_boundaries =
            get_regex(&RegexType::V4V6, exact_no_boundaries);
        let expected = "(?:^(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}$)|(?:^(?:(?:[a-fA-F\\d]{1,4}:){7}(?:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){6}(?:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){5}(?::(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,2}|:)|(?:[a-fA-F\\d]{1,4}:){4}(?:(?::[a-fA-F\\d]{1,4}){0,1}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,3}|:)|(?:[a-fA-F\\d]{1,4}:){3}(?:(?::[a-fA-F\\d]{1,4}){0,2}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,4}|:)|(?:[a-fA-F\\d]{1,4}:){2}(?:(?::[a-fA-F\\d]{1,4}){0,3}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,5}|:)|(?:[a-fA-F\\d]{1,4}:){1}(?:(?::[a-fA-F\\d]{1,4}){0,4}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,6}|:)|(?::(?:(?::[a-fA-F\\d]{1,4}){0,5}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,7}|:)))(?:%[0-9a-zA-Z]{1,})?$)";
        assert_eq!(both_exact_no_boundaries, expected);

        let v4_inexact_no_boundaries =
            get_regex(&RegexType::V4, inexact_no_boundaries);
        let expected = "(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}";
        assert_eq!(v4_inexact_no_boundaries, expected);

        let v4_inexact_boundaries =
            get_regex(&RegexType::V4, inexact_boundaries);
        let expected = "(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$))(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$))";
        assert_eq!(v4_inexact_boundaries, expected);

        let v4_exact_no_boundaries =
            get_regex(&RegexType::V4, exact_no_boundaries);
        let expected = "^(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}$";
        assert_eq!(v4_exact_no_boundaries, expected);

        let v6_inexact_no_boundaries =
            get_regex(&RegexType::V6, inexact_no_boundaries);
        let expected = "(?:(?:[a-fA-F\\d]{1,4}:){7}(?:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){6}(?:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){5}(?::(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,2}|:)|(?:[a-fA-F\\d]{1,4}:){4}(?:(?::[a-fA-F\\d]{1,4}){0,1}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,3}|:)|(?:[a-fA-F\\d]{1,4}:){3}(?:(?::[a-fA-F\\d]{1,4}){0,2}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,4}|:)|(?:[a-fA-F\\d]{1,4}:){2}(?:(?::[a-fA-F\\d]{1,4}){0,3}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,5}|:)|(?:[a-fA-F\\d]{1,4}:){1}(?:(?::[a-fA-F\\d]{1,4}){0,4}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,6}|:)|(?::(?:(?::[a-fA-F\\d]{1,4}){0,5}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,7}|:)))(?:%[0-9a-zA-Z]{1,})?";
        assert_eq!(v6_inexact_no_boundaries, expected);

        let v6_inexact_boundaries =
            get_regex(&RegexType::V6, inexact_boundaries);
        let expected = "(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$))(?:(?:[a-fA-F\\d]{1,4}:){7}(?:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){6}(?:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){5}(?::(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,2}|:)|(?:[a-fA-F\\d]{1,4}:){4}(?:(?::[a-fA-F\\d]{1,4}){0,1}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,3}|:)|(?:[a-fA-F\\d]{1,4}:){3}(?:(?::[a-fA-F\\d]{1,4}){0,2}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,4}|:)|(?:[a-fA-F\\d]{1,4}:){2}(?:(?::[a-fA-F\\d]{1,4}){0,3}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,5}|:)|(?:[a-fA-F\\d]{1,4}:){1}(?:(?::[a-fA-F\\d]{1,4}){0,4}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,6}|:)|(?::(?:(?::[a-fA-F\\d]{1,4}){0,5}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,7}|:)))(?:%[0-9a-zA-Z]{1,})?(?:(?<=\\s|^)(?=[a-fA-F\\d:])|(?<=[a-fA-F\\d:])(?=\\s|$))";
        assert_eq!(v6_inexact_boundaries, expected);

        let v6_exact_no_boundaries =
            get_regex(&RegexType::V6, exact_no_boundaries);
        let expected = "^(?:(?:[a-fA-F\\d]{1,4}:){7}(?:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){6}(?:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|:[a-fA-F\\d]{1,4}|:)|(?:[a-fA-F\\d]{1,4}:){5}(?::(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,2}|:)|(?:[a-fA-F\\d]{1,4}:){4}(?:(?::[a-fA-F\\d]{1,4}){0,1}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,3}|:)|(?:[a-fA-F\\d]{1,4}:){3}(?:(?::[a-fA-F\\d]{1,4}){0,2}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,4}|:)|(?:[a-fA-F\\d]{1,4}:){2}(?:(?::[a-fA-F\\d]{1,4}){0,3}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,5}|:)|(?:[a-fA-F\\d]{1,4}:){1}(?:(?::[a-fA-F\\d]{1,4}){0,4}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,6}|:)|(?::(?:(?::[a-fA-F\\d]{1,4}){0,5}:(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}|(?::[a-fA-F\\d]{1,4}){1,7}|:)))(?:%[0-9a-zA-Z]{1,})?$";
        assert_eq!(v6_exact_no_boundaries, expected);
    }

    fn v4() -> Vec<String> {
        let vec = vec![
            "0.0.0.0",
            "8.8.8.8",
            "127.0.0.1",
            "100.100.100.100",
            "192.168.0.1",
            "18.101.25.153",
            "123.23.34.2",
            "172.26.168.134",
            "212.58.241.131",
            "128.0.0.0",
            "23.71.254.72",
            "223.255.255.255",
            "192.0.2.235",
            "99.198.122.146",
            "46.51.197.88",
            "173.194.34.134",
        ];

        vec.into_iter().map(String::from).collect()
    }

    fn v4not() -> Vec<String> {
        let vec = vec![
            ".100.100.100.100",
            "100..100.100.100.",
            "100.100.100.100.",
            "999.999.999.999",
            "256.256.256.256",
            "256.100.100.100.100",
            "123.123.123",
            "http://123.123.123",
            "1000.2.3.4",
            "999.2.3.4",
        ];

        vec.into_iter().map(String::from).collect()
    }

    #[expect(
        clippy::too_many_lines,
        reason = "comprehensive test fixture matrix"
    )]
    fn v6() -> Vec<String> {
        let vec = vec![
            "::",
            "1::",
            "::1",
            "1::8",
            "1::7:8",
            "1:2:3:4:5:6:7:8",
            "1:2:3:4:5:6::8",
            "1:2:3:4:5:6:7::",
            "1:2:3:4:5::7:8",
            "1:2:3:4:5::8",
            "1:2:3::8",
            "1::4:5:6:7:8",
            "1::6:7:8",
            "1::3:4:5:6:7:8",
            "1:2:3:4::6:7:8",
            "1:2::4:5:6:7:8",
            "::2:3:4:5:6:7:8",
            "1:2::8",
            "2001:0000:1234:0000:0000:C1C0:ABCD:0876",
            "3ffe:0b00:0000:0000:0001:0000:0000:000a",
            "FF02:0000:0000:0000:0000:0000:0000:0001",
            "0000:0000:0000:0000:0000:0000:0000:0001",
            "0000:0000:0000:0000:0000:0000:0000:0000",
            "::ffff:192.168.1.26",
            "2::10",
            "ff02::1",
            "fe80::",
            "2002::",
            "2001:db8::",
            "2001:0db8:1234::",
            "::ffff:0:0",
            "::ffff:192.168.1.1",
            "1:2:3:4::8",
            "1::2:3:4:5:6:7",
            "1::2:3:4:5:6",
            "1::2:3:4:5",
            "1::2:3:4",
            "1::2:3",
            "::2:3:4:5:6:7",
            "::2:3:4:5:6",
            "::2:3:4:5",
            "::2:3:4",
            "::2:3",
            "::8",
            "1:2:3:4:5:6::",
            "1:2:3:4:5::",
            "1:2:3:4::",
            "1:2:3::",
            "1:2::",
            "1:2:3:4::7:8",
            "1:2:3::7:8",
            "1:2::7:8",
            "1:2:3:4:5:6:1.2.3.4",
            "1:2:3:4:5::1.2.3.4",
            "1:2:3:4::1.2.3.4",
            "1:2:3::1.2.3.4",
            "1:2::1.2.3.4",
            "1::1.2.3.4",
            "1:2:3:4::5:1.2.3.4",
            "1:2:3::5:1.2.3.4",
            "1:2::5:1.2.3.4",
            "1::5:1.2.3.4",
            "1::5:11.22.33.44",
            "fe80::217:f2ff:254.7.237.98",
            "fe80::217:f2ff:fe07:ed62",
            "2001:DB8:0:0:8:800:200C:417A",
            "FF01:0:0:0:0:0:0:101",
            "0:0:0:0:0:0:0:1",
            "0:0:0:0:0:0:0:0",
            "2001:DB8::8:800:200C:417A",
            "FF01::101",
            "0:0:0:0:0:0:13.1.68.3",
            "0:0:0:0:0:FFFF:129.144.52.38",
            "::13.1.68.3",
            "::FFFF:129.144.52.38",
            "fe80:0000:0000:0000:0204:61ff:fe9d:f156",
            "fe80:0:0:0:204:61ff:fe9d:f156",
            "fe80::204:61ff:fe9d:f156",
            "fe80:0:0:0:204:61ff:254.157.241.86",
            "fe80::204:61ff:254.157.241.86",
            "fe80::1",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "2001:db8:85a3:0:0:8a2e:370:7334",
            "2001:db8:85a3::8a2e:370:7334",
            "2001:0db8:0000:0000:0000:0000:1428:57ab",
            "2001:0db8:0000:0000:0000::1428:57ab",
            "2001:0db8:0:0:0:0:1428:57ab",
            "2001:0db8:0:0::1428:57ab",
            "2001:0db8::1428:57ab",
            "2001:db8::1428:57ab",
            "::ffff:12.34.56.78",
            "::ffff:0c22:384e",
            "2001:0db8:1234:0000:0000:0000:0000:0000",
            "2001:0db8:1234:ffff:ffff:ffff:ffff:ffff",
            "2001:db8:a::123",
            "::ffff:192.0.2.128",
            "::ffff:c000:280",
            "a:b:c:d:e:f:f1:f2",
            "a:b:c::d:e:f:f1",
            "a:b:c::d:e:f",
            "a:b:c::d:e",
            "a:b:c::d",
            "::a",
            "::a:b:c",
            "::a:b:c:d:e:f:f1",
            "a::",
            "a:b:c::",
            "a:b:c:d:e:f:f1::",
            "a:bb:ccc:dddd:000e:00f:0f::",
            "0:a:0:a:0:0:0:a",
            "0:a:0:0:a:0:0:a",
            "2001:db8:1:1:1:1:0:0",
            "2001:db8:1:1:1:0:0:0",
            "2001:db8:1:1:0:0:0:0",
            "2001:db8:1:0:0:0:0:0",
            "2001:db8:0:0:0:0:0:0",
            "2001:0:0:0:0:0:0:0",
            "A:BB:CCC:DDDD:000E:00F:0F::",
            "0:0:0:0:0:0:0:a",
            "0:0:0:0:a:0:0:0",
            "0:0:0:a:0:0:0:0",
            "a:0:0:a:0:0:a:a",
            "a:0:0:a:0:0:0:a",
            "a:0:0:0:a:0:0:a",
            "a:0:0:0:a:0:0:0",
            "a:0:0:0:0:0:0:0",
            "fe80::7:8%eth0",
            "fe80::7:8%1",
        ];

        vec.into_iter().map(String::from).collect()
    }

    fn v6not() -> Vec<String> {
        let vec = vec![
            "",
            "1:",
            ":1",
            "11:36:12",
            "02001:0000:1234:0000:0000:C1C0:ABCD:0876",
            "2001:0000:1234:0000:00001:C1C0:ABCD:0876",
            "2001:0000:1234: 0000:0000:C1C0:ABCD:0876",
            "2001:1:1:1:1:1:255Z255X255Y255",
            "3ffe:0b00:0000:0001:0000:0000:000a",
            "FF02:0000:0000:0000:0000:0000:0000:0000:0001",
            "3ffe:b00::1::a",
            "::1111:2222:3333:4444:5555:6666::",
            "1:2:3::4:5::7:8",
            "12345::6:7:8",
            "1::5:400.2.3.4",
            "1::5:260.2.3.4",
            "1::5:256.2.3.4",
            "1::5:1.256.3.4",
            "1::5:1.2.256.4",
            "1::5:1.2.3.256",
            "1::5:300.2.3.4",
            "1::5:1.300.3.4",
            "1::5:1.2.300.4",
            "1::5:1.2.3.300",
            "1::5:900.2.3.4",
            "1::5:1.900.3.4",
            "1::5:1.2.900.4",
            "1::5:1.2.3.900",
            "1::5:300.300.300.300",
            "1::5:3000.30.30.30",
            "1::400.2.3.4",
            "1::260.2.3.4",
            "1::256.2.3.4",
            "1::1.256.3.4",
            "1::1.2.256.4",
            "1::1.2.3.256",
            "1::300.2.3.4",
            "1::1.300.3.4",
            "1::1.2.300.4",
            "1::1.2.3.300",
            "1::900.2.3.4",
            "1::1.900.3.4",
            "1::1.2.900.4",
            "1::1.2.3.900",
            "1::300.300.300.300",
            "1::3000.30.30.30",
            "::400.2.3.4",
            "::260.2.3.4",
            "::256.2.3.4",
            "::1.256.3.4",
            "::1.2.256.4",
            "::1.2.3.256",
            "::300.2.3.4",
            "::1.300.3.4",
            "::1.2.300.4",
            "::1.2.3.300",
            "::900.2.3.4",
            "::1.900.3.4",
            "::1.2.900.4",
            "::1.2.3.900",
            "::300.300.300.300",
            "::3000.30.30.30",
            "2001:DB8:0:0:8:800:200C:417A:221",
            "FF01::101::2",
            "1111:2222:3333:4444::5555:",
            "1111:2222:3333::5555:",
            "1111:2222::5555:",
            "1111::5555:",
            "::5555:",
            ":::",
            "1111:",
            ":",
            ":1111:2222:3333:4444::5555",
            ":1111:2222:3333::5555",
            ":1111:2222::5555",
            ":1111::5555",
            ":::5555",
            "1.2.3.4:1111:2222:3333:4444::5555",
            "1.2.3.4:1111:2222:3333::5555",
            "1.2.3.4:1111:2222::5555",
            "1.2.3.4:1111::5555",
            "1.2.3.4::5555",
            "1.2.3.4::",
            "fe80:0000:0000:0000:0204:61ff:254.157.241.086",
            "123",
            "ldkfj",
            "2001::FFD3::57ab",
            "2001:db8:85a3::8a2e:37023:7334",
            "2001:db8:85a3::8a2e:370k:7334",
            "1:2:3:4:5:6:7:8:9",
            "1::2::3",
            "1:::3:4:5",
            "1:2:3::4:5:6:7:8:9",
            "::ffff:2.3.4",
            "::ffff:257.1.2.3",
            "::ffff:12345678901234567890.1.26",
        ];

        vec.into_iter().map(String::from).collect()
    }
}
