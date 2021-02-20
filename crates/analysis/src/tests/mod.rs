mod support;

use support::check;

#[test]
fn arrow() {
  check(include_str!("data/arrow.c0"))
}

#[test]
fn bad_op() {
  check(include_str!("data/bad_op.c0"))
}

#[test]
fn cond() {
  check(include_str!("data/cond.c0"))
}

#[test]
fn decl_in_for() {
  check(include_str!("data/decl_in_for.c0"))
}

#[test]
fn def() {
  check(include_str!("data/def.c0"))
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
fn lv_bad() {
  check(include_str!("data/lv_bad.c0"))
}

#[test]
fn mismatched_types_any() {
  check(include_str!("data/mismatched_types_any.c0"))
}

#[test]
fn not_in_loop() {
  check(include_str!("data/not_in_loop.c0"))
}

#[test]
fn recur() {
  check(include_str!("data/recur.c0"))
}

#[test]
fn return_bad() {
  check(include_str!("data/return_bad.c0"))
}

#[test]
fn ty_bad() {
  check(include_str!("data/ty_bad.c0"))
}

#[test]
fn undefined() {
  check(include_str!("data/undefined.c0"))
}

#[test]
fn use_lib() {
  check(include_str!("data/use_lib.c0"))
}
