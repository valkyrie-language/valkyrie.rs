use super::*;

impl ExpressionStream {
    /// term (~ infix ~ term)*
    /// 1 + (1 + +3? + 4)
    pub fn parse(state: ParseState) -> ParseResult<Vec<ExpressionStream>> {
        let mut stream = Vec::new();
        let (state, _) = state.match_fn(|s| parse_term(s, &mut stream))?;
        let (state, _) = state.match_repeats(|s| parse_infix_term(s, &mut stream))?;
        state.finish(stream)
    }
}

/// `~ infix ~ term`
#[inline(always)]
fn parse_infix_term<'i>(input: ParseState<'i>, stream: &mut Vec<ExpressionStream>) -> ParseResult<'i, ()> {
    let (state, infix) = input.skip(ignore).match_fn(ValkyrieInfix::parse).map_inner(ExpressionStream::Infix)?;
    stream.push(infix);
    let (state, _) = state.skip(ignore).match_fn(|s| parse_term(s, stream))?;
    state.finish(())
}

/// `(~ prefix)* ~ value (~ suffix)*`
fn parse_term<'i>(state: ParseState<'i>, stream: &mut Vec<ExpressionStream>) -> ParseResult<'i, ()> {
    let (state, prefix) = state.match_repeats(parse_prefix)?;
    stream.extend(prefix);
    let (state, value) = parse_expr_value(state)?;
    stream.push(value);
    let (state, suffix) = state.match_repeats(parse_suffix)?;
    stream.extend(suffix);
    state.finish(())
}

#[inline(always)]
fn parse_prefix(input: ParseState) -> ParseResult<ExpressionStream> {
    let (state, prefix) = input.skip(ignore).match_fn(ValkyriePrefix::parse)?;
    state.finish(ExpressionStream::Prefix(prefix))
}

#[inline(always)]
fn parse_suffix(input: ParseState) -> ParseResult<ExpressionStream> {
    let (state, suffix) = input.skip(ignore).match_fn(ValkyrieSuffix::parse)?;
    state.finish(ExpressionStream::Postfix(suffix))
}

#[inline(always)]
fn parse_expr_value(input: ParseState) -> ParseResult<ExpressionStream> {
    input.skip(ignore).match_fn(parse_value).map_inner(ExpressionStream::Term)
}
