use crate::db::SyntaxData;
use syntax::rowan::TokenAtOffset;
use syntax::{SyntaxKind, SyntaxToken};
use text_pos::Position;

pub(crate) fn get_token(
  syntax_data: &SyntaxData,
  pos: Position,
) -> Option<SyntaxToken> {
  let idx = syntax_data.positions.text_size(pos);
  let ret = match syntax_data.ast_root.as_ref().token_at_offset(idx) {
    TokenAtOffset::None => return None,
    TokenAtOffset::Single(t) => t,
    TokenAtOffset::Between(t1, t2) => {
      // right biased when eq
      if priority(t1.kind()) > priority(t2.kind()) {
        t1
      } else {
        t2
      }
    }
  };
  Some(ret)
}

// heuristic for how much we should care about some token
fn priority(kind: SyntaxKind) -> u8 {
  match kind {
    SyntaxKind::Ident => 4,
    SyntaxKind::DecLit
    | SyntaxKind::HexLit
    | SyntaxKind::StringLit
    | SyntaxKind::CharLit => 3,
    SyntaxKind::IntKw
    | SyntaxKind::BoolKw
    | SyntaxKind::StringKw
    | SyntaxKind::CharKw
    | SyntaxKind::VoidKw => 2,
    SyntaxKind::Whitespace
    | SyntaxKind::LineComment
    | SyntaxKind::BlockComment
    | SyntaxKind::Invalid => 0,
    _ => 1,
  }
}
