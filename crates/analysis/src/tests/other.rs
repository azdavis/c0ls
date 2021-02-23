use super::support::uri;
use crate::db::Db;
use crate::types::Update;
use rustc_hash::FxHashMap;

#[test]
fn change() {
  let header = r#"
    int foo();
  "#;
  let source = r#"
    #use "h.h0"

    int main() {
      return foo();
    }
  "#;
  let mut db = Db::new(vec![
    (uri("/h.h0"), header.to_owned()),
    (uri("/c.c0"), source.to_owned()),
  ]);
  assert!(db.all_diagnostics().iter().all(|(_, ds)| ds.is_empty()));
  db.update_files(vec![
    Update::Create(uri("/new.h0"), header.to_owned()),
    Update::Delete(uri("/h.h0")),
  ]);
  let ds: FxHashMap<_, _> = db.all_diagnostics().into_iter().collect();
  assert_eq!(ds.len(), 2);
  assert!(ds[&uri("/new.h0")].is_empty());
  let c_ds = ds.get(&uri("/c.c0")).unwrap();
  assert_eq!(c_ds.len(), 2);
  assert_eq!(c_ds[0].message, "no such path");
  assert_eq!(c_ds[1].message, "undefined function `foo`");
}
