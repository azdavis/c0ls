use crate::expr::get as get_expr;
use crate::simp::get as get_simp;
use crate::util::{Cx, INDENT};
use syntax::ast::{BlockStmt, Stmt};

#[must_use]
pub(crate) fn get_block(cx: &mut Cx, stmt: BlockStmt) -> Option<()> {
  get_many(cx, 0, stmt.stmts())
}

#[must_use]
fn get_many<I>(cx: &mut Cx, level: u8, stmts: I) -> Option<()>
where
  I: Iterator<Item = Stmt>,
{
  cx.push("{\n");
  for s in stmts {
    for _ in 0..(level + 1) {
      cx.push(INDENT);
    }
    get_one(cx, level + 1, s)?;
    cx.push("\n");
  }
  for _ in 0..level {
    cx.push(INDENT);
  }
  cx.push("}");
  Some(())
}

#[must_use]
fn get_always_block(cx: &mut Cx, level: u8, stmt: Stmt) -> Option<()> {
  match stmt {
    Stmt::BlockStmt(stmt) => get_many(cx, level, stmt.stmts()),
    _ => get_many(cx, level, std::iter::once(stmt)),
  }
}

#[must_use]
fn get_one(cx: &mut Cx, level: u8, stmt: Stmt) -> Option<()> {
  match stmt {
    Stmt::SimpStmt(stmt) => {
      get_simp(cx, stmt.simp()?)?;
      cx.push(";");
    }
    Stmt::IfStmt(stmt) => {
      cx.push("if (");
      get_expr(cx, stmt.cond()?)?;
      cx.push(") ");
      get_always_block(cx, level, stmt.yes()?)?;
      if let Some(no) = stmt.no() {
        cx.push(" else ");
        match no.stmt()? {
          Stmt::BlockStmt(stmt) => {
            let mut stmts: Vec<_> = stmt.stmts().collect();
            match stmts.as_slice() {
              [Stmt::IfStmt(_)] => get_one(cx, level, stmts.pop().unwrap())?,
              _ => get_many(cx, level, stmts.into_iter())?,
            }
          }
          stmt @ Stmt::IfStmt(_) => get_one(cx, level, stmt)?,
          stmt => get_many(cx, level, std::iter::once(stmt))?,
        }
      }
    }
    Stmt::WhileStmt(stmt) => {
      cx.push("while (");
      get_expr(cx, stmt.cond()?)?;
      cx.push(") ");
      get_always_block(cx, level, stmt.body()?)?;
    }
    Stmt::ForStmt(stmt) => {
      cx.push("for (");
      match stmt.init() {
        None => cx.push(" "),
        Some(init) => get_simp(cx, init.simp()?)?,
      }
      cx.push("; ");
      get_expr(cx, stmt.cond()?)?;
      cx.push("; ");
      if let Some(step) = stmt.step() {
        get_simp(cx, step.simp()?)?;
      }
      cx.push(") ");
      get_always_block(cx, level, stmt.body()?)?;
    }
    Stmt::ReturnStmt(stmt) => match stmt.expr() {
      None => cx.push("return;"),
      Some(e) => {
        cx.push("return ");
        get_expr(cx, e)?;
        cx.push(";");
      }
    },
    Stmt::BlockStmt(stmt) => get_many(cx, level, stmt.stmts())?,
    Stmt::AssertStmt(stmt) => {
      cx.push("assert(");
      get_expr(cx, stmt.expr()?)?;
      cx.push(");");
    }
    Stmt::ErrorStmt(stmt) => {
      cx.push("error(");
      get_expr(cx, stmt.expr()?)?;
      cx.push(");");
    }
    Stmt::BreakStmt(_) => cx.push("break;"),
    Stmt::ContinueStmt(_) => cx.push("continue;"),
  }
  Some(())
}
