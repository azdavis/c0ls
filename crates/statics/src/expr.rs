use crate::ty;
use crate::util::ty::{Ty, TyData, TyDb};
use crate::util::{unify, ItemDb, NameToTy};
use syntax::ast::{BinOp, Expr, UnOp};
use syntax::SyntaxToken;

pub fn get(
  items: &ItemDb,
  vars: &NameToTy,
  tys: &mut TyDb,
  expr: Expr,
) -> Option<Ty> {
  match expr {
    Expr::DecExpr(_) | Expr::HexExpr(_) => Some(Ty::Int),
    Expr::StringExpr(_) => Some(Ty::String),
    Expr::CharExpr(_) => Some(Ty::Char),
    Expr::TrueExpr(_) | Expr::FalseExpr(_) => Some(Ty::Bool),
    Expr::NullExpr(_) => Some(Ty::PtrTop),
    Expr::IdentExpr(expr) => vars.get(expr.ident()?.text()).copied(),
    Expr::ParenExpr(expr) => get(items, vars, tys, expr.expr()?),
    Expr::BinOpExpr(expr) => {
      let (params, ret) = bin_op_ty(expr.op()?);
      let lhs_ty = get(items, vars, tys, expr.lhs()?)?;
      let rhs_ty = get(items, vars, tys, expr.rhs()?)?;
      for &param in params {
        if unify(tys, param, lhs_ty).is_some() {
          unify(tys, param, rhs_ty)?;
          return Some(ret);
        }
      }
      None
    }
    Expr::UnOpExpr(expr) => {
      let ty = get(items, vars, tys, expr.expr()?)?;
      match expr.op()? {
        UnOp::Bang(_) => {
          unify(tys, Ty::Bool, ty)?;
          Some(Ty::Bool)
        }
        UnOp::Tilde(_) | UnOp::Minus(_) => {
          unify(tys, Ty::Int, ty)?;
          Some(Ty::Int)
        }
        UnOp::Star(_) => match *tys.get(ty) {
          TyData::Ptr(inner) => {
            if inner == Ty::Top {
              None
            } else {
              Some(inner)
            }
          }
          _ => None,
        },
      }
    }
    Expr::TernaryExpr(expr) => {
      let cond_ty = get(items, vars, tys, expr.cond()?)?;
      let yes_ty = get(items, vars, tys, expr.yes()?)?;
      let no_ty = get(items, vars, tys, expr.no()?)?;
      unify(tys, Ty::Bool, cond_ty)?;
      let ret_ty = unify(tys, yes_ty, no_ty)?;
      no_void(no_struct(tys, ret_ty)?)
    }
    Expr::CallExpr(expr) => {
      let fn_name = expr.ident()?;
      let fn_name = fn_name.text();
      if vars.contains_key(fn_name) {
        // variables shadow function names, and variables can never have
        // function type
        return None;
      }
      let fn_data = items.fns.get(fn_name)?;
      if fn_data.params.len() != expr.args().count() {
        // wrong number of arguments
        return None;
      }
      for (&(_, param_ty), arg) in fn_data.params.iter().zip(expr.args()) {
        let arg_expr = arg.expr()?;
        let arg_ty = get(items, vars, tys, arg_expr)?;
        unify(tys, param_ty, arg_ty)?;
      }
      Some(fn_data.ret_ty)
    }
    Expr::DotExpr(expr) => {
      let struct_ty = get(items, vars, tys, expr.expr()?)?;
      struct_field(items, tys, struct_ty, expr.ident()?)
    }
    Expr::ArrowExpr(expr) => {
      let ptr_ty = get(items, vars, tys, expr.expr()?)?;
      let struct_ty = match tys.get(ptr_ty) {
        TyData::Ptr(t) => *t,
        _ => return None,
      };
      struct_field(items, tys, struct_ty, expr.ident()?)
    }
    Expr::SubscriptExpr(expr) => {
      let array_ty = get(items, vars, tys, expr.array()?)?;
      let idx_ty = get(items, vars, tys, expr.idx()?)?;
      unify(tys, Ty::Int, idx_ty)?;
      match tys.get(array_ty) {
        TyData::Array(ty) => Some(*ty),
        _ => None,
      }
    }
    Expr::AllocExpr(expr) => {
      let inner_ty = ty::get(&items.type_defs, tys, expr.ty()?)?;
      Some(tys.mk(TyData::Ptr(inner_ty)))
    }
    Expr::AllocArrayExpr(expr) => {
      let len_ty = get(items, vars, tys, expr.expr()?)?;
      unify(tys, Ty::Int, len_ty)?;
      let inner_ty = ty::get(&items.type_defs, tys, expr.ty()?)?;
      Some(tys.mk(TyData::Array(inner_ty)))
    }
  }
}

fn struct_field(
  items: &ItemDb,
  tys: &TyDb,
  ty: Ty,
  field: SyntaxToken,
) -> Option<Ty> {
  let struct_name = match tys.get(ty) {
    TyData::Struct(n) => n,
    _ => return None,
  };
  items.structs.get(struct_name)?.get(field.text()).copied()
}

fn no_void(ty: Ty) -> Option<Ty> {
  if ty == Ty::Void {
    None
  } else {
    Some(ty)
  }
}

fn no_struct(tys: &TyDb, ty: Ty) -> Option<Ty> {
  match tys.get(ty) {
    TyData::Struct(_) => None,
    _ => Some(ty),
  }
}

fn bin_op_ty(op: BinOp) -> (&'static [Ty], Ty) {
  let params: &'static [Ty];
  let ret;
  match op {
    BinOp::Plus(_)
    | BinOp::Minus(_)
    | BinOp::Star(_)
    | BinOp::Slash(_)
    | BinOp::Percent(_)
    | BinOp::LtLt(_)
    | BinOp::GtGt(_)
    | BinOp::And(_)
    | BinOp::Carat(_)
    | BinOp::Bar(_) => {
      params = &[Ty::Int];
      ret = Ty::Int;
    }
    BinOp::EqEq(_) | BinOp::BangEq(_) => {
      params = &[Ty::Int, Ty::Bool, Ty::Char, Ty::PtrTop, Ty::ArrayTop];
      ret = Ty::Bool;
    }
    BinOp::Lt(_) | BinOp::LtEq(_) | BinOp::Gt(_) | BinOp::GtEq(_) => {
      params = &[Ty::Int, Ty::Char];
      ret = Ty::Bool;
    }
    BinOp::AndAnd(_) | BinOp::BarBar(_) => {
      params = &[Ty::Bool];
      ret = Ty::Bool;
    }
  }
  (params, ret)
}
