#![allow(unused_variables)]
use super::*;
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ProgramNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PROGRAM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PROGRAM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ProgramNode<'i> {
    pub fn shebang(&self) -> Option<ShebangNode<'i>> {
        self.pair.take_tagged_option("shebang")
    }
    pub fn statement(&self) -> Vec<StatementNode<'i>> {
        self.pair.take_tagged_items("statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("define_namespace") {
            return Ok(Self::DefineNamespace(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_class") {
            return Ok(Self::DefineClass(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_union") {
            return Ok(Self::DefineUnion(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_enumerate") {
            return Ok(Self::DefineEnumerate(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_trait") {
            return Ok(Self::DefineTrait(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_extends") {
            return Ok(Self::DefineExtends(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_function") {
            return Ok(Self::DefineFunction(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_variable") {
            return Ok(Self::DefineVariable(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_import") {
            return Ok(Self::DefineImport(s));
        }
        if let Ok(s) = pair.take_tagged_one("control_flow") {
            return Ok(Self::ControlFlow(s));
        }
        if let Ok(s) = pair.take_tagged_one("loop_each_statement") {
            return Ok(Self::LoopEachStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("loop_while_statement") {
            return Ok(Self::LoopWhileStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("loop_until_statement") {
            return Ok(Self::LoopUntilStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("loop_statement") {
            return Ok(Self::LoopStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("expression_root") {
            return Ok(Self::ExpressionRoot(s));
        }
        if let Ok(s) = pair.take_tagged_one("eos") {
            return Ok(Self::Eos(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::STATEMENT, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STATEMENT
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::DefineNamespace(s) => s.get_str(),
            Self::DefineClass(s) => s.get_str(),
            Self::DefineUnion(s) => s.get_str(),
            Self::DefineEnumerate(s) => s.get_str(),
            Self::DefineTrait(s) => s.get_str(),
            Self::DefineExtends(s) => s.get_str(),
            Self::DefineFunction(s) => s.get_str(),
            Self::DefineVariable(s) => s.get_str(),
            Self::DefineImport(s) => s.get_str(),
            Self::ControlFlow(s) => s.get_str(),
            Self::LoopEachStatement(s) => s.get_str(),
            Self::LoopWhileStatement(s) => s.get_str(),
            Self::LoopUntilStatement(s) => s.get_str(),
            Self::LoopStatement(s) => s.get_str(),
            Self::ExpressionRoot(s) => s.get_str(),
            Self::Eos(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DefineNamespace(s) => s.get_range(),
            Self::DefineClass(s) => s.get_range(),
            Self::DefineUnion(s) => s.get_range(),
            Self::DefineEnumerate(s) => s.get_range(),
            Self::DefineTrait(s) => s.get_range(),
            Self::DefineExtends(s) => s.get_range(),
            Self::DefineFunction(s) => s.get_range(),
            Self::DefineVariable(s) => s.get_range(),
            Self::DefineImport(s) => s.get_range(),
            Self::ControlFlow(s) => s.get_range(),
            Self::LoopEachStatement(s) => s.get_range(),
            Self::LoopWhileStatement(s) => s.get_range(),
            Self::LoopUntilStatement(s) => s.get_range(),
            Self::LoopStatement(s) => s.get_range(),
            Self::ExpressionRoot(s) => s.get_range(),
            Self::Eos(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for EosNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EOS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EOS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> EosNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for EosFreeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EOS_FREE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EOS_FREE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> EosFreeNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineNamespaceNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_NAMESPACE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_NAMESPACE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineNamespaceNode<'i> {
    pub fn namepath_free(&self) -> NamepathFreeNode<'i> {
        self.pair.take_tagged_one("namepath_free").unwrap()
    }
    pub fn op_namespace(&self) -> Option<OpNamespaceNode<'i>> {
        self.pair.take_tagged_option("op_namespace")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpNamespaceNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_NAMESPACE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("main") {
            return Ok(Self::Main(s));
        }
        if let Ok(s) = pair.take_tagged_one("test") {
            return Ok(Self::Test(s));
        }
        if let Ok(s) = pair.take_tagged_one("hide") {
            return Ok(Self::Hide(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::OP_NAMESPACE, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_NAMESPACE
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Main(s) => s.get_str(),
            Self::Test(s) => s.get_str(),
            Self::Hide(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Main(s) => s.get_range(),
            Self::Test(s) => s.get_range(),
            Self::Hide(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineImportNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_IMPORT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_IMPORT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineImportNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn eos_free(&self) -> Option<EosFreeNode<'i>> {
        self.pair.take_tagged_option("eos_free")
    }
    pub fn import_block(&self) -> Option<ImportBlockNode<'i>> {
        self.pair.take_tagged_option("import_block")
    }
    pub fn import_term(&self) -> Option<ImportTermNode<'i>> {
        self.pair.take_tagged_option("import_term")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IMPORT_BLOCK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IMPORT_BLOCK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ImportBlockNode<'i> {
    pub fn import_term(&self) -> Vec<ImportTermNode<'i>> {
        self.pair.take_tagged_items("import_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IMPORT_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("import_all") {
            return Ok(Self::ImportAll(s));
        }
        if let Ok(s) = pair.take_tagged_one("import_space") {
            return Ok(Self::ImportSpace(s));
        }
        if let Ok(s) = pair.take_tagged_one("import_name") {
            return Ok(Self::ImportName(s));
        }
        if let Ok(s) = pair.take_tagged_one("eos_free") {
            return Ok(Self::EosFree(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::IMPORT_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IMPORT_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ImportAll(s) => s.get_str(),
            Self::ImportSpace(s) => s.get_str(),
            Self::ImportName(s) => s.get_str(),
            Self::EosFree(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ImportAll(s) => s.get_range(),
            Self::ImportSpace(s) => s.get_range(),
            Self::ImportName(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportAllNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IMPORT_ALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IMPORT_ALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ImportAllNode<'i> {
    pub fn path(&self) -> Vec<IdentifierNode<'i>> {
        self.pair.take_tagged_items("path").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportSpaceNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IMPORT_SPACE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IMPORT_SPACE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ImportSpaceNode<'i> {
    pub fn body(&self) -> ImportBlockNode<'i> {
        self.pair.take_tagged_one("body").unwrap()
    }
    pub fn path(&self) -> Vec<IdentifierNode<'i>> {
        self.pair.take_tagged_items("path").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportNameNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IMPORT_NAME)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IMPORT_NAME
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ImportNameNode<'i> {
    pub fn alias(&self) -> ImportAsNode<'i> {
        self.pair.take_tagged_one("alias").unwrap()
    }
    pub fn item(&self) -> ImportNameItemNode<'i> {
        self.pair.take_tagged_one("item").unwrap()
    }
    pub fn path(&self) -> Vec<IdentifierNode<'i>> {
        self.pair.take_tagged_items("path").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportAsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IMPORT_AS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IMPORT_AS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ImportAsNode<'i> {
    pub fn alias(&self) -> Option<ImportNameItemNode<'i>> {
        self.pair.take_tagged_option("alias")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportNameItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IMPORT_NAME_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("procedural_name") {
            return Ok(Self::ProceduralName(s));
        }
        if let Ok(s) = pair.take_tagged_one("attribute_name") {
            return Ok(Self::AttributeName(s));
        }
        if let Ok(s) = pair.take_tagged_one("identifier") {
            return Ok(Self::Identifier(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::IMPORT_NAME_ITEM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IMPORT_NAME_ITEM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ProceduralName(s) => s.get_str(),
            Self::AttributeName(s) => s.get_str(),
            Self::Identifier(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralName(s) => s.get_range(),
            Self::AttributeName(s) => s.get_range(),
            Self::Identifier(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineConstraintNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_CONSTRAINT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_CONSTRAINT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineConstraintNode<'i> {
    pub fn annotation_head(&self) -> AnnotationHeadNode<'i> {
        self.pair.take_tagged_one("annotation_head").unwrap()
    }
    pub fn constraint_block(&self) -> ConstraintBlockNode<'i> {
        self.pair.take_tagged_one("constraint_block").unwrap()
    }
    pub fn constraint_parameters(&self) -> Option<ConstraintParametersNode<'i>> {
        self.pair.take_tagged_option("constraint_parameters")
    }
    pub fn kw_constraint(&self) -> KwConstraintNode<'i> {
        self.pair.take_tagged_one("kw_constraint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ConstraintParametersNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CONSTRAINT_PARAMETERS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CONSTRAINT_PARAMETERS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ConstraintParametersNode<'i> {
    pub fn identifier(&self) -> Vec<IdentifierNode<'i>> {
        self.pair.take_tagged_items("identifier").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ConstraintBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CONSTRAINT_BLOCK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CONSTRAINT_BLOCK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ConstraintBlockNode<'i> {
    pub fn constraint_implements(&self) -> Vec<ConstraintImplementsNode<'i>> {
        self.pair.take_tagged_items("constraint_implements").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn constraint_statement(&self) -> Vec<ConstraintStatementNode<'i>> {
        self.pair.take_tagged_items("constraint_statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn eos_free(&self) -> Vec<EosFreeNode<'i>> {
        self.pair.take_tagged_items("eos_free").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ConstraintStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CONSTRAINT_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CONSTRAINT_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ConstraintStatementNode<'i> {
    pub fn where_block(&self) -> WhereBlockNode<'i> {
        self.pair.take_tagged_one("where_block").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ConstraintImplementsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CONSTRAINT_IMPLEMENTS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CONSTRAINT_IMPLEMENTS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ConstraintImplementsNode<'i> {
    pub fn kw_implements(&self) -> KwImplementsNode<'i> {
        self.pair.take_tagged_one("kw_implements").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for WhereBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WHERE_BLOCK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::WHERE_BLOCK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> WhereBlockNode<'i> {
    pub fn kw_where(&self) -> KwWhereNode<'i> {
        self.pair.take_tagged_one("kw_where").unwrap()
    }
    pub fn where_bound(&self) -> Vec<WhereBoundNode<'i>> {
        self.pair.take_tagged_items("where_bound").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for WhereBoundNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WHERE_BOUND)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::WHERE_BOUND
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> WhereBoundNode<'i> {
    pub fn eos_free(&self) -> EosFreeNode<'i> {
        self.pair.take_tagged_one("eos_free").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineClassNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_CLASS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_CLASS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineClassNode<'i> {
    pub fn annotation_head(&self) -> AnnotationHeadNode<'i> {
        self.pair.take_tagged_one("annotation_head").unwrap()
    }
    pub fn class_block(&self) -> ClassBlockNode<'i> {
        self.pair.take_tagged_one("class_block").unwrap()
    }
    pub fn class_kind(&self) -> ClassKindNode<'i> {
        self.pair.take_tagged_one("class_kind").unwrap()
    }
    pub fn define_constraint(&self) -> Option<DefineConstraintNode<'i>> {
        self.pair.take_tagged_option("define_constraint")
    }
    pub fn define_generic(&self) -> Option<DefineGenericNode<'i>> {
        self.pair.take_tagged_option("define_generic")
    }
    pub fn define_inherit(&self) -> Option<DefineInheritNode<'i>> {
        self.pair.take_tagged_option("define_inherit")
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ClassKindNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CLASS_KIND)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("kw_class") {
            return Ok(Self::KwClass(s));
        }
        if let Ok(s) = pair.take_tagged_one("kw_structure") {
            return Ok(Self::KwStructure(s));
        }
        if let Ok(s) = pair.take_tagged_one("kw_widget") {
            return Ok(Self::KwWidget(s));
        }
        if let Ok(s) = pair.take_tagged_one("kw_neural") {
            return Ok(Self::KwNeural(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::CLASS_KIND, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CLASS_KIND
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::KwClass(s) => s.get_str(),
            Self::KwStructure(s) => s.get_str(),
            Self::KwWidget(s) => s.get_str(),
            Self::KwNeural(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::KwClass(s) => s.get_range(),
            Self::KwStructure(s) => s.get_range(),
            Self::KwWidget(s) => s.get_range(),
            Self::KwNeural(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ClassBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CLASS_BLOCK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CLASS_BLOCK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ClassBlockNode<'i> {
    pub fn class_term(&self) -> Vec<ClassTermNode<'i>> {
        self.pair.take_tagged_items("class_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ClassTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CLASS_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("procedural_call") {
            return Ok(Self::ProceduralCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_method") {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_domain") {
            return Ok(Self::DefineDomain(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_field") {
            return Ok(Self::DefineField(s));
        }
        if let Ok(s) = pair.take_tagged_one("eos_free") {
            return Ok(Self::EosFree(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::CLASS_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CLASS_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ProceduralCall(s) => s.get_str(),
            Self::DefineMethod(s) => s.get_str(),
            Self::DefineDomain(s) => s.get_str(),
            Self::DefineField(s) => s.get_str(),
            Self::EosFree(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::DefineDomain(s) => s.get_range(),
            Self::DefineField(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineFieldNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_FIELD)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_FIELD
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineFieldNode<'i> {
    pub fn annotation_mix(&self) -> AnnotationMixNode<'i> {
        self.pair.take_tagged_one("annotation_mix").unwrap()
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn parameter_default(&self) -> Option<ParameterDefaultNode<'i>> {
        self.pair.take_tagged_option("parameter_default")
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ParameterDefaultNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PARAMETER_DEFAULT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PARAMETER_DEFAULT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ParameterDefaultNode<'i> {
    pub fn default(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("default").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineMethodNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_METHOD)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_METHOD
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineMethodNode<'i> {
    pub fn annotation_mix(&self) -> AnnotationMixNode<'i> {
        self.pair.take_tagged_one("annotation_mix").unwrap()
    }
    pub fn continuation(&self) -> Option<ContinuationNode<'i>> {
        self.pair.take_tagged_option("continuation")
    }
    pub fn function_middle(&self) -> FunctionMiddleNode<'i> {
        self.pair.take_tagged_one("function_middle").unwrap()
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineDomainNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_DOMAIN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_DOMAIN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineDomainNode<'i> {
    pub fn annotation_mix(&self) -> AnnotationMixNode<'i> {
        self.pair.take_tagged_one("annotation_mix").unwrap()
    }
    pub fn domain_term(&self) -> DomainTermNode<'i> {
        self.pair.take_tagged_one("domain_term").unwrap()
    }
    pub fn statement(&self) -> Vec<StatementNode<'i>> {
        self.pair.take_tagged_items("statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DomainTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DOMAIN_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("identifier") {
            return Ok(Self::Identifier(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::DOMAIN_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DOMAIN_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Identifier(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Identifier(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineInheritNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_INHERIT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_INHERIT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineInheritNode<'i> {
    pub fn inherit_term(&self) -> Vec<InheritTermNode<'i>> {
        self.pair.take_tagged_items("inherit_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InheritTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INHERIT_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INHERIT_TERM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> InheritTermNode<'i> {
    pub fn annotation_mix(&self) -> AnnotationMixNode<'i> {
        self.pair.take_tagged_one("annotation_mix").unwrap()
    }
    pub fn type_expression(&self) -> TypeExpressionNode<'i> {
        self.pair.take_tagged_one("type_expression").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ObjectStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OBJECT_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OBJECT_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ObjectStatementNode<'i> {
    pub fn class_block(&self) -> ClassBlockNode<'i> {
        self.pair.take_tagged_one("class_block").unwrap()
    }
    pub fn define_inherit(&self) -> Option<DefineInheritNode<'i>> {
        self.pair.take_tagged_option("define_inherit")
    }
    pub fn kw_object(&self) -> KwObjectNode<'i> {
        self.pair.take_tagged_one("kw_object").unwrap()
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineEnumerateNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_ENUMERATE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_ENUMERATE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineEnumerateNode<'i> {
    pub fn annotation_head(&self) -> AnnotationHeadNode<'i> {
        self.pair.take_tagged_one("annotation_head").unwrap()
    }
    pub fn define_inherit(&self) -> Option<DefineInheritNode<'i>> {
        self.pair.take_tagged_option("define_inherit")
    }
    pub fn flag_term(&self) -> Vec<FlagTermNode<'i>> {
        self.pair.take_tagged_items("flag_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn kw_enumerate(&self) -> KwEnumerateNode<'i> {
        self.pair.take_tagged_one("kw_enumerate").unwrap()
    }
    pub fn layout(&self) -> Option<TypeExpressionNode<'i>> {
        self.pair.take_tagged_option("layout")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for FlagTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FLAG_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("procedural_call") {
            return Ok(Self::ProceduralCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_method") {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one("flag_field") {
            return Ok(Self::FlagField(s));
        }
        if let Ok(s) = pair.take_tagged_one("eos_free") {
            return Ok(Self::EosFree(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::FLAG_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FLAG_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ProceduralCall(s) => s.get_str(),
            Self::DefineMethod(s) => s.get_str(),
            Self::FlagField(s) => s.get_str(),
            Self::EosFree(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::FlagField(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for FlagFieldNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FLAG_FIELD)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FLAG_FIELD
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> FlagFieldNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn parameter_default(&self) -> Option<ParameterDefaultNode<'i>> {
        self.pair.take_tagged_option("parameter_default")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineUnionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_UNION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_UNION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineUnionNode<'i> {
    pub fn annotation_head(&self) -> AnnotationHeadNode<'i> {
        self.pair.take_tagged_one("annotation_head").unwrap()
    }
    pub fn define_constraint(&self) -> Option<DefineConstraintNode<'i>> {
        self.pair.take_tagged_option("define_constraint")
    }
    pub fn define_generic(&self) -> Option<DefineGenericNode<'i>> {
        self.pair.take_tagged_option("define_generic")
    }
    pub fn define_inherit(&self) -> Option<DefineInheritNode<'i>> {
        self.pair.take_tagged_option("define_inherit")
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn kw_union(&self) -> KwUnionNode<'i> {
        self.pair.take_tagged_one("kw_union").unwrap()
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
    pub fn union_term(&self) -> Vec<UnionTermNode<'i>> {
        self.pair.take_tagged_items("union_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for UnionTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::UNION_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("procedural_call") {
            return Ok(Self::ProceduralCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_method") {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_variant") {
            return Ok(Self::DefineVariant(s));
        }
        if let Ok(s) = pair.take_tagged_one("eos_free") {
            return Ok(Self::EosFree(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::UNION_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::UNION_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ProceduralCall(s) => s.get_str(),
            Self::DefineMethod(s) => s.get_str(),
            Self::DefineVariant(s) => s.get_str(),
            Self::EosFree(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::DefineVariant(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineVariantNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_VARIANT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_VARIANT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineVariantNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn class_block(&self) -> Option<ClassBlockNode<'i>> {
        self.pair.take_tagged_option("class_block")
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwUnionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_UNION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_UNION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwUnionNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineTraitNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_TRAIT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_TRAIT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineTraitNode<'i> {
    pub fn annotation_head(&self) -> AnnotationHeadNode<'i> {
        self.pair.take_tagged_one("annotation_head").unwrap()
    }
    pub fn define_constraint(&self) -> Option<DefineConstraintNode<'i>> {
        self.pair.take_tagged_option("define_constraint")
    }
    pub fn define_generic(&self) -> Option<DefineGenericNode<'i>> {
        self.pair.take_tagged_option("define_generic")
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn kw_trait(&self) -> KwTraitNode<'i> {
        self.pair.take_tagged_one("kw_trait").unwrap()
    }
    pub fn trait_block(&self) -> TraitBlockNode<'i> {
        self.pair.take_tagged_one("trait_block").unwrap()
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TraitBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TRAIT_BLOCK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TRAIT_BLOCK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TraitBlockNode<'i> {
    pub fn trait_term(&self) -> Vec<TraitTermNode<'i>> {
        self.pair.take_tagged_items("trait_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TraitTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TRAIT_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("procedural_call") {
            return Ok(Self::ProceduralCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_method") {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_field") {
            return Ok(Self::DefineField(s));
        }
        if let Ok(s) = pair.take_tagged_one("eos_free") {
            return Ok(Self::EosFree(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TRAIT_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TRAIT_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ProceduralCall(s) => s.get_str(),
            Self::DefineMethod(s) => s.get_str(),
            Self::DefineField(s) => s.get_str(),
            Self::EosFree(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::DefineField(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineExtendsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_EXTENDS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_EXTENDS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineExtendsNode<'i> {
    pub fn annotation_head(&self) -> AnnotationHeadNode<'i> {
        self.pair.take_tagged_one("annotation_head").unwrap()
    }
    pub fn define_constraint(&self) -> Option<DefineConstraintNode<'i>> {
        self.pair.take_tagged_option("define_constraint")
    }
    pub fn kw_extends(&self) -> KwExtendsNode<'i> {
        self.pair.take_tagged_one("kw_extends").unwrap()
    }
    pub fn namepath(&self) -> NamepathNode<'i> {
        self.pair.take_tagged_one("namepath").unwrap()
    }
    pub fn trait_block(&self) -> TraitBlockNode<'i> {
        self.pair.take_tagged_one("trait_block").unwrap()
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineFunctionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_FUNCTION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_FUNCTION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineFunctionNode<'i> {
    pub fn annotation_head(&self) -> AnnotationHeadNode<'i> {
        self.pair.take_tagged_one("annotation_head").unwrap()
    }
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn function_middle(&self) -> FunctionMiddleNode<'i> {
        self.pair.take_tagged_one("function_middle").unwrap()
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn kw_function(&self) -> KwFunctionNode<'i> {
        self.pair.take_tagged_one("kw_function").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineLambdaNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_LAMBDA)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_LAMBDA
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineLambdaNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn function_middle(&self) -> FunctionMiddleNode<'i> {
        self.pair.take_tagged_one("function_middle").unwrap()
    }
    pub fn kw_lambda(&self) -> KwLambdaNode<'i> {
        self.pair.take_tagged_one("kw_lambda").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for FunctionMiddleNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FUNCTION_MIDDLE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FUNCTION_MIDDLE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> FunctionMiddleNode<'i> {
    pub fn define_generic(&self) -> Option<DefineGenericNode<'i>> {
        self.pair.take_tagged_option("define_generic")
    }
    pub fn function_parameters(&self) -> FunctionParametersNode<'i> {
        self.pair.take_tagged_one("function_parameters").unwrap()
    }
    pub fn type_effect(&self) -> Option<TypeEffectNode<'i>> {
        self.pair.take_tagged_option("type_effect")
    }
    pub fn type_return(&self) -> Option<TypeReturnNode<'i>> {
        self.pair.take_tagged_option("type_return")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeHintNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_HINT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_HINT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeHintNode<'i> {
    pub fn hint(&self) -> TypeExpressionNode<'i> {
        self.pair.take_tagged_one("hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeReturnNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_RETURN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_RETURN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeReturnNode<'i> {
    pub fn arrow_1(&self) -> Arrow1Node<'i> {
        self.pair.take_tagged_one("arrow_1").unwrap()
    }
    pub fn type_expression(&self) -> TypeExpressionNode<'i> {
        self.pair.take_tagged_one("type_expression").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeEffectNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_EFFECT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_EFFECT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeEffectNode<'i> {
    pub fn type_expression(&self) -> TypeExpressionNode<'i> {
        self.pair.take_tagged_one("type_expression").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for FunctionParametersNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FUNCTION_PARAMETERS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FUNCTION_PARAMETERS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> FunctionParametersNode<'i> {
    pub fn parameter_item(&self) -> Vec<ParameterItemNode<'i>> {
        self.pair.take_tagged_items("parameter_item").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ParameterItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PARAMETER_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("parameter_item_control") {
            return Ok(Self::ParameterItemControl(s));
        }
        if let Ok(s) = pair.take_tagged_one("parameter_pair") {
            return Ok(Self::ParameterPair(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::PARAMETER_ITEM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PARAMETER_ITEM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ParameterItemControl(s) => s.get_str(),
            Self::ParameterPair(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ParameterItemControl(s) => s.get_range(),
            Self::ParameterPair(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ParameterItemControlNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PARAMETER_ITEM_CONTROL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PARAMETER_ITEM_CONTROL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ParameterItemControlNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ParameterPairNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PARAMETER_PAIR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PARAMETER_PAIR
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ParameterPairNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn modifier_ahead(&self) -> Vec<ModifierAheadNode<'i>> {
        self.pair.take_tagged_items("modifier_ahead").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn parameter_default(&self) -> Option<ParameterDefaultNode<'i>> {
        self.pair.take_tagged_option("parameter_default")
    }
    pub fn parameter_hint(&self) -> Option<ParameterHintNode<'i>> {
        self.pair.take_tagged_option("parameter_hint")
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ParameterHintNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PARAMETER_HINT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PARAMETER_HINT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ParameterHintNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ContinuationNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CONTINUATION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CONTINUATION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ContinuationNode<'i> {
    pub fn statement(&self) -> Vec<StatementNode<'i>> {
        self.pair.take_tagged_items("statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwFunctionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_FUNCTION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_FUNCTION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwFunctionNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineVariableNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_VARIABLE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_VARIABLE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineVariableNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn kw_let(&self) -> KwLetNode<'i> {
        self.pair.take_tagged_one("kw_let").unwrap()
    }
    pub fn let_pattern(&self) -> LetPatternNode<'i> {
        self.pair.take_tagged_one("let_pattern").unwrap()
    }
    pub fn parameter_default(&self) -> Option<ParameterDefaultNode<'i>> {
        self.pair.take_tagged_option("parameter_default")
    }
    pub fn type_hint(&self) -> Option<TypeHintNode<'i>> {
        self.pair.take_tagged_option("type_hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for LetPatternNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LET_PATTERN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("standard_pattern") {
            return Ok(Self::StandardPattern(s));
        }
        if let Ok(s) = pair.take_tagged_one("bare_pattern") {
            return Ok(Self::BarePattern(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::LET_PATTERN, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::LET_PATTERN
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::StandardPattern(s) => s.get_str(),
            Self::BarePattern(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::StandardPattern(s) => s.get_range(),
            Self::BarePattern(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StandardPatternNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STANDARD_PATTERN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("tuple_pattern") {
            return Ok(Self::TuplePattern(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::STANDARD_PATTERN, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STANDARD_PATTERN
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::TuplePattern(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::TuplePattern(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for BarePatternNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BARE_PATTERN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::BARE_PATTERN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> BarePatternNode<'i> {
    pub fn bare_pattern_item(&self) -> Vec<BarePatternItemNode<'i>> {
        self.pair.take_tagged_items("bare_pattern_item").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for BarePatternItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BARE_PATTERN_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::BARE_PATTERN_ITEM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> BarePatternItemNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn modifier_ahead(&self) -> Vec<ModifierAheadNode<'i>> {
        self.pair.take_tagged_items("modifier_ahead").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TuplePatternNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_PATTERN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_PATTERN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TuplePatternNode<'i> {
    pub fn namepath(&self) -> Option<NamepathNode<'i>> {
        self.pair.take_tagged_option("namepath")
    }
    pub fn pattern_item(&self) -> Vec<PatternItemNode<'i>> {
        self.pair.take_tagged_items("pattern_item").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for PatternItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PATTERN_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("tuple_pattern_item") {
            return Ok(Self::TuplePatternItem(s));
        }
        if let Ok(s) = pair.take_tagged_one("omit_dict") {
            return Ok(Self::OmitDict(s));
        }
        if let Ok(s) = pair.take_tagged_one("omit_list") {
            return Ok(Self::OmitList(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::PATTERN_ITEM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PATTERN_ITEM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::TuplePatternItem(s) => s.get_str(),
            Self::OmitDict(s) => s.get_str(),
            Self::OmitList(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::TuplePatternItem(s) => s.get_range(),
            Self::OmitDict(s) => s.get_range(),
            Self::OmitList(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TuplePatternItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_PATTERN_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_PATTERN_ITEM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TuplePatternItemNode<'i> {
    pub fn annotation_mix(&self) -> AnnotationMixNode<'i> {
        self.pair.take_tagged_one("annotation_mix").unwrap()
    }
    pub fn colon(&self) -> Option<ColonNode<'i>> {
        self.pair.take_tagged_option("colon")
    }
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn parameter_hint(&self) -> Option<ParameterHintNode<'i>> {
        self.pair.take_tagged_option("parameter_hint")
    }
    pub fn standard_pattern(&self) -> Option<StandardPatternNode<'i>> {
        self.pair.take_tagged_option("standard_pattern")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for LoopStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LOOP_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::LOOP_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> LoopStatementNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn kw_loop(&self) -> KwLoopNode<'i> {
        self.pair.take_tagged_one("kw_loop").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for LoopWhileStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LOOP_WHILE_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::LOOP_WHILE_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> LoopWhileStatementNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn inline_expression(&self) -> InlineExpressionNode<'i> {
        self.pair.take_tagged_one("inline_expression").unwrap()
    }
    pub fn kw_loop(&self) -> KwLoopNode<'i> {
        self.pair.take_tagged_one("kw_loop").unwrap()
    }
    pub fn kw_while(&self) -> KwWhileNode<'i> {
        self.pair.take_tagged_one("kw_while").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for LoopUntilStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LOOP_UNTIL_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::LOOP_UNTIL_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> LoopUntilStatementNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn inline_expression(&self) -> InlineExpressionNode<'i> {
        self.pair.take_tagged_one("inline_expression").unwrap()
    }
    pub fn kw_loop(&self) -> KwLoopNode<'i> {
        self.pair.take_tagged_one("kw_loop").unwrap()
    }
    pub fn kw_until(&self) -> KwUntilNode<'i> {
        self.pair.take_tagged_one("kw_until").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for LoopEachStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LOOP_EACH_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::LOOP_EACH_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> LoopEachStatementNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn if_guard(&self) -> IfGuardNode<'i> {
        self.pair.take_tagged_one("if_guard").unwrap()
    }
    pub fn inline_expression(&self) -> Option<InlineExpressionNode<'i>> {
        self.pair.take_tagged_option("inline_expression")
    }
    pub fn kw_each(&self) -> Option<KwEachNode<'i>> {
        self.pair.take_tagged_option("kw_each")
    }
    pub fn kw_in(&self) -> KwInNode<'i> {
        self.pair.take_tagged_one("kw_in").unwrap()
    }
    pub fn kw_loop(&self) -> KwLoopNode<'i> {
        self.pair.take_tagged_one("kw_loop").unwrap()
    }
    pub fn let_pattern(&self) -> LetPatternNode<'i> {
        self.pair.take_tagged_one("let_pattern").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IfGuardNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IF_GUARD)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IF_GUARD
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> IfGuardNode<'i> {
    pub fn condition(&self) -> Option<InlineExpressionNode<'i>> {
        self.pair.take_tagged_option("condition")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ControlFlowNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CONTROL_FLOW)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CONTROL_FLOW
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ControlFlowNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn jump_label(&self) -> JumpLabelNode<'i> {
        self.pair.take_tagged_one("jump_label").unwrap()
    }
    pub fn kw_control(&self) -> KwControlNode<'i> {
        self.pair.take_tagged_one("kw_control").unwrap()
    }
    pub fn main_expression(&self) -> Option<MainExpressionNode<'i>> {
        self.pair.take_tagged_option("main_expression")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for JumpLabelNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::JUMP_LABEL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::JUMP_LABEL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> JumpLabelNode<'i> {
    pub fn identifier(&self) -> Option<IdentifierNode<'i>> {
        self.pair.take_tagged_option("identifier")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ExpressionRootNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EXPRESSION_ROOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EXPRESSION_ROOT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ExpressionRootNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn eos(&self) -> Option<EosNode<'i>> {
        self.pair.take_tagged_option("eos")
    }
    pub fn main_expression(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("main_expression").unwrap()
    }
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchExpressionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_EXPRESSION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_EXPRESSION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchExpressionNode<'i> {
    pub fn bind_l(&self) -> Option<BindLNode<'i>> {
        self.pair.take_tagged_option("bind_l")
    }
    pub fn bind_r(&self) -> Option<BindRNode<'i>> {
        self.pair.take_tagged_option("bind_r")
    }
    pub fn identifier(&self) -> Option<IdentifierNode<'i>> {
        self.pair.take_tagged_option("identifier")
    }
    pub fn inline_expression(&self) -> InlineExpressionNode<'i> {
        self.pair.take_tagged_one("inline_expression").unwrap()
    }
    pub fn kw_match(&self) -> KwMatchNode<'i> {
        self.pair.take_tagged_one("kw_match").unwrap()
    }
    pub fn match_block(&self) -> MatchBlockNode<'i> {
        self.pair.take_tagged_one("match_block").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SwitchStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SWITCH_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SWITCH_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> SwitchStatementNode<'i> {
    pub fn kw_switch(&self) -> KwSwitchNode<'i> {
        self.pair.take_tagged_one("kw_switch").unwrap()
    }
    pub fn match_block(&self) -> MatchBlockNode<'i> {
        self.pair.take_tagged_one("match_block").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_BLOCK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_BLOCK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchBlockNode<'i> {
    pub fn match_terms(&self) -> Vec<MatchTermsNode<'i>> {
        self.pair.take_tagged_items("match_terms").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchTermsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_TERMS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("match_type") {
            return Ok(Self::MatchType(s));
        }
        if let Ok(s) = pair.take_tagged_one("match_case") {
            return Ok(Self::MatchCase(s));
        }
        if let Ok(s) = pair.take_tagged_one("match_when") {
            return Ok(Self::MatchWhen(s));
        }
        if let Ok(s) = pair.take_tagged_one("match_else") {
            return Ok(Self::MatchElse(s));
        }
        if let Ok(s) = pair.take_tagged_one("comma") {
            return Ok(Self::Comma(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MATCH_TERMS, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_TERMS
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::MatchType(s) => s.get_str(),
            Self::MatchCase(s) => s.get_str(),
            Self::MatchWhen(s) => s.get_str(),
            Self::MatchElse(s) => s.get_str(),
            Self::Comma(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::MatchType(s) => s.get_range(),
            Self::MatchCase(s) => s.get_range(),
            Self::MatchWhen(s) => s.get_range(),
            Self::MatchElse(s) => s.get_range(),
            Self::Comma(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchTypeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_TYPE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_TYPE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchTypeNode<'i> {
    pub fn if_guard(&self) -> IfGuardNode<'i> {
        self.pair.take_tagged_one("if_guard").unwrap()
    }
    pub fn kw_type(&self) -> KwTypeNode<'i> {
        self.pair.take_tagged_one("kw_type").unwrap()
    }
    pub fn match_statement(&self) -> Vec<MatchStatementNode<'i>> {
        self.pair.take_tagged_items("match_statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn type_expression(&self) -> TypeExpressionNode<'i> {
        self.pair.take_tagged_one("type_expression").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchCaseNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_CASE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_CASE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchCaseNode<'i> {
    pub fn case_pattern(&self) -> CasePatternNode<'i> {
        self.pair.take_tagged_one("case_pattern").unwrap()
    }
    pub fn if_guard(&self) -> IfGuardNode<'i> {
        self.pair.take_tagged_one("if_guard").unwrap()
    }
    pub fn kw_case(&self) -> KwCaseNode<'i> {
        self.pair.take_tagged_one("kw_case").unwrap()
    }
    pub fn match_statement(&self) -> Vec<MatchStatementNode<'i>> {
        self.pair.take_tagged_items("match_statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for CasePatternNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CASE_PATTERN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("standard_pattern") {
            return Ok(Self::StandardPattern(s));
        }
        if let Ok(s) = pair.take_tagged_one("namepath") {
            return Ok(Self::Namepath(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::CASE_PATTERN, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CASE_PATTERN
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::StandardPattern(s) => s.get_str(),
            Self::Namepath(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::StandardPattern(s) => s.get_range(),
            Self::Namepath(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchWhenNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_WHEN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_WHEN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchWhenNode<'i> {
    pub fn inline_expression(&self) -> InlineExpressionNode<'i> {
        self.pair.take_tagged_one("inline_expression").unwrap()
    }
    pub fn kw_when(&self) -> KwWhenNode<'i> {
        self.pair.take_tagged_one("kw_when").unwrap()
    }
    pub fn match_statement(&self) -> Vec<MatchStatementNode<'i>> {
        self.pair.take_tagged_items("match_statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchElseNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_ELSE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_ELSE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchElseNode<'i> {
    pub fn kw_else(&self) -> KwElseNode<'i> {
        self.pair.take_tagged_one("kw_else").unwrap()
    }
    pub fn match_statement(&self) -> Vec<MatchStatementNode<'i>> {
        self.pair.take_tagged_items("match_statement").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MATCH_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MATCH_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchStatementNode<'i> {
    pub fn statement(&self) -> StatementNode<'i> {
        self.pair.take_tagged_one("statement").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwMatchNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_MATCH)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("match") {
            return Ok(Self::Match(s));
        }
        if let Ok(s) = pair.take_tagged_one("catch") {
            return Ok(Self::Catch(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::KW_MATCH, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_MATCH
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Match(s) => s.get_str(),
            Self::Catch(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Match(s) => s.get_range(),
            Self::Catch(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for BindLNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BIND_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::BIND_L
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BIND_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::BIND_R
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
impl<'i> YggdrasilNode<'i> for DotMatchCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DOT_MATCH_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DOT_MATCH_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DotMatchCallNode<'i> {
    pub fn bind_r(&self) -> Option<BindRNode<'i>> {
        self.pair.take_tagged_option("bind_r")
    }
    pub fn identifier(&self) -> Option<IdentifierNode<'i>> {
        self.pair.take_tagged_option("identifier")
    }
    pub fn kw_match(&self) -> KwMatchNode<'i> {
        self.pair.take_tagged_one("kw_match").unwrap()
    }
    pub fn match_block(&self) -> MatchBlockNode<'i> {
        self.pair.take_tagged_one("match_block").unwrap()
    }
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainExpressionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_EXPRESSION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_EXPRESSION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainExpressionNode<'i> {
    pub fn main_infix(&self) -> Vec<MainInfixNode<'i>> {
        self.pair.take_tagged_items("main_infix").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn main_term(&self) -> Vec<MainTermNode<'i>> {
        self.pair.take_tagged_items("main_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_TERM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainTermNode<'i> {
    pub fn main_factor(&self) -> MainFactorNode<'i> {
        self.pair.take_tagged_one("main_factor").unwrap()
    }
    pub fn main_prefix(&self) -> Vec<MainPrefixNode<'i>> {
        self.pair.take_tagged_items("main_prefix").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn main_suffix_term(&self) -> Vec<MainSuffixTermNode<'i>> {
        self.pair.take_tagged_items("main_suffix_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainFactorNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_FACTOR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("switch_statement") {
            return Ok(Self::SwitchStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("try_statement") {
            return Ok(Self::TryStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("match_expression") {
            return Ok(Self::MatchExpression(s));
        }
        if let Ok(s) = pair.take_tagged_one("define_lambda") {
            return Ok(Self::DefineLambda(s));
        }
        if let Ok(s) = pair.take_tagged_one("object_statement") {
            return Ok(Self::ObjectStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("new_statement") {
            return Ok(Self::NewStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("group_factor") {
            return Ok(Self::GroupFactor(s));
        }
        if let Ok(s) = pair.take_tagged_one("leading") {
            return Ok(Self::Leading(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MAIN_FACTOR, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_FACTOR
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::SwitchStatement(s) => s.get_str(),
            Self::TryStatement(s) => s.get_str(),
            Self::MatchExpression(s) => s.get_str(),
            Self::DefineLambda(s) => s.get_str(),
            Self::ObjectStatement(s) => s.get_str(),
            Self::NewStatement(s) => s.get_str(),
            Self::GroupFactor(s) => s.get_str(),
            Self::Leading(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::SwitchStatement(s) => s.get_range(),
            Self::TryStatement(s) => s.get_range(),
            Self::MatchExpression(s) => s.get_range(),
            Self::DefineLambda(s) => s.get_range(),
            Self::ObjectStatement(s) => s.get_range(),
            Self::NewStatement(s) => s.get_range(),
            Self::GroupFactor(s) => s.get_range(),
            Self::Leading(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for GroupFactorNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GROUP_FACTOR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GROUP_FACTOR
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> GroupFactorNode<'i> {
    pub fn main_expression(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("main_expression").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for LeadingNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LEADING)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("procedural_call") {
            return Ok(Self::ProceduralCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("tuple_literal_strict") {
            return Ok(Self::TupleLiteralStrict(s));
        }
        if let Ok(s) = pair.take_tagged_one("range_literal") {
            return Ok(Self::RangeLiteral(s));
        }
        if let Ok(s) = pair.take_tagged_one("text_literal") {
            return Ok(Self::TextLiteral(s));
        }
        if let Ok(s) = pair.take_tagged_one("slot") {
            return Ok(Self::Slot(s));
        }
        if let Ok(s) = pair.take_tagged_one("number") {
            return Ok(Self::Number(s));
        }
        if let Ok(s) = pair.take_tagged_one("special") {
            return Ok(Self::Special(s));
        }
        if let Ok(s) = pair.take_tagged_one("namepath") {
            return Ok(Self::Namepath(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::LEADING, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::LEADING
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ProceduralCall(s) => s.get_str(),
            Self::TupleLiteralStrict(s) => s.get_str(),
            Self::RangeLiteral(s) => s.get_str(),
            Self::TextLiteral(s) => s.get_str(),
            Self::Slot(s) => s.get_str(),
            Self::Number(s) => s.get_str(),
            Self::Special(s) => s.get_str(),
            Self::Namepath(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::TupleLiteralStrict(s) => s.get_range(),
            Self::RangeLiteral(s) => s.get_range(),
            Self::TextLiteral(s) => s.get_range(),
            Self::Slot(s) => s.get_range(),
            Self::Number(s) => s.get_range(),
            Self::Special(s) => s.get_range(),
            Self::Namepath(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainSuffixTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_SUFFIX_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("main_suffix_term_0") {
            return Ok(Self::MainSuffixTerm0(s));
        }
        if let Ok(s) = pair.take_tagged_one("main_suffix_term_1") {
            return Ok(Self::MainSuffixTerm1(s));
        }
        if let Ok(s) = pair.take_tagged_one("tuple_call") {
            return Ok(Self::TupleCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("inline_suffix_term") {
            return Ok(Self::InlineSuffixTerm(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MAIN_SUFFIX_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_SUFFIX_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::MainSuffixTerm0(s) => s.get_str(),
            Self::MainSuffixTerm1(s) => s.get_str(),
            Self::TupleCall(s) => s.get_str(),
            Self::InlineSuffixTerm(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::MainSuffixTerm0(s) => s.get_range(),
            Self::MainSuffixTerm1(s) => s.get_range(),
            Self::TupleCall(s) => s.get_range(),
            Self::InlineSuffixTerm(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainPrefixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_PREFIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_PREFIX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainPrefixNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypePrefixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_PREFIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_PREFIX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypePrefixNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainInfixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_INFIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_INFIX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainInfixNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeInfixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_INFIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_INFIX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeInfixNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainSuffixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_SUFFIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_SUFFIX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainSuffixNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeSuffixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_SUFFIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_SUFFIX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeSuffixNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InlineExpressionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INLINE_EXPRESSION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INLINE_EXPRESSION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> InlineExpressionNode<'i> {
    pub fn inline_term(&self) -> Vec<InlineTermNode<'i>> {
        self.pair.take_tagged_items("inline_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn main_infix(&self) -> Vec<MainInfixNode<'i>> {
        self.pair.take_tagged_items("main_infix").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InlineTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INLINE_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INLINE_TERM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> InlineTermNode<'i> {
    pub fn inline_suffix_term(&self) -> Vec<InlineSuffixTermNode<'i>> {
        self.pair.take_tagged_items("inline_suffix_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn main_factor(&self) -> MainFactorNode<'i> {
        self.pair.take_tagged_one("main_factor").unwrap()
    }
    pub fn main_prefix(&self) -> Vec<MainPrefixNode<'i>> {
        self.pair.take_tagged_items("main_prefix").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InlineSuffixTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INLINE_SUFFIX_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("main_suffix") {
            return Ok(Self::MainSuffix(s));
        }
        if let Ok(s) = pair.take_tagged_one("dot_call") {
            return Ok(Self::DotCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("inline_tuple_call") {
            return Ok(Self::InlineTupleCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("range_call") {
            return Ok(Self::RangeCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("generic_call") {
            return Ok(Self::GenericCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::INLINE_SUFFIX_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INLINE_SUFFIX_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::MainSuffix(s) => s.get_str(),
            Self::DotCall(s) => s.get_str(),
            Self::InlineTupleCall(s) => s.get_str(),
            Self::RangeCall(s) => s.get_str(),
            Self::GenericCall(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::MainSuffix(s) => s.get_range(),
            Self::DotCall(s) => s.get_range(),
            Self::InlineTupleCall(s) => s.get_range(),
            Self::RangeCall(s) => s.get_range(),
            Self::GenericCall(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeExpressionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_EXPRESSION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_EXPRESSION
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeExpressionNode<'i> {
    pub fn type_infix(&self) -> Vec<TypeInfixNode<'i>> {
        self.pair.take_tagged_items("type_infix").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn type_term(&self) -> Vec<TypeTermNode<'i>> {
        self.pair.take_tagged_items("type_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_TERM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeTermNode<'i> {
    pub fn main_factor(&self) -> MainFactorNode<'i> {
        self.pair.take_tagged_one("main_factor").unwrap()
    }
    pub fn type_prefix(&self) -> Vec<TypePrefixNode<'i>> {
        self.pair.take_tagged_items("type_prefix").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn type_suffix_term(&self) -> Vec<TypeSuffixTermNode<'i>> {
        self.pair.take_tagged_items("type_suffix_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeFactorNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_FACTOR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("type_expression") {
            return Ok(Self::TypeExpression(s));
        }
        if let Ok(s) = pair.take_tagged_one("leading") {
            return Ok(Self::Leading(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TYPE_FACTOR, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_FACTOR
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::TypeExpression(s) => s.get_str(),
            Self::Leading(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::TypeExpression(s) => s.get_range(),
            Self::Leading(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeSuffixTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_SUFFIX_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("generic_hide") {
            return Ok(Self::GenericHide(s));
        }
        if let Ok(s) = pair.take_tagged_one("type_suffix") {
            return Ok(Self::TypeSuffix(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TYPE_SUFFIX_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_SUFFIX_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::GenericHide(s) => s.get_str(),
            Self::TypeSuffix(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::GenericHide(s) => s.get_range(),
            Self::TypeSuffix(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TryStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TRY_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TRY_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TryStatementNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn kw_try(&self) -> KwTryNode<'i> {
        self.pair.take_tagged_one("kw_try").unwrap()
    }
    pub fn type_expression(&self) -> Option<TypeExpressionNode<'i>> {
        self.pair.take_tagged_option("type_expression")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NewStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NEW_STATEMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NEW_STATEMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> NewStatementNode<'i> {
    pub fn generic_hide(&self) -> Option<GenericHideNode<'i>> {
        self.pair.take_tagged_option("generic_hide")
    }
    pub fn kw_new(&self) -> KwNewNode<'i> {
        self.pair.take_tagged_one("kw_new").unwrap()
    }
    pub fn modifier_ahead(&self) -> Vec<ModifierAheadNode<'i>> {
        self.pair.take_tagged_items("modifier_ahead").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn namepath(&self) -> NamepathNode<'i> {
        self.pair.take_tagged_one("namepath").unwrap()
    }
    pub fn new_block(&self) -> Option<NewBlockNode<'i>> {
        self.pair.take_tagged_option("new_block")
    }
    pub fn tuple_literal(&self) -> Option<TupleLiteralNode<'i>> {
        self.pair.take_tagged_option("tuple_literal")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NewBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NEW_BLOCK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NEW_BLOCK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> NewBlockNode<'i> {
    pub fn eos_free(&self) -> Vec<EosFreeNode<'i>> {
        self.pair.take_tagged_items("eos_free").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn new_pair(&self) -> Vec<NewPairNode<'i>> {
        self.pair.take_tagged_items("new_pair").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NewPairNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NEW_PAIR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NEW_PAIR
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> NewPairNode<'i> {
    pub fn colon(&self) -> Option<ColonNode<'i>> {
        self.pair.take_tagged_option("colon")
    }
    pub fn main_expression(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("main_expression").unwrap()
    }
    pub fn new_pair_key(&self) -> Option<NewPairKeyNode<'i>> {
        self.pair.take_tagged_option("new_pair_key")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NewPairKeyNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NEW_PAIR_KEY)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("identifier") {
            return Ok(Self::Identifier(s));
        }
        if let Ok(s) = pair.take_tagged_one("text_raw") {
            return Ok(Self::TextRaw(s));
        }
        if let Ok(s) = pair.take_tagged_one("range_literal") {
            return Ok(Self::RangeLiteral(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::NEW_PAIR_KEY, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NEW_PAIR_KEY
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Identifier(s) => s.get_str(),
            Self::TextRaw(s) => s.get_str(),
            Self::RangeLiteral(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Identifier(s) => s.get_range(),
            Self::TextRaw(s) => s.get_range(),
            Self::RangeLiteral(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DotCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DOT_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DOT_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DotCallNode<'i> {
    pub fn dot_call_item(&self) -> DotCallItemNode<'i> {
        self.pair.take_tagged_one("dot_call_item").unwrap()
    }
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DotCallItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DOT_CALL_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("namepath") {
            return Ok(Self::Namepath(s));
        }
        if let Ok(s) = pair.take_tagged_one("integer") {
            return Ok(Self::Integer(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::DOT_CALL_ITEM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DOT_CALL_ITEM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Namepath(s) => s.get_str(),
            Self::Integer(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Namepath(s) => s.get_range(),
            Self::Integer(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DotClosureCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DOT_CLOSURE_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DOT_CLOSURE_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DotClosureCallNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InlineTupleCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INLINE_TUPLE_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INLINE_TUPLE_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> InlineTupleCallNode<'i> {
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
    pub fn tuple_literal(&self) -> TupleLiteralNode<'i> {
        self.pair.take_tagged_one("tuple_literal").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TupleCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TupleCallNode<'i> {
    pub fn continuation(&self) -> Option<ContinuationNode<'i>> {
        self.pair.take_tagged_option("continuation")
    }
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
    pub fn tuple_literal(&self) -> Option<TupleLiteralNode<'i>> {
        self.pair.take_tagged_option("tuple_literal")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TupleLiteralNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_LITERAL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_LITERAL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TupleLiteralNode<'i> {
    pub fn tuple_terms(&self) -> TupleTermsNode<'i> {
        self.pair.take_tagged_one("tuple_terms").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TupleLiteralStrictNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_LITERAL_STRICT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_LITERAL_STRICT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TupleLiteralStrictNode<'i> {
    pub fn tuple_pair(&self) -> Vec<TuplePairNode<'i>> {
        self.pair.take_tagged_items("tuple_pair").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TupleTermsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_TERMS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_TERMS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TupleTermsNode<'i> {
    pub fn tuple_pair(&self) -> Vec<TuplePairNode<'i>> {
        self.pair.take_tagged_items("tuple_pair").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TuplePairNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_PAIR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_PAIR
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TuplePairNode<'i> {
    pub fn colon(&self) -> Option<ColonNode<'i>> {
        self.pair.take_tagged_option("colon")
    }
    pub fn main_expression(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("main_expression").unwrap()
    }
    pub fn tuple_key(&self) -> Option<TupleKeyNode<'i>> {
        self.pair.take_tagged_option("tuple_key")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TupleKeyNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TUPLE_KEY)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("identifier") {
            return Ok(Self::Identifier(s));
        }
        if let Ok(s) = pair.take_tagged_one("integer") {
            return Ok(Self::Integer(s));
        }
        if let Ok(s) = pair.take_tagged_one("text_raw") {
            return Ok(Self::TextRaw(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TUPLE_KEY, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TUPLE_KEY
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Identifier(s) => s.get_str(),
            Self::Integer(s) => s.get_str(),
            Self::TextRaw(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Identifier(s) => s.get_range(),
            Self::Integer(s) => s.get_range(),
            Self::TextRaw(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for RangeCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RANGE_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RANGE_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> RangeCallNode<'i> {
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
    pub fn range_literal(&self) -> RangeLiteralNode<'i> {
        self.pair.take_tagged_one("range_literal").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for RangeLiteralNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RANGE_LITERAL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("range_literal_index_0") {
            return Ok(Self::RangeLiteralIndex0(s));
        }
        if let Ok(s) = pair.take_tagged_one("range_literal_index_1") {
            return Ok(Self::RangeLiteralIndex1(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::RANGE_LITERAL, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RANGE_LITERAL
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::RangeLiteralIndex0(s) => s.get_str(),
            Self::RangeLiteralIndex1(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::RangeLiteralIndex0(s) => s.get_range(),
            Self::RangeLiteralIndex1(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for RangeLiteralIndex0Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RANGE_LITERAL_INDEX0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RANGE_LITERAL_INDEX0
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> RangeLiteralIndex0Node<'i> {
    pub fn subscript_axis(&self) -> Vec<SubscriptAxisNode<'i>> {
        self.pair.take_tagged_items("subscript_axis").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for RangeLiteralIndex1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RANGE_LITERAL_INDEX1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RANGE_LITERAL_INDEX1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> RangeLiteralIndex1Node<'i> {
    pub fn subscript_axis(&self) -> Vec<SubscriptAxisNode<'i>> {
        self.pair.take_tagged_items("subscript_axis").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SubscriptAxisNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SUBSCRIPT_AXIS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("subscript_range") {
            return Ok(Self::SubscriptRange(s));
        }
        if let Ok(s) = pair.take_tagged_one("subscript_only") {
            return Ok(Self::SubscriptOnly(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::SUBSCRIPT_AXIS, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SUBSCRIPT_AXIS
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::SubscriptRange(s) => s.get_str(),
            Self::SubscriptOnly(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::SubscriptRange(s) => s.get_range(),
            Self::SubscriptOnly(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SubscriptOnlyNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SUBSCRIPT_ONLY)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SUBSCRIPT_ONLY
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> SubscriptOnlyNode<'i> {
    pub fn index(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("index").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SubscriptRangeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SUBSCRIPT_RANGE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SUBSCRIPT_RANGE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> SubscriptRangeNode<'i> {
    pub fn head(&self) -> Option<MainExpressionNode<'i>> {
        self.pair.take_tagged_option("head")
    }
    pub fn step(&self) -> Option<MainExpressionNode<'i>> {
        self.pair.take_tagged_option("step")
    }
    pub fn tail(&self) -> Option<MainExpressionNode<'i>> {
        self.pair.take_tagged_option("tail")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for RangeOmitNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RANGE_OMIT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RANGE_OMIT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> RangeOmitNode<'i> {
    pub fn colon(&self) -> Vec<ColonNode<'i>> {
        self.pair.take_tagged_items("colon").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn proportion(&self) -> Option<ProportionNode<'i>> {
        self.pair.take_tagged_option("proportion")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineGenericNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DEFINE_GENERIC)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DEFINE_GENERIC
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DefineGenericNode<'i> {
    pub fn generic_parameter(&self) -> GenericParameterNode<'i> {
        self.pair.take_tagged_one("generic_parameter").unwrap()
    }
    pub fn proportion(&self) -> Option<ProportionNode<'i>> {
        self.pair.take_tagged_option("proportion")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for GenericParameterNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GENERIC_PARAMETER)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GENERIC_PARAMETER
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> GenericParameterNode<'i> {
    pub fn generic_parameter_pair(&self) -> Vec<GenericParameterPairNode<'i>> {
        self.pair.take_tagged_items("generic_parameter_pair").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for GenericParameterPairNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GENERIC_PARAMETER_PAIR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GENERIC_PARAMETER_PAIR
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> GenericParameterPairNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
    pub fn bound(&self) -> Option<TypeExpressionNode<'i>> {
        self.pair.take_tagged_option("bound")
    }
    pub fn default(&self) -> Option<TypeExpressionNode<'i>> {
        self.pair.take_tagged_option("default")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for GenericCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GENERIC_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GENERIC_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> GenericCallNode<'i> {
    pub fn generic_terms(&self) -> GenericTermsNode<'i> {
        self.pair.take_tagged_one("generic_terms").unwrap()
    }
    pub fn namepath(&self) -> Option<NamepathNode<'i>> {
        self.pair.take_tagged_option("namepath")
    }
    pub fn op_and_then(&self) -> Option<OpAndThenNode<'i>> {
        self.pair.take_tagged_option("op_and_then")
    }
    pub fn proportion(&self) -> Vec<ProportionNode<'i>> {
        self.pair.take_tagged_items("proportion").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for GenericHideNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GENERIC_HIDE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GENERIC_HIDE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> GenericHideNode<'i> {
    pub fn generic_terms(&self) -> GenericTermsNode<'i> {
        self.pair.take_tagged_one("generic_terms").unwrap()
    }
    pub fn proportion(&self) -> Option<ProportionNode<'i>> {
        self.pair.take_tagged_option("proportion")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for GenericTermsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GENERIC_TERMS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GENERIC_TERMS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> GenericTermsNode<'i> {
    pub fn generic_pair(&self) -> Vec<GenericPairNode<'i>> {
        self.pair.take_tagged_items("generic_pair").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for GenericPairNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GENERIC_PAIR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GENERIC_PAIR
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> GenericPairNode<'i> {
    pub fn colon(&self) -> Option<ColonNode<'i>> {
        self.pair.take_tagged_option("colon")
    }
    pub fn identifier(&self) -> Option<IdentifierNode<'i>> {
        self.pair.take_tagged_option("identifier")
    }
    pub fn type_expression(&self) -> TypeExpressionNode<'i> {
        self.pair.take_tagged_one("type_expression").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AnnotationHeadNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ANNOTATION_HEAD)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ANNOTATION_HEAD
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AnnotationHeadNode<'i> {
    pub fn annotation_term(&self) -> Vec<AnnotationTermNode<'i>> {
        self.pair.take_tagged_items("annotation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn modifier_call(&self) -> Vec<ModifierCallNode<'i>> {
        self.pair.take_tagged_items("modifier_call").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AnnotationMixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ANNOTATION_MIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ANNOTATION_MIX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AnnotationMixNode<'i> {
    pub fn annotation_term_mix(&self) -> Vec<AnnotationTermMixNode<'i>> {
        self.pair.take_tagged_items("annotation_term_mix").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn modifier_ahead(&self) -> Vec<ModifierAheadNode<'i>> {
        self.pair.take_tagged_items("modifier_ahead").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AnnotationTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ANNOTATION_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("attribute_below_call") {
            return Ok(Self::AttributeBelowCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ANNOTATION_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ANNOTATION_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::AttributeBelowCall(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::AttributeBelowCall(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AnnotationTermMixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ANNOTATION_TERM_MIX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("attribute_below_call") {
            return Ok(Self::AttributeBelowCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("procedural_call") {
            return Ok(Self::ProceduralCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ANNOTATION_TERM_MIX, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ANNOTATION_TERM_MIX
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::AttributeBelowCall(s) => s.get_str(),
            Self::ProceduralCall(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::AttributeBelowCall(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AttributeBelowCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ATTRIBUTE_BELOW_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ATTRIBUTE_BELOW_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AttributeBelowCallNode<'i> {
    pub fn attribute_below_mark(&self) -> AttributeBelowMarkNode<'i> {
        self.pair.take_tagged_one("attribute_below_mark").unwrap()
    }
    pub fn attribute_item(&self) -> Vec<AttributeItemNode<'i>> {
        self.pair.take_tagged_items("attribute_item").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AttributeBelowMarkNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ATTRIBUTE_BELOW_MARK)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ATTRIBUTE_BELOW_MARK
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AttributeBelowMarkNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AttributeItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ATTRIBUTE_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ATTRIBUTE_ITEM
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AttributeItemNode<'i> {
    pub fn continuation(&self) -> Option<ContinuationNode<'i>> {
        self.pair.take_tagged_option("continuation")
    }
    pub fn identifier(&self) -> Vec<IdentifierNode<'i>> {
        self.pair.take_tagged_items("identifier").collect::<Result<Vec<_>, _>>().unwrap()
    }
    pub fn namepath(&self) -> NamepathNode<'i> {
        self.pair.take_tagged_one("namepath").unwrap()
    }
    pub fn tuple_literal(&self) -> Option<TupleLiteralNode<'i>> {
        self.pair.take_tagged_option("tuple_literal")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AttributeNameNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ATTRIBUTE_NAME)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ATTRIBUTE_NAME
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AttributeNameNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ProceduralCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PROCEDURAL_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PROCEDURAL_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ProceduralCallNode<'i> {
    pub fn continuation(&self) -> Option<ContinuationNode<'i>> {
        self.pair.take_tagged_option("continuation")
    }
    pub fn namepath(&self) -> NamepathNode<'i> {
        self.pair.take_tagged_one("namepath").unwrap()
    }
    pub fn tuple_literal(&self) -> Option<TupleLiteralNode<'i>> {
        self.pair.take_tagged_option("tuple_literal")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ProceduralNameNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PROCEDURAL_NAME)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PROCEDURAL_NAME
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ProceduralNameNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextLiteralNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_LITERAL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_LITERAL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextLiteralNode<'i> {
    pub fn identifier(&self) -> Option<IdentifierNode<'i>> {
        self.pair.take_tagged_option("identifier")
    }
    pub fn text_raw(&self) -> TextRawNode<'i> {
        self.pair.take_tagged_one("text_raw").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextRawNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_RAW)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_RAW
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextRawNode<'i> {
    pub fn text_content_1(&self) -> Option<TextContent1Node<'i>> {
        self.pair.take_tagged_option("text_content_1")
    }
    pub fn text_content_2(&self) -> Option<TextContent2Node<'i>> {
        self.pair.take_tagged_option("text_content_2")
    }
    pub fn text_content_3(&self) -> Option<TextContent3Node<'i>> {
        self.pair.take_tagged_option("text_content_3")
    }
    pub fn text_content_4(&self) -> Option<TextContent4Node<'i>> {
        self.pair.take_tagged_option("text_content_4")
    }
    pub fn text_content_5(&self) -> Option<TextContent5Node<'i>> {
        self.pair.take_tagged_option("text_content_5")
    }
    pub fn text_content_6(&self) -> Option<TextContent6Node<'i>> {
        self.pair.take_tagged_option("text_content_6")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextLNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_L
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextLNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextRNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_R
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextRNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextXNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_X)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_X
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextXNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextContent1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_CONTENT1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextContent1Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextContent2Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT2)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_CONTENT2
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextContent2Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextContent3Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT3)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_CONTENT3
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextContent3Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextContent4Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT4)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_CONTENT4
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextContent4Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextContent5Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT5)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_CONTENT5
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextContent5Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TextContent6Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT6)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEXT_CONTENT6
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TextContent6Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ModifierCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MODIFIER_CALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MODIFIER_CALL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ModifierCallNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ModifierAheadNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MODIFIER_AHEAD)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MODIFIER_AHEAD
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ModifierAheadNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KeywordsStopNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KEYWORDS_STOP)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KEYWORDS_STOP
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KeywordsStopNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IdentifierStopNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IDENTIFIER_STOP)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IDENTIFIER_STOP
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> IdentifierStopNode<'i> {
    pub fn identifier(&self) -> IdentifierNode<'i> {
        self.pair.take_tagged_one("identifier").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SlotNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SLOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SLOT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> SlotNode<'i> {
    pub fn op_slot(&self) -> OpSlotNode<'i> {
        self.pair.take_tagged_one("op_slot").unwrap()
    }
    pub fn slot_item(&self) -> Option<SlotItemNode<'i>> {
        self.pair.take_tagged_option("slot_item")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SlotItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SLOT_ITEM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("integer") {
            return Ok(Self::Integer(s));
        }
        if let Ok(s) = pair.take_tagged_one("identifier") {
            return Ok(Self::Identifier(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::SLOT_ITEM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SLOT_ITEM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Integer(s) => s.get_str(),
            Self::Identifier(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Integer(s) => s.get_range(),
            Self::Identifier(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NamepathFreeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NAMEPATH_FREE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NAMEPATH_FREE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> NamepathFreeNode<'i> {
    pub fn identifier(&self) -> Vec<IdentifierNode<'i>> {
        self.pair.take_tagged_items("identifier").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NamepathNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NAMEPATH)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NAMEPATH
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> NamepathNode<'i> {
    pub fn identifier(&self) -> Vec<IdentifierNode<'i>> {
        self.pair.take_tagged_items("identifier").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IdentifierNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IDENTIFIER)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("identifier_bare") {
            return Ok(Self::IdentifierBare(s));
        }
        if let Ok(s) = pair.take_tagged_one("identifier_raw") {
            return Ok(Self::IdentifierRaw(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::IDENTIFIER, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IDENTIFIER
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::IdentifierBare(s) => s.get_str(),
            Self::IdentifierRaw(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::IdentifierBare(s) => s.get_range(),
            Self::IdentifierRaw(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IdentifierBareNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IDENTIFIER_BARE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IDENTIFIER_BARE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> IdentifierBareNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IdentifierRawNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IDENTIFIER_RAW)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IDENTIFIER_RAW
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> IdentifierRawNode<'i> {
    pub fn identifier_raw_text(&self) -> IdentifierRawTextNode<'i> {
        self.pair.take_tagged_one("identifier_raw_text").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IdentifierRawTextNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IDENTIFIER_RAW_TEXT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IDENTIFIER_RAW_TEXT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> IdentifierRawTextNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SpecialNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SPECIAL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SPECIAL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> SpecialNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NumberNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NUMBER)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("decimal_x") {
            return Ok(Self::DecimalX(s));
        }
        if let Ok(s) = pair.take_tagged_one("decimal") {
            return Ok(Self::Decimal(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::NUMBER, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NUMBER
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::DecimalX(s) => s.get_str(),
            Self::Decimal(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DecimalX(s) => s.get_range(),
            Self::Decimal(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SignNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SIGN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("positive") {
            return Ok(Self::Positive(s));
        }
        if let Ok(s) = pair.take_tagged_one("netative") {
            return Ok(Self::Netative(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::SIGN, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SIGN
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::Positive(s) => s.get_str(),
            Self::Netative(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Positive(s) => s.get_range(),
            Self::Netative(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IntegerNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INTEGER)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INTEGER
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> IntegerNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DigitsXNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DIGITS_X)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DIGITS_X
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DigitsXNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DecimalNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DECIMAL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DECIMAL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DecimalNode<'i> {
    pub fn dot(&self) -> Option<DotNode<'i>> {
        self.pair.take_tagged_option("dot")
    }
    pub fn lhs(&self) -> IntegerNode<'i> {
        self.pair.take_tagged_one("lhs").unwrap()
    }
    pub fn rhs(&self) -> Option<IntegerNode<'i>> {
        self.pair.take_tagged_option("rhs")
    }
    pub fn shift(&self) -> Option<IntegerNode<'i>> {
        self.pair.take_tagged_option("shift")
    }
    pub fn sign(&self) -> Option<SignNode<'i>> {
        self.pair.take_tagged_option("sign")
    }
    pub fn unit(&self) -> Option<IdentifierNode<'i>> {
        self.pair.take_tagged_option("unit")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DecimalXNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DECIMAL_X)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DECIMAL_X
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DecimalXNode<'i> {
    pub fn base(&self) -> IntegerNode<'i> {
        self.pair.take_tagged_one("base").unwrap()
    }
    pub fn dot(&self) -> Option<DotNode<'i>> {
        self.pair.take_tagged_option("dot")
    }
    pub fn lhs(&self) -> DigitsXNode<'i> {
        self.pair.take_tagged_one("lhs").unwrap()
    }
    pub fn rhs(&self) -> Option<DigitsXNode<'i>> {
        self.pair.take_tagged_option("rhs")
    }
    pub fn shift(&self) -> Option<IntegerNode<'i>> {
        self.pair.take_tagged_option("shift")
    }
    pub fn sign(&self) -> Option<SignNode<'i>> {
        self.pair.take_tagged_option("sign")
    }
    pub fn unit(&self) -> Option<IdentifierNode<'i>> {
        self.pair.take_tagged_option("unit")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ProportionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PROPORTION)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PROPORTION
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NS_CONCAT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NS_CONCAT
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::COLON)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::COLON
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
impl<'i> YggdrasilNode<'i> for EqualNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EQUAL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EQUAL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> EqualNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for Arrow1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ARROW1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ARROW1
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::COMMA)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::COMMA
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DOT
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_SLOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_SLOT
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OFFSET_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OFFSET_L
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OFFSET_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OFFSET_R
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
impl<'i> YggdrasilNode<'i> for Proportion2Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PROPORTION2)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PROPORTION2
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> Proportion2Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpImportAllNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_IMPORT_ALL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_IMPORT_ALL
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_AND_THEN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_AND_THEN
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_BIND)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_BIND
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
impl<'i> YggdrasilNode<'i> for KwControlNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CONTROL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_CONTROL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwControlNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwNamespaceNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_NAMESPACE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_NAMESPACE
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IMPORT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_IMPORT
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CONSTRAINT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_CONSTRAINT
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WHERE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_WHERE
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IMPLEMENTS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_IMPLEMENTS
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
impl<'i> YggdrasilNode<'i> for KwTraitNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_TRAIT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_TRAIT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwTraitNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwExtendsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_EXTENDS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_EXTENDS
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_INHERITS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_INHERITS
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
impl<'i> YggdrasilNode<'i> for KwClassNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CLASS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_CLASS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwClassNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwStructureNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_STRUCTURE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_STRUCTURE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwStructureNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwWidgetNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WIDGET)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_WIDGET
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwWidgetNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwNeuralNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_NEURAL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_NEURAL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwNeuralNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwEnumerateNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_ENUMERATE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_ENUMERATE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwEnumerateNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwFlagsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_FLAGS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_FLAGS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwFlagsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwLoopNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_LOOP)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_LOOP
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwLoopNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwEachNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_EACH)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_EACH
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwEachNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwWhileNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WHILE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_WHILE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwWhileNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwUntilNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_UNTIL)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_UNTIL
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwUntilNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwLetNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_LET)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_LET
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_NEW)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_NEW
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_OBJECT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_OBJECT
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_LAMBDA)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_LAMBDA
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IF)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_IF
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_SWITCH)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_SWITCH
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_TRY)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_TRY
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_TYPE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_TYPE
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CASE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_CASE
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WHEN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_WHEN
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_ELSE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_ELSE
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_NOT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_NOT
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_IN
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_IS
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_AS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_AS
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
impl<'i> YggdrasilNode<'i> for KwEndNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_END)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_END
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
impl<'i> YggdrasilNode<'i> for ShebangNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SHEBANG)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SHEBANG
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ShebangNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for WhiteSpaceNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WHITE_SPACE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::WHITE_SPACE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> WhiteSpaceNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SkipSpaceNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SKIP_SPACE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SKIP_SPACE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> SkipSpaceNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for CommentNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::COMMENT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::COMMENT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> CommentNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringInterpolationsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_INTERPOLATIONS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_INTERPOLATIONS
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> StringInterpolationsNode<'i> {
    pub fn string_interpolation_term(&self) -> Vec<StringInterpolationTermNode<'i>> {
        self.pair.take_tagged_items("string_interpolation_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringInterpolationTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_INTERPOLATION_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("escape_unicode") {
            return Ok(Self::EscapeUnicode(s));
        }
        if let Ok(s) = pair.take_tagged_one("escape_character") {
            return Ok(Self::EscapeCharacter(s));
        }
        if let Ok(s) = pair.take_tagged_one("string_interpolation_simple") {
            return Ok(Self::StringInterpolationSimple(s));
        }
        if let Ok(s) = pair.take_tagged_one("string_interpolation_complex") {
            return Ok(Self::StringInterpolationComplex(s));
        }
        if let Ok(s) = pair.take_tagged_one("string_interpolation_text") {
            return Ok(Self::StringInterpolationText(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::STRING_INTERPOLATION_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_INTERPOLATION_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::EscapeUnicode(s) => s.get_str(),
            Self::EscapeCharacter(s) => s.get_str(),
            Self::StringInterpolationSimple(s) => s.get_str(),
            Self::StringInterpolationComplex(s) => s.get_str(),
            Self::StringInterpolationText(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::EscapeUnicode(s) => s.get_range(),
            Self::EscapeCharacter(s) => s.get_range(),
            Self::StringInterpolationSimple(s) => s.get_range(),
            Self::StringInterpolationComplex(s) => s.get_range(),
            Self::StringInterpolationText(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for EscapeCharacterNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ESCAPE_CHARACTER)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ESCAPE_CHARACTER
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> EscapeCharacterNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for EscapeUnicodeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ESCAPE_UNICODE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ESCAPE_UNICODE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> EscapeUnicodeNode<'i> {
    pub fn code(&self) -> EscapeUnicodeCodeNode<'i> {
        self.pair.take_tagged_one("code").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for EscapeUnicodeCodeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ESCAPE_UNICODE_CODE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ESCAPE_UNICODE_CODE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> EscapeUnicodeCodeNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringInterpolationSimpleNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_INTERPOLATION_SIMPLE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_INTERPOLATION_SIMPLE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> StringInterpolationSimpleNode<'i> {
    pub fn main_expression(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("main_expression").unwrap()
    }
    pub fn string_formatter(&self) -> Option<StringFormatterNode<'i>> {
        self.pair.take_tagged_option("string_formatter")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringInterpolationTextNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_INTERPOLATION_TEXT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_INTERPOLATION_TEXT
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> StringInterpolationTextNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringFormatterNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_FORMATTER)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_FORMATTER
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> StringFormatterNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringInterpolationComplexNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_INTERPOLATION_COMPLEX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_INTERPOLATION_COMPLEX
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> StringInterpolationComplexNode<'i> {
    pub fn main_expression(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("main_expression").unwrap()
    }
    pub fn tuple_pair(&self) -> Vec<TuplePairNode<'i>> {
        self.pair.take_tagged_items("tuple_pair").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringTemplatesNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_TEMPLATES)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_TEMPLATES
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> StringTemplatesNode<'i> {
    pub fn string_template_term(&self) -> Vec<StringTemplateTermNode<'i>> {
        self.pair.take_tagged_items("string_template_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for StringTemplateTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::STRING_TEMPLATE_TERM)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("for_template") {
            return Ok(Self::ForTemplate(s));
        }
        if let Ok(s) = pair.take_tagged_one("expression_template") {
            return Ok(Self::ExpressionTemplate(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::STRING_TEMPLATE_TERM, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::STRING_TEMPLATE_TERM
    }

    fn get_str(&self) -> &'i str {
        match self {
            Self::ForTemplate(s) => s.get_str(),
            Self::ExpressionTemplate(s) => s.get_str(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ForTemplate(s) => s.get_range(),
            Self::ExpressionTemplate(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ExpressionTemplateNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EXPRESSION_TEMPLATE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EXPRESSION_TEMPLATE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ExpressionTemplateNode<'i> {
    pub fn main_expression(&self) -> MainExpressionNode<'i> {
        self.pair.take_tagged_one("main_expression").unwrap()
    }
    pub fn template_e(&self) -> TemplateENode<'i> {
        self.pair.take_tagged_one("template_e").unwrap()
    }
    pub fn template_s(&self) -> TemplateSNode<'i> {
        self.pair.take_tagged_one("template_s").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ForTemplateNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FOR_TEMPLATE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FOR_TEMPLATE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ForTemplateNode<'i> {
    pub fn for_template_begin(&self) -> ForTemplateBeginNode<'i> {
        self.pair.take_tagged_one("for_template_begin").unwrap()
    }
    pub fn for_template_else(&self) -> Option<ForTemplateElseNode<'i>> {
        self.pair.take_tagged_option("for_template_else")
    }
    pub fn for_template_end(&self) -> ForTemplateEndNode<'i> {
        self.pair.take_tagged_one("for_template_end").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ForTemplateBeginNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FOR_TEMPLATE_BEGIN)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FOR_TEMPLATE_BEGIN
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ForTemplateBeginNode<'i> {
    pub fn if_guard(&self) -> IfGuardNode<'i> {
        self.pair.take_tagged_one("if_guard").unwrap()
    }
    pub fn inline_expression(&self) -> Option<InlineExpressionNode<'i>> {
        self.pair.take_tagged_option("inline_expression")
    }
    pub fn kw_in(&self) -> KwInNode<'i> {
        self.pair.take_tagged_one("kw_in").unwrap()
    }
    pub fn kw_loop(&self) -> KwLoopNode<'i> {
        self.pair.take_tagged_one("kw_loop").unwrap()
    }
    pub fn let_pattern(&self) -> LetPatternNode<'i> {
        self.pair.take_tagged_one("let_pattern").unwrap()
    }
    pub fn template_e(&self) -> TemplateENode<'i> {
        self.pair.take_tagged_one("template_e").unwrap()
    }
    pub fn template_s(&self) -> TemplateSNode<'i> {
        self.pair.take_tagged_one("template_s").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ForTemplateElseNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FOR_TEMPLATE_ELSE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FOR_TEMPLATE_ELSE
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ForTemplateElseNode<'i> {
    pub fn kw_else(&self) -> KwElseNode<'i> {
        self.pair.take_tagged_one("kw_else").unwrap()
    }
    pub fn template_e(&self) -> TemplateENode<'i> {
        self.pair.take_tagged_one("template_e").unwrap()
    }
    pub fn template_s(&self) -> TemplateSNode<'i> {
        self.pair.take_tagged_one("template_s").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ForTemplateEndNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FOR_TEMPLATE_END)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FOR_TEMPLATE_END
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ForTemplateEndNode<'i> {
    pub fn kw_end(&self) -> KwEndNode<'i> {
        self.pair.take_tagged_one("kw_end").unwrap()
    }
    pub fn kw_loop(&self) -> Option<KwLoopNode<'i>> {
        self.pair.take_tagged_option("kw_loop")
    }
    pub fn template_e(&self) -> TemplateENode<'i> {
        self.pair.take_tagged_one("template_e").unwrap()
    }
    pub fn template_s(&self) -> TemplateSNode<'i> {
        self.pair.take_tagged_one("template_s").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TemplateSNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_S)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEMPLATE_S
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TemplateSNode<'i> {
    pub fn template_m(&self) -> Option<TemplateMNode<'i>> {
        self.pair.take_tagged_option("template_m")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TemplateENode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_E)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEMPLATE_E
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TemplateENode<'i> {
    pub fn template_m(&self) -> Option<TemplateMNode<'i>> {
        self.pair.take_tagged_option("template_m")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TemplateLNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEMPLATE_L
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEMPLATE_R
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
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_M)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TEMPLATE_M
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TemplateMNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpNamespace0Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_NAMESPACE0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_NAMESPACE0
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OpNamespace0Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpNamespace1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_NAMESPACE1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_NAMESPACE1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OpNamespace1Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OpNamespace2Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_NAMESPACE2)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OP_NAMESPACE2
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OpNamespace2Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for PatternItem1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PATTERN_ITEM1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PATTERN_ITEM1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> PatternItem1Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for PatternItem2Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PATTERN_ITEM2)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PATTERN_ITEM2
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> PatternItem2Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwMatch0Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_MATCH0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_MATCH0
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwMatch0Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwMatch1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_MATCH1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_MATCH1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwMatch1Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainSuffixTerm0Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_SUFFIX_TERM0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_SUFFIX_TERM0
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainSuffixTerm0Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainSuffixTerm1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MAIN_SUFFIX_TERM1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MAIN_SUFFIX_TERM1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainSuffixTerm1Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InlineSuffixTerm0Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INLINE_SUFFIX_TERM0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INLINE_SUFFIX_TERM0
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> InlineSuffixTerm0Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InlineSuffixTerm1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::INLINE_SUFFIX_TERM1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::INLINE_SUFFIX_TERM1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> InlineSuffixTerm1Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeFactor0Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TYPE_FACTOR0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TYPE_FACTOR0
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeFactor0Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for Sign0Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SIGN0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SIGN0
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> Sign0Node<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for Sign1Node<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SIGN1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SIGN1
    }

    fn get_str(&self) -> &'i str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> Sign1Node<'i> {}
