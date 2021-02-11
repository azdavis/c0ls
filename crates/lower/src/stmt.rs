use crate::expr::get as get_expr;
use crate::ptr::AstPtr;
use crate::simp::get as get_simp;
use crate::util::Cx;
use syntax::ast::{BlockStmt, Stmt};

pub(crate) fn get_block(cx: &mut Cx, stmt: BlockStmt) -> hir::StmtId {
  get(cx, Some(Stmt::BlockStmt(stmt)))
}

fn get(cx: &mut Cx, stmt: Option<Stmt>) -> hir::StmtId {
  let (ptr, data) = stmt.map_or((None, hir::Stmt::None), |stmt| {
    (Some(AstPtr::new(&stmt)), get_impl(cx, stmt))
  });
  let ret = cx.arenas.stmt.alloc(data);
  if let Some(ptr) = ptr {
    cx.ptrs.stmt.insert(ptr, ret);
    cx.ptrs.stmt_back.insert(ret, ptr);
  }
  ret
}

fn get_impl(cx: &mut Cx, stmt: Stmt) -> hir::Stmt {
  match stmt {
    Stmt::SimpStmt(stmt) => stmt
      .simp()
      .and_then(|simp| get_simp(cx, simp))
      .map_or(hir::Stmt::None, hir::Stmt::Simp),
    Stmt::IfStmt(stmt) => {
      let cond = get_expr(cx, stmt.cond());
      let yes = get(cx, stmt.yes());
      let no = stmt.no().map(|else_branch| get(cx, else_branch.stmt()));
      hir::Stmt::If(cond, yes, no)
    }
    Stmt::WhileStmt(stmt) => {
      let cond = get_expr(cx, stmt.cond());
      let body = get(cx, stmt.body());
      hir::Stmt::While(cond, body)
    }
    Stmt::ForStmt(stmt) => {
      let init = stmt.init().and_then(|x| get_simp(cx, x.simp()?));
      let cond = get_expr(cx, stmt.cond());
      let step = stmt.step().and_then(|x| get_simp(cx, x.simp()?));
      let body = get(cx, stmt.body());
      hir::Stmt::For(init, cond, step, body)
    }
    Stmt::ReturnStmt(stmt) => {
      // this one is a little weird, but it's because the expression is actually
      // allowed to be optional.
      let expr = stmt.expr().map(|x| get_expr(cx, Some(x)));
      hir::Stmt::Return(expr)
    }
    Stmt::BlockStmt(stmt) => {
      let stmts: Vec<_> =
        stmt.stmts().map(|stmt| get(cx, Some(stmt))).collect();
      hir::Stmt::Block(stmts)
    }
    Stmt::AssertStmt(stmt) => hir::Stmt::Assert(get_expr(cx, stmt.expr())),
    Stmt::ErrorStmt(stmt) => hir::Stmt::Error(get_expr(cx, stmt.expr())),
    Stmt::BreakStmt(_) => hir::Stmt::Break,
    Stmt::ContinueStmt(_) => hir::Stmt::Continue,
  }
}
