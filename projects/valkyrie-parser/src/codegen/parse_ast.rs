#![allow(unused_variables)]
use super::*;
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ProgramNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Program)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Program
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Statement)?)
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
        if let Ok(s) = pair.take_tagged_one("while_statement") {
            return Ok(Self::WhileStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("for_statement") {
            return Ok(Self::ForStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one("expression_root") {
            return Ok(Self::ExpressionRoot(s));
        }
        if let Ok(s) = pair.take_tagged_one("eos") {
            return Ok(Self::EOS(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Statement, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Statement
    }

    fn get_text(&self) -> &str {
        match self {
            Self::DefineNamespace(s) => s.get_text(),
            Self::DefineClass(s) => s.get_text(),
            Self::DefineUnion(s) => s.get_text(),
            Self::DefineEnumerate(s) => s.get_text(),
            Self::DefineTrait(s) => s.get_text(),
            Self::DefineExtends(s) => s.get_text(),
            Self::DefineFunction(s) => s.get_text(),
            Self::DefineVariable(s) => s.get_text(),
            Self::DefineImport(s) => s.get_text(),
            Self::ControlFlow(s) => s.get_text(),
            Self::WhileStatement(s) => s.get_text(),
            Self::ForStatement(s) => s.get_text(),
            Self::ExpressionRoot(s) => s.get_text(),
            Self::EOS(s) => s.get_text(),
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
            Self::WhileStatement(s) => s.get_range(),
            Self::ForStatement(s) => s.get_range(),
            Self::ExpressionRoot(s) => s.get_range(),
            Self::EOS(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OmitNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Omit)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Omit
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OmitNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ShowNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Show)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Show
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ShowNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for EosNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EOS)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("omit") {
            return Ok(Self::Omit(s));
        }
        if let Ok(s) = pair.take_tagged_one("show") {
            return Ok(Self::Show(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::EOS, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EOS
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Omit(s) => s.get_text(),
            Self::Show(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Omit(s) => s.get_range(),
            Self::Show(s) => s.get_range(),
        }
    }
}
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

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineNamespace)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineNamespace
    }

    fn get_text(&self) -> &str {
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
impl<'i> YggdrasilNode<'i> for MainNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Main)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Main
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MainNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TestNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Test)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Test
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TestNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for HideNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Hide)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Hide
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> HideNode<'i> {}
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

    fn get_text(&self) -> &str {
        match self {
            Self::Main(s) => s.get_text(),
            Self::Test(s) => s.get_text(),
            Self::Hide(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineImport)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineImport
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportBlock)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ImportBlock
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportTerm)?)
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
            return Ok(Self::EOS_FREE(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ImportTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ImportTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ImportAll(s) => s.get_text(),
            Self::ImportSpace(s) => s.get_text(),
            Self::ImportName(s) => s.get_text(),
            Self::EOS_FREE(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ImportAll(s) => s.get_range(),
            Self::ImportSpace(s) => s.get_range(),
            Self::ImportName(s) => s.get_range(),
            Self::EOS_FREE(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ImportAllNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportAll)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ImportAll
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportSpace)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ImportSpace
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportName)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ImportName
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportAs)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ImportAs
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportNameItem)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::ImportNameItem, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ImportNameItem
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ProceduralName(s) => s.get_text(),
            Self::AttributeName(s) => s.get_text(),
            Self::Identifier(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineConstraint)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineConstraint
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintParameters)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ConstraintParameters
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintBlock)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ConstraintBlock
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ConstraintStatement
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintImplements)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ConstraintImplements
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhereBlock)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::WhereBlock
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhereBound)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::WhereBound
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineClass)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineClass
    }

    fn get_text(&self) -> &str {
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
    pub fn kw_class(&self) -> KwClassNode<'i> {
        self.pair.take_tagged_one("kw_class").unwrap()
    }
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ClassBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ClassBlock)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ClassBlock
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ClassTerm)?)
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
            return Ok(Self::EOS_FREE(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ClassTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ClassTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ProceduralCall(s) => s.get_text(),
            Self::DefineMethod(s) => s.get_text(),
            Self::DefineDomain(s) => s.get_text(),
            Self::DefineField(s) => s.get_text(),
            Self::EOS_FREE(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::DefineDomain(s) => s.get_range(),
            Self::DefineField(s) => s.get_range(),
            Self::EOS_FREE(s) => s.get_range(),
        }
    }
}
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwClassNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineFieldNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineField)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineField
    }

    fn get_text(&self) -> &str {
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
    pub fn parameter_default(&self) -> ParameterDefaultNode<'i> {
        self.pair.take_tagged_one("parameter_default").unwrap()
    }
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ParameterDefaultNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterDefault)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ParameterDefault
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ParameterDefaultNode<'i> {
    pub fn main_expression(&self) -> Option<MainExpressionNode<'i>> {
        self.pair.take_tagged_option("main_expression")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineMethodNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineMethod)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineMethod
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineDomain)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineDomain
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DomainTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("identifier") {
            return Ok(Self::Identifier(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::DomainTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DomainTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Identifier(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineInherit)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineInherit
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InheritTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::InheritTerm
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ObjectStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ObjectStatement
    }

    fn get_text(&self) -> &str {
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
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineEnumerateNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineEnumerate)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineEnumerate
    }

    fn get_text(&self) -> &str {
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
    pub fn kw_flags(&self) -> KwFlagsNode<'i> {
        self.pair.take_tagged_one("kw_flags").unwrap()
    }
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for FlagTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FlagTerm)?)
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
            return Ok(Self::EOS_FREE(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::FlagTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FlagTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ProceduralCall(s) => s.get_text(),
            Self::DefineMethod(s) => s.get_text(),
            Self::FlagField(s) => s.get_text(),
            Self::EOS_FREE(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::FlagField(s) => s.get_range(),
            Self::EOS_FREE(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for FlagFieldNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FlagField)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FlagField
    }

    fn get_text(&self) -> &str {
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
    pub fn parameter_default(&self) -> ParameterDefaultNode<'i> {
        self.pair.take_tagged_one("parameter_default").unwrap()
    }
}
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwFlagsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineUnionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineUnion)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineUnion
    }

    fn get_text(&self) -> &str {
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
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
    pub fn union_term(&self) -> Vec<UnionTermNode<'i>> {
        self.pair.take_tagged_items("union_term").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for UnionTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::UnionTerm)?)
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
            return Ok(Self::EOS_FREE(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::UnionTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::UnionTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ProceduralCall(s) => s.get_text(),
            Self::DefineMethod(s) => s.get_text(),
            Self::DefineVariant(s) => s.get_text(),
            Self::EOS_FREE(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::DefineVariant(s) => s.get_range(),
            Self::EOS_FREE(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineVariantNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineVariant)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineVariant
    }

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineTrait)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineTrait
    }

    fn get_text(&self) -> &str {
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
    pub fn define_inherit(&self) -> Option<DefineInheritNode<'i>> {
        self.pair.take_tagged_option("define_inherit")
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
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineExtendsNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineExtends)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineExtends
    }

    fn get_text(&self) -> &str {
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
    pub fn trait_block(&self) -> TraitBlockNode<'i> {
        self.pair.take_tagged_one("trait_block").unwrap()
    }
    pub fn type_expression(&self) -> TypeExpressionNode<'i> {
        self.pair.take_tagged_one("type_expression").unwrap()
    }
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TraitBlockNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TraitBlock)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TraitBlock
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TraitTerm)?)
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
            return Ok(Self::EOS_FREE(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TraitTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TraitTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ProceduralCall(s) => s.get_text(),
            Self::DefineMethod(s) => s.get_text(),
            Self::DefineField(s) => s.get_text(),
            Self::EOS_FREE(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ProceduralCall(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::DefineField(s) => s.get_range(),
            Self::EOS_FREE(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TraitNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Trait)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Trait
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TraitNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for InterfaceNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Interface)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Interface
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> InterfaceNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwTraitNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_TRAIT)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("trait") {
            return Ok(Self::Trait(s));
        }
        if let Ok(s) = pair.take_tagged_one("interface") {
            return Ok(Self::Interface(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::KW_TRAIT, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_TRAIT
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Trait(s) => s.get_text(),
            Self::Interface(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Trait(s) => s.get_range(),
            Self::Interface(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for DefineFunctionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineFunction)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineFunction
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineLambda)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineLambda
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FunctionMiddle)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FunctionMiddle
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeHint)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeHint
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeHintNode<'i> {
    pub fn hint(&self) -> Option<TypeExpressionNode<'i>> {
        self.pair.take_tagged_option("hint")
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeReturnNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeReturn)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeReturn
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeEffect)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeEffect
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FunctionParameters)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::FunctionParameters
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterItem)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("parameter_item_control") {
            return Ok(Self::ParameterItemControl(s));
        }
        if let Ok(s) = pair.take_tagged_one("parameter_pair") {
            return Ok(Self::ParameterPair(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ParameterItem, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ParameterItem
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ParameterItemControl(s) => s.get_text(),
            Self::ParameterPair(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterItemControl)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ParameterItemControl
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterPair)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ParameterPair
    }

    fn get_text(&self) -> &str {
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
    pub fn parameter_default(&self) -> ParameterDefaultNode<'i> {
        self.pair.take_tagged_one("parameter_default").unwrap()
    }
    pub fn parameter_hint(&self) -> Option<ParameterHintNode<'i>> {
        self.pair.take_tagged_option("parameter_hint")
    }
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ParameterHintNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterHint)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ParameterHint
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Continuation)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Continuation
    }

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineVariable)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineVariable
    }

    fn get_text(&self) -> &str {
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
    pub fn parameter_default(&self) -> ParameterDefaultNode<'i> {
        self.pair.take_tagged_one("parameter_default").unwrap()
    }
    pub fn type_hint(&self) -> TypeHintNode<'i> {
        self.pair.take_tagged_one("type_hint").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for LetPatternNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LetPattern)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("standard_pattern") {
            return Ok(Self::StandardPattern(s));
        }
        if let Ok(s) = pair.take_tagged_one("bare_pattern") {
            return Ok(Self::BarePattern(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::LetPattern, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::LetPattern
    }

    fn get_text(&self) -> &str {
        match self {
            Self::StandardPattern(s) => s.get_text(),
            Self::BarePattern(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StandardPattern)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("tuple_pattern") {
            return Ok(Self::TuplePattern(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::StandardPattern, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StandardPattern
    }

    fn get_text(&self) -> &str {
        match self {
            Self::TuplePattern(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BarePattern)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::BarePattern
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BarePatternItem)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::BarePatternItem
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TuplePattern)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TuplePattern
    }

    fn get_text(&self) -> &str {
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
impl<'i> YggdrasilNode<'i> for OmitDictNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OmitDict)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OmitDict
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OmitDictNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for OmitListNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OmitList)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::OmitList
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> OmitListNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for PatternItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PatternItem)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::PatternItem, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::PatternItem
    }

    fn get_text(&self) -> &str {
        match self {
            Self::TuplePatternItem(s) => s.get_text(),
            Self::OmitDict(s) => s.get_text(),
            Self::OmitList(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TuplePatternItem)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TuplePatternItem
    }

    fn get_text(&self) -> &str {
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
impl<'i> YggdrasilNode<'i> for WhileStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhileStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::WhileStatement
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> WhileStatementNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn inline_expression(&self) -> Option<InlineExpressionNode<'i>> {
        self.pair.take_tagged_option("inline_expression")
    }
    pub fn kw_while(&self) -> KwWhileNode<'i> {
        self.pair.take_tagged_one("kw_while").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for WhileNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::While)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::While
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> WhileNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for UntilNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Until)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Until
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> UntilNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwWhileNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WHILE)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("while") {
            return Ok(Self::While(s));
        }
        if let Ok(s) = pair.take_tagged_one("until") {
            return Ok(Self::Until(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::KW_WHILE, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_WHILE
    }

    fn get_text(&self) -> &str {
        match self {
            Self::While(s) => s.get_text(),
            Self::Until(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::While(s) => s.get_range(),
            Self::Until(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ForStatementNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ForStatement
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ForStatementNode<'i> {
    pub fn continuation(&self) -> ContinuationNode<'i> {
        self.pair.take_tagged_one("continuation").unwrap()
    }
    pub fn if_guard(&self) -> IfGuardNode<'i> {
        self.pair.take_tagged_one("if_guard").unwrap()
    }
    pub fn inline_expression(&self) -> Option<InlineExpressionNode<'i>> {
        self.pair.take_tagged_option("inline_expression")
    }
    pub fn kw_for(&self) -> KwForNode<'i> {
        self.pair.take_tagged_one("kw_for").unwrap()
    }
    pub fn kw_in(&self) -> KwInNode<'i> {
        self.pair.take_tagged_one("kw_in").unwrap()
    }
    pub fn let_pattern(&self) -> LetPatternNode<'i> {
        self.pair.take_tagged_one("let_pattern").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for IfGuardNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IfGuard)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IfGuard
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ControlFlow)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ControlFlow
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::JumpLabel)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::JumpLabel
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ExpressionRoot)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ExpressionRoot
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchExpression)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchExpression
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SwitchStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SwitchStatement
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchBlock)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchBlock
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchTerms)?)
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
            return Ok(Self::COMMA(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MatchTerms, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchTerms
    }

    fn get_text(&self) -> &str {
        match self {
            Self::MatchType(s) => s.get_text(),
            Self::MatchCase(s) => s.get_text(),
            Self::MatchWhen(s) => s.get_text(),
            Self::MatchElse(s) => s.get_text(),
            Self::COMMA(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::MatchType(s) => s.get_range(),
            Self::MatchCase(s) => s.get_range(),
            Self::MatchWhen(s) => s.get_range(),
            Self::MatchElse(s) => s.get_range(),
            Self::COMMA(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MatchTypeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchType)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchType
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchCase)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchCase
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CasePattern)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("standard_pattern") {
            return Ok(Self::StandardPattern(s));
        }
        if let Ok(s) = pair.take_tagged_one("namepath") {
            return Ok(Self::Namepath(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::CasePattern, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::CasePattern
    }

    fn get_text(&self) -> &str {
        match self {
            Self::StandardPattern(s) => s.get_text(),
            Self::Namepath(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchWhen)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchWhen
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchElse)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchElse
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MatchStatement
    }

    fn get_text(&self) -> &str {
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
impl<'i> YggdrasilNode<'i> for MatchNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Match)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Match
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> MatchNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for CatchNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Catch)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Catch
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> CatchNode<'i> {}
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

    fn get_text(&self) -> &str {
        match self {
            Self::Match(s) => s.get_text(),
            Self::Catch(s) => s.get_text(),
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotMatchCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DotMatchCall
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> DotMatchCallNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainExpressionNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainExpression)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MainExpression
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MainTerm
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainFactor)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::MainFactor, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MainFactor
    }

    fn get_text(&self) -> &str {
        match self {
            Self::SwitchStatement(s) => s.get_text(),
            Self::TryStatement(s) => s.get_text(),
            Self::MatchExpression(s) => s.get_text(),
            Self::DefineLambda(s) => s.get_text(),
            Self::ObjectStatement(s) => s.get_text(),
            Self::NewStatement(s) => s.get_text(),
            Self::GroupFactor(s) => s.get_text(),
            Self::Leading(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GroupFactor)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GroupFactor
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Leading)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::Leading, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Leading
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ProceduralCall(s) => s.get_text(),
            Self::TupleLiteralStrict(s) => s.get_text(),
            Self::RangeLiteral(s) => s.get_text(),
            Self::TextLiteral(s) => s.get_text(),
            Self::Slot(s) => s.get_text(),
            Self::Number(s) => s.get_text(),
            Self::Special(s) => s.get_text(),
            Self::Namepath(s) => s.get_text(),
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
impl<'i> YggdrasilNode<'i> for DotClosureCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotClosureCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DotClosureCall
    }

    fn get_text(&self) -> &str {
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
impl<'i> YggdrasilNode<'i> for MainSuffixTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainSuffixTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("dot_match_call") {
            return Ok(Self::DotMatchCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("dot_closure_call") {
            return Ok(Self::DotClosureCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("tuple_call") {
            return Ok(Self::TupleCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("inline_suffix_term") {
            return Ok(Self::InlineSuffixTerm(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MainSuffixTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MainSuffixTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::DotMatchCall(s) => s.get_text(),
            Self::DotClosureCall(s) => s.get_text(),
            Self::TupleCall(s) => s.get_text(),
            Self::InlineSuffixTerm(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DotMatchCall(s) => s.get_range(),
            Self::DotClosureCall(s) => s.get_range(),
            Self::TupleCall(s) => s.get_range(),
            Self::InlineSuffixTerm(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for MainPrefixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainPrefix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MainPrefix
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypePrefix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypePrefix
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainInfix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MainInfix
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeInfix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeInfix
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainSuffix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::MainSuffix
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeSuffix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeSuffix
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineExpression)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::InlineExpression
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::InlineTerm
    }

    fn get_text(&self) -> &str {
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
impl<'i> YggdrasilNode<'i> for DotCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DotCall
    }

    fn get_text(&self) -> &str {
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
impl<'i> YggdrasilNode<'i> for InlineSuffixTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineSuffixTerm)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::InlineSuffixTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::InlineSuffixTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::MainSuffix(s) => s.get_text(),
            Self::DotCall(s) => s.get_text(),
            Self::InlineTupleCall(s) => s.get_text(),
            Self::RangeCall(s) => s.get_text(),
            Self::GenericCall(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeExpression)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeExpression
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TypeExpressionNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for TypeTermNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeTerm
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeFactor)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("type_expression") {
            return Ok(Self::TypeExpression(s));
        }
        if let Ok(s) = pair.take_tagged_one("leading") {
            return Ok(Self::Leading(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TypeFactor, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeFactor
    }

    fn get_text(&self) -> &str {
        match self {
            Self::TypeExpression(s) => s.get_text(),
            Self::Leading(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeSuffixTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("generic_hide") {
            return Ok(Self::GenericHide(s));
        }
        if let Ok(s) = pair.take_tagged_one("type_suffix") {
            return Ok(Self::TypeSuffix(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TypeSuffixTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TypeSuffixTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::GenericHide(s) => s.get_text(),
            Self::TypeSuffix(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TryStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TryStatement
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewStatement)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NewStatement
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewBlock)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NewBlock
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewPair)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NewPair
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewPairKey)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::NewPairKey, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NewPairKey
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Identifier(s) => s.get_text(),
            Self::TextRaw(s) => s.get_text(),
            Self::RangeLiteral(s) => s.get_text(),
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
impl<'i> YggdrasilNode<'i> for DotCallItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotCallItem)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("namepath") {
            return Ok(Self::Namepath(s));
        }
        if let Ok(s) = pair.take_tagged_one("integer") {
            return Ok(Self::Integer(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::DotCallItem, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DotCallItem
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Namepath(s) => s.get_text(),
            Self::Integer(s) => s.get_text(),
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
impl<'i> YggdrasilNode<'i> for InlineTupleCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineTupleCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::InlineTupleCall
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TupleCall
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleLiteral)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TupleLiteral
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleLiteralStrict)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TupleLiteralStrict
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleTerms)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TupleTerms
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TuplePair)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TuplePair
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleKey)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::TupleKey, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TupleKey
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Identifier(s) => s.get_text(),
            Self::Integer(s) => s.get_text(),
            Self::TextRaw(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RangeCall
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeLiteral)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("range_literal_index_0") {
            return Ok(Self::RangeLiteralIndex0(s));
        }
        if let Ok(s) = pair.take_tagged_one("range_literal_index_1") {
            return Ok(Self::RangeLiteralIndex1(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::RangeLiteral, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RangeLiteral
    }

    fn get_text(&self) -> &str {
        match self {
            Self::RangeLiteralIndex0(s) => s.get_text(),
            Self::RangeLiteralIndex1(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeLiteralIndex0)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RangeLiteralIndex0
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeLiteralIndex1)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RangeLiteralIndex1
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SubscriptAxis)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("subscript_range") {
            return Ok(Self::SubscriptRange(s));
        }
        if let Ok(s) = pair.take_tagged_one("subscript_only") {
            return Ok(Self::SubscriptOnly(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::SubscriptAxis, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SubscriptAxis
    }

    fn get_text(&self) -> &str {
        match self {
            Self::SubscriptRange(s) => s.get_text(),
            Self::SubscriptOnly(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SubscriptOnly)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SubscriptOnly
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SubscriptRange)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SubscriptRange
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeOmit)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::RangeOmit
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineGeneric)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DefineGeneric
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericParameter)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GenericParameter
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericParameterPair)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GenericParameterPair
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GenericCall
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericHide)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GenericHide
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericTerms)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GenericTerms
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericPair)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::GenericPair
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationHead)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AnnotationHead
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationMix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AnnotationMix
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("attribute_list") {
            return Ok(Self::AttributeList(s));
        }
        if let Ok(s) = pair.take_tagged_one("attribute_call") {
            return Ok(Self::AttributeCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::AnnotationTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AnnotationTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::AttributeList(s) => s.get_text(),
            Self::AttributeCall(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::AttributeList(s) => s.get_range(),
            Self::AttributeCall(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AnnotationTermMixNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationTermMix)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("attribute_list") {
            return Ok(Self::AttributeList(s));
        }
        if let Ok(s) = pair.take_tagged_one("attribute_call") {
            return Ok(Self::AttributeCall(s));
        }
        if let Ok(s) = pair.take_tagged_one("procedural_call") {
            return Ok(Self::ProceduralCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::AnnotationTermMix, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AnnotationTermMix
    }

    fn get_text(&self) -> &str {
        match self {
            Self::AttributeList(s) => s.get_text(),
            Self::AttributeCall(s) => s.get_text(),
            Self::ProceduralCall(s) => s.get_text(),
        }
    }

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::AttributeList(s) => s.get_range(),
            Self::AttributeCall(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
        }
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AttributeListNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeList)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AttributeList
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AttributeListNode<'i> {
    pub fn attribute_item(&self) -> Vec<AttributeItemNode<'i>> {
        self.pair.take_tagged_items("attribute_item").collect::<Result<Vec<_>, _>>().unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AttributeCallNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AttributeCall
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> AttributeCallNode<'i> {
    pub fn attribute_item(&self) -> AttributeItemNode<'i> {
        self.pair.take_tagged_one("attribute_item").unwrap()
    }
}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for AttributeItemNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeItem)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AttributeItem
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeName)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::AttributeName
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ProceduralCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ProceduralCall
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ProceduralName)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ProceduralName
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TextLiteral)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TextLiteral
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TextRaw)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::TextRaw
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Text_L)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Text_L
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Text_R)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Text_R
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Text_X)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Text_X
    }

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ModifierCall)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ModifierCall
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ModifierAhead)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ModifierAhead
    }

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Slot)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Slot
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SlotItem)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("integer") {
            return Ok(Self::Integer(s));
        }
        if let Ok(s) = pair.take_tagged_one("identifier") {
            return Ok(Self::Identifier(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::SlotItem, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SlotItem
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Integer(s) => s.get_text(),
            Self::Identifier(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NamepathFree)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::NamepathFree
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Namepath)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Namepath
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Identifier)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("identifier_bare") {
            return Ok(Self::IdentifierBare(s));
        }
        if let Ok(s) = pair.take_tagged_one("identifier_raw") {
            return Ok(Self::IdentifierRaw(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Identifier, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Identifier
    }

    fn get_text(&self) -> &str {
        match self {
            Self::IdentifierBare(s) => s.get_text(),
            Self::IdentifierRaw(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IdentifierBare)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IdentifierBare
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IdentifierRaw)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IdentifierRaw
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IdentifierRawText)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::IdentifierRawText
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Special)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Special
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Number)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("decimal_x") {
            return Ok(Self::DecimalX(s));
        }
        if let Ok(s) = pair.take_tagged_one("decimal") {
            return Ok(Self::Decimal(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Number, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Number
    }

    fn get_text(&self) -> &str {
        match self {
            Self::DecimalX(s) => s.get_text(),
            Self::Decimal(s) => s.get_text(),
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
impl<'i> YggdrasilNode<'i> for PositiveNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Positive)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Positive
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> PositiveNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for NetativeNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Netative)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Netative
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> NetativeNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for SignNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Sign)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("positive") {
            return Ok(Self::Positive(s));
        }
        if let Ok(s) = pair.take_tagged_one("netative") {
            return Ok(Self::Netative(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Sign, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Sign
    }

    fn get_text(&self) -> &str {
        match self {
            Self::Positive(s) => s.get_text(),
            Self::Netative(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Integer)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Integer
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DigitsX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DigitsX
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Decimal)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Decimal
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DecimalX)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::DecimalX
    }

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> ColonNode<'i> {}
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwImplementsNode<'i> {}
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwInheritsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for KwForNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_FOR)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::KW_FOR
    }

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwForNode<'i> {}
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwEndNode<'i> {}
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> KwAsNode<'i> {}
#[automatically_derived]
impl<'i> YggdrasilNode<'i> for ShebangNode<'i> {
    type Rule = ValkyrieRule;

    fn from_str(input: &'i str, offset: usize) -> Result<Self, YggdrasilError<Self::Rule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Shebang)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Shebang
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhiteSpace)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::WhiteSpace
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SkipSpace)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::SkipSpace
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Comment)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::Comment
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolations)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringInterpolations
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationTerm)?)
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
        Err(YggdrasilError::invalid_node(ValkyrieRule::StringInterpolationTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringInterpolationTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::EscapeUnicode(s) => s.get_text(),
            Self::EscapeCharacter(s) => s.get_text(),
            Self::StringInterpolationSimple(s) => s.get_text(),
            Self::StringInterpolationComplex(s) => s.get_text(),
            Self::StringInterpolationText(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EscapeCharacter)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EscapeCharacter
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EscapeUnicode)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EscapeUnicode
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EscapeUnicodeCode)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::EscapeUnicodeCode
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationSimple)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringInterpolationSimple
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationText)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringInterpolationText
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringFormatter)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringFormatter
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationComplex)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringInterpolationComplex
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringTemplates)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringTemplates
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringTemplateTerm)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one("for_template") {
            return Ok(Self::ForTemplate(s));
        }
        if let Ok(s) = pair.take_tagged_one("expression_template") {
            return Ok(Self::ExpressionTemplate(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::StringTemplateTerm, _span))
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::StringTemplateTerm
    }

    fn get_text(&self) -> &str {
        match self {
            Self::ForTemplate(s) => s.get_text(),
            Self::ExpressionTemplate(s) => s.get_text(),
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ExpressionTemplate)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ExpressionTemplate
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplate)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ForTemplate
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplateBegin)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ForTemplateBegin
    }

    fn get_text(&self) -> &str {
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
    pub fn kw_for(&self) -> KwForNode<'i> {
        self.pair.take_tagged_one("kw_for").unwrap()
    }
    pub fn kw_in(&self) -> KwInNode<'i> {
        self.pair.take_tagged_one("kw_in").unwrap()
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplateElse)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ForTemplateElse
    }

    fn get_text(&self) -> &str {
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
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplateEnd)?)
    }
    fn from_pair(pair: TokenPair<'i, Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        Ok(Self { pair })
    }

    fn get_rule(&self) -> Self::Rule {
        ValkyrieRule::ForTemplateEnd
    }

    fn get_text(&self) -> &str {
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
    pub fn kw_for(&self) -> Option<KwForNode<'i>> {
        self.pair.take_tagged_option("kw_for")
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
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

    fn get_text(&self) -> &str {
        self.pair.get_span().as_str()
    }

    fn get_range(&self) -> Range<usize> {
        self.pair.get_span().get_range()
    }
}
impl<'i> TemplateMNode<'i> {}
