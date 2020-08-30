// vim: tw=80
//! Test double adapter for use with Mockall
//!
//! This crate provides `[#double]`, which can swap in Mock objects for real
//! objects while in test mode.  It's intended to be used in tandeom with the
//! [`mockall`](https://docs.rs/mockall/latest/mockall) crate.  However, it gets
//! its own crate so that you don't have to build all of Mockall when your crate
//! isn't in test mode.

#![cfg_attr(feature = "nightly", feature(proc_macro_diagnostic))]
//#![cfg_attr(test, deny(warnings))]
extern crate proc_macro;

use cfg_if::cfg_if;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    *,
    //punctuated::Punctuated,
    spanned::Spanned
};

cfg_if! {
    // proc-macro2's Span::unstable method requires the nightly feature, and it
    // doesn't work in test mode.
    // https://github.com/alexcrichton/proc-macro2/issues/159
    if #[cfg(all(feature = "nightly", not(test)))] {
        fn compile_error(span: Span, msg: &'static str) {
            span.unstable()
                .error(msg)
                .emit();
        }
    } else {
        fn compile_error(_span: Span, msg: &str) {
            panic!("{}.  More information may be available when mockall_double is built with the \"nightly\" feature.", msg);
        }
    }
}

fn do_double(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut use_stmt: ItemUse = match parse2(input.clone()) {
        Ok(u) => u,
        Err(e) => return e.to_compile_error()
    };
    mock_itemuse(&mut use_stmt);
    quote!(
        #[cfg(not(test))]
        #input
        #[cfg(test)]
        #use_stmt
    )
}

#[proc_macro_attribute]
pub fn double(attrs: proc_macro::TokenStream, input: proc_macro::TokenStream)
    -> proc_macro::TokenStream
{
    // TODO
    do_double(attrs.into(), input.into()).into()
}

fn mock_itemuse(orig: &mut ItemUse) {
    if let UseTree::Name(un) = &orig.tree {
        compile_error(un.span(),
            "Cannot double types in the current module.  Use a submodule (use foo::Foo) or a rename (use Foo as Bar)");
    } else {
        mock_usetree(&mut orig.tree)
    }
}

fn mock_usetree(mut orig: &mut UseTree) {
    match &mut orig {
        UseTree::Glob(star) => {
            compile_error(star.span(),
                "Cannot double glob imports.  Import by fully qualified name instead.");
        },
        UseTree::Group(ug) => {
            for ut in ug.items.iter_mut() {
                mock_usetree(ut);
            }
        },
        UseTree::Name(un) => {
            *orig = UseTree::Rename(UseRename {
                ident: format_ident!("Mock{}", &un.ident),
                as_token: <Token![as]>::default(),
                rename: un.ident.clone()
            });
        },
        UseTree::Path(up) => {
            mock_usetree(up.tree.as_mut());
        },
        UseTree::Rename(ur) => {
            ur.ident = format_ident!("Mock{}", ur.ident)
        },
    }
}

#[cfg(test)]
mod t {
    use super::*;

mod double {
    use super::*;
    use std::str::FromStr;

    fn cmp(attrs: &str, code: &str, expected: &str) {
        let attrs_ts = TokenStream::from_str(attrs).unwrap();
        let code_ts = TokenStream::from_str(code).unwrap();
        let output = do_double(attrs_ts, code_ts);
        let output = output.to_string();
        // Round-trip expected through proc_macro2 so whitespace will be
        // identically formatted
        let expected = TokenStream::from_str(expected)
            .unwrap()
            .to_string();
        assert_eq!(output, expected);
    }

    #[test]
    #[should_panic(expected = "Cannot double glob")]
    fn glob() {
        let code = r#"use foo::*;"#;
        cmp("", &code, "");
    }

    #[test]
    fn group() {
        let code = r#"
            use foo::bar::{
                Baz,
                Bean
            };
        "#;
        let expected = r#"
            #[cfg(not(test))]
            use foo::bar::{
                Baz,
                Bean
            };
            #[cfg(test)]
            use foo::bar::{
                MockBaz as Baz,
                MockBean as Bean
            };
        "#;
        cmp("", &code, &expected);
    }

    #[test]
    #[should_panic(expected = "Cannot double types in the current module")]
    fn name() {
        let code = r#"use Foo;"#;
        cmp("", &code, "");
    }

    #[test]
    fn path() {
        let code = r#"use foo::bar::Baz;"#;
        let expected = r#"
            #[cfg(not(test))]
            use foo::bar::Baz;
            #[cfg(test)]
            use foo::bar::MockBaz as Baz;
        "#;
        cmp("", &code, &expected);
    }

    #[test]
    fn rename() {
        let code = r#"use Foo as Bar;"#;
        let expected = r#"
            #[cfg(not(test))]
            use Foo as Bar;
            #[cfg(test)]
            use MockFoo as Bar;
        "#;
        cmp("", &code, &expected);
    }

    #[test]
    fn not_use_stmt() {
        let code = r#"struct Foo{}"#;
        cmp("", &code, "compile_error!{\"expected `use`\"}");
    }
}
}

