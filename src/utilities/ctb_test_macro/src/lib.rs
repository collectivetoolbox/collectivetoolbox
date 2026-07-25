// Copyright (C) 2019-2025 Daniel Mueller <deso@posteo.net>
// Copyright (c) 2019 Daniel Henry-Mantilla
// Copyright (C) 2020-2023 Threema GmbH, Danilo Bargen
// SPDX-License-Identifier for parts derived from test-log and tracing-test: (Apache-2.0 OR MIT)
// Parts derived from rust-function_name: MIT
/**
 * `ctb_test` procedural macro for running code before tests.
 * Based on <https://github.com/d-e-s-o/test-log>
 * and <https://github.com/danielhenrymantilla/rust-function_name>
 * with some additional modifications.
 * See licenses at end of file.
 */
use std::borrow::Cow;
use std::sync::{Mutex, OnceLock};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;

use quote::quote;

use ::proc_macro::{Delimiter, Group, Literal, TokenTree};
use syn::Attribute;
use syn::Expr;
use syn::ItemFn;
use syn::Lit;
use syn::Meta;
use syn::parse::Parse;
use syn::parse_macro_input;

use syn::{Stmt, parse};

enum CtbTestAttr {
    Default,
    TokioMultiThread,
    Passthrough(Tokens),
}

fn parse_ctb_test_attr(attr: TokenStream) -> syn::Result<CtbTestAttr> {
    if attr.is_empty() {
        return Ok(CtbTestAttr::Default);
    }

    if let Ok(lit) = syn::parse::<syn::LitStr>(attr.clone()) {
        return match lit.value().as_str() {
            "tokio" => Ok(CtbTestAttr::TokioMultiThread),
            _ => Err(syn::Error::new_spanned(
                lit,
                "Unknown ctb_test string option. Expected \"tokio\".",
            )),
        };
    }

    Ok(CtbTestAttr::Passthrough(Tokens::from(attr)))
}

/// Mini `quote!` implementation,
/// can only interpolate `impl IntoIterator<Item = TokenTree>`.
macro_rules! mini_quote {
    (
        @$q:tt
        { $($code:tt)* } $($rest:tt)*
    ) => (
        $q.push(
            TokenTree::Group(Group::new(
                Delimiter::Brace,
                mini_quote!($($code)*)
            ))
        );
        mini_quote!(@$q $($rest)*);
    );

    (
        @$q:tt
        [ $($code:tt)* ]
        $($rest:tt)*
    ) => (
        $q.push(
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                mini_quote!($($code)*)
            ))
        );
        mini_quote!(@$q $($rest)*);
    );

    (
        @$q:tt
        ( $($code:tt)* )
        $($rest:tt)*
    ) => (
        $q.push(
            TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                mini_quote!($($code)*)
            ))
        );
        mini_quote!(@$q $($rest)*);
    );

    (
        @$q:tt
        #$var:ident
        $($rest:tt)*
    ) => (
        $q.extend($var);
        mini_quote!(@$q $($rest)*);
    );

    (
        @$q:tt
        $tt:tt $($rest:tt)*
    ) => (
        $q.extend(
            stringify!($tt)
                .parse::<TokenStream>()
                .unwrap()
        );
        mini_quote!(@$q $($rest)*);
    );

    (
        @$q:tt
        /* nothign left */
    ) => ();

    (
        $($code:tt)*
    ) => ({
        let mut _q = Vec::<TokenTree>::new();
        mini_quote!(@_q $($code)*);
        _q.into_iter().collect::<TokenStream>()
    });
}

// Documented in `test-log` crate's re-export.
#[expect(missing_docs, reason = "ctb_test is documented in the re-export")]
#[proc_macro_attribute]
pub fn ctb_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_itemfn = item.clone();
    let item_itemfn = parse_macro_input!(item_itemfn as ItemFn);
    let test_log_applied = try_test(attr.clone(), item, item_itemfn)
        .unwrap_or_else(syn::Error::into_compile_error);

    named_impl(test_log_applied.into()).unwrap_or_else(|err| {
        let err = Some(TokenTree::from(Literal::string(err)));
        mini_quote!(::core::compile_error! { #err })
    })
}

fn parse_attrs(
    attrs: Vec<Attribute>,
) -> syn::Result<(AttributeArgs, Vec<Attribute>)> {
    let attribute_args = AttributeArgs::default();

    Ok((attribute_args, attrs))
}

// Check whether given attribute is a test attribute of forms:
// * `#[test]`
// * `#[core::prelude::*::test]` or `#[::core::prelude::*::test]`
// * `#[std::prelude::*::test]` or `#[::std::prelude::*::test]`
fn is_test_attribute(attr: &Attribute) -> bool {
    let path = match &attr.meta {
        syn::Meta::Path(path) => path,
        _ => return false,
    };
    let candidates = [
        ["core", "prelude", "*", "test"],
        ["std", "prelude", "*", "test"],
    ];
    let is_single_test_segment = path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments.first().is_some_and(|seg| seg.arguments.is_none() && seg.ident == "test");
    if is_single_test_segment {
        return true;
    } else if path.segments.len() != candidates.first().map_or(0, |c| c.len()) {
        return false;
    }
    candidates.into_iter().any(|segments| {
        path.segments.iter().zip(segments).all(|(segment, path)| {
            segment.arguments.is_none()
                && (path == "*" || segment.ident == path)
        })
    })
}

/// Registered scopes.
///
/// By default, every traced test registers a span with the function name.
/// However, since multiple tests can share the same function name, in case
/// of conflict, a counter is appended.
///
/// This vector is used to store all already registered scopes.
fn registered_scopes() -> &'static Mutex<Vec<String>> {
    static REGISTERED_SCOPES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    REGISTERED_SCOPES.get_or_init(|| Mutex::new(vec![]))
}

/// Check whether this test function name is already taken as scope. If yes, a
/// counter is appended to make it unique. In the end, a unique scope is returned.
fn get_free_scope(mut test_fn_name: String) -> String {
    let mut vec = registered_scopes().lock().unwrap();
    let mut counter = 1_usize;
    let len = test_fn_name.len();
    while vec.contains(&test_fn_name) {
        counter = counter.saturating_add(1);
        test_fn_name.replace_range(len.., &counter.to_string());
    }
    vec.push(test_fn_name.clone());
    test_fn_name
}

#[expect(clippy::too_many_lines, reason = "try_test implements complex macro expansion logic")]
fn try_test(
    attr: TokenStream,
    item: TokenStream,
    _input: ItemFn,
) -> syn::Result<Tokens> {
    // Parse annotated function
    let mut function: ItemFn = parse(item).expect("Could not parse ItemFn");

    // Determine scope
    let scope = get_free_scope(function.sig.ident.to_string());

    // Prepare code that should be injected at the start of the function
    let init = parse::<Stmt>(
        quote! {
            crate::testing::logging_test_internal::INITIALIZED.call_once(|| {
                // no-op, uses subscriber from logging.rs instead
            });
        }
        .into(),
    )
    .expect("Could not parse quoted statement init");
    let span = parse::<Stmt>(
        quote! {
            let span = tracing::info_span!(#scope);
        }
        .into(),
    )
    .expect("Could not parse quoted statement span");
    let enter = parse::<Stmt>(
        quote! {
            let _enter = span.enter();
        }
        .into(),
    )
    .expect("Could not parse quoted statement enter");
    let logs_contain_fn = parse::<Stmt>(
        quote! {
            fn logs_contain(val: &str) -> bool {
                crate::testing::logging_test_internal::logs_with_scope_contain(#scope, val)
            }

        }
        .into(),
    )
    .expect("Could not parse quoted statement logs_contain_fn");
    let logs_assert_fn = parse::<Stmt>(
        quote! {
            /// Run a function against the log lines. If the function returns
            /// an `Err`, panic. This can be used to run arbitrary assertion
            /// logic against the logs.
            fn logs_assert(f: impl Fn(&[&str]) -> std::result::Result<(), String>) {
                match crate::testing::logging_test_internal::logs_assert(#scope, f) {
                    Ok(()) => {},
                    Err(msg) => panic!("The logs_assert function returned an error: {}", msg),
                };
            }
        }
        .into(),
    )
    .expect("Could not parse quoted statement logs_assert_fn");

    // Inject code into function
    function.block.stmts.insert(0, init);
    function.block.stmts.insert(1, span);
    function.block.stmts.insert(2, enter);
    function.block.stmts.insert(3, logs_contain_fn);
    function.block.stmts.insert(4, logs_assert_fn);

    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = function;

    let (attribute_args, ignored_attrs) = parse_attrs(attrs)?;
    let run_pre_test = run_pre_test(&attribute_args);

    let ctb_attr = parse_ctb_test_attr(attr)?;
    let has_test = ignored_attrs.iter().any(is_test_attribute);

    let mut tokio_returns_unit = false;

    let (inner_test, generated_test) = match ctb_attr {
        CtbTestAttr::Default => {
            let generated_test = if has_test {
                quote! {}
            } else {
                quote! { #[::core::prelude::v1::test] }
            };
            (quote! {}, generated_test)
        }
        CtbTestAttr::Passthrough(ref attr) => (quote! { #[#attr] }, quote! {}),
        CtbTestAttr::TokioMultiThread => {
            if sig.asyncness.is_none() {
                return Err(syn::Error::new_spanned(
                    sig.fn_token,
                    r#"#[ctb_test("tokio")] requires an async fn"#,
                ));
            }

            // Make the outer test sync, but preserve its return type.
            sig.asyncness = None;
            tokio_returns_unit = matches!(sig.output, syn::ReturnType::Default);

            let generated_test = if has_test {
                quote! {}
            } else {
                quote! { #[::core::prelude::v1::test] }
            };

            (quote! {}, generated_test)
        }
    };

    let result = if let CtbTestAttr::TokioMultiThread = ctb_attr {
        if tokio_returns_unit {
            quote! {
                #(#ignored_attrs)*
                #generated_test
                #vis #sig {
                    mod init {
                        pub fn init() {
                            #run_pre_test
                        }
                    }

                    let runtime = match ::tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                    {
                        Ok(runtime) => runtime,
                        Err(err) => {
                            // No `Result` to return here; failing the test
                            // is the only reasonable option.
                            panic!("Failed to build Tokio runtime: {err}");
                        }
                    };

                    runtime.block_on(crate::testing::scope_current_test_name(
                        function_name!().to_string(),
                        async move {
                            init::init();
                            #block
                        }
                    ));
                }
            }
        } else {
            quote! {
                #(#ignored_attrs)*
                #generated_test
                #vis #sig {
                    mod init {
                        pub fn init() {
                            #run_pre_test
                        }
                    }

                    let runtime = ::tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()?;

                    runtime.block_on(crate::testing::scope_current_test_name(
                        function_name!().to_string(),
                        async move {
                            init::init();
                            #block
                        }
                    ))
                }
            }
        }
    } else {
        quote! {
            #inner_test
            #(#ignored_attrs)*
            #generated_test
            #vis #sig {
                mod init {
                    pub fn init() {
                        #run_pre_test
                    }
                }

                init::init();

                #block
            }
        }
    };

    Ok(result)
}

#[derive(Debug, Default)]
struct AttributeArgs {
    default_log_filter: Option<Cow<'static, str>>,
}

impl AttributeArgs {
    fn try_parse_attr_single(&mut self, attr: &Attribute) -> syn::Result<bool> {
        if !attr.path().is_ident("test_log") {
            return Ok(false);
        }

        let nested_meta = attr.parse_args_with(Meta::parse)?;
        let name_value = if let Meta::NameValue(name_value) = nested_meta {
            name_value
        } else {
            return Err(syn::Error::new_spanned(
                &nested_meta,
                "Expected NameValue syntax, e.g. 'default_log_filter = \"debug\"'.",
            ));
        };

        let ident = if let Some(ident) = name_value.path.get_ident() {
            ident
        } else {
            return Err(syn::Error::new_spanned(
                &name_value.path,
                "Expected NameValue syntax, e.g. 'default_log_filter = \"debug\"'.",
            ));
        };

        let arg_ref = if ident == "default_log_filter" {
            &mut self.default_log_filter
        } else {
            return Err(syn::Error::new_spanned(
                &name_value.path,
                "Unrecognized attribute, see documentation for details.",
            ));
        };

        if let Expr::Lit(lit) = &name_value.value
            && let Lit::Str(lit_str) = &lit.lit
        {
            *arg_ref = Some(Cow::from(lit_str.value()));
        }

        // If we couldn't parse the value on the right-hand side because it was some
        // unexpected type, e.g. #[test_log::log(default_log_filter=10)], return an error.
        if arg_ref.is_none() {
            return Err(syn::Error::new_spanned(
                &name_value.value,
                "Failed to parse value, expected a string",
            ));
        }

        Ok(true)
    }
}

/// Expand the initialization code.
fn run_pre_test(_attribute_args: &AttributeArgs) -> Tokens {
    quote! {
        {
            crate::testing::set_current_test_name(function_name!().to_string());
            #[allow(unexpected_cfgs)]
            {
                #[cfg(feature = "ctb-harness")]
                {
                    crate::workspace::boot_for_test();
                }
                #[cfg(not(feature = "ctb-harness"))]
                {
                    let ctb_macro_logging_temp = crate::logging::setup_logger(
                        "workspace".to_string(),
                        "workspace".to_string()
                    );
                    if (!ctb_macro_logging_temp.is_ok()) {
                        let ctb_macro_logging_err_mess = ctb_macro_logging_temp.err().unwrap_or(crate::utilities::anyhow::anyhow!("Error getting macro logging error message"));
                        let ctb_macro_logging_err_mess = format!("{:?}", ctb_macro_logging_err_mess);
                        if (!ctb_macro_logging_err_mess.contains("global default trace dispatcher has already been set")) {
                            println!(
                                "###### ERROR ###### Failed to set up test logger: {}",
                                ctb_macro_logging_err_mess
                            );
                        }
                    }
                }
            }
        }
    }
}

fn named_impl(input: TokenStream) -> Result<TokenStream, &'static str> {
    let tts = &mut input.into_iter().peekable();

    let mut input = Vec::<TokenTree>::new();

    // `#` `[…]` attributes:
    while matches!(tts.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '#')
    {
        input.extend(tts.take(2));
    }

    // rest but scan the tt right after `fn`.
    let fname = loop {
        let tt = tts.next().unwrap();
        if matches!(tt, TokenTree::Ident(ref ident) if ident.to_string() == "fn")
        {
            input.push(tt);
            let fname = tts.peek().unwrap().to_string();
            input.extend(tts);
            break Some(TokenTree::from(Literal::string(&fname)));
        }
        input.push(tt);
    };

    let g = match input.last_mut() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g,
        _ => return Err("expected a `fn`"),
    };
    let g_span = g.span();
    *g = Group::new(g.delimiter(), {
        let mut body = mini_quote!(
            #[allow(unused_macros)]
            macro_rules! function_name {() => (
                #fname
            )}
        );
        body.extend(g.stream());
        body
    });
    g.set_span(g_span);
    Ok(input.into_iter().collect())
}

/*

// From tracing-test:

MIT License

Copyright (C) 2020-2023 Threema GmbH, Danilo Bargen

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS


// From rust-function-name:
MIT License

Copyright (c) 2019 Daniel Henry-Mantilla

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


// From test-log:
MIT License

Copyright (c) 2020 Daniel Müller

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


                              Apache License
                        Version 2.0, January 2004
                     https://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS

APPENDIX: How to apply the Apache License to your work.

   To apply the Apache License to your work, attach the following
   boilerplate notice, with the fields enclosed by brackets "[]"
   replaced with your own identifying information. (Don't include
   the brackets!)  The text should be enclosed in the appropriate
   comment syntax for the file format. We also recommend that a
   file or class name and description of purpose be included on the
   same "printed page" as the copyright notice for easier
   identification within third-party archives.

Copyright [yyyy] [name of copyright owner]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

*/
