use super::support::uri;
use crate::db::Db;
use crate::types::Update;

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
}
