//! End-to-end tests for C0 analysis.

#![cfg(test)]

mod support;

use support::check;

#[test]
fn decl_in_for() {
  check(include_str!("data/decl_in_for.c0"))
}

#[test]
fn deref_null() {
  check(include_str!("data/deref_null.c0"))
}

#[test]
fn div() {
  check(include_str!("data/div.c0"))
}

#[test]
fn duplicate_var() {
  check(include_str!("data/duplicate_var.c0"))
}

#[test]
fn hover_ty() {
  check(include_str!("data/hover_ty.c0"))
}

#[test]
fn mismatched_types_any() {
  check(include_str!("data/mismatched_types_any.c0"))
}

#[test]
fn undefined() {
  check(include_str!("data/undefined.c0"))
}
