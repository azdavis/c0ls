use crate::ty;
use crate::util::error::{ErrorKind, Thing};
use crate::util::ty::{Ty, TyData};
use crate::util::{unify, unify_impl, Cx, ItemDb, NameToTy};
use syntax::ast::{BinOpKind, Expr, Syntax as _, UnOpKind};
use syntax::SyntaxToken;

pub(crate) fn get(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &NameToTy,
  expr: Expr,
) -> Ty {
  match expr {
    Expr::DecExpr(_) | Expr::HexExpr(_) => Ty::Int,
    Expr::StringExpr(_) => Ty::String,
    Expr::CharExpr(_) => Ty::Char,
    Expr::TrueExpr(_) | Expr::FalseExpr(_) => Ty::Bool,
    Expr::NullExpr(_) => Ty::PtrTop,
    Expr::IdentExpr(expr) => match expr.ident() {
      None => Ty::Error,
      Some(ident) => match vars.get(ident.text()) {
        None => {
          cx.errors
            .push(ident.text_range(), ErrorKind::Undefined(Thing::Variable));
          Ty::Error
        }
        Some(&ty) => ty,
      },
    },
    Expr::ParenExpr(expr) => get_opt(cx, items, vars, expr.expr()),
    Expr::BinOpExpr(expr) => {
      let lhs_ty = get_opt(cx, items, vars, expr.lhs());
      let rhs_ty = get_opt(cx, items, vars, expr.rhs());
      let op = unwrap_or!(expr.op(), return Ty::Error);
      let (params, ret) = bin_op_ty(op.kind);
      for &param in params {
        if unify_impl(cx, param, lhs_ty).is_some() {
          unify(cx, param, rhs_ty);
          return ret;
        }
      }
      // TODO push 'mismatched types: expected any of ..., found ...'
      ret
    }
    Expr::UnOpExpr(expr) => {
      let ty = get_opt(cx, items, vars, expr.expr());
      let op = unwrap_or!(expr.op(), return Ty::Error);
      match op.kind {
        UnOpKind::Bang => {
          unify(cx, Ty::Bool, ty);
          Ty::Bool
        }
        UnOpKind::Tilde | UnOpKind::Minus => {
          unify(cx, Ty::Int, ty);
          Ty::Int
        }
        UnOpKind::Star => match *cx.tys.get(ty) {
          TyData::Ptr(inner) => {
            if inner == Ty::Top {
              cx.errors
                .push(expr.syntax().text_range(), ErrorKind::DerefNull);
              Ty::Error
            } else {
              inner
            }
          }
          _ => {
            cx.errors
              .push(expr.syntax().text_range(), ErrorKind::DerefNonPtr(ty));
            Ty::Error
          }
        },
      }
    }
    Expr::TernaryExpr(expr) => {
      let cond_ty = get_opt(cx, items, vars, expr.cond());
      let yes_ty = get_opt(cx, items, vars, expr.yes());
      let no_ty = get_opt(cx, items, vars, expr.no());
      unify(cx, Ty::Bool, cond_ty);
      let ret_ty = unify(cx, yes_ty, no_ty);
      no_void(ret_ty);
      no_struct(cx, ret_ty);
      ret_ty
    }
    Expr::CallExpr(expr) => {
      let fn_ident = unwrap_or!(expr.ident(), return Ty::Error);
      let fn_name = fn_ident.text();
      if vars.contains_key(fn_name) {
        todo!("variables shadow function names and variables are not functions")
      }
      let arg_tys: Vec<_> = expr
        .args()
        .map(|arg| get_opt(cx, items, vars, arg.expr()))
        .collect();
      let fn_data = unwrap_or!(items.fns.get(fn_name), {
        cx.errors
          .push(fn_ident.text_range(), ErrorKind::Undefined(Thing::Function));
        return Ty::Error;
      });
      if fn_data.params.len() != arg_tys.len() {
        cx.errors.push(
          expr.syntax().text_range(),
          ErrorKind::WrongNumArgs(fn_data.params.len(), arg_tys.len()),
        );
      }
      for (&(_, param_ty), arg_ty) in fn_data.params.iter().zip(arg_tys) {
        unify(cx, param_ty, arg_ty);
      }
      fn_data.ret_ty
    }
    Expr::DotExpr(expr) => {
      let struct_ty = get_opt(cx, items, vars, expr.expr());
      struct_field(cx, items, struct_ty, expr.ident())
    }
    Expr::ArrowExpr(expr) => {
      let ptr_ty = get_opt(cx, items, vars, expr.expr());
      let struct_ty = match *cx.tys.get(ptr_ty) {
        TyData::Ptr(ty) => ty,
        _ => {
          cx.errors
            .push(expr.syntax().text_range(), ErrorKind::DerefNonPtr(ptr_ty));
          Ty::Error
        }
      };
      struct_field(cx, items, struct_ty, expr.ident())
    }
    Expr::SubscriptExpr(expr) => {
      let array_ty = get_opt(cx, items, vars, expr.array());
      let idx_ty = get_opt(cx, items, vars, expr.idx());
      unify(cx, Ty::Int, idx_ty);
      match *cx.tys.get(array_ty) {
        TyData::Array(ty) => ty,
        _ => {
          cx.errors.push(
            expr.syntax().text_range(),
            ErrorKind::SubscriptNonArray(array_ty),
          );
          Ty::Error
        }
      }
    }
    Expr::AllocExpr(expr) => {
      let inner_ty = ty::get_opt(cx, &items.type_defs, expr.ty());
      cx.tys.mk(TyData::Ptr(inner_ty))
    }
    Expr::AllocArrayExpr(expr) => {
      let inner_ty = ty::get_opt(cx, &items.type_defs, expr.ty());
      let len_ty = get_opt(cx, items, vars, expr.expr());
      unify(cx, Ty::Int, len_ty);
      cx.tys.mk(TyData::Array(inner_ty))
    }
  }
}

/// does NOT report an error if it is None, so only call this with optional
/// things from the AST (that have a corresponding parse error).
pub(crate) fn get_opt(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &NameToTy,
  expr: Option<Expr>,
) -> Ty {
  expr.map_or(Ty::Error, |expr| get(cx, items, vars, expr))
}

fn struct_field(
  cx: &mut Cx,
  items: &ItemDb,
  ty: Ty,
  field: Option<SyntaxToken>,
) -> Ty {
  let field = unwrap_or!(field, return Ty::Error);
  let struct_name = match cx.tys.get(ty) {
    TyData::Struct(n) => n,
    _ => todo!("field get on non-struct type"),
  };
  let struct_data = unwrap_or!(items.structs.get(struct_name), {
    todo!("no such struct defined")
  });
  unwrap_or!(struct_data.get(field.text()).copied(), {
    todo!("no such field on struct")
  })
}

fn no_void(ty: Ty) {
  if ty == Ty::Void {
    todo!("expression cannot have void type")
  }
}

fn no_struct(cx: &mut Cx, ty: Ty) {
  if let TyData::Struct(_) = cx.tys.get(ty) {
    todo!("expression cannot have struct type")
  }
}

fn bin_op_ty(op: BinOpKind) -> (&'static [Ty], Ty) {
  let params: &'static [Ty];
  let ret;
  match op {
    BinOpKind::Plus
    | BinOpKind::Minus
    | BinOpKind::Star
    | BinOpKind::Slash
    | BinOpKind::Percent
    | BinOpKind::LtLt
    | BinOpKind::GtGt
    | BinOpKind::And
    | BinOpKind::Carat
    | BinOpKind::Bar => {
      params = &[Ty::Int];
      ret = Ty::Int;
    }
    BinOpKind::EqEq | BinOpKind::BangEq => {
      params = &[Ty::Int, Ty::Bool, Ty::Char, Ty::PtrTop, Ty::ArrayTop];
      ret = Ty::Bool;
    }
    BinOpKind::Lt | BinOpKind::LtEq | BinOpKind::Gt | BinOpKind::GtEq => {
      params = &[Ty::Int, Ty::Char];
      ret = Ty::Bool;
    }
    BinOpKind::AndAnd | BinOpKind::BarBar => {
      params = &[Ty::Bool];
      ret = Ty::Bool;
    }
  }
  (params, ret)
}
