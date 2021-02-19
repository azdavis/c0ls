//! Topological sorting.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A graph, represented as a map between nodes and their neighbors.
pub type Graph<T> = BTreeMap<T, BTreeSet<T>>;

/// Returns a reverse topological ordering of the graph, or an error if the
/// graph has a cycle.
pub fn get<T>(graph: &Graph<T>) -> Result<Vec<T>, CycleError<T>>
where
  T: Copy + Eq + Ord,
{
  let mut active = BTreeSet::new();
  let mut done = BTreeSet::new();
  let mut ret = Vec::with_capacity(graph.len());
  let mut stack: Vec<_> = graph.keys().map(|&x| (Action::Start, x)).collect();
  while let Some((ac, cur)) = stack.pop() {
    match ac {
      Action::Start => {
        if done.contains(&cur) {
          continue;
        }
        if !active.insert(cur) {
          return Err(CycleError(cur));
        }
        stack.push((Action::Finish, cur));
        if let Some(ns) = graph.get(&cur) {
          stack.extend(ns.iter().map(|&x| (Action::Start, x)));
        }
      }
      Action::Finish => {
        assert!(active.remove(&cur));
        assert!(done.insert(cur));
        ret.push(cur);
      }
    }
  }
  Ok(ret)
}

/// An error when the graph contained a cycle.
#[derive(Debug)]
pub struct CycleError<T>(T);

impl<T> CycleError<T> {
  /// Returns one of the `T` involved in the cycle.
  pub fn witness(self) -> T {
    self.0
  }
}

impl<T> fmt::Display for CycleError<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "graph has a cycle")
  }
}

impl<T> std::error::Error for CycleError<T> where T: std::fmt::Debug {}

enum Action {
  Start,
  Finish,
}

#[cfg(test)]
mod tests {
  use super::{get, CycleError, Graph};
  use maplit::{btreemap, btreeset};

  fn check(graph: Graph<u32>, order: &[u32]) {
    assert_eq!(get(&graph).unwrap(), order);
  }

  fn check_cycle(graph: Graph<u32>) {
    let err = get(&graph).unwrap_err();
    assert!(matches!(err, CycleError(_)));
  }

  #[test]
  fn empty() {
    check(btreemap![], &[]);
  }

  #[test]
  fn one() {
    let graph = btreemap![
      1 => btreeset![],
    ];
    check(graph, &[1]);
  }

  #[test]
  fn separate() {
    let graph = btreemap![
      1 => btreeset![],
      2 => btreeset![],
    ];
    check(graph, &[2, 1]);
  }

  #[test]
  fn simple() {
    let graph = btreemap![
      1 => btreeset![2],
      2 => btreeset![],
    ];
    check(graph, &[2, 1]);
  }

  #[test]
  fn cycle() {
    let graph = btreemap![
      2 => btreeset![1],
      1 => btreeset![2],
    ];
    check_cycle(graph);
  }

  #[test]
  fn bigger() {
    let graph = btreemap![
      1 => btreeset![4],
      2 => btreeset![1, 7],
      3 => btreeset![4, 6, 8],
      4 => btreeset![5],
      5 => btreeset![6, 8],
      6 => btreeset![],
      7 => btreeset![3, 8, 9],
      8 => btreeset![9],
      9 => btreeset![],
    ];
    check(graph, &[9, 8, 6, 5, 4, 3, 7, 1, 2]);
  }

  #[test]
  fn bigger_cycle() {
    let graph = btreemap![
      1 => btreeset![2],
      2 => btreeset![],
      3 => btreeset![6],
      4 => btreeset![5],
      5 => btreeset![3, 2],
      6 => btreeset![1, 4],
    ];
    check_cycle(graph);
  }

  #[test]
  fn hm_cycle() {
    let graph = btreemap![
      1 => btreeset![2],
      2 => btreeset![1],
      3 => btreeset![1],
    ];
    check_cycle(graph);
  }
}
