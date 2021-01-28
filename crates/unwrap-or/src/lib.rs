//! A macro for unwrapping options.

#![deny(missing_doc_code_examples)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![no_std]

/// Given `opt` and `or`, this evaluates to `x` if `opt` is `Some(x)`, and `or`
/// if `opt` is `None`.
///
/// This is just the macro form of the `unwrap_or` method. But because it's a
/// macro, the expression `or` can do things with the control flow of the
/// surrounding expression, like `return` or `break`.
/// ```
/// use unwrap_or::unwrap_or;
///
/// let xs = vec![Some(3), None, Some(5)];
/// let mut sum = 0;
/// for x in xs {
///   sum += unwrap_or!(x, continue);
/// }
/// assert_eq!(sum, 8);
/// ```
#[macro_export]
macro_rules! unwrap_or {
  ($opt:expr, $or:expr) => {
    match $opt {
      ::core::option::Option::Some(x) => x,
      ::core::option::Option::None => $or,
    }
  };
}
