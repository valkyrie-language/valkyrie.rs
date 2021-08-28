use super::*;

pub(super) fn parse_cst(input: &str, rule: Rule) -> OutputResult<Rule> {
    state(input, |state| match rule {
        Rule::BIND_L => parse_bind_l(state),
        Rule::BIND_R => parse_bind_r(state),
        Rule::PROPORTION => parse_proportion(state),
        Rule::NS_CONCAT => parse_ns_concat(state),
        Rule::COLON => parse_colon(state),
        Rule::ARROW1 => parse_arrow_1(state),
        Rule::COMMA => parse_comma(state),
        Rule::DOT => parse_dot(state),
        Rule::OP_SLOT => parse_op_slot(state),
        Rule::OFFSET_L => parse_offset_l(state),
        Rule::OFFSET_R => parse_offset_r(state),
        Rule::OP_IMPORT_ALL => parse_op_import_all(state),
        Rule::OP_AND_THEN => parse_op_and_then(state),
        Rule::OP_BIND => parse_op_bind(state),
        Rule::KW_NAMESPACE => parse_kw_namespace(state),
        Rule::KW_IMPORT => parse_kw_import(state),
        Rule::KW_CONSTRAINT => parse_kw_constraint(state),
        Rule::KW_WHERE => parse_kw_where(state),
        Rule::KW_IMPLEMENTS => parse_kw_implements(state),
        Rule::KW_EXTENDS => parse_kw_extends(state),
        Rule::KW_INHERITS => parse_kw_inherits(state),
        Rule::KW_FOR => parse_kw_for(state),
        Rule::KW_END => parse_kw_end(state),
        Rule::KW_LET => parse_kw_let(state),
        Rule::KW_NEW => parse_kw_new(state),
        Rule::KW_OBJECT => parse_kw_object(state),
        Rule::KW_LAMBDA => parse_kw_lambda(state),
        Rule::KW_IF => parse_kw_if(state),
        Rule::KW_SWITCH => parse_kw_switch(state),
        Rule::KW_TRY => parse_kw_try(state),
        Rule::KW_TYPE => parse_kw_type(state),
        Rule::KW_CASE => parse_kw_case(state),
        Rule::KW_WHEN => parse_kw_when(state),
        Rule::KW_ELSE => parse_kw_else(state),
        Rule::KW_NOT => parse_kw_not(state),
        Rule::KW_IN => parse_kw_in(state),
        Rule::KW_IS => parse_kw_is(state),
        Rule::KW_AS => parse_kw_as(state),
        Rule::TEMPLATE_L => parse_template_l(state),
        Rule::TEMPLATE_R => parse_template_r(state),
        Rule::TEMPLATE_M => parse_template_m(state),
        Rule::HiddenText => unreachable!(),
    })
}
#[inline]
fn parse_bind_l(state: Input) -> Output {
    state.rule(Rule::BIND_L, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/≔|:=/)").unwrap())
        })
    })
}
#[inline]
fn parse_bind_r(state: Input) -> Output {
    state.rule(Rule::BIND_R, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/≕|=:/)").unwrap())
        })
    })
}
#[inline]
fn parse_proportion(state: Input) -> Output {
    state.rule(Rule::PROPORTION, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/∷|::/)").unwrap())
        })
    })
}
#[inline]
fn parse_ns_concat(state: Input) -> Output {
    state.rule(Rule::NS_CONCAT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/[.∷]|::/)").unwrap())
        })
    })
}
#[inline]
fn parse_colon(state: Input) -> Output {
    state.rule(Rule::COLON, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([:：])").unwrap())
        })
    })
}
#[inline]
fn parse_arrow_1(state: Input) -> Output {
    state.rule(Rule::ARROW1, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/[:：⟶]|->/)").unwrap())
        })
    })
}
#[inline]
fn parse_comma(state: Input) -> Output {
    state.rule(Rule::COMMA, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([,，])").unwrap())
        })
    })
}
#[inline]
fn parse_dot(state: Input) -> Output {
    state.rule(Rule::DOT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([.．])").unwrap())
        })
    })
}
#[inline]
fn parse_op_slot(state: Input) -> Output {
    state.rule(Rule::OP_SLOT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/[$]{1,3}/)").unwrap())
        })
    })
}
#[inline]
fn parse_offset_l(state: Input) -> Output {
    state.rule(Rule::OFFSET_L, |s| s.match_string("⁅", false))
}
#[inline]
fn parse_offset_r(state: Input) -> Output {
    state.rule(Rule::OFFSET_R, |s| s.match_string("⁆", false))
}
#[inline]
fn parse_op_import_all(state: Input) -> Output {
    state.rule(Rule::OP_IMPORT_ALL, |s| s.match_string("*", false))
}
#[inline]
fn parse_op_and_then(state: Input) -> Output {
    state.rule(Rule::OP_AND_THEN, |s| s.match_string("?", false))
}
#[inline]
fn parse_op_bind(state: Input) -> Output {
    state.rule(Rule::OP_BIND, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/≔|:=/)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_namespace(state: Input) -> Output {
    state.rule(Rule::KW_NAMESPACE, |s| s.match_string("namespace", false))
}
#[inline]
fn parse_kw_import(state: Input) -> Output {
    state.rule(Rule::KW_IMPORT, |s| s.match_string("using", false))
}
#[inline]
fn parse_kw_constraint(state: Input) -> Output {
    state.rule(Rule::KW_CONSTRAINT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/template|generic|constraint|∀/)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_where(state: Input) -> Output {
    state.rule(Rule::KW_WHERE, |s| s.match_string("where", false))
}
#[inline]
fn parse_kw_implements(state: Input) -> Output {
    state.rule(Rule::KW_IMPLEMENTS, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/implements?/)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_extends(state: Input) -> Output {
    state.rule(Rule::KW_EXTENDS, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/extends?/)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_inherits(state: Input) -> Output {
    state.rule(Rule::KW_INHERITS, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/inherits?/)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_for(state: Input) -> Output {
    state.rule(Rule::KW_FOR, |s| s.match_string("for", false))
}
#[inline]
fn parse_kw_end(state: Input) -> Output {
    state.rule(Rule::KW_END, |s| s.match_string("end", false))
}
#[inline]
fn parse_kw_let(state: Input) -> Output {
    state.rule(Rule::KW_LET, |s| s.match_string("let", false))
}
#[inline]
fn parse_kw_new(state: Input) -> Output {
    state.rule(Rule::KW_NEW, |s| s.match_string("new", false))
}
#[inline]
fn parse_kw_object(state: Input) -> Output {
    state.rule(Rule::KW_OBJECT, |s| s.match_string("object", false))
}
#[inline]
fn parse_kw_lambda(state: Input) -> Output {
    state.rule(Rule::KW_LAMBDA, |s| s.match_string("lambda", false))
}
#[inline]
fn parse_kw_if(state: Input) -> Output {
    state.rule(Rule::KW_IF, |s| s.match_string("if", false))
}
#[inline]
fn parse_kw_switch(state: Input) -> Output {
    state.rule(Rule::KW_SWITCH, |s| s.match_string("switch", false))
}
#[inline]
fn parse_kw_try(state: Input) -> Output {
    state.rule(Rule::KW_TRY, |s| s.match_string("try", false))
}
#[inline]
fn parse_kw_type(state: Input) -> Output {
    state.rule(Rule::KW_TYPE, |s| s.match_string("type", false))
}
#[inline]
fn parse_kw_case(state: Input) -> Output {
    state.rule(Rule::KW_CASE, |s| s.match_string("case", false))
}
#[inline]
fn parse_kw_when(state: Input) -> Output {
    state.rule(Rule::KW_WHEN, |s| s.match_string("when", false))
}
#[inline]
fn parse_kw_else(state: Input) -> Output {
    state.rule(Rule::KW_ELSE, |s| s.match_string("else", false))
}
#[inline]
fn parse_kw_not(state: Input) -> Output {
    state.rule(Rule::KW_NOT, |s| s.match_string("not", false))
}
#[inline]
fn parse_kw_in(state: Input) -> Output {
    state.rule(Rule::KW_IN, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(/in|∈/)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_is(state: Input) -> Output {
    state.rule(Rule::KW_IS, |s| s.match_string("is", false))
}
#[inline]
fn parse_kw_as(state: Input) -> Output {
    state.rule(Rule::KW_AS, |s| s.match_string("as", false))
}
#[inline]
fn parse_template_l(state: Input) -> Output {
    state.rule(Rule::TEMPLATE_L, |s| s.match_string("<%", false))
}
#[inline]
fn parse_template_r(state: Input) -> Output {
    state.rule(Rule::TEMPLATE_R, |s| s.match_string("%>", false))
}
#[inline]
fn parse_template_m(state: Input) -> Output {
    state.rule(Rule::TEMPLATE_M, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([-_~.=])").unwrap())
        })
    })
}

/// All rules ignored in ast mode, inline is not recommended
fn builtin_ignore(state: Input) -> Output {
    state.repeat(0..u32::MAX, |s| {})
}

fn builtin_any(state: Input) -> Output {
    state.rule(Rule::HiddenText, |s| s.match_char_if(|_| true))
}

fn builtin_text<'i>(state: Input<'i>, text: &'static str, case: bool) -> Output<'i> {
    state.rule(Rule::HiddenText, |s| s.match_string(text, case))
}

fn builtin_regex<'i, 'r>(state: Input<'i>, regex: &'r Regex) -> Output<'i> {
    state.rule(Rule::HiddenText, |s| s.match_regex(regex))
}
