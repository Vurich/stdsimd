//! Implementation of the `#[simd_test]` macro
//!
//! This macro expands to a `#[test]` function which tests the local machine
//! for the appropriate cfg before calling the inner test function.

#![feature(proc_macro)]

extern crate proc_macro2;
extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro2::{Term, TokenNode, TokenStream, TokenTree};
use proc_macro2::Literal;

fn string(s: &str) -> TokenTree {
    TokenNode::Literal(Literal::string(s)).into()
}

#[proc_macro_attribute]
pub fn simd_test(
    attr: proc_macro::TokenStream, item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let tokens = TokenStream::from(attr).into_iter().collect::<Vec<_>>();
    if tokens.len() != 2 {
        panic!("expected #[simd_test = \"feature\"]");
    }
    match tokens[0].kind {
        TokenNode::Op('=', _) => {}
        _ => panic!("expected #[simd_test = \"feature\"]"),
    }
    let target_features = match tokens[1].kind {
        TokenNode::Literal(ref l) => l.to_string(),
        _ => panic!("expected #[simd_test = \"feature\"]"),
    };
    let target_features: Vec<String> = target_features
        .replace('"', "")
        .replace('+', "")
        .split(',')
        .map(|v| String::from(v))
        .collect();

    let enable_feature = match tokens[1].kind {
        TokenNode::Literal(ref l) => l.to_string(),
        _ => panic!("expected #[simd_test = \"feature\"]"),
    };
    let enable_feature = enable_feature
        .trim_left_matches('"')
        .trim_right_matches('"');
    let enable_feature =
        string(&(format!("+{}", enable_feature).replace(',', ",+")));
    let item = TokenStream::from(item);
    let name = find_name(item.clone());

    let name: TokenStream = name.as_str().parse().unwrap();

    let mut cfg_target_features = quote::Tokens::new();
    use quote::ToTokens;
    for feature in target_features {
        let q = quote_spanned! {
            proc_macro2::Span::call_site() =>
            cfg_feature_enabled!(#feature) &&
        };
        q.to_tokens(&mut cfg_target_features);
    }
    let q = quote!{ true };
    q.to_tokens(&mut cfg_target_features);

    let ret: TokenStream = quote_spanned! {
        proc_macro2::Span::call_site() =>
        #[allow(non_snake_case)]
        #[test]
        fn #name() {
            if #cfg_target_features {
                return unsafe { #name() };
            } else {
                ::stdsimd_test::assert_skip_test_ok(stringify!(#name));
            }

            #[target_feature = #enable_feature]
            #item
        }
    }.into();
    ret.into()
}

fn find_name(item: TokenStream) -> Term {
    let mut tokens = item.into_iter();
    while let Some(tok) = tokens.next() {
        if let TokenNode::Term(word) = tok.kind {
            if word.as_str() == "fn" {
                break;
            }
        }
    }

    match tokens.next().map(|t| t.kind) {
        Some(TokenNode::Term(word)) => word,
        _ => panic!("failed to find function name"),
    }
}
