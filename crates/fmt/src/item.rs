use crate::stmt::get_block;
use crate::ty::get as get_ty;
use crate::util::Cx;
use crate::util::INDENT;
use syntax::ast::{Field, FnTail, Item};

#[must_use]
pub(crate) fn get(cx: &mut Cx, item: Item) -> Option<()> {
  match item {
    Item::StructItem(item) => {
      cx.push("struct ");
      cx.push(item.ident()?.text());
      match item.fields() {
        None => cx.push(";"),
        Some(fields) => {
          cx.push(" {");
          let mut fields = fields.fields();
          if let Some(field) = fields.next() {
            cx.push("\n");
            get_field(cx, field)?;
          }
          for field in fields {
            get_field(cx, field)?;
          }
          cx.push("};\n");
        }
      }
    }
    Item::FnItem(item) => {
      get_ty(cx, item.ret_ty()?)?;
      cx.push(" ");
      cx.push(item.ident()?.text());
      cx.push("(");
      let mut params = item.params();
      if let Some(param) = params.next() {
        get_ty(cx, param.ty()?)?;
        cx.push(" ");
        cx.push(param.ident()?.text());
      }
      for param in params {
        cx.push(", ");
        get_ty(cx, param.ty()?)?;
        cx.push(" ");
        cx.push(param.ident()?.text());
      }
      cx.push(")");
      match item.tail()? {
        FnTail::SemicolonTail(_) => cx.push(";"),
        FnTail::BlockStmt(stmt) => {
          cx.push(" ");
          get_block(cx, stmt)?;
        }
      }
      cx.push("\n");
    }
    Item::TypedefItem(item) => {
      cx.push("typedef ");
      get_ty(cx, item.ty()?)?;
      cx.push(item.ident()?.text());
      cx.push(";\n");
    }
    Item::PragmaItem(item) => {
      // TODO format pragmas more?
      cx.push(item.pragma()?.text());
      cx.push("\n");
    }
  }
  Some(())
}

#[must_use]
fn get_field(cx: &mut Cx, field: Field) -> Option<()> {
  cx.push(INDENT);
  get_ty(cx, field.ty()?)?;
  cx.push(" ");
  cx.push(field.ident()?.text());
  cx.push(";\n");
  Some(())
}
