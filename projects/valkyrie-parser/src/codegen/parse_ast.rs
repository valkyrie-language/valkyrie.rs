use super::*;
#[automatically_derived]
impl YggdrasilNode for ProgramNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            shebang: pair.take_tagged_option::<ShebangNode>(Cow::Borrowed("shebang")),
            statement: pair.take_tagged_items::<StatementNode>(Cow::Borrowed("statement")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ProgramNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Program)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ControlFlow(s) => s.get_range(),
            Self::DefineClass(s) => s.get_range(),
            Self::DefineEnumerate(s) => s.get_range(),
            Self::DefineExtends(s) => s.get_range(),
            Self::DefineFunction(s) => s.get_range(),
            Self::DefineImport(s) => s.get_range(),
            Self::DefineNamespace(s) => s.get_range(),
            Self::DefineTrait(s) => s.get_range(),
            Self::DefineUnion(s) => s.get_range(),
            Self::DefineVariable(s) => s.get_range(),
            Self::Eos(s) => s.get_range(),
            Self::ExpressionRoot(s) => s.get_range(),
            Self::ForStatement(s) => s.get_range(),
            Self::WhileStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<ControlFlowNode>(Cow::Borrowed("control_flow")) {
            return Ok(Self::ControlFlow(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineClassNode>(Cow::Borrowed("define_class")) {
            return Ok(Self::DefineClass(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineEnumerateNode>(Cow::Borrowed("define_enumerate")) {
            return Ok(Self::DefineEnumerate(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineExtendsNode>(Cow::Borrowed("define_extends")) {
            return Ok(Self::DefineExtends(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineFunctionNode>(Cow::Borrowed("define_function")) {
            return Ok(Self::DefineFunction(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineImportNode>(Cow::Borrowed("define_import")) {
            return Ok(Self::DefineImport(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineNamespaceNode>(Cow::Borrowed("define_namespace")) {
            return Ok(Self::DefineNamespace(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineTraitNode>(Cow::Borrowed("define_trait")) {
            return Ok(Self::DefineTrait(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineUnionNode>(Cow::Borrowed("define_union")) {
            return Ok(Self::DefineUnion(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineVariableNode>(Cow::Borrowed("define_variable")) {
            return Ok(Self::DefineVariable(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EosNode>(Cow::Borrowed("eos")) {
            return Ok(Self::Eos(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ExpressionRootNode>(Cow::Borrowed("expression_root")) {
            return Ok(Self::ExpressionRoot(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ForStatementNode>(Cow::Borrowed("for_statement")) {
            return Ok(Self::ForStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<WhileStatementNode>(Cow::Borrowed("while_statement")) {
            return Ok(Self::WhileStatement(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Statement, _span))
    }
}
#[automatically_derived]
impl FromStr for StatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Statement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EosNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Omit => Range::default(),
            Self::Show => Range::default(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("omit") {
            return Ok(Self::Omit);
        }
        if let Some(_) = pair.find_first_tag("show") {
            return Ok(Self::Show);
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::EOS, _span))
    }
}
#[automatically_derived]
impl FromStr for EosNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EOS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EosFreeNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for EosFreeNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EOS_FREE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineNamespaceNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            namepath_free: pair.take_tagged_one::<NamepathFreeNode>(Cow::Borrowed("namepath_free"))?,
            op_namespace: pair.take_tagged_option::<OpNamespaceNode>(Cow::Borrowed("op_namespace")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineNamespaceNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineNamespace)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OpNamespaceNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Hide => Range::default(),
            Self::Main => Range::default(),
            Self::Test => Range::default(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("hide") {
            return Ok(Self::Hide);
        }
        if let Some(_) = pair.find_first_tag("main") {
            return Ok(Self::Main);
        }
        if let Some(_) = pair.find_first_tag("test") {
            return Ok(Self::Test);
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::OP_NAMESPACE, _span))
    }
}
#[automatically_derived]
impl FromStr for OpNamespaceNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_NAMESPACE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineImportNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term: pair
                .take_tagged_items::<AnnotationTermNode>(Cow::Borrowed("annotation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            eos_free: pair.take_tagged_option::<EosFreeNode>(Cow::Borrowed("eos_free")),
            import_block: pair.take_tagged_option::<ImportBlockNode>(Cow::Borrowed("import_block")),
            import_term: pair.take_tagged_option::<ImportTermNode>(Cow::Borrowed("import_term")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineImportNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineImport)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ImportBlockNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            import_term: pair
                .take_tagged_items::<ImportTermNode>(Cow::Borrowed("import_term"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ImportBlockNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ImportTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::EosFree(s) => s.get_range(),
            Self::ImportAll(s) => s.get_range(),
            Self::ImportName(s) => s.get_range(),
            Self::ImportSpace(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<EosFreeNode>(Cow::Borrowed("eos_free")) {
            return Ok(Self::EosFree(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ImportAllNode>(Cow::Borrowed("import_all")) {
            return Ok(Self::ImportAll(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ImportNameNode>(Cow::Borrowed("import_name")) {
            return Ok(Self::ImportName(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ImportSpaceNode>(Cow::Borrowed("import_space")) {
            return Ok(Self::ImportSpace(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ImportTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for ImportTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ImportAllNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            path: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("path")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ImportAllNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportAll)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ImportSpaceNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            body: pair.take_tagged_one::<ImportBlockNode>(Cow::Borrowed("body"))?,
            path: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("path")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ImportSpaceNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportSpace)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ImportNameNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            alias: pair.take_tagged_one::<ImportAsNode>(Cow::Borrowed("alias"))?,
            item: pair.take_tagged_one::<ImportNameItemNode>(Cow::Borrowed("item"))?,
            path: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("path")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ImportNameNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportName)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ImportAsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            alias: pair.take_tagged_option::<ImportNameItemNode>(Cow::Borrowed("alias")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ImportAsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportAs)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ImportNameItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::AttributeName(s) => s.get_range(),
            Self::Identifier(s) => s.get_range(),
            Self::ProceduralName(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<AttributeNameNode>(Cow::Borrowed("attribute_name")) {
            return Ok(Self::AttributeName(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier")) {
            return Ok(Self::Identifier(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ProceduralNameNode>(Cow::Borrowed("procedural_name")) {
            return Ok(Self::ProceduralName(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ImportNameItem, _span))
    }
}
#[automatically_derived]
impl FromStr for ImportNameItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ImportNameItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineConstraintNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_head: pair.take_tagged_one::<AnnotationHeadNode>(Cow::Borrowed("annotation_head"))?,
            constraint_block: pair.take_tagged_one::<ConstraintBlockNode>(Cow::Borrowed("constraint_block"))?,
            constraint_parameters: pair.take_tagged_option::<ConstraintParametersNode>(Cow::Borrowed("constraint_parameters")),
            kw_constraint: pair.take_tagged_one::<KwConstraintNode>(Cow::Borrowed("kw_constraint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineConstraintNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineConstraint)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ConstraintParametersNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("identifier")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ConstraintParametersNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintParameters)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ConstraintBlockNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            constraint_implements: pair
                .take_tagged_items::<ConstraintImplementsNode>(Cow::Borrowed("constraint_implements"))
                .collect::<Result<Vec<_>, _>>()?,
            constraint_statement: pair
                .take_tagged_items::<ConstraintStatementNode>(Cow::Borrowed("constraint_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            eos_free: pair.take_tagged_items::<EosFreeNode>(Cow::Borrowed("eos_free")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ConstraintBlockNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ConstraintStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            where_block: pair.take_tagged_one::<WhereBlockNode>(Cow::Borrowed("where_block"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ConstraintStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ConstraintImplementsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            kw_implements: pair.take_tagged_one::<KwImplementsNode>(Cow::Borrowed("kw_implements"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ConstraintImplementsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ConstraintImplements)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for WhereBlockNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            kw_where: pair.take_tagged_one::<KwWhereNode>(Cow::Borrowed("kw_where"))?,
            where_bound: pair
                .take_tagged_items::<WhereBoundNode>(Cow::Borrowed("where_bound"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for WhereBlockNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhereBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for WhereBoundNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            eos_free: pair.take_tagged_one::<EosFreeNode>(Cow::Borrowed("eos_free"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for WhereBoundNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhereBound)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineClassNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_head: pair.take_tagged_one::<AnnotationHeadNode>(Cow::Borrowed("annotation_head"))?,
            class_block: pair.take_tagged_one::<ClassBlockNode>(Cow::Borrowed("class_block"))?,
            define_constraint: pair.take_tagged_option::<DefineConstraintNode>(Cow::Borrowed("define_constraint")),
            define_generic: pair.take_tagged_option::<DefineGenericNode>(Cow::Borrowed("define_generic")),
            define_inherit: pair.take_tagged_option::<DefineInheritNode>(Cow::Borrowed("define_inherit")),
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            kw_class: pair.take_tagged_one::<KwClassNode>(Cow::Borrowed("kw_class"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineClassNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineClass)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ClassBlockNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            class_term: pair.take_tagged_items::<ClassTermNode>(Cow::Borrowed("class_term")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ClassBlockNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ClassBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ClassTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DefineDomain(s) => s.get_range(),
            Self::DefineField(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DefineDomainNode>(Cow::Borrowed("define_domain")) {
            return Ok(Self::DefineDomain(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineFieldNode>(Cow::Borrowed("define_field")) {
            return Ok(Self::DefineField(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineMethodNode>(Cow::Borrowed("define_method")) {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EosFreeNode>(Cow::Borrowed("eos_free")) {
            return Ok(Self::EosFree(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ProceduralCallNode>(Cow::Borrowed("procedural_call")) {
            return Ok(Self::ProceduralCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ClassTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for ClassTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ClassTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwClassNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwClassNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CLASS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineFieldNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_mix: pair.take_tagged_one::<AnnotationMixNode>(Cow::Borrowed("annotation_mix"))?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            parameter_default: pair.take_tagged_one::<ParameterDefaultNode>(Cow::Borrowed("parameter_default"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineFieldNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineField)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ParameterDefaultNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_expression: pair.take_tagged_option::<MainExpressionNode>(Cow::Borrowed("main_expression")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ParameterDefaultNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterDefault)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineMethodNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_mix: pair.take_tagged_one::<AnnotationMixNode>(Cow::Borrowed("annotation_mix"))?,
            continuation: pair.take_tagged_option::<ContinuationNode>(Cow::Borrowed("continuation")),
            function_middle: pair.take_tagged_one::<FunctionMiddleNode>(Cow::Borrowed("function_middle"))?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineMethodNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineMethod)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineDomainNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_mix: pair.take_tagged_one::<AnnotationMixNode>(Cow::Borrowed("annotation_mix"))?,
            domain_term: pair.take_tagged_one::<DomainTermNode>(Cow::Borrowed("domain_term"))?,
            statement: pair.take_tagged_items::<StatementNode>(Cow::Borrowed("statement")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineDomainNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineDomain)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DomainTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Identifier(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier")) {
            return Ok(Self::Identifier(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::DomainTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for DomainTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DomainTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineInheritNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            inherit_term: pair
                .take_tagged_items::<InheritTermNode>(Cow::Borrowed("inherit_term"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineInheritNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineInherit)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for InheritTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_mix: pair.take_tagged_one::<AnnotationMixNode>(Cow::Borrowed("annotation_mix"))?,
            type_expression: pair.take_tagged_one::<TypeExpressionNode>(Cow::Borrowed("type_expression"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for InheritTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InheritTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ObjectStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            class_block: pair.take_tagged_one::<ClassBlockNode>(Cow::Borrowed("class_block"))?,
            define_inherit: pair.take_tagged_option::<DefineInheritNode>(Cow::Borrowed("define_inherit")),
            kw_object: pair.take_tagged_one::<KwObjectNode>(Cow::Borrowed("kw_object"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ObjectStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ObjectStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineEnumerateNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_head: pair.take_tagged_one::<AnnotationHeadNode>(Cow::Borrowed("annotation_head"))?,
            define_inherit: pair.take_tagged_option::<DefineInheritNode>(Cow::Borrowed("define_inherit")),
            flag_term: pair.take_tagged_items::<FlagTermNode>(Cow::Borrowed("flag_term")).collect::<Result<Vec<_>, _>>()?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            kw_flags: pair.take_tagged_one::<KwFlagsNode>(Cow::Borrowed("kw_flags"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineEnumerateNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineEnumerate)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for FlagTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DefineMethod(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
            Self::FlagField(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DefineMethodNode>(Cow::Borrowed("define_method")) {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EosFreeNode>(Cow::Borrowed("eos_free")) {
            return Ok(Self::EosFree(s));
        }
        if let Ok(s) = pair.take_tagged_one::<FlagFieldNode>(Cow::Borrowed("flag_field")) {
            return Ok(Self::FlagField(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ProceduralCallNode>(Cow::Borrowed("procedural_call")) {
            return Ok(Self::ProceduralCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::FlagTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for FlagTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FlagTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for FlagFieldNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            parameter_default: pair.take_tagged_one::<ParameterDefaultNode>(Cow::Borrowed("parameter_default"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for FlagFieldNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FlagField)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwFlagsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwFlagsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_FLAGS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineUnionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_head: pair.take_tagged_one::<AnnotationHeadNode>(Cow::Borrowed("annotation_head"))?,
            define_constraint: pair.take_tagged_option::<DefineConstraintNode>(Cow::Borrowed("define_constraint")),
            define_generic: pair.take_tagged_option::<DefineGenericNode>(Cow::Borrowed("define_generic")),
            define_inherit: pair.take_tagged_option::<DefineInheritNode>(Cow::Borrowed("define_inherit")),
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            kw_union: pair.take_tagged_one::<KwUnionNode>(Cow::Borrowed("kw_union"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            union_term: pair.take_tagged_items::<UnionTermNode>(Cow::Borrowed("union_term")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineUnionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineUnion)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for UnionTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DefineMethod(s) => s.get_range(),
            Self::DefineVariant(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DefineMethodNode>(Cow::Borrowed("define_method")) {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineVariantNode>(Cow::Borrowed("define_variant")) {
            return Ok(Self::DefineVariant(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EosFreeNode>(Cow::Borrowed("eos_free")) {
            return Ok(Self::EosFree(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ProceduralCallNode>(Cow::Borrowed("procedural_call")) {
            return Ok(Self::ProceduralCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::UnionTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for UnionTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::UnionTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineVariantNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term: pair
                .take_tagged_items::<AnnotationTermNode>(Cow::Borrowed("annotation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            class_block: pair.take_tagged_option::<ClassBlockNode>(Cow::Borrowed("class_block")),
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineVariantNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineVariant)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwUnionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwUnionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_UNION)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineTraitNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_head: pair.take_tagged_one::<AnnotationHeadNode>(Cow::Borrowed("annotation_head"))?,
            define_constraint: pair.take_tagged_option::<DefineConstraintNode>(Cow::Borrowed("define_constraint")),
            define_generic: pair.take_tagged_option::<DefineGenericNode>(Cow::Borrowed("define_generic")),
            define_inherit: pair.take_tagged_option::<DefineInheritNode>(Cow::Borrowed("define_inherit")),
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            kw_trait: pair.take_tagged_one::<KwTraitNode>(Cow::Borrowed("kw_trait"))?,
            trait_block: pair.take_tagged_one::<TraitBlockNode>(Cow::Borrowed("trait_block"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineTraitNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineTrait)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineExtendsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_head: pair.take_tagged_one::<AnnotationHeadNode>(Cow::Borrowed("annotation_head"))?,
            define_constraint: pair.take_tagged_option::<DefineConstraintNode>(Cow::Borrowed("define_constraint")),
            kw_extends: pair.take_tagged_one::<KwExtendsNode>(Cow::Borrowed("kw_extends"))?,
            trait_block: pair.take_tagged_one::<TraitBlockNode>(Cow::Borrowed("trait_block"))?,
            type_expression: pair.take_tagged_one::<TypeExpressionNode>(Cow::Borrowed("type_expression"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineExtendsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineExtends)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TraitBlockNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            trait_term: pair.take_tagged_items::<TraitTermNode>(Cow::Borrowed("trait_term")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TraitBlockNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TraitBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TraitTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DefineField(s) => s.get_range(),
            Self::DefineMethod(s) => s.get_range(),
            Self::EosFree(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DefineFieldNode>(Cow::Borrowed("define_field")) {
            return Ok(Self::DefineField(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DefineMethodNode>(Cow::Borrowed("define_method")) {
            return Ok(Self::DefineMethod(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EosFreeNode>(Cow::Borrowed("eos_free")) {
            return Ok(Self::EosFree(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ProceduralCallNode>(Cow::Borrowed("procedural_call")) {
            return Ok(Self::ProceduralCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TraitTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for TraitTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TraitTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwTraitNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Interface => Range::default(),
            Self::Trait => Range::default(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("interface") {
            return Ok(Self::Interface);
        }
        if let Some(_) = pair.find_first_tag("trait") {
            return Ok(Self::Trait);
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::KW_TRAIT, _span))
    }
}
#[automatically_derived]
impl FromStr for KwTraitNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_TRAIT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineFunctionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_head: pair.take_tagged_one::<AnnotationHeadNode>(Cow::Borrowed("annotation_head"))?,
            continuation: pair.take_tagged_one::<ContinuationNode>(Cow::Borrowed("continuation"))?,
            function_middle: pair.take_tagged_one::<FunctionMiddleNode>(Cow::Borrowed("function_middle"))?,
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            kw_function: pair.take_tagged_one::<KwFunctionNode>(Cow::Borrowed("kw_function"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineFunctionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineFunction)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineLambdaNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term: pair
                .take_tagged_items::<AnnotationTermNode>(Cow::Borrowed("annotation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            continuation: pair.take_tagged_one::<ContinuationNode>(Cow::Borrowed("continuation"))?,
            function_middle: pair.take_tagged_one::<FunctionMiddleNode>(Cow::Borrowed("function_middle"))?,
            kw_lambda: pair.take_tagged_one::<KwLambdaNode>(Cow::Borrowed("kw_lambda"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineLambdaNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineLambda)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for FunctionMiddleNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            define_generic: pair.take_tagged_option::<DefineGenericNode>(Cow::Borrowed("define_generic")),
            function_parameters: pair.take_tagged_one::<FunctionParametersNode>(Cow::Borrowed("function_parameters"))?,
            type_effect: pair.take_tagged_option::<TypeEffectNode>(Cow::Borrowed("type_effect")),
            type_return: pair.take_tagged_option::<TypeReturnNode>(Cow::Borrowed("type_return")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for FunctionMiddleNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FunctionMiddle)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeHintNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            hint: pair.take_tagged_option::<TypeExpressionNode>(Cow::Borrowed("hint")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TypeHintNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeHint)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeReturnNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            arrow_1: pair.take_tagged_one::<Arrow1Node>(Cow::Borrowed("arrow_1"))?,
            type_expression: pair.take_tagged_one::<TypeExpressionNode>(Cow::Borrowed("type_expression"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TypeReturnNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeReturn)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeEffectNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            type_expression: pair.take_tagged_one::<TypeExpressionNode>(Cow::Borrowed("type_expression"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TypeEffectNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeEffect)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for FunctionParametersNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            parameter_item: pair
                .take_tagged_items::<ParameterItemNode>(Cow::Borrowed("parameter_item"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for FunctionParametersNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::FunctionParameters)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ParameterItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ParameterItemControl(s) => s.get_range(),
            Self::ParameterPair(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<ParameterItemControlNode>(Cow::Borrowed("parameter_item_control")) {
            return Ok(Self::ParameterItemControl(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ParameterPairNode>(Cow::Borrowed("parameter_pair")) {
            return Ok(Self::ParameterPair(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::ParameterItem, _span))
    }
}
#[automatically_derived]
impl FromStr for ParameterItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ParameterItemControlNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for ParameterItemControlNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterItemControl)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ParameterPairNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            modifier_ahead: pair
                .take_tagged_items::<ModifierAheadNode>(Cow::Borrowed("modifier_ahead"))
                .collect::<Result<Vec<_>, _>>()?,
            parameter_default: pair.take_tagged_one::<ParameterDefaultNode>(Cow::Borrowed("parameter_default"))?,
            parameter_hint: pair.take_tagged_option::<ParameterHintNode>(Cow::Borrowed("parameter_hint")),
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ParameterPairNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterPair)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ParameterHintNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for ParameterHintNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ParameterHint)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ContinuationNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            statement: pair.take_tagged_items::<StatementNode>(Cow::Borrowed("statement")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ContinuationNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Continuation)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwFunctionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwFunctionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_FUNCTION)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineVariableNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term: pair
                .take_tagged_items::<AnnotationTermNode>(Cow::Borrowed("annotation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            kw_let: pair.take_tagged_one::<KwLetNode>(Cow::Borrowed("kw_let"))?,
            let_pattern: pair.take_tagged_one::<LetPatternNode>(Cow::Borrowed("let_pattern"))?,
            parameter_default: pair.take_tagged_one::<ParameterDefaultNode>(Cow::Borrowed("parameter_default"))?,
            type_hint: pair.take_tagged_one::<TypeHintNode>(Cow::Borrowed("type_hint"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineVariableNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineVariable)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for LetPatternNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::BarePattern(s) => s.get_range(),
            Self::StandardPattern(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<BarePatternNode>(Cow::Borrowed("bare_pattern")) {
            return Ok(Self::BarePattern(s));
        }
        if let Ok(s) = pair.take_tagged_one::<StandardPatternNode>(Cow::Borrowed("standard_pattern")) {
            return Ok(Self::StandardPattern(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::LetPattern, _span))
    }
}
#[automatically_derived]
impl FromStr for LetPatternNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::LetPattern)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StandardPatternNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::TuplePattern(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<TuplePatternNode>(Cow::Borrowed("tuple_pattern")) {
            return Ok(Self::TuplePattern(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::StandardPattern, _span))
    }
}
#[automatically_derived]
impl FromStr for StandardPatternNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StandardPattern)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for BarePatternNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            bare_pattern_item: pair
                .take_tagged_items::<BarePatternItemNode>(Cow::Borrowed("bare_pattern_item"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for BarePatternNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BarePattern)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for BarePatternItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            modifier_ahead: pair
                .take_tagged_items::<ModifierAheadNode>(Cow::Borrowed("modifier_ahead"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for BarePatternItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BarePatternItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TuplePatternNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            namepath: pair.take_tagged_option::<NamepathNode>(Cow::Borrowed("namepath")),
            pattern_item: pair
                .take_tagged_items::<PatternItemNode>(Cow::Borrowed("pattern_item"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TuplePatternNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TuplePattern)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for PatternItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::OmitDict => Range::default(),
            Self::OmitList => Range::default(),
            Self::TuplePatternItem(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("omit_dict") {
            return Ok(Self::OmitDict);
        }
        if let Some(_) = pair.find_first_tag("omit_list") {
            return Ok(Self::OmitList);
        }
        if let Ok(s) = pair.take_tagged_one::<TuplePatternItemNode>(Cow::Borrowed("tuple_pattern_item")) {
            return Ok(Self::TuplePatternItem(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::PatternItem, _span))
    }
}
#[automatically_derived]
impl FromStr for PatternItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PatternItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TuplePatternItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_mix: pair.take_tagged_one::<AnnotationMixNode>(Cow::Borrowed("annotation_mix"))?,
            colon: pair.take_tagged_option::<ColonNode>(Cow::Borrowed("colon")),
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            parameter_hint: pair.take_tagged_option::<ParameterHintNode>(Cow::Borrowed("parameter_hint")),
            standard_pattern: pair.take_tagged_option::<StandardPatternNode>(Cow::Borrowed("standard_pattern")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TuplePatternItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TuplePatternItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for WhileStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            continuation: pair.take_tagged_one::<ContinuationNode>(Cow::Borrowed("continuation"))?,
            inline_expression: pair.take_tagged_option::<InlineExpressionNode>(Cow::Borrowed("inline_expression")),
            kw_while: pair.take_tagged_one::<KwWhileNode>(Cow::Borrowed("kw_while"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for WhileStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhileStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwWhileNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Until => Range::default(),
            Self::While => Range::default(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("until") {
            return Ok(Self::Until);
        }
        if let Some(_) = pair.find_first_tag("while") {
            return Ok(Self::While);
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::KW_WHILE, _span))
    }
}
#[automatically_derived]
impl FromStr for KwWhileNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WHILE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ForStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            continuation: pair.take_tagged_one::<ContinuationNode>(Cow::Borrowed("continuation"))?,
            if_guard: pair.take_tagged_one::<IfGuardNode>(Cow::Borrowed("if_guard"))?,
            inline_expression: pair.take_tagged_option::<InlineExpressionNode>(Cow::Borrowed("inline_expression")),
            kw_for: pair.take_tagged_one::<KwForNode>(Cow::Borrowed("kw_for"))?,
            kw_in: pair.take_tagged_one::<KwInNode>(Cow::Borrowed("kw_in"))?,
            let_pattern: pair.take_tagged_one::<LetPatternNode>(Cow::Borrowed("let_pattern"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ForStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IfGuardNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            condition: pair.take_tagged_option::<InlineExpressionNode>(Cow::Borrowed("condition")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for IfGuardNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IfGuard)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ControlFlowNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term: pair
                .take_tagged_items::<AnnotationTermNode>(Cow::Borrowed("annotation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            jump_label: pair.take_tagged_one::<JumpLabelNode>(Cow::Borrowed("jump_label"))?,
            kw_control: pair.take_tagged_one::<KwControlNode>(Cow::Borrowed("kw_control"))?,
            main_expression: pair.take_tagged_option::<MainExpressionNode>(Cow::Borrowed("main_expression")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ControlFlowNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ControlFlow)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for JumpLabelNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_option::<IdentifierNode>(Cow::Borrowed("identifier")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for JumpLabelNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::JumpLabel)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ExpressionRootNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term: pair
                .take_tagged_items::<AnnotationTermNode>(Cow::Borrowed("annotation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            eos: pair.take_tagged_option::<EosNode>(Cow::Borrowed("eos")),
            main_expression: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("main_expression"))?,
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ExpressionRootNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ExpressionRoot)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchExpressionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            bind_l: pair.take_tagged_option::<BindLNode>(Cow::Borrowed("bind_l")),
            bind_r: pair.take_tagged_option::<BindRNode>(Cow::Borrowed("bind_r")),
            identifier: pair.take_tagged_option::<IdentifierNode>(Cow::Borrowed("identifier")),
            inline_expression: pair.take_tagged_one::<InlineExpressionNode>(Cow::Borrowed("inline_expression"))?,
            kw_match: pair.take_tagged_one::<KwMatchNode>(Cow::Borrowed("kw_match"))?,
            match_block: pair.take_tagged_one::<MatchBlockNode>(Cow::Borrowed("match_block"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MatchExpressionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchExpression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SwitchStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            kw_switch: pair.take_tagged_one::<KwSwitchNode>(Cow::Borrowed("kw_switch"))?,
            match_block: pair.take_tagged_one::<MatchBlockNode>(Cow::Borrowed("match_block"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for SwitchStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SwitchStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchBlockNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            match_terms: pair
                .take_tagged_items::<MatchTermsNode>(Cow::Borrowed("match_terms"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MatchBlockNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchTermsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Comma(s) => s.get_range(),
            Self::MatchCase(s) => s.get_range(),
            Self::MatchElse(s) => s.get_range(),
            Self::MatchType(s) => s.get_range(),
            Self::MatchWhen(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<CommaNode>(Cow::Borrowed("comma")) {
            return Ok(Self::Comma(s));
        }
        if let Ok(s) = pair.take_tagged_one::<MatchCaseNode>(Cow::Borrowed("match_case")) {
            return Ok(Self::MatchCase(s));
        }
        if let Ok(s) = pair.take_tagged_one::<MatchElseNode>(Cow::Borrowed("match_else")) {
            return Ok(Self::MatchElse(s));
        }
        if let Ok(s) = pair.take_tagged_one::<MatchTypeNode>(Cow::Borrowed("match_type")) {
            return Ok(Self::MatchType(s));
        }
        if let Ok(s) = pair.take_tagged_one::<MatchWhenNode>(Cow::Borrowed("match_when")) {
            return Ok(Self::MatchWhen(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MatchTerms, _span))
    }
}
#[automatically_derived]
impl FromStr for MatchTermsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchTerms)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchTypeNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            if_guard: pair.take_tagged_one::<IfGuardNode>(Cow::Borrowed("if_guard"))?,
            kw_type: pair.take_tagged_one::<KwTypeNode>(Cow::Borrowed("kw_type"))?,
            match_statement: pair
                .take_tagged_items::<MatchStatementNode>(Cow::Borrowed("match_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            type_expression: pair.take_tagged_one::<TypeExpressionNode>(Cow::Borrowed("type_expression"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MatchTypeNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchType)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchCaseNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            case_pattern: pair.take_tagged_one::<CasePatternNode>(Cow::Borrowed("case_pattern"))?,
            if_guard: pair.take_tagged_one::<IfGuardNode>(Cow::Borrowed("if_guard"))?,
            kw_case: pair.take_tagged_one::<KwCaseNode>(Cow::Borrowed("kw_case"))?,
            match_statement: pair
                .take_tagged_items::<MatchStatementNode>(Cow::Borrowed("match_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MatchCaseNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchCase)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for CasePatternNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Namepath(s) => s.get_range(),
            Self::StandardPattern(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<NamepathNode>(Cow::Borrowed("namepath")) {
            return Ok(Self::Namepath(s));
        }
        if let Ok(s) = pair.take_tagged_one::<StandardPatternNode>(Cow::Borrowed("standard_pattern")) {
            return Ok(Self::StandardPattern(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::CasePattern, _span))
    }
}
#[automatically_derived]
impl FromStr for CasePatternNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::CasePattern)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchWhenNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            inline_expression: pair.take_tagged_one::<InlineExpressionNode>(Cow::Borrowed("inline_expression"))?,
            kw_when: pair.take_tagged_one::<KwWhenNode>(Cow::Borrowed("kw_when"))?,
            match_statement: pair
                .take_tagged_items::<MatchStatementNode>(Cow::Borrowed("match_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MatchWhenNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchWhen)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchElseNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            kw_else: pair.take_tagged_one::<KwElseNode>(Cow::Borrowed("kw_else"))?,
            match_statement: pair
                .take_tagged_items::<MatchStatementNode>(Cow::Borrowed("match_statement"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MatchElseNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchElse)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MatchStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            statement: pair.take_tagged_one::<StatementNode>(Cow::Borrowed("statement"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MatchStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MatchStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwMatchNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Catch => Range::default(),
            Self::Match => Range::default(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("catch") {
            return Ok(Self::Catch);
        }
        if let Some(_) = pair.find_first_tag("match") {
            return Ok(Self::Match);
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::KW_MATCH, _span))
    }
}
#[automatically_derived]
impl FromStr for KwMatchNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_MATCH)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for BindLNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for BindLNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BIND_L)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for BindRNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for BindRNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::BIND_R)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DotMatchCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            bind_r: pair.take_tagged_option::<BindRNode>(Cow::Borrowed("bind_r")),
            identifier: pair.take_tagged_option::<IdentifierNode>(Cow::Borrowed("identifier")),
            kw_match: pair.take_tagged_one::<KwMatchNode>(Cow::Borrowed("kw_match"))?,
            match_block: pair.take_tagged_one::<MatchBlockNode>(Cow::Borrowed("match_block"))?,
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DotMatchCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotMatchCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MainExpressionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_infix: pair.take_tagged_items::<MainInfixNode>(Cow::Borrowed("main_infix")).collect::<Result<Vec<_>, _>>()?,
            main_term: pair.take_tagged_items::<MainTermNode>(Cow::Borrowed("main_term")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MainExpressionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainExpression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MainTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_factor: pair.take_tagged_one::<MainFactorNode>(Cow::Borrowed("main_factor"))?,
            main_prefix: pair
                .take_tagged_items::<MainPrefixNode>(Cow::Borrowed("main_prefix"))
                .collect::<Result<Vec<_>, _>>()?,
            main_suffix_term: pair
                .take_tagged_items::<MainSuffixTermNode>(Cow::Borrowed("main_suffix_term"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for MainTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MainFactorNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DefineLambda(s) => s.get_range(),
            Self::GroupFactor(s) => s.get_range(),
            Self::Leading(s) => s.get_range(),
            Self::MatchExpression(s) => s.get_range(),
            Self::NewStatement(s) => s.get_range(),
            Self::ObjectStatement(s) => s.get_range(),
            Self::SwitchStatement(s) => s.get_range(),
            Self::TryStatement(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DefineLambdaNode>(Cow::Borrowed("define_lambda")) {
            return Ok(Self::DefineLambda(s));
        }
        if let Ok(s) = pair.take_tagged_one::<GroupFactorNode>(Cow::Borrowed("group_factor")) {
            return Ok(Self::GroupFactor(s));
        }
        if let Ok(s) = pair.take_tagged_one::<LeadingNode>(Cow::Borrowed("leading")) {
            return Ok(Self::Leading(s));
        }
        if let Ok(s) = pair.take_tagged_one::<MatchExpressionNode>(Cow::Borrowed("match_expression")) {
            return Ok(Self::MatchExpression(s));
        }
        if let Ok(s) = pair.take_tagged_one::<NewStatementNode>(Cow::Borrowed("new_statement")) {
            return Ok(Self::NewStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ObjectStatementNode>(Cow::Borrowed("object_statement")) {
            return Ok(Self::ObjectStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<SwitchStatementNode>(Cow::Borrowed("switch_statement")) {
            return Ok(Self::SwitchStatement(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TryStatementNode>(Cow::Borrowed("try_statement")) {
            return Ok(Self::TryStatement(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MainFactor, _span))
    }
}
#[automatically_derived]
impl FromStr for MainFactorNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainFactor)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for GroupFactorNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_expression: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("main_expression"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for GroupFactorNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GroupFactor)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for LeadingNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Namepath(s) => s.get_range(),
            Self::Number(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
            Self::RangeLiteral(s) => s.get_range(),
            Self::Slot(s) => s.get_range(),
            Self::Special(s) => s.get_range(),
            Self::TextLiteral(s) => s.get_range(),
            Self::TupleLiteralStrict(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<NamepathNode>(Cow::Borrowed("namepath")) {
            return Ok(Self::Namepath(s));
        }
        if let Ok(s) = pair.take_tagged_one::<NumberNode>(Cow::Borrowed("number")) {
            return Ok(Self::Number(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ProceduralCallNode>(Cow::Borrowed("procedural_call")) {
            return Ok(Self::ProceduralCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RangeLiteralNode>(Cow::Borrowed("range_literal")) {
            return Ok(Self::RangeLiteral(s));
        }
        if let Ok(s) = pair.take_tagged_one::<SlotNode>(Cow::Borrowed("slot")) {
            return Ok(Self::Slot(s));
        }
        if let Ok(s) = pair.take_tagged_one::<SpecialNode>(Cow::Borrowed("special")) {
            return Ok(Self::Special(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TextLiteralNode>(Cow::Borrowed("text_literal")) {
            return Ok(Self::TextLiteral(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TupleLiteralStrictNode>(Cow::Borrowed("tuple_literal_strict")) {
            return Ok(Self::TupleLiteralStrict(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Leading, _span))
    }
}
#[automatically_derived]
impl FromStr for LeadingNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Leading)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MainSuffixTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DotClosureCall(s) => s.get_range(),
            Self::DotMatchCall(s) => s.get_range(),
            Self::InlineSuffixTerm(s) => s.get_range(),
            Self::TupleCall(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DotClosureCallNode>(Cow::Borrowed("dot_closure_call")) {
            return Ok(Self::DotClosureCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DotMatchCallNode>(Cow::Borrowed("dot_match_call")) {
            return Ok(Self::DotMatchCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<InlineSuffixTermNode>(Cow::Borrowed("inline_suffix_term")) {
            return Ok(Self::InlineSuffixTerm(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TupleCallNode>(Cow::Borrowed("tuple_call")) {
            return Ok(Self::TupleCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::MainSuffixTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for MainSuffixTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainSuffixTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MainPrefixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for MainPrefixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainPrefix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypePrefixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TypePrefixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypePrefix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MainInfixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for MainInfixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainInfix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeInfixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TypeInfixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeInfix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for MainSuffixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for MainSuffixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::MainSuffix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeSuffixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TypeSuffixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeSuffix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for InlineExpressionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            inline_term: pair
                .take_tagged_items::<InlineTermNode>(Cow::Borrowed("inline_term"))
                .collect::<Result<Vec<_>, _>>()?,
            main_infix: pair.take_tagged_items::<MainInfixNode>(Cow::Borrowed("main_infix")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for InlineExpressionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineExpression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for InlineTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            inline_suffix_term: pair
                .take_tagged_items::<InlineSuffixTermNode>(Cow::Borrowed("inline_suffix_term"))
                .collect::<Result<Vec<_>, _>>()?,
            main_factor: pair.take_tagged_one::<MainFactorNode>(Cow::Borrowed("main_factor"))?,
            main_prefix: pair
                .take_tagged_items::<MainPrefixNode>(Cow::Borrowed("main_prefix"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for InlineTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for InlineSuffixTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::DotCall(s) => s.get_range(),
            Self::GenericCall(s) => s.get_range(),
            Self::InlineTupleCall(s) => s.get_range(),
            Self::MainSuffix(s) => s.get_range(),
            Self::RangeCall(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DotCallNode>(Cow::Borrowed("dot_call")) {
            return Ok(Self::DotCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<GenericCallNode>(Cow::Borrowed("generic_call")) {
            return Ok(Self::GenericCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<InlineTupleCallNode>(Cow::Borrowed("inline_tuple_call")) {
            return Ok(Self::InlineTupleCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<MainSuffixNode>(Cow::Borrowed("main_suffix")) {
            return Ok(Self::MainSuffix(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RangeCallNode>(Cow::Borrowed("range_call")) {
            return Ok(Self::RangeCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::InlineSuffixTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for InlineSuffixTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineSuffixTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeExpressionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            type_infix: pair.take_tagged_items::<TypeInfixNode>(Cow::Borrowed("type_infix")).collect::<Result<Vec<_>, _>>()?,
            type_term: pair.take_tagged_items::<TypeTermNode>(Cow::Borrowed("type_term")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TypeExpressionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeExpression)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_factor: pair.take_tagged_one::<MainFactorNode>(Cow::Borrowed("main_factor"))?,
            type_prefix: pair
                .take_tagged_items::<TypePrefixNode>(Cow::Borrowed("type_prefix"))
                .collect::<Result<Vec<_>, _>>()?,
            type_suffix_term: pair
                .take_tagged_items::<TypeSuffixTermNode>(Cow::Borrowed("type_suffix_term"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TypeTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeFactorNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Leading(s) => s.get_range(),
            Self::TypeExpression(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<LeadingNode>(Cow::Borrowed("leading")) {
            return Ok(Self::Leading(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TypeExpressionNode>(Cow::Borrowed("type_expression")) {
            return Ok(Self::TypeExpression(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TypeFactor, _span))
    }
}
#[automatically_derived]
impl FromStr for TypeFactorNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeFactor)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TypeSuffixTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::GenericHide(s) => s.get_range(),
            Self::TypeSuffix(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<GenericHideNode>(Cow::Borrowed("generic_hide")) {
            return Ok(Self::GenericHide(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TypeSuffixNode>(Cow::Borrowed("type_suffix")) {
            return Ok(Self::TypeSuffix(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TypeSuffixTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for TypeSuffixTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TypeSuffixTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TryStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            continuation: pair.take_tagged_one::<ContinuationNode>(Cow::Borrowed("continuation"))?,
            kw_try: pair.take_tagged_one::<KwTryNode>(Cow::Borrowed("kw_try"))?,
            type_expression: pair.take_tagged_option::<TypeExpressionNode>(Cow::Borrowed("type_expression")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TryStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TryStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NewStatementNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            generic_hide: pair.take_tagged_option::<GenericHideNode>(Cow::Borrowed("generic_hide")),
            kw_new: pair.take_tagged_one::<KwNewNode>(Cow::Borrowed("kw_new"))?,
            modifier_ahead: pair
                .take_tagged_items::<ModifierAheadNode>(Cow::Borrowed("modifier_ahead"))
                .collect::<Result<Vec<_>, _>>()?,
            namepath: pair.take_tagged_one::<NamepathNode>(Cow::Borrowed("namepath"))?,
            new_block: pair.take_tagged_option::<NewBlockNode>(Cow::Borrowed("new_block")),
            tuple_literal: pair.take_tagged_option::<TupleLiteralNode>(Cow::Borrowed("tuple_literal")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for NewStatementNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewStatement)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NewBlockNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            eos_free: pair.take_tagged_items::<EosFreeNode>(Cow::Borrowed("eos_free")).collect::<Result<Vec<_>, _>>()?,
            new_pair: pair.take_tagged_items::<NewPairNode>(Cow::Borrowed("new_pair")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for NewBlockNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewBlock)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NewPairNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            colon: pair.take_tagged_option::<ColonNode>(Cow::Borrowed("colon")),
            main_expression: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("main_expression"))?,
            new_pair_key: pair.take_tagged_option::<NewPairKeyNode>(Cow::Borrowed("new_pair_key")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for NewPairNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewPair)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NewPairKeyNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Identifier(s) => s.get_range(),
            Self::RangeLiteral(s) => s.get_range(),
            Self::TextRaw(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier")) {
            return Ok(Self::Identifier(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RangeLiteralNode>(Cow::Borrowed("range_literal")) {
            return Ok(Self::RangeLiteral(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TextRawNode>(Cow::Borrowed("text_raw")) {
            return Ok(Self::TextRaw(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::NewPairKey, _span))
    }
}
#[automatically_derived]
impl FromStr for NewPairKeyNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NewPairKey)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DotCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            dot_call_item: pair.take_tagged_one::<DotCallItemNode>(Cow::Borrowed("dot_call_item"))?,
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DotCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DotCallItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Integer(s) => s.get_range(),
            Self::Namepath(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("integer")) {
            return Ok(Self::Integer(s));
        }
        if let Ok(s) = pair.take_tagged_one::<NamepathNode>(Cow::Borrowed("namepath")) {
            return Ok(Self::Namepath(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::DotCallItem, _span))
    }
}
#[automatically_derived]
impl FromStr for DotCallItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotCallItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DotClosureCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            continuation: pair.take_tagged_one::<ContinuationNode>(Cow::Borrowed("continuation"))?,
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DotClosureCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DotClosureCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for InlineTupleCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            tuple_literal: pair.take_tagged_one::<TupleLiteralNode>(Cow::Borrowed("tuple_literal"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for InlineTupleCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::InlineTupleCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TupleCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            continuation: pair.take_tagged_option::<ContinuationNode>(Cow::Borrowed("continuation")),
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            tuple_literal: pair.take_tagged_option::<TupleLiteralNode>(Cow::Borrowed("tuple_literal")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TupleCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TupleLiteralNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            tuple_terms: pair.take_tagged_one::<TupleTermsNode>(Cow::Borrowed("tuple_terms"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TupleLiteralNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleLiteral)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TupleLiteralStrictNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            tuple_pair: pair.take_tagged_items::<TuplePairNode>(Cow::Borrowed("tuple_pair")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TupleLiteralStrictNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleLiteralStrict)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TupleTermsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            tuple_pair: pair.take_tagged_items::<TuplePairNode>(Cow::Borrowed("tuple_pair")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TupleTermsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleTerms)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TuplePairNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            colon: pair.take_tagged_option::<ColonNode>(Cow::Borrowed("colon")),
            main_expression: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("main_expression"))?,
            tuple_key: pair.take_tagged_option::<TupleKeyNode>(Cow::Borrowed("tuple_key")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TuplePairNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TuplePair)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TupleKeyNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Identifier(s) => s.get_range(),
            Self::Integer(s) => s.get_range(),
            Self::TextRaw(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier")) {
            return Ok(Self::Identifier(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("integer")) {
            return Ok(Self::Integer(s));
        }
        if let Ok(s) = pair.take_tagged_one::<TextRawNode>(Cow::Borrowed("text_raw")) {
            return Ok(Self::TextRaw(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::TupleKey, _span))
    }
}
#[automatically_derived]
impl FromStr for TupleKeyNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TupleKey)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for RangeCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            range_literal: pair.take_tagged_one::<RangeLiteralNode>(Cow::Borrowed("range_literal"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for RangeCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for RangeLiteralNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::RangeLiteralIndex0(s) => s.get_range(),
            Self::RangeLiteralIndex1(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<RangeLiteralIndex0Node>(Cow::Borrowed("range_literal_index_0")) {
            return Ok(Self::RangeLiteralIndex0(s));
        }
        if let Ok(s) = pair.take_tagged_one::<RangeLiteralIndex1Node>(Cow::Borrowed("range_literal_index_1")) {
            return Ok(Self::RangeLiteralIndex1(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::RangeLiteral, _span))
    }
}
#[automatically_derived]
impl FromStr for RangeLiteralNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeLiteral)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for RangeLiteralIndex0Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            subscript_axis: pair
                .take_tagged_items::<SubscriptAxisNode>(Cow::Borrowed("subscript_axis"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for RangeLiteralIndex0Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeLiteralIndex0)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for RangeLiteralIndex1Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            subscript_axis: pair
                .take_tagged_items::<SubscriptAxisNode>(Cow::Borrowed("subscript_axis"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for RangeLiteralIndex1Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeLiteralIndex1)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SubscriptAxisNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::SubscriptOnly(s) => s.get_range(),
            Self::SubscriptRange(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<SubscriptOnlyNode>(Cow::Borrowed("subscript_only")) {
            return Ok(Self::SubscriptOnly(s));
        }
        if let Ok(s) = pair.take_tagged_one::<SubscriptRangeNode>(Cow::Borrowed("subscript_range")) {
            return Ok(Self::SubscriptRange(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::SubscriptAxis, _span))
    }
}
#[automatically_derived]
impl FromStr for SubscriptAxisNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SubscriptAxis)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SubscriptOnlyNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            index: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("index"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for SubscriptOnlyNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SubscriptOnly)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SubscriptRangeNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            head: pair.take_tagged_option::<MainExpressionNode>(Cow::Borrowed("head")),
            step: pair.take_tagged_option::<MainExpressionNode>(Cow::Borrowed("step")),
            tail: pair.take_tagged_option::<MainExpressionNode>(Cow::Borrowed("tail")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for SubscriptRangeNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SubscriptRange)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for RangeOmitNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            colon: pair.take_tagged_items::<ColonNode>(Cow::Borrowed("colon")).collect::<Result<Vec<_>, _>>()?,
            proportion: pair.take_tagged_option::<ProportionNode>(Cow::Borrowed("proportion")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for RangeOmitNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::RangeOmit)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DefineGenericNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            generic_parameter: pair.take_tagged_one::<GenericParameterNode>(Cow::Borrowed("generic_parameter"))?,
            proportion: pair.take_tagged_option::<ProportionNode>(Cow::Borrowed("proportion")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DefineGenericNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DefineGeneric)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for GenericParameterNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            generic_parameter_pair: pair
                .take_tagged_items::<GenericParameterPairNode>(Cow::Borrowed("generic_parameter_pair"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for GenericParameterNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericParameter)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for GenericParameterPairNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            bound: pair.take_tagged_option::<TypeExpressionNode>(Cow::Borrowed("bound")),
            default: pair.take_tagged_option::<TypeExpressionNode>(Cow::Borrowed("default")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for GenericParameterPairNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericParameterPair)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for GenericCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            generic_terms: pair.take_tagged_one::<GenericTermsNode>(Cow::Borrowed("generic_terms"))?,
            namepath: pair.take_tagged_option::<NamepathNode>(Cow::Borrowed("namepath")),
            op_and_then: pair.take_tagged_option::<OpAndThenNode>(Cow::Borrowed("op_and_then")),
            proportion: pair.take_tagged_items::<ProportionNode>(Cow::Borrowed("proportion")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for GenericCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for GenericHideNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            generic_terms: pair.take_tagged_one::<GenericTermsNode>(Cow::Borrowed("generic_terms"))?,
            proportion: pair.take_tagged_option::<ProportionNode>(Cow::Borrowed("proportion")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for GenericHideNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericHide)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for GenericTermsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            generic_pair: pair
                .take_tagged_items::<GenericPairNode>(Cow::Borrowed("generic_pair"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for GenericTermsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericTerms)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for GenericPairNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            colon: pair.take_tagged_option::<ColonNode>(Cow::Borrowed("colon")),
            identifier: pair.take_tagged_option::<IdentifierNode>(Cow::Borrowed("identifier")),
            type_expression: pair.take_tagged_one::<TypeExpressionNode>(Cow::Borrowed("type_expression"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for GenericPairNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::GenericPair)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AnnotationHeadNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term: pair
                .take_tagged_items::<AnnotationTermNode>(Cow::Borrowed("annotation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            modifier_call: pair
                .take_tagged_items::<ModifierCallNode>(Cow::Borrowed("modifier_call"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for AnnotationHeadNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationHead)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AnnotationMixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            annotation_term_mix: pair
                .take_tagged_items::<AnnotationTermMixNode>(Cow::Borrowed("annotation_term_mix"))
                .collect::<Result<Vec<_>, _>>()?,
            modifier_ahead: pair
                .take_tagged_items::<ModifierAheadNode>(Cow::Borrowed("modifier_ahead"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for AnnotationMixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationMix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AnnotationTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::AttributeCall(s) => s.get_range(),
            Self::AttributeList(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<AttributeCallNode>(Cow::Borrowed("attribute_call")) {
            return Ok(Self::AttributeCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<AttributeListNode>(Cow::Borrowed("attribute_list")) {
            return Ok(Self::AttributeList(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::AnnotationTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for AnnotationTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AnnotationTermMixNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::AttributeCall(s) => s.get_range(),
            Self::AttributeList(s) => s.get_range(),
            Self::ProceduralCall(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<AttributeCallNode>(Cow::Borrowed("attribute_call")) {
            return Ok(Self::AttributeCall(s));
        }
        if let Ok(s) = pair.take_tagged_one::<AttributeListNode>(Cow::Borrowed("attribute_list")) {
            return Ok(Self::AttributeList(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ProceduralCallNode>(Cow::Borrowed("procedural_call")) {
            return Ok(Self::ProceduralCall(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::AnnotationTermMix, _span))
    }
}
#[automatically_derived]
impl FromStr for AnnotationTermMixNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AnnotationTermMix)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AttributeListNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            attribute_item: pair
                .take_tagged_items::<AttributeItemNode>(Cow::Borrowed("attribute_item"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for AttributeListNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeList)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AttributeCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            attribute_item: pair.take_tagged_one::<AttributeItemNode>(Cow::Borrowed("attribute_item"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for AttributeCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AttributeItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            continuation: pair.take_tagged_option::<ContinuationNode>(Cow::Borrowed("continuation")),
            identifier: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("identifier")).collect::<Result<Vec<_>, _>>()?,
            namepath: pair.take_tagged_one::<NamepathNode>(Cow::Borrowed("namepath"))?,
            tuple_literal: pair.take_tagged_option::<TupleLiteralNode>(Cow::Borrowed("tuple_literal")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for AttributeItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for AttributeNameNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for AttributeNameNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::AttributeName)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ProceduralCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            continuation: pair.take_tagged_option::<ContinuationNode>(Cow::Borrowed("continuation")),
            namepath: pair.take_tagged_one::<NamepathNode>(Cow::Borrowed("namepath"))?,
            tuple_literal: pair.take_tagged_option::<TupleLiteralNode>(Cow::Borrowed("tuple_literal")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ProceduralCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ProceduralCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ProceduralNameNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ProceduralNameNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ProceduralName)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextLiteralNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_option::<IdentifierNode>(Cow::Borrowed("identifier")),
            text_raw: pair.take_tagged_one::<TextRawNode>(Cow::Borrowed("text_raw"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TextLiteralNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TextLiteral)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextRawNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            text_content_1: pair.take_tagged_option::<TextContent1Node>(Cow::Borrowed("text_content_1")),
            text_content_2: pair.take_tagged_option::<TextContent2Node>(Cow::Borrowed("text_content_2")),
            text_content_3: pair.take_tagged_option::<TextContent3Node>(Cow::Borrowed("text_content_3")),
            text_content_4: pair.take_tagged_option::<TextContent4Node>(Cow::Borrowed("text_content_4")),
            text_content_5: pair.take_tagged_option::<TextContent5Node>(Cow::Borrowed("text_content_5")),
            text_content_6: pair.take_tagged_option::<TextContent6Node>(Cow::Borrowed("text_content_6")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TextRawNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TextRaw)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextLNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextLNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Text_L)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextRNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextRNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Text_R)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextXNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextXNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Text_X)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextContent1Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextContent1Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT1)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextContent2Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextContent2Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT2)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextContent3Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextContent3Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT3)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextContent4Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextContent4Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT4)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextContent5Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextContent5Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT5)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TextContent6Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TextContent6Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEXT_CONTENT6)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ModifierCallNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ModifierCallNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ModifierCall)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ModifierAheadNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ModifierAheadNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ModifierAhead)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KeywordsStopNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KeywordsStopNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KEYWORDS_STOP)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IdentifierStopNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for IdentifierStopNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IDENTIFIER_STOP)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SlotNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            op_slot: pair.take_tagged_one::<OpSlotNode>(Cow::Borrowed("op_slot"))?,
            slot_item: pair.take_tagged_option::<SlotItemNode>(Cow::Borrowed("slot_item")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for SlotNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Slot)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SlotItemNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Identifier(s) => s.get_range(),
            Self::Integer(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<IdentifierNode>(Cow::Borrowed("identifier")) {
            return Ok(Self::Identifier(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("integer")) {
            return Ok(Self::Integer(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::SlotItem, _span))
    }
}
#[automatically_derived]
impl FromStr for SlotItemNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SlotItem)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NamepathFreeNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("identifier")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for NamepathFreeNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NamepathFree)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NamepathNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier: pair.take_tagged_items::<IdentifierNode>(Cow::Borrowed("identifier")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for NamepathNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Namepath)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IdentifierNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::IdentifierBare(s) => s.get_range(),
            Self::IdentifierRaw(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<IdentifierBareNode>(Cow::Borrowed("identifier_bare")) {
            return Ok(Self::IdentifierBare(s));
        }
        if let Ok(s) = pair.take_tagged_one::<IdentifierRawNode>(Cow::Borrowed("identifier_raw")) {
            return Ok(Self::IdentifierRaw(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Identifier, _span))
    }
}
#[automatically_derived]
impl FromStr for IdentifierNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Identifier)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IdentifierBareNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for IdentifierBareNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IdentifierBare)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IdentifierRawNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            identifier_raw_text: pair.take_tagged_one::<IdentifierRawTextNode>(Cow::Borrowed("identifier_raw_text"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for IdentifierRawNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IdentifierRaw)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IdentifierRawTextNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for IdentifierRawTextNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::IdentifierRawText)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SpecialNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for SpecialNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Special)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NumberNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Decimal(s) => s.get_range(),
            Self::DecimalX(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<DecimalNode>(Cow::Borrowed("decimal")) {
            return Ok(Self::Decimal(s));
        }
        if let Ok(s) = pair.take_tagged_one::<DecimalXNode>(Cow::Borrowed("decimal_x")) {
            return Ok(Self::DecimalX(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Number, _span))
    }
}
#[automatically_derived]
impl FromStr for NumberNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Number)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SignNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::Netative => Range::default(),
            Self::Positive => Range::default(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Some(_) = pair.find_first_tag("netative") {
            return Ok(Self::Netative);
        }
        if let Some(_) = pair.find_first_tag("positive") {
            return Ok(Self::Positive);
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::Sign, _span))
    }
}
#[automatically_derived]
impl FromStr for SignNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Sign)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for IntegerNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for IntegerNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Integer)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DigitsXNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for DigitsXNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DigitsX)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DecimalNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            dot: pair.take_tagged_option::<DotNode>(Cow::Borrowed("dot")),
            lhs: pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("lhs"))?,
            rhs: pair.take_tagged_option::<IntegerNode>(Cow::Borrowed("rhs")),
            shift: pair.take_tagged_option::<IntegerNode>(Cow::Borrowed("shift")),
            sign: pair.take_tagged_option::<SignNode>(Cow::Borrowed("sign")),
            unit: pair.take_tagged_option::<IdentifierNode>(Cow::Borrowed("unit")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DecimalNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Decimal)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DecimalXNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            base: pair.take_tagged_one::<IntegerNode>(Cow::Borrowed("base"))?,
            dot: pair.take_tagged_option::<DotNode>(Cow::Borrowed("dot")),
            lhs: pair.take_tagged_one::<DigitsXNode>(Cow::Borrowed("lhs"))?,
            rhs: pair.take_tagged_option::<DigitsXNode>(Cow::Borrowed("rhs")),
            shift: pair.take_tagged_option::<IntegerNode>(Cow::Borrowed("shift")),
            sign: pair.take_tagged_option::<SignNode>(Cow::Borrowed("sign")),
            unit: pair.take_tagged_option::<IdentifierNode>(Cow::Borrowed("unit")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for DecimalXNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DecimalX)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ProportionNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for ProportionNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PROPORTION)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for NsConcatNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for NsConcatNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::NS_CONCAT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ColonNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for ColonNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::COLON)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for Arrow1Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for Arrow1Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ARROW1)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for CommaNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for CommaNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::COMMA)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for DotNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for DotNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::DOT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OpSlotNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for OpSlotNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_SLOT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OffsetLNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for OffsetLNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OFFSET_L)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OffsetRNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for OffsetRNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OFFSET_R)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for Proportion2Node {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for Proportion2Node {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::PROPORTION2)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OpImportAllNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for OpImportAllNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_IMPORT_ALL)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OpAndThenNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for OpAndThenNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_AND_THEN)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for OpBindNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for OpBindNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::OP_BIND)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwControlNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwControlNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CONTROL)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwNamespaceNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwNamespaceNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_NAMESPACE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwImportNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwImportNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IMPORT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwConstraintNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwConstraintNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CONSTRAINT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwWhereNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwWhereNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WHERE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwImplementsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwImplementsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IMPLEMENTS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwExtendsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwExtendsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_EXTENDS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwInheritsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwInheritsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_INHERITS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwForNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwForNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_FOR)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwEndNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwEndNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_END)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwLetNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwLetNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_LET)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwNewNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwNewNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_NEW)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwObjectNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwObjectNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_OBJECT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwLambdaNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwLambdaNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_LAMBDA)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwIfNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwIfNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IF)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwSwitchNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwSwitchNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_SWITCH)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwTryNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwTryNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_TRY)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwTypeNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwTypeNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_TYPE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwCaseNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwCaseNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_CASE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwWhenNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwWhenNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_WHEN)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwElseNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwElseNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_ELSE)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwNotNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwNotNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_NOT)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwInNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwInNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IN)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwIsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwIsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_IS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for KwAsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for KwAsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::KW_AS)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ShebangNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for ShebangNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Shebang)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for WhiteSpaceNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for WhiteSpaceNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::WhiteSpace)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for SkipSpaceNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for SkipSpaceNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::SkipSpace)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for CommentNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for CommentNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::Comment)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringInterpolationsNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            string_interpolation_term: pair
                .take_tagged_items::<StringInterpolationTermNode>(Cow::Borrowed("string_interpolation_term"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for StringInterpolationsNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolations)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringInterpolationTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::EscapeCharacter(s) => s.get_range(),
            Self::EscapeUnicode(s) => s.get_range(),
            Self::StringInterpolationComplex(s) => s.get_range(),
            Self::StringInterpolationSimple(s) => s.get_range(),
            Self::StringInterpolationText(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<EscapeCharacterNode>(Cow::Borrowed("escape_character")) {
            return Ok(Self::EscapeCharacter(s));
        }
        if let Ok(s) = pair.take_tagged_one::<EscapeUnicodeNode>(Cow::Borrowed("escape_unicode")) {
            return Ok(Self::EscapeUnicode(s));
        }
        if let Ok(s) = pair.take_tagged_one::<StringInterpolationComplexNode>(Cow::Borrowed("string_interpolation_complex")) {
            return Ok(Self::StringInterpolationComplex(s));
        }
        if let Ok(s) = pair.take_tagged_one::<StringInterpolationSimpleNode>(Cow::Borrowed("string_interpolation_simple")) {
            return Ok(Self::StringInterpolationSimple(s));
        }
        if let Ok(s) = pair.take_tagged_one::<StringInterpolationTextNode>(Cow::Borrowed("string_interpolation_text")) {
            return Ok(Self::StringInterpolationText(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::StringInterpolationTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for StringInterpolationTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EscapeCharacterNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for EscapeCharacterNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EscapeCharacter)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EscapeUnicodeNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            code: pair.take_tagged_one::<EscapeUnicodeCodeNode>(Cow::Borrowed("code"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for EscapeUnicodeNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EscapeUnicode)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for EscapeUnicodeCodeNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for EscapeUnicodeCodeNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::EscapeUnicodeCode)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringInterpolationSimpleNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_expression: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("main_expression"))?,
            string_formatter: pair.take_tagged_option::<StringFormatterNode>(Cow::Borrowed("string_formatter")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for StringInterpolationSimpleNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationSimple)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringInterpolationTextNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for StringInterpolationTextNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationText)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringFormatterNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { text: pair.get_string(), span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for StringFormatterNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringFormatter)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringInterpolationComplexNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_expression: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("main_expression"))?,
            tuple_pair: pair.take_tagged_items::<TuplePairNode>(Cow::Borrowed("tuple_pair")).collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for StringInterpolationComplexNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringInterpolationComplex)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringTemplatesNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            string_template_term: pair
                .take_tagged_items::<StringTemplateTermNode>(Cow::Borrowed("string_template_term"))
                .collect::<Result<Vec<_>, _>>()?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for StringTemplatesNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringTemplates)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for StringTemplateTermNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        match self {
            Self::ExpressionTemplate(s) => s.get_range(),
            Self::ForTemplate(s) => s.get_range(),
        }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        if let Ok(s) = pair.take_tagged_one::<ExpressionTemplateNode>(Cow::Borrowed("expression_template")) {
            return Ok(Self::ExpressionTemplate(s));
        }
        if let Ok(s) = pair.take_tagged_one::<ForTemplateNode>(Cow::Borrowed("for_template")) {
            return Ok(Self::ForTemplate(s));
        }
        Err(YggdrasilError::invalid_node(ValkyrieRule::StringTemplateTerm, _span))
    }
}
#[automatically_derived]
impl FromStr for StringTemplateTermNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::StringTemplateTerm)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ExpressionTemplateNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            main_expression: pair.take_tagged_one::<MainExpressionNode>(Cow::Borrowed("main_expression"))?,
            template_e: pair.take_tagged_one::<TemplateENode>(Cow::Borrowed("template_e"))?,
            template_s: pair.take_tagged_one::<TemplateSNode>(Cow::Borrowed("template_s"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ExpressionTemplateNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ExpressionTemplate)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ForTemplateNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            for_template_begin: pair.take_tagged_one::<ForTemplateBeginNode>(Cow::Borrowed("for_template_begin"))?,
            for_template_else: pair.take_tagged_option::<ForTemplateElseNode>(Cow::Borrowed("for_template_else")),
            for_template_end: pair.take_tagged_one::<ForTemplateEndNode>(Cow::Borrowed("for_template_end"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ForTemplateNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplate)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ForTemplateBeginNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            if_guard: pair.take_tagged_one::<IfGuardNode>(Cow::Borrowed("if_guard"))?,
            inline_expression: pair.take_tagged_option::<InlineExpressionNode>(Cow::Borrowed("inline_expression")),
            kw_for: pair.take_tagged_one::<KwForNode>(Cow::Borrowed("kw_for"))?,
            kw_in: pair.take_tagged_one::<KwInNode>(Cow::Borrowed("kw_in"))?,
            let_pattern: pair.take_tagged_one::<LetPatternNode>(Cow::Borrowed("let_pattern"))?,
            template_e: pair.take_tagged_one::<TemplateENode>(Cow::Borrowed("template_e"))?,
            template_s: pair.take_tagged_one::<TemplateSNode>(Cow::Borrowed("template_s"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ForTemplateBeginNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplateBegin)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ForTemplateElseNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            kw_else: pair.take_tagged_one::<KwElseNode>(Cow::Borrowed("kw_else"))?,
            template_e: pair.take_tagged_one::<TemplateENode>(Cow::Borrowed("template_e"))?,
            template_s: pair.take_tagged_one::<TemplateSNode>(Cow::Borrowed("template_s"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ForTemplateElseNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplateElse)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for ForTemplateEndNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            kw_end: pair.take_tagged_one::<KwEndNode>(Cow::Borrowed("kw_end"))?,
            kw_for: pair.take_tagged_option::<KwForNode>(Cow::Borrowed("kw_for")),
            template_e: pair.take_tagged_one::<TemplateENode>(Cow::Borrowed("template_e"))?,
            template_s: pair.take_tagged_one::<TemplateSNode>(Cow::Borrowed("template_s"))?,
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for ForTemplateEndNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::ForTemplateEnd)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TemplateSNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            template_m: pair.take_tagged_option::<TemplateMNode>(Cow::Borrowed("template_m")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TemplateSNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_S)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TemplateENode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self {
            template_m: pair.take_tagged_option::<TemplateMNode>(Cow::Borrowed("template_m")),
            span: Range { start: _span.start() as u32, end: _span.end() as u32 },
        })
    }
}
#[automatically_derived]
impl FromStr for TemplateENode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_E)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TemplateLNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TemplateLNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_L)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TemplateRNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TemplateRNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_R)?)
    }
}
#[automatically_derived]
impl YggdrasilNode for TemplateMNode {
    type Rule = ValkyrieRule;

    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
    fn from_pair(pair: TokenPair<Self::Rule>) -> Result<Self, YggdrasilError<Self::Rule>> {
        let _span = pair.get_span();
        Ok(Self { span: Range { start: _span.start() as u32, end: _span.end() as u32 } })
    }
}
#[automatically_derived]
impl FromStr for TemplateMNode {
    type Err = YggdrasilError<ValkyrieRule>;

    fn from_str(input: &str) -> Result<Self, YggdrasilError<ValkyrieRule>> {
        Self::from_cst(ValkyrieParser::parse_cst(input, ValkyrieRule::TEMPLATE_M)?)
    }
}
