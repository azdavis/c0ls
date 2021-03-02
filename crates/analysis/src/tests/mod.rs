mod other;
mod support;

use support::{check, check_many};

#[test]
fn transitive() {
  check_many(&[
    ("/1.h0", include_str!("data/transitive/1.h0")),
    ("/2.h0", include_str!("data/transitive/2.h0")),
    ("/3.c0", include_str!("data/transitive/3.c0")),
  ])
}

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
fn hover_ambiguous() {
  // FIXME hovering `a * b;` should work but does not
  check(include_str!("data/hover_ambiguous.c0"))
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
fn mismatched_num() {
  check(include_str!("data/mismatched_num.c0"))
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
fn unreachable() {
  check(include_str!("data/unreachable.c0"))
}

#[test]
fn use_lib() {
  check(include_str!("data/use_lib.c0"))
}
