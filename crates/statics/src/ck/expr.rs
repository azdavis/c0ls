use crate::error::{ErrorKind, Thing};
use crate::name::Name;
use crate::ty::{Ty, TyData};
use crate::util::{no_struct, no_void, unify, unify_impl, Cx, ItemDb, VarDb};
use syntax::ast::{BinOpKind, Expr, Syntax as _, UnOpKind};
use syntax::{rowan::TextRange, SyntaxToken};
use unwrap_or::unwrap_or;

fn get_impl(cx: &mut Cx, items: &ItemDb, vars: &VarDb, expr: Expr) -> Ty {
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
          cx.error(ident.text_range(), ErrorKind::Undefined(Thing::Variable));
          Ty::Error
        }
        Some(&data) => {
          if !data.defined {
            cx.error(ident.text_range(), ErrorKind::UninitializedVar);
          }
          data.ty
        }
      },
    },
    Expr::ParenExpr(expr) => get_opt_or(cx, items, vars, expr.expr()),
    Expr::BinOpExpr(expr) => {
      let lhs_ty = get_opt(cx, items, vars, expr.lhs());
      let rhs_ty = get_opt(cx, items, vars, expr.rhs());
      let op = unwrap_or!(expr.op(), return Ty::Error);
      let (params, ret) = bin_op_ty(op.kind);
      if let Some((range, lhs_ty)) = lhs_ty {
        for &param in params {
          if let Some(param) = unify_impl(cx, param, lhs_ty) {
            unify(cx, param, rhs_ty);
            return ret;
          }
        }
        cx.error(range, ErrorKind::MismatchedTypesAny(params, lhs_ty));
      }
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
        UnOpKind::Star => ty.map_or(Ty::Error, |(r, t)| deref(cx, r, t)),
      }
    }
    Expr::TernaryExpr(expr) => {
      let cond_ty = get_opt(cx, items, vars, expr.cond());
      let yes_ty = get_opt_or(cx, items, vars, expr.yes());
      let no_ty = get_opt(cx, items, vars, expr.no());
      unify(cx, Ty::Bool, cond_ty);
      let ret_ty = unify(cx, yes_ty, no_ty);
      let range = expr.syntax().text_range();
      no_void(cx, range, ret_ty);
      no_struct(cx, range, ret_ty);
      ret_ty
    }
    Expr::CallExpr(expr) => {
      let arg_tys: Vec<_> = expr
        .args()
        .map(|arg| get_opt(cx, items, vars, arg.expr()))
        .collect();
      let fn_ident = unwrap_or!(expr.ident(), return Ty::Error);
      let fn_name = fn_ident.text();
      if vars.contains_key(fn_name) {
        cx.error(fn_ident.text_range(), ErrorKind::ShadowedFunction);
      }
      let fn_data = unwrap_or!(items.fns.get(fn_name), {
        cx.error(fn_ident.text_range(), ErrorKind::Undefined(Thing::Function));
        return Ty::Error;
      });
      cx.called.insert(Name::new(fn_name));
      if fn_data.params.len() != arg_tys.len() {
        cx.error(
          expr.syntax().text_range(),
          ErrorKind::MismatchedNumArgs(fn_data.params.len(), arg_tys.len()),
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
      let struct_ty = ptr_ty.map(|(r, t)| (r, deref(cx, r, t)));
      struct_field(cx, items, struct_ty, expr.ident())
    }
    Expr::SubscriptExpr(expr) => {
      let array_ty = get_opt_or(cx, items, vars, expr.array());
      let idx_ty = get_opt(cx, items, vars, expr.idx());
      unify(cx, Ty::Int, idx_ty);
      match *cx.tys.get(array_ty) {
        TyData::Array(ty) => ty,
        _ => {
          cx.error(
            expr.syntax().text_range(),
            ErrorKind::SubscriptNonArray(array_ty),
          );
          Ty::Error
        }
      }
    }
    Expr::AllocExpr(expr) => {
      let inner_ty = super::ty::get_sized_opt_or(cx, items, expr.ty());
      cx.tys.mk(TyData::Ptr(inner_ty))
    }
    Expr::AllocArrayExpr(expr) => {
      let inner_ty = super::ty::get_sized_opt_or(cx, items, expr.ty());
      let len_ty = get_opt(cx, items, vars, expr.expr());
      unify(cx, Ty::Int, len_ty);
      cx.tys.mk(TyData::Array(inner_ty))
    }
  }
}

/// also makes sure this isn't a struct type. we call this 'get' and not
/// 'get_opt' or 'get_no_struct' because
/// 1. it's the only function exported
/// 2. i don't feel like renaming everything
pub(crate) fn get(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &VarDb,
  expr: Option<Expr>,
) -> Option<(TextRange, Ty)> {
  let ret = get_opt(cx, items, vars, expr);
  if let Some((range, ty)) = ret {
    no_struct(cx, range, ty);
  }
  ret
}

fn get_opt(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &VarDb,
  expr: Option<Expr>,
) -> Option<(TextRange, Ty)> {
  expr.map(|expr| (expr.syntax().text_range(), get_impl(cx, items, vars, expr)))
}

/// does NOT report an error if it is None, so only call this with optional
/// things from the AST (that have a corresponding parse error).
fn get_opt_or(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &VarDb,
  expr: Option<Expr>,
) -> Ty {
  expr.map_or(Ty::Error, |expr| get_impl(cx, items, vars, expr))
}

fn deref(cx: &mut Cx, range: TextRange, ty: Ty) -> Ty {
  match *cx.tys.get(ty) {
    TyData::Ptr(inner) => {
      if inner == Ty::Top {
        cx.error(range, ErrorKind::DerefNull);
        Ty::Error
      } else {
        inner
      }
    }
    _ => {
      cx.error(range, ErrorKind::DerefNonPtr(ty));
      Ty::Error
    }
  }
}

fn struct_field(
  cx: &mut Cx,
  items: &ItemDb,
  ty: Option<(TextRange, Ty)>,
  field: Option<SyntaxToken>,
) -> Ty {
  let (range, ty) = unwrap_or!(ty, return Ty::Error);
  let field = unwrap_or!(field, return Ty::Error);
  let struct_name = match cx.tys.get(ty) {
    TyData::Struct(n) => n,
    _ => {
      cx.error(range, ErrorKind::FieldGetNonStruct(ty));
      return Ty::Error;
    }
  };
  let struct_data = unwrap_or!(items.structs.get(struct_name), {
    cx.error(range, ErrorKind::Undefined(Thing::Struct));
    return Ty::Error;
  });
  unwrap_or!(struct_data.get(field.text()).copied(), {
    cx.error(range, ErrorKind::Undefined(Thing::Field));
    Ty::Error
  })
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
