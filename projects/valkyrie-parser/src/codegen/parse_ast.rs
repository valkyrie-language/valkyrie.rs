#![allow(unused_variables)]
use super::*;
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for BindLNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::BIND_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::BIND_L
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> BindLNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for BindRNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::BIND_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::BIND_R
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> BindRNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ProportionNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::PROPORTION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::PROPORTION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ProportionNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NsConcatNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::NS_CONCAT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::NS_CONCAT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> NsConcatNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ColonNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::COLON)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::COLON
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ColonNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for Arrow1Node<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::ARROW1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::ARROW1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> Arrow1Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for CommaNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::COMMA)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::COMMA
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> CommaNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DotNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::DOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::DOT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DotNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpSlotNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::OP_SLOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::OP_SLOT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OpSlotNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OffsetLNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::OFFSET_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::OFFSET_L
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OffsetLNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OffsetRNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::OFFSET_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::OFFSET_R
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OffsetRNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpImportAllNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::OP_IMPORT_ALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::OP_IMPORT_ALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OpImportAllNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpAndThenNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::OP_AND_THEN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::OP_AND_THEN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OpAndThenNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpBindNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::OP_BIND)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::OP_BIND
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OpBindNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwNamespaceNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_NAMESPACE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_NAMESPACE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwNamespaceNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwImportNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_IMPORT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_IMPORT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwImportNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwConstraintNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_CONSTRAINT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_CONSTRAINT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwConstraintNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwWhereNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_WHERE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_WHERE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwWhereNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwImplementsNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_IMPLEMENTS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_IMPLEMENTS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwImplementsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwExtendsNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_EXTENDS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_EXTENDS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwExtendsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwInheritsNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_INHERITS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_INHERITS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwInheritsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwForNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_FOR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_FOR
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwForNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwEndNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_END)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_END
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwEndNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwLetNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_LET)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_LET
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwLetNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwNewNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_NEW)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_NEW
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwNewNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwObjectNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_OBJECT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_OBJECT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwObjectNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwLambdaNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_LAMBDA)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_LAMBDA
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwLambdaNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwIfNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_IF)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_IF
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwIfNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwSwitchNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_SWITCH)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_SWITCH
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwSwitchNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwTryNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_TRY)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_TRY
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwTryNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwTypeNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_TYPE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_TYPE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwTypeNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwCaseNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_CASE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_CASE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwCaseNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwWhenNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_WHEN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_WHEN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwWhenNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwElseNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_ELSE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_ELSE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwElseNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwNotNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_NOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_NOT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwNotNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwInNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_IN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_IN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwInNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwIsNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_IS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_IS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwIsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwAsNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::KW_AS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::KW_AS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwAsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TemplateLNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::TEMPLATE_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::TEMPLATE_L
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TemplateLNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TemplateRNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::TEMPLATE_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::TEMPLATE_R
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TemplateRNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TemplateMNode<'i> {
    type Rule = Rule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(Parser::parse_cst(input, Rule::TEMPLATE_M)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        Rule::TEMPLATE_M
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TemplateMNode<'i> {}
