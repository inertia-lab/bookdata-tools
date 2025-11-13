//! Utilities for cleaning strings.
use std::iter::FromIterator;

use std::borrow::Cow;
use unicode_normalization::*;

/// Normalize Unicode character representations in a string.
pub fn norm_unicode<'a>(s: &'a str) -> Cow<'a, str> {
    if is_nfc_quick(s.chars()) == IsNormalized::Yes {
        s.into()
    } else {
        String::from_iter(s.nfc()).into()
    }
}

#[test]
fn test_nu_empty() {
    let text = "";
    let res = norm_unicode(&text);
    assert_eq!(res.as_ref(), "");
}

#[test]
fn test_nu_basic() {
    let text = "foo";
    let res = norm_unicode(&text);
    assert_eq!(res.as_ref(), "foo");
}

#[test]
fn test_nu_metal() {
    let text = "metäl";
    let res = norm_unicode(&text);
    assert_eq!(res.as_ref(), "met\u{00e4}l");
}
