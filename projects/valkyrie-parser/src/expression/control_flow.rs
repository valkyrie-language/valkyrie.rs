use super::*;

// r#"^(?x)(
//       [.]{2}[=<]
//     | [‥…]
//     # | [.]{1,3}
//     # | [⟦⟧⟬⟭{}\[\]()]
//     | [$§¶^]
//     | @[*!?@]?
//     # contains
//     | (\b(not)\b\s+)?\b(in)\b
//     | [∈∊∉]
//     # is
//     | \b(is)\b(\s+\b(not)\b)?
//     | [⊑⋢]
//     # start with <, >
//     | === | ≡
//     | !== | =!= | ≢
//     | == | ≖
//     | != | ≠
//     | <=>
//     # start with <, >
//     | >{1,3} | >= | /> | ≥ | ⩾ | ≫ | ⋙
//     | <{1,3} | <= | </ | ≤ | ⩽ | ≪ | ⋘ |  <: | <!
//     # start with :
//     | :>
//     # start with -
//     | -> | ⟶ | -{1,2}=?
//     # start with ~
//     | ~> | ~
//     # start with +
//     | [+]> | [+]{1,2}=?
//     # start with *
//     | [*]=?
//     # start with / or % or ÷
//     | /=?
//     | ÷=?
//     | %=?
//     # start with &
//     | &> | &{1,2} | ≻
//     | [|]> | [|]{1,2} | ⊁
//     | ⊻=? | ⊼=? | ⊽=? | [⩕⩖]
//     # start with =
//     | => | ⇒
//     | =
//     # unicode
//     | [⨳∀∁∂∃∄¬±√∛∜⊹⋗]
//     | [↻↺⇆↹⇄⇋⇌⇅]
// )"#

impl crate::ControlFlowNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ControlNode> {
        let expression = match &self.main_expression {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        Ok(ControlNode {
            kind: self.kw_control.build(ctx),
            label: self.jump_label.build(ctx),
            expression,
            span: self.get_range32(),
        })
    }
}

impl crate::KwControlNode {
    pub(crate) fn build(&self, _: &mut ProgramState) -> ControlKind {
        match self.text.as_str() {
            "break" => ControlKind::Break,
            "continue" => ControlKind::Continue,
            "fallthrough" => ControlKind::Fallthrough,
            "fallthrough!" => ControlKind::FallthroughUnchecked,
            "goto" => ControlKind::Goto,
            "raise" => ControlKind::Raise,
            "return" => ControlKind::Return,
            "resume" => ControlKind::Resume,
            s if s.starts_with("yield") => {
                if s.ends_with("break") {
                    ControlKind::YieldBreak
                }
                else if s.ends_with("from") {
                    ControlKind::YieldFrom
                }
                else if s.ends_with("wait") {
                    ControlKind::YieldSend
                }
                else {
                    ControlKind::YieldReturn
                }
            }
            _ => unreachable!("Invalid control flow keyword `{}`", self.text),
        }
    }
}
