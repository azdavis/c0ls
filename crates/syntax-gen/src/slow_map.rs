//! TODO this is a HACK until hopefully Token implements Hash

use std::borrow::Borrow;

#[derive(Debug)]
pub(crate) struct SlowMap<K, V> {
  store: Vec<(K, V)>,
}

impl<K, V> SlowMap<K, V> {
  pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
    self.store.iter().map(|&(_, ref v)| v)
  }
}

impl<K, V> SlowMap<K, V>
where
  K: Eq,
{
  pub(crate) fn insert(&mut self, key: K, val: V) -> Option<V> {
    let ret = self
      .store
      .iter()
      .position(|(k, _)| key == *k)
      .map(|pos| self.store.remove(pos).1);
    self.store.push((key, val));
    ret
  }

  pub(crate) fn get<Q>(&self, key: &Q) -> Option<&V>
  where
    Q: Eq + ?Sized,
    K: Borrow<Q>,
  {
    self
      .store
      .iter()
      .find_map(|(k, v)| if key == k.borrow() { Some(v) } else { None })
  }
}

impl<K, V> Default for SlowMap<K, V> {
  fn default() -> Self {
    Self { store: Vec::new() }
  }
}

impl<K, V> IntoIterator for SlowMap<K, V> {
  type Item = (K, V);
  type IntoIter = IntoIter<K, V>;
  fn into_iter(self) -> Self::IntoIter {
    IntoIter(self.store.into_iter())
  }
}

pub struct IntoIter<K, V>(std::vec::IntoIter<(K, V)>);

impl<K, V> Iterator for IntoIter<K, V> {
  type Item = (K, V);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next()
  }
}
