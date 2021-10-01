use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use miette::{Diagnostic, LabeledSpan as MietteLabeledSpan, Severity};

mod control_flow;
mod match_patterns;

use crate::{
    ast::{
        Annotations, AttributeArgument, AttributeDeclaration, AttributeItem, AttributeList, BinaryOperator, ClassDeclaration, DeclarationBody,
        DeclarationStatement, FunctionDeclaration, FunctionParameter, GenericParameterDeclaration, IdentifierNode, IfStatement,
        ImplyAssociatedConstBinding, ImplyAssociatedTypeBinding, ImplyDeclaration, InheritanceItem, LiteralExpression, LoopStatement, NamePath,
        NamespaceDeclaration, ObjectBody, ObjectFieldDeclaration, ObjectMethodDeclaration, Statement, StringLiteral, StringSegment,
        SubscriptKind, TermAsExpression, TermBinaryExpression, TermExpression, TermUnaryExpression, TraitAssociatedConstDeclaration,
        TraitAssociatedTypeDeclaration, TraitDeclaration, TypeAliasDeclaration, TypeExpression, TypePath, UnaryOperator, UniteDeclaration,
        UniteVariantDeclaration, UsingStatement, ValkyrieRoot, WhereConstraintDeclaration,
    },
    lexer::{Keyword, Lexer, Token, TokenKind},
};
use std::ops::Range;

const DECLARATION_MODIFIERS: &[&str] = &[
    "public",
    "private",
    "protected",
    "internal",
    "open",
    "sealed",
    "abstract",
    "final",
    "readonly",
    "virtual",
    "override",
    "static",
    "get",
    "set",
    "unsafe",
];

/// Parser-side error type.
#[derive(Debug)]
pub enum ParseError {
    /// File read failure.
    Io(std::io::Error),
    /// Syntactically invalid source text.
    Invalid {
        /// 错误消息。
        message: String,
        /// 可选源码范围。
        span: Option<Range<usize>>,
    },
}

impl ParseError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid { message: message.into(), span: None }
    }

    pub fn invalid_at(message: impl Into<String>, span: Range<usize>) -> Self {
        Self::Invalid { message: message.into(), span: Some(span) }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, f),
            Self::Invalid { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for ParseError {}

impl Diagnostic for ParseError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(match self {
            ParseError::Io(_) => "valkyrie::parser::io",
            ParseError::Invalid { .. } => "valkyrie::parser::invalid",
        }))
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(match self {
            ParseError::Io(_) => "请确认源文件存在且当前进程具备读取权限",
            ParseError::Invalid { .. } => "请检查语法是否完整，尤其是声明头、括号、属性与对象体",
        }))
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = MietteLabeledSpan> + '_>> {
        match self {
            ParseError::Invalid { span: Some(span), .. } => {
                let labeled =
                    MietteLabeledSpan::new_with_span(Some("解析失败位置".to_string()), (span.start, span.end.saturating_sub(span.start)));
                Some(Box::new(std::iter::once(labeled)))
            }
            _ => None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    index: usize,
    /// 当为 true 时，`{` 不作为结构体构造 postfix 处理（用于 if/while/loop/match 条件解析）。
    suppress_struct_constructor: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Self { source, tokens, index: 0, suppress_struct_constructor: false }
    }

    fn parse_root(&mut self) -> Result<ValkyrieRoot, ParseError> {
        let mut statements = Vec::new();
        while !self.is_eof() {
            if self.match_symbol(TokenKind::Semicolon) {
                continue;
            }

            let annotations = self.parse_annotations()?;
            if self.is_eof() {
                if annotations.attribute_lists.is_empty() && annotations.modifiers.is_empty() {
                    break;
                }
                return Err(ParseError::invalid("dangling attributes or modifiers at end of file"));
            }

            statements.push(self.parse_declaration(annotations)?);
        }
        Ok(ValkyrieRoot { statements })
    }

    fn parse_annotations(&mut self) -> Result<Annotations, ParseError> {
        let mut attribute_lists = Vec::new();
        let mut modifiers = Vec::new();

        while self.check_symbol(TokenKind::LBracket) {
            attribute_lists.push(self.parse_attribute_list()?);
        }

        while let Some(text) = self.current_modifier_text() {
            if !DECLARATION_MODIFIERS.contains(&text) {
                break;
            }
            let modifier_start = self.current().span.start;
            let modifier_text = text.to_string();
            self.bump();
            modifiers
                .push(IdentifierNode::new(valkyrie_types::Identifier::new(&modifier_text), span(modifier_start, self.previous().span.end)));
        }

        Ok(Annotations { documents: Vec::new(), attribute_lists, modifiers })
    }

    fn parse_declaration(&mut self, annotations: Annotations) -> Result<DeclarationStatement, ParseError> {
        if self.check_token_keyword(Keyword::Namespace) {
            return Ok(DeclarationStatement::Namespace(self.parse_namespace()?));
        }
        if self.check_token_keyword(Keyword::Using) {
            return Ok(DeclarationStatement::Using(self.parse_using()?));
        }
        if self.check_token_keyword(Keyword::Micro) {
            return Ok(DeclarationStatement::Function(self.parse_function_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Class) || self.check_token_keyword(Keyword::Structure) {
            return Ok(DeclarationStatement::Class(self.parse_class_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Trait) {
            return Ok(DeclarationStatement::Trait(self.parse_trait_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Imply) {
            return Ok(DeclarationStatement::Imply(self.parse_imply_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Unite) {
            return Ok(DeclarationStatement::Unite(self.parse_unite_declaration(annotations)?));
        }
        if self.check_identifier_text_eq("attribute") {
            return Ok(DeclarationStatement::Attribute(self.parse_attribute_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Type) {
            return Ok(DeclarationStatement::TypeAlias(self.parse_type_alias_declaration(annotations)?));
        }
        if self.check_identifier_text_eq("extern") {
            return Ok(DeclarationStatement::Function(self.parse_extern_function_declaration(annotations)?));
        }
        Err(self.error_here("expected declaration"))
    }

    fn parse_namespace(&mut self) -> Result<NamespaceDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Namespace)?.span.start;
        // 支持 `namespace!` 宏式语法，`!` 被消费后按普通 `namespace` 处理。
        self.match_symbol(TokenKind::Bang);
        // 源码使用 `.` 作为命名空间路径分隔符（如 `namespace std.data.text.wit;`）。
        let name = self.parse_dotted_name_path()?;
        let body = if self.check_symbol(TokenKind::LBrace) {
            let open_start = self.bump().span.start;
            let mut statements = Vec::new();
            let tail_expression = None;

            while !self.check_symbol(TokenKind::RBrace) {
                if self.is_eof() {
                    return Err(ParseError::invalid("unterminated namespace body"));
                }
                if self.match_symbol(TokenKind::Semicolon) {
                    continue;
                }
                let decl = self.parse_declaration(Annotations::default())?;
                match decl {
                    DeclarationStatement::Function(func) => {
                        let path = NamePath { parts: vec![func.name.as_str().to_string()], span: func.span.clone() };
                        statements.push(Statement::Expr {
                            span: func.span.clone(),
                            expression: TermExpression::Name { path, span: func.span.clone() },
                        });
                    }
                    _ => {
                        return Err(self.error_here("unexpected declaration in namespace body"));
                    }
                }
            }
            let close_end = self.expect_symbol(TokenKind::RBrace)?.span.end;
            Some(DeclarationBody { statements, tail_expression, span: span(open_start, close_end) })
        }
        else {
            self.expect_symbol(TokenKind::Semicolon)?;
            None
        };
        Ok(NamespaceDeclaration { name, body, span: span(start, self.previous().span.end) })
    }

    fn parse_using(&mut self) -> Result<UsingStatement, ParseError> {
        let start = self.expect_token_keyword(Keyword::Using)?.span.start;
        // 支持 `using!` 宏式语法，`!` 被消费后按普通 `using` 处理。
        self.match_symbol(TokenKind::Bang);
        let path = self.parse_dotted_name_path()?;

        // 支持选择性导入：`using a.b.{C, D};`
        // 支持通配导入：`using a.b.*;`
        // 同时保留裸导入：`using a.b;`，其语义由后续绑定阶段决定。
        let mut selective_imports = Vec::new();
        let mut glob_import = false;
        if self.match_symbol(TokenKind::Dot) {
            if self.match_symbol(TokenKind::LBrace) {
                selective_imports = self.parse_comma_separated_until(TokenKind::RBrace, |parser| {
                    let name = parser.expect_identifier_text()?.to_string();
                    Ok(name)
                })?;
                self.expect_symbol(TokenKind::RBrace)?;
            }
            else if self.match_symbol(TokenKind::Star) {
                glob_import = true;
            }
            else {
                return Err(self.error_here("expected `{` or `*` after `using <module>.`"));
            }
        }

        // 分号可选：`using!` 宏式语法常省略分号，以下一个声明边界作为隐式终止。
        self.match_symbol(TokenKind::Semicolon);
        Ok(UsingStatement { path, selective_imports, glob_import, span: span(start, self.previous().span.end) })
    }

    fn parse_function_declaration(&mut self, annotations: Annotations) -> Result<FunctionDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Micro)?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        self.skip_generic_parameter_clause()?;
        let params = self.parse_parameter_list()?;
        let return_type = if self.match_symbol(TokenKind::Arrow) || self.match_symbol(TokenKind::Colon) {
            Some(self.parse_type_expression_bp(0)?)
        }
        else {
            None
        };

        // 解析可选的 `where` 子句，当前仅消费不存储（自举阶段不做类型检查）。
        let _ = self.parse_where_constraints()?;

        let signature_end = self.current().span.start;
        let body = if self.check_symbol(TokenKind::LBrace) {
            Some(self.parse_block_body()?)
        }
        else {
            // 允许函数声明省略分号，以下一个声明边界作为隐式终止。
            self.expect_implicit_or_explicit_terminator(&["micro", "class", "structure", "trait", "imply", "unite", "namespace", "using"])?;
            None
        };

        Ok(FunctionDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            annotations,
            signature: self.slice(span(start, signature_end)).trim().to_string(),
            params,
            return_type,
            body,
            span: span(start, self.previous().span.end),
        })
    }

    /// 解析 `extern` 函数声明。
    ///
    /// 语法：`extern name(params): return_type`
    /// 无 `micro` 关键字，无函数体，用于声明外部导入函数（如 WASI 接口）。
    fn parse_extern_function_declaration(&mut self, annotations: Annotations) -> Result<FunctionDeclaration, ParseError> {
        let start = self.expect_identifier_text_eq("extern")?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        self.skip_generic_parameter_clause()?;
        let params = self.parse_parameter_list()?;
        let return_type = if self.match_symbol(TokenKind::Arrow) || self.match_symbol(TokenKind::Colon) {
            Some(self.parse_type_expression_bp(0)?)
        }
        else {
            None
        };
        let signature_end = self.current().span.start;

        Ok(FunctionDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            annotations,
            signature: self.slice(span(start, signature_end)).trim().to_string(),
            params,
            return_type,
            body: None,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_class_declaration(&mut self, annotations: Annotations) -> Result<ClassDeclaration, ParseError> {
        // `structure` 与 `class` 共用同一解析路径，区别仅在语义（值类型 vs 引用类型）。
        // `structure` 关键字声明值类型，`class` 关键字声明引用类型。
        let (start, is_value_type) = if self.check_token_keyword(Keyword::Class) {
            (self.expect_token_keyword(Keyword::Class)?.span.start, false)
        }
        else {
            (self.expect_token_keyword(Keyword::Structure)?.span.start, true)
        };
        let name_start = self.current().span.start;
        let name = self.expect_name_text()?;
        self.skip_generic_parameter_clause()?;
        let inheritance = if self.match_symbol(TokenKind::LParen) {
            let items = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_inheritance_item())?;
            self.expect_symbol(TokenKind::RParen)?;
            items
        }
        else {
            Vec::new()
        };
        let body = self.parse_object_body(true)?;
        Ok(ClassDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            annotations,
            inheritance,
            body,
            is_value_type,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_trait_declaration(&mut self, annotations: Annotations) -> Result<TraitDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Trait)?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_name_text()?;
        let generic_parameters = self.parse_generic_parameter_clause()?;
        if self.match_symbol(TokenKind::Equal) {
            let alias_targets = self.parse_trait_inheritance_list()?;
            self.expect_implicit_or_explicit_terminator(&["namespace", "using", "micro", "class", "trait", "imply", "unite"])?;
            return Ok(TraitDeclaration {
                name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
                annotations,
                generic_parameters,
                inheritance: Vec::new(),
                alias_targets,
                is_alias: true,
                body: ObjectBody::default(),
                span: span(start, self.previous().span.end),
            });
        }

        let inheritance = if self.match_symbol(TokenKind::Colon) { self.parse_trait_inheritance_list()? } else { Vec::new() };
        let body = self.parse_object_body(false)?;
        Ok(TraitDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            annotations,
            generic_parameters,
            inheritance,
            alias_targets: Vec::new(),
            is_alias: false,
            body,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_imply_declaration(&mut self, annotations: Annotations) -> Result<ImplyDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Imply)?.span.start;
        let generic_parameters = self.parse_structured_generic_parameter_clause()?;
        let target_type = self.parse_type_expression_bp(0)?;
        let trait_type = if self.match_symbol(TokenKind::Colon) { Some(self.parse_type_expression_bp(0)?) } else { None };
        let where_constraints = self.parse_where_constraints()?;
        let (methods, associated_type_bindings, associated_const_bindings) = self.parse_imply_body()?;

        Ok(ImplyDeclaration {
            annotations,
            generic_parameters,
            target_type,
            trait_type,
            where_constraints,
            methods,
            associated_type_bindings,
            associated_const_bindings,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_object_body(&mut self, allow_fields: bool) -> Result<ObjectBody, ParseError> {
        self.expect_symbol(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut associated_types = Vec::new();
        let mut associated_constants = Vec::new();

        while !self.check_symbol(TokenKind::RBrace) {
            let annotations = self.parse_annotations()?;
            if self.check_symbol(TokenKind::RBrace) {
                break;
            }

            if self.check_token_keyword(Keyword::Micro)
                || self.check_identifier_text_eq("infix")
                || self.check_identifier_text_eq("prefix")
                || self.check_identifier_text_eq("postfix")
                || self.check_identifier_text_eq("suffix")
                || annotations.modifiers.iter().any(|modifier| modifier.as_str() == "get" || modifier.as_str() == "set")
            {
                methods.push(self.parse_object_method_declaration(annotations)?);
                continue;
            }

            if !allow_fields && self.check_token_keyword(Keyword::Type) {
                associated_types.push(self.parse_trait_associated_type(annotations)?);
                continue;
            }

            if !allow_fields && self.check_token_keyword(Keyword::Const) {
                associated_constants.push(self.parse_trait_associated_const(annotations)?);
                continue;
            }

            // 当允许字段时，标识符后跟 `(` 是方法，后跟 `:` 或 `=` 是字段。
            if allow_fields && matches!(self.current().kind, TokenKind::Identifier) {
                if matches!(self.peek().kind, TokenKind::LParen) {
                    methods.push(self.parse_object_method_declaration(annotations)?);
                }
                else {
                    fields.push(self.parse_object_field(annotations)?);
                }
                continue;
            }

            // 当不允许字段时（trait 体），标识符是方法。
            if !allow_fields
                && matches!(self.current().kind, TokenKind::Identifier)
                && !self.check_token_keyword(Keyword::Type)
                && !self.check_token_keyword(Keyword::Const)
            {
                methods.push(self.parse_object_method_declaration(annotations)?);
                continue;
            }

            return Err(self.error_here("expected object field or method"));
        }

        self.expect_symbol(TokenKind::RBrace)?;
        Ok(ObjectBody { fields, methods, associated_types, associated_constants })
    }

    fn parse_object_field(&mut self, annotations: Annotations) -> Result<ObjectFieldDeclaration, ParseError> {
        let start = self.current().span.start;
        let name_start = self.current().span.start;
        let name_text = self.expect_identifier_text()?.to_string();
        let name = IdentifierNode::new(valkyrie_types::Identifier::new(&name_text), span(name_start, self.previous().span.end));
        self.expect_symbol(TokenKind::Colon)?;
        let field_type = self.parse_type_expression_bp(0)?;
        let default_value = if self.match_symbol(TokenKind::Equal) {
            let value = self.parse_expression_bp(0)?;
            Some(value)
        }
        else {
            None
        };
        // 对象字段可省略分号，以换行分隔。下一个字段以标识符开头，或以 `}` 结束。
        // 也支持逗号分隔的字段列表。
        if !self.match_symbol(TokenKind::Semicolon)
            && !self.match_symbol(TokenKind::Comma)
            && !self.check_symbol(TokenKind::RBrace)
            && !self.is_eof()
            && !self.check_symbol(TokenKind::LBracket)
            && !matches!(self.current().kind, TokenKind::Identifier)
        {
            return Err(self.error_here("expected ';' or '}' or next field after object field"));
        }

        Ok(ObjectFieldDeclaration { annotations, name, field_type, default_value, span: span(start, self.previous().span.end) })
    }

    fn parse_object_method_declaration(&mut self, annotations: Annotations) -> Result<ObjectMethodDeclaration, ParseError> {
        let is_accessor = annotations.modifiers.iter().any(|modifier| modifier.as_str() == "get" || modifier.as_str() == "set");
        let is_operator = self.check_identifier_text_eq("infix")
            || self.check_identifier_text_eq("prefix")
            || self.check_identifier_text_eq("postfix")
            || self.check_identifier_text_eq("suffix");
        // 支持不带 `micro` 前缀的普通方法名（如 `bit_and(self, ...)`）。
        let is_plain_method = matches!(self.current().kind, TokenKind::Identifier)
            && !self.check_token_keyword(Keyword::Micro)
            && !self.check_token_keyword(Keyword::Type)
            && !self.check_token_keyword(Keyword::Const)
            && !self.check_identifier_text_eq("infix")
            && !self.check_identifier_text_eq("prefix")
            && !self.check_identifier_text_eq("postfix")
            && !self.check_identifier_text_eq("suffix");
        let start = if is_accessor || is_operator || is_plain_method {
            self.current().span.start
        }
        else {
            self.expect_token_keyword(Keyword::Micro)?.span.start
        };
        let name = if is_accessor {
            let name_start = self.current().span.start;
            let name_text = self.expect_identifier_text()?.to_string();
            IdentifierNode::new(valkyrie_types::Identifier::new(&name_text), span(name_start, self.previous().span.end))
        }
        else {
            let parsed_name = self.parse_method_name()?;
            IdentifierNode::new(valkyrie_types::Identifier::new(&parsed_name), span(start, self.previous().span.end))
        };
        self.skip_generic_parameter_clause()?;
        let params = self.parse_parameter_list()?;
        let return_type = if self.match_symbol(TokenKind::Arrow) || self.match_symbol(TokenKind::Colon) {
            Some(self.parse_type_expression_bp(0)?)
        }
        else {
            None
        };

        // 解析可选的 `where` 子句，当前仅消费不存储（自举阶段不做类型检查）。
        let _ = self.parse_where_constraints()?;

        let signature_end = self.current().span.start;
        let body = if self.check_symbol(TokenKind::LBrace) {
            Some(self.parse_block_body()?)
        }
        else {
            self.expect_implicit_or_explicit_terminator(&["micro", "type", "const", "infix", "prefix", "postfix", "suffix"])?;
            None
        };

        Ok(ObjectMethodDeclaration {
            name,
            annotations,
            signature: self.slice(span(start, signature_end)).trim().to_string(),
            params,
            return_type,
            body,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_method_name(&mut self) -> Result<String, ParseError> {
        if self.check_identifier_text_eq("infix")
            || self.check_identifier_text_eq("prefix")
            || self.check_identifier_text_eq("postfix")
            || self.check_identifier_text_eq("suffix")
        {
            let fixity = self.expect_identifier_text()?.to_string();
            let operator = self.parse_operator_method_symbol()?;
            return Ok(format!("{fixity} {operator}"));
        }

        Ok(self.expect_identifier_text()?.to_string())
    }

    /// 解析运算符方法符号。
    ///
    /// 支持两种形式：
    /// - 预定义运算符 token（如 `+`、`==`）
    /// - 反引号包裹的任意运算符（如 `` `+=` ``、`` `>>=` ``）
    fn parse_operator_method_symbol(&mut self) -> Result<String, ParseError> {
        if matches!(self.current().kind, TokenKind::BacktickSymbol) {
            let span = self.bump().span.clone();
            let text = self.slice(span);
            // 剥离首尾反引号，提取运算符文本。
            let trimmed = text.trim_start_matches('`').trim_end_matches('`');
            return Ok(trimmed.to_string());
        }

        let operator = match self.current().kind {
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Bang => "!",
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::LAngle => "<",
            TokenKind::RAngle => ">",
            TokenKind::LessEq => "<=",
            TokenKind::GreaterEq => ">=",
            _ => {
                return Err(self.error_here("expected operator symbol"));
            }
        };
        self.bump();
        Ok(operator.to_string())
    }

    fn parse_imply_body(
        &mut self,
    ) -> Result<(Vec<ObjectMethodDeclaration>, Vec<ImplyAssociatedTypeBinding>, Vec<ImplyAssociatedConstBinding>), ParseError> {
        self.expect_symbol(TokenKind::LBrace)?;
        let mut methods = Vec::new();
        let mut associated_type_bindings = Vec::new();
        let mut associated_const_bindings = Vec::new();

        while !self.check_symbol(TokenKind::RBrace) {
            let annotations = self.parse_annotations()?;
            if self.check_symbol(TokenKind::RBrace) {
                break;
            }

            if self.check_token_keyword(Keyword::Micro)
                || self.check_identifier_text_eq("infix")
                || self.check_identifier_text_eq("prefix")
                || self.check_identifier_text_eq("postfix")
                || self.check_identifier_text_eq("suffix")
                || (matches!(self.current().kind, TokenKind::Identifier)
                    && !self.check_token_keyword(Keyword::Type)
                    && !self.check_token_keyword(Keyword::Const))
            {
                methods.push(self.parse_object_method_declaration(annotations)?);
                continue;
            }

            if self.check_token_keyword(Keyword::Type) {
                associated_type_bindings.push(self.parse_imply_associated_type_binding(annotations)?);
                continue;
            }

            if self.check_token_keyword(Keyword::Const) {
                associated_const_bindings.push(self.parse_imply_associated_const_binding(annotations)?);
                continue;
            }

            return Err(self.error_here("expected imply method or associated member binding"));
        }

        self.expect_symbol(TokenKind::RBrace)?;
        Ok((methods, associated_type_bindings, associated_const_bindings))
    }

    fn parse_imply_associated_type_binding(&mut self, annotations: Annotations) -> Result<ImplyAssociatedTypeBinding, ParseError> {
        let start = self.expect_token_keyword(Keyword::Type)?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        let generic_parameters = self.parse_structured_generic_parameter_clause()?;
        self.expect_symbol(TokenKind::Equal)?;
        let concrete_type = self.parse_type_expression_bp(0)?;
        self.expect_implicit_or_explicit_terminator(&["micro", "type", "const"])?;

        Ok(ImplyAssociatedTypeBinding {
            annotations,
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            generic_parameters,
            concrete_type,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_imply_associated_const_binding(&mut self, annotations: Annotations) -> Result<ImplyAssociatedConstBinding, ParseError> {
        let start = self.expect_token_keyword(Keyword::Const)?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        let const_type = if self.match_symbol(TokenKind::Colon) { Some(self.parse_type_expression_bp(0)?) } else { None };
        self.expect_symbol(TokenKind::Equal)?;
        let value = self.parse_expression_bp(0)?;
        self.expect_implicit_or_explicit_terminator(&["micro", "type", "const"])?;

        Ok(ImplyAssociatedConstBinding {
            annotations,
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            const_type,
            value,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_trait_associated_type(&mut self, annotations: Annotations) -> Result<TraitAssociatedTypeDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Type)?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        let generic_parameters = self.parse_generic_parameter_clause()?;
        let bounds = if self.match_symbol(TokenKind::Colon) { self.parse_trait_bound_list()? } else { Vec::new() };
        let default_type = if self.match_symbol(TokenKind::Equal) { Some(self.parse_type_expression_bp(0)?) } else { None };
        self.expect_implicit_or_explicit_terminator(&["micro", "type", "const"])?;

        Ok(TraitAssociatedTypeDeclaration {
            annotations,
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            generic_parameters,
            bounds,
            default_type,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_trait_associated_const(&mut self, annotations: Annotations) -> Result<TraitAssociatedConstDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Const)?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        self.expect_symbol(TokenKind::Colon)?;
        let const_type = self.parse_type_expression_bp(0)?;
        let default_value = if self.match_symbol(TokenKind::Equal) { Some(self.parse_expression_bp(0)?) } else { None };
        self.expect_implicit_or_explicit_terminator(&["micro", "type", "const"])?;

        Ok(TraitAssociatedConstDeclaration {
            annotations,
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            const_type,
            default_value,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_unite_declaration(&mut self, annotations: Annotations) -> Result<UniteDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Unite)?.span.start;
        let name_start = self.current().span.start;
        let name_text = self.expect_identifier_text()?.to_string();
        let name = IdentifierNode::new(valkyrie_types::Identifier::new(&name_text), span(name_start, self.previous().span.end));
        self.skip_generic_parameter_clause()?;
        self.expect_symbol(TokenKind::LBrace)?;
        let mut variants = Vec::new();

        while !self.check_symbol(TokenKind::RBrace) {
            let variant_annotations = self.parse_annotations()?;
            if self.check_symbol(TokenKind::RBrace) {
                break;
            }
            variants.push(self.parse_unite_variant(variant_annotations)?);
        }

        self.expect_symbol(TokenKind::RBrace)?;
        Ok(UniteDeclaration { name, annotations, variants, span: span(start, self.previous().span.end) })
    }

    fn parse_unite_variant(&mut self, annotations: Annotations) -> Result<UniteVariantDeclaration, ParseError> {
        let start = self.current().span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        let result_type = if self.match_symbol(TokenKind::Colon) { Some(self.parse_type_expression_bp(0)?) } else { None };
        let mut fields = Vec::new();
        let mut tuple_types = Vec::new();

        // 元组变体：`name(T1, T2, ...)`，如 `Some(T)`、`Ok(T, E)`。
        if self.match_symbol(TokenKind::LParen) {
            if !self.check_symbol(TokenKind::RParen) {
                loop {
                    tuple_types.push(self.parse_type_expression_bp(0)?);
                    if !self.match_symbol(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect_symbol(TokenKind::RParen)?;
        }

        if self.match_symbol(TokenKind::LBrace) {
            while !self.check_symbol(TokenKind::RBrace) {
                let field_annotations = self.parse_annotations()?;
                if self.check_symbol(TokenKind::RBrace) {
                    break;
                }
                let field_start = self.current().span.start;
                let field_name_start = self.current().span.start;
                let field_name_text = self.expect_identifier_text()?.to_string();
                let field_name =
                    IdentifierNode::new(valkyrie_types::Identifier::new(&field_name_text), span(field_name_start, self.previous().span.end));
                self.expect_symbol(TokenKind::Colon)?;
                let field_type = self.parse_type_expression_bp(0)?;
                let default_value = if self.match_symbol(TokenKind::Equal) { Some(self.parse_expression_bp(0)?) } else { None };
                if self.match_symbol(TokenKind::Comma) || self.match_symbol(TokenKind::Semicolon) {}
                fields.push(ObjectFieldDeclaration {
                    annotations: field_annotations,
                    name: field_name,
                    field_type,
                    default_value,
                    span: span(field_start, self.previous().span.end),
                });
            }
            self.expect_symbol(TokenKind::RBrace)?;
        }

        // 消费可选的尾随逗号或分号。
        self.match_symbol(TokenKind::Comma);
        self.match_symbol(TokenKind::Semicolon);

        Ok(UniteVariantDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            annotations,
            fields,
            tuple_types,
            result_type,
            span: span(start, self.previous().span.end),
        })
    }

    /// 解析 `attribute name;` 标记属性声明。
    ///
    /// 语法：`attribute <identifier>;`
    /// 用于声明可在类型上使用的标记属性。
    fn parse_attribute_declaration(&mut self, _annotations: Annotations) -> Result<AttributeDeclaration, ParseError> {
        let start = self.expect_identifier_text_eq("attribute")?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        self.expect_symbol(TokenKind::Semicolon)?;
        Ok(AttributeDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            span: span(start, self.previous().span.end),
        })
    }

    /// 解析 `type Name = Target;` 类型别名声明。
    fn parse_type_alias_declaration(&mut self, _annotations: Annotations) -> Result<TypeAliasDeclaration, ParseError> {
        let start = self.expect_token_keyword(Keyword::Type)?.span.start;
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        self.expect_symbol(TokenKind::Equal)?;
        let target = self.parse_type_expression_bp(0)?;
        // 分号可选：某些源文件省略分号。
        self.match_symbol(TokenKind::Semicolon);
        Ok(TypeAliasDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            target,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_attribute_list(&mut self) -> Result<AttributeList, ParseError> {
        self.expect_symbol(TokenKind::LBracket)?;
        let items = self.parse_comma_separated_until(TokenKind::RBracket, |parser| parser.parse_attribute_item())?;
        self.expect_symbol(TokenKind::RBracket)?;
        Ok(AttributeList { items })
    }

    fn parse_attribute_item(&mut self) -> Result<AttributeItem, ParseError> {
        let start = self.current().span.start;
        let name = self.parse_name_path()?;
        let arguments = if self.match_symbol(TokenKind::LParen) {
            let args = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_attribute_argument())?;
            self.expect_symbol(TokenKind::RParen)?;
            args
        }
        else {
            Vec::new()
        };
        Ok(AttributeItem { name, arguments, span: span(start, self.previous().span.end) })
    }

    fn parse_attribute_argument(&mut self) -> Result<AttributeArgument, ParseError> {
        if matches!(self.current().kind, TokenKind::Identifier) && self.nth_is_symbol(1, TokenKind::Equal) {
            let key = self.expect_identifier_text()?.to_string();
            self.expect_symbol(TokenKind::Equal)?;
            let value = self.parse_expression_bp(0)?;
            return Ok(AttributeArgument { key: Some(key), value });
        }

        let value = self.parse_expression_bp(0)?;
        Ok(AttributeArgument { key: None, value })
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<FunctionParameter>, ParseError> {
        self.expect_symbol(TokenKind::LParen)?;
        let params = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_parameter())?;
        self.expect_symbol(TokenKind::RParen)?;
        Ok(params)
    }

    fn parse_parameter(&mut self) -> Result<FunctionParameter, ParseError> {
        let start = self.current().span.start;
        let is_mutable = self.match_token_keyword(Keyword::Mut);
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        let parameter_type = if self.match_symbol(TokenKind::Colon) { Some(self.parse_type_expression_bp(0)?) } else { None };

        Ok(FunctionParameter {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            parameter_type,
            is_mutable,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_inheritance_item(&mut self) -> Result<InheritanceItem, ParseError> {
        let start = self.current().span.start;
        let alias = if matches!(self.current().kind, TokenKind::Identifier) && self.nth_is_symbol(1, TokenKind::Colon) {
            let value = self.expect_identifier_text()?.to_string();
            self.expect_symbol(TokenKind::Colon)?;
            Some(value)
        }
        else {
            None
        };
        let base_type = self.parse_type_expression_bp(0)?;
        Ok(InheritanceItem { alias, base_type, span: span(start, self.previous().span.end) })
    }

    fn parse_trait_inheritance_list(&mut self) -> Result<Vec<InheritanceItem>, ParseError> {
        self.parse_separated_while(|parser| parser.parse_inheritance_item(), &[TokenKind::Comma, TokenKind::Plus])
    }

    fn parse_trait_bound_list(&mut self) -> Result<Vec<TypeExpression>, ParseError> {
        // trait bounds 只由 `+` 分隔（如 `T: Iterator + Clone`）。
        // 不包含 `,`，因为 `,` 用于分隔不同的约束或参数。
        self.parse_separated_while(|parser| parser.parse_type_expression_bp(0), &[TokenKind::Plus])
    }

    fn parse_name_path(&mut self) -> Result<NamePath, ParseError> {
        self.parse_name_path_with(false)
    }

    /// 解析 `using` 语句中的点分路径。
    ///
    /// Valkyrie 的 `using` 使用 `.` 作为模块路径分隔符
    /// （如 `using std.data.text.von;`），与表达式中的成员访问 `.` 不同。
    fn parse_dotted_name_path(&mut self) -> Result<NamePath, ParseError> {
        self.parse_name_path_with(true)
    }

    fn parse_name_path_with(&mut self, allow_dot: bool) -> Result<NamePath, ParseError> {
        let start = self.current().span.start;
        let mut parts = vec![self.expect_identifier_text()?.to_string()];
        loop {
            let is_colon = self.check_symbol(TokenKind::DoubleColon) && self.nth_is_identifier(1);
            let is_dot = allow_dot && self.check_symbol(TokenKind::Dot) && self.nth_is_identifier(1);
            if !is_colon && !is_dot {
                break;
            }
            self.bump();
            parts.push(self.expect_identifier_text()?.to_string());
        }
        Ok(NamePath { parts, span: span(start, self.previous().span.end) })
    }

    fn parse_type_expression_bp(&mut self, min_bp: u8) -> Result<TypeExpression, ParseError> {
        let mut lhs = self.parse_type_expression_prefix()?;
        loop {
            let Some((left_bp, _right_bp)) = self.type_postfix_binding_power()
            else {
                break;
            };
            if left_bp < min_bp {
                break;
            }
            lhs = self.parse_type_expression_postfix(lhs)?;
        }
        Ok(lhs)
    }

    /// 解析可能带命名字段的类型表达式。
    ///
    /// 支持 `name: Type` 形式（跳过字段名和 `:`，只解析类型），
    /// 以及普通类型表达式 `Type`。
    fn parse_named_or_plain_type(&mut self) -> Result<TypeExpression, ParseError> {
        // 检查 `identifier :` 模式（命名字段）。
        if self.current_identifier_text().is_some() && self.nth_is_symbol(1, TokenKind::Colon) {
            self.bump(); // 跳过字段名
            self.bump(); // 跳过 `:`
        }
        self.parse_type_expression_bp(0)
    }

    fn parse_type_expression_prefix(&mut self) -> Result<TypeExpression, ParseError> {
        // `&Type` 引用类型：跳过 `&`，解析底层类型（自举阶段不区分值/引用）。
        if self.match_symbol(TokenKind::Ampersand) {
            return self.parse_type_expression_bp(0);
        }

        if self.match_symbol(TokenKind::LBracket) {
            let open = self.previous().span.clone();
            let first = self.parse_type_expression_bp(0)?;
            // `[T1, T2, ...]` — 元组类型（多个类型用逗号分隔）。
            if self.match_symbol(TokenKind::Comma) {
                let mut items = vec![first];
                loop {
                    items.push(self.parse_type_expression_bp(0)?);
                    if !self.match_symbol(TokenKind::Comma) {
                        break;
                    }
                }
                let close = self.expect_symbol(TokenKind::RBracket)?;
                return Ok(TypeExpression::Tuple { items, span: span(open.start, close.span.end) });
            }
            let close = self.expect_symbol(TokenKind::RBracket)?;
            return Ok(TypeExpression::Array { item: Box::new(first), span: span(open.start, close.span.end) });
        }

        if self.match_symbol(TokenKind::LParen) {
            let open = self.previous().span.clone();
            if self.match_symbol(TokenKind::RParen) {
                return Ok(TypeExpression::Tuple { items: Vec::new(), span: span(open.start, self.previous().span.end) });
            }

            // 支持带命名字段的元组类型 `(name: Type, name2: Type2)`。
            // 自举阶段跳过字段名，只解析类型。
            let first = self.parse_named_or_plain_type()?;
            if self.match_symbol(TokenKind::Comma) {
                let mut items = vec![first];
                loop {
                    items.push(self.parse_named_or_plain_type()?);
                    if !self.match_symbol(TokenKind::Comma) {
                        break;
                    }
                }
                let close = self.expect_symbol(TokenKind::RParen)?;
                return Ok(TypeExpression::Tuple { items, span: span(open.start, close.span.end) });
            }

            self.expect_symbol(TokenKind::RParen)?;
            return Ok(with_type_span(first, span(open.start, self.previous().span.end)));
        }

        // 函数类型：`micro(P1, P2) -> R`。
        if self.check_token_keyword(Keyword::Micro) && self.nth_is_symbol(1, TokenKind::LParen) {
            let start = self.current().span.start;
            self.bump(); // 消费 `micro`
            self.expect_symbol(TokenKind::LParen)?;
            let params = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_type_expression_bp(0))?;
            self.expect_symbol(TokenKind::RParen)?;
            let return_type = if self.match_symbol(TokenKind::Arrow) || self.match_symbol(TokenKind::Colon) {
                self.parse_type_expression_bp(0)?
            }
            else {
                TypeExpression::Tuple { items: Vec::new(), span: span(self.previous().span.end, self.previous().span.end) }
            };
            let end = self.previous().span.end;
            return Ok(TypeExpression::Function { params, return_type: Box::new(return_type), span: span(start, end) });
        }

        let path = self.parse_type_path()?;
        Ok(TypeExpression::Path(path))
    }

    fn parse_type_expression_postfix(&mut self, lhs: TypeExpression) -> Result<TypeExpression, ParseError> {
        match self.current().kind {
            TokenKind::LAngle => match lhs {
                TypeExpression::Path(mut path) => {
                    path.arguments = self.parse_type_argument_clause()?;
                    path.span = span(path.span.start, self.previous().span.end);
                    Ok(TypeExpression::Path(path))
                }
                _ => Err(self.error_here("generic arguments can only follow a path type")),
            },
            // 后缀 `[]` 数组类型：`utf16[]` 表示 `Array<utf16>`。
            TokenKind::LBracket => {
                let start = self.current().span.start;
                self.expect_symbol(TokenKind::LBracket)?;
                self.expect_symbol(TokenKind::RBracket)?;
                let end = self.previous().span.end;
                Ok(TypeExpression::Array { item: Box::new(lhs), span: span(start, end) })
            }
            // 后缀 `?` 可空类型：`T?` 表示 `T` 或 `null`。
            TokenKind::Question => {
                let start = lhs.span().start;
                self.bump();
                let end = self.previous().span.end;
                Ok(TypeExpression::Nullable { item: Box::new(lhs), span: span(start, end) })
            }
            _ => Err(self.error_here("expected generic arguments `<...>` or array suffix `[]` after type")),
        }
    }

    fn type_postfix_binding_power(&self) -> Option<(u8, u8)> {
        match self.current().kind {
            TokenKind::LAngle => Some((90, 91)),
            // 仅当 `[` 后紧跟 `]` 时才视为数组后缀 `[]`，
            // 避免将属性列表 `[clr(...)]` 误认为数组类型。
            TokenKind::LBracket if self.nth_is_symbol(1, TokenKind::RBracket) => Some((90, 91)),
            // `?` 可空类型后缀，绑定力高于 `<` 和 `[]`。
            TokenKind::Question => Some((95, 96)),
            _ => None,
        }
    }

    fn parse_type_path(&mut self) -> Result<TypePath, ParseError> {
        let start = self.current().span.start;
        let mut parts = vec![self.expect_type_name_text()?];
        // 同时接受 `::` 和 `.` 作为类型路径分隔符（源码中两种写法均有使用）。
        loop {
            let is_colon = self.check_symbol(TokenKind::DoubleColon) && self.nth_is_type_name(1);
            let is_dot = self.check_symbol(TokenKind::Dot) && self.nth_is_type_name(1);
            if !is_colon && !is_dot {
                break;
            }
            self.bump();
            parts.push(self.expect_type_name_text()?);
        }
        let name_span = span(start, self.previous().span.end);
        Ok(TypePath { name: NamePath { parts, span: name_span.clone() }, arguments: Vec::new(), span: name_span })
    }

    fn parse_type_argument_clause(&mut self) -> Result<Vec<TypeExpression>, ParseError> {
        self.expect_symbol(TokenKind::LAngle)?;
        let arguments = self.parse_comma_separated_until(TokenKind::RAngle, |parser| parser.parse_type_argument())?;
        self.expect_symbol(TokenKind::RAngle)?;
        Ok(arguments)
    }

    /// 解析类型参数：可能是普通类型表达式，也可能是 `Name = Type` 关联类型绑定。
    fn parse_type_argument(&mut self) -> Result<TypeExpression, ParseError> {
        let start = self.current().span.start;
        // 尝试解析 `Name = Type` 形式的关联类型绑定。
        if matches!(self.current().kind, TokenKind::Identifier)
            && !self.check_token_keyword(Keyword::Where)
            && self.peek().kind == TokenKind::Equal
        {
            let name_start = self.current().span.start;
            let name_text = self.expect_identifier_text()?.to_string();
            let name = IdentifierNode::new(valkyrie_types::Identifier::new(&name_text), span(name_start, self.previous().span.end));
            self.expect_symbol(TokenKind::Equal)?;
            let ty = self.parse_type_expression_bp(0)?;
            let end = self.previous().span.end;
            return Ok(TypeExpression::Associated { name, ty: Box::new(ty), span: span(start, end) });
        }
        self.parse_type_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, min_bp: u8) -> Result<TermExpression, ParseError> {
        let mut lhs = self.parse_expression_prefix()?;

        loop {
            if let Some((left_bp, _right_bp)) = self.expression_postfix_binding_power() {
                if left_bp < min_bp {
                    break;
                }
                lhs = self.parse_expression_postfix(lhs)?;
                continue;
            }

            if self.check_token_keyword(Keyword::As) {
                let (left_bp, _right_bp) = (45, 46);
                if left_bp < min_bp {
                    break;
                }
                let start = lhs.span().start;
                self.bump();
                let ty = self.parse_type_expression_bp(0)?;
                let end = self.previous().span.end;
                lhs = TermExpression::As(Box::new(TermAsExpression { base: lhs, target: ty, span: span(start, end) }));
                continue;
            }

            if self.check_symbol(TokenKind::Equal) {
                let (left_bp, right_bp) = (10, 9);
                if left_bp < min_bp {
                    break;
                }
                let start = lhs.span().start;
                self.bump();
                let value = self.parse_expression_bp(right_bp)?;
                let end = value.span().end;
                lhs = TermExpression::Assign { target: Box::new(lhs), value: Box::new(value), span: span(start, end) };
                continue;
            }

            let Some((op, left_bp, right_bp)) = self.expression_infix_binding_power()
            else {
                break;
            };
            if left_bp < min_bp {
                break;
            }

            let start = lhs.span().start;
            self.bump();
            let rhs = self.parse_expression_bp(right_bp)?;
            let end = rhs.span().end;
            lhs = TermExpression::Binary(Box::new(TermBinaryExpression { operator: op, lhs, rhs, span: span(start, end) }));
        }

        Ok(lhs)
    }

    fn parse_expression_prefix(&mut self) -> Result<TermExpression, ParseError> {
        let token = self.current().clone();
        if self.check_identifier_text_eq("unsafe") && self.nth_is_symbol(1, TokenKind::LBrace) {
            return self.parse_unsafe_block_expression();
        }
        match token.kind {
            TokenKind::Keyword(Keyword::True) | TokenKind::Keyword(Keyword::False) => {
                let value = token.kind == TokenKind::Keyword(Keyword::True);
                self.bump();
                Ok(TermExpression::Literal { literal: LiteralExpression::Bool(value), span: token.span })
            }
            TokenKind::Apostrophe => self.parse_labeled_loop_expression(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_expression(),
            TokenKind::Keyword(Keyword::Break) => self.parse_break_expression(),
            TokenKind::Keyword(Keyword::Continue) => self.parse_continue_expression(),
            TokenKind::Keyword(Keyword::Yield) => self.parse_yield_expression(),
            TokenKind::Keyword(Keyword::Raise) => self.parse_raise_expression(),
            TokenKind::Keyword(Keyword::Resume) => self.parse_resume_expression(),
            TokenKind::Keyword(Keyword::Catch) => self.parse_catch_expression(),
            TokenKind::Keyword(Keyword::Fallthrough) => {
                self.bump();
                Ok(TermExpression::Fallthrough { span: token.span })
            }
            TokenKind::Keyword(Keyword::If) => self.parse_if_expression(),
            TokenKind::Keyword(Keyword::Loop) => self.parse_loop_expression(),
            TokenKind::Keyword(Keyword::While) => self.parse_while_expression(),
            TokenKind::Keyword(Keyword::Case) => self.parse_case_expression(),
            TokenKind::Keyword(Keyword::Match) => self.parse_match_expression(),
            TokenKind::Keyword(Keyword::Micro) => self.parse_lambda_expression(),
            TokenKind::Identifier => {
                let path = self.parse_name_path()?;
                let span = path.span.clone();
                Ok(TermExpression::Name { path, span })
            }
            TokenKind::StringLiteral => {
                self.bump();
                Ok(TermExpression::Literal {
                    literal: LiteralExpression::String(parse_string_literal(self.slice(token.span.clone()))?),
                    span: token.span,
                })
            }
            TokenKind::IntegerLiteral => {
                self.bump();
                Ok(TermExpression::Literal {
                    literal: LiteralExpression::Integer(self.slice(token.span.clone()).to_string()),
                    span: token.span,
                })
            }
            TokenKind::FloatLiteral => {
                self.bump();
                Ok(TermExpression::Literal { literal: LiteralExpression::Float(self.slice(token.span.clone()).to_string()), span: token.span })
            }
            TokenKind::Minus | TokenKind::Bang => {
                let start = token.span.start;
                let op = match token.kind {
                    TokenKind::Minus => UnaryOperator::Neg,
                    TokenKind::Bang => UnaryOperator::Not,
                    _ => unreachable!(),
                };
                self.bump();
                let rhs = self.parse_expression_bp(80)?;
                let end = rhs.span().end;
                Ok(TermExpression::Unary(Box::new(TermUnaryExpression { operator: op, base: rhs, span: span(start, end) })))
            }
            TokenKind::LParen => self.parse_parenthesized_expression(),
            TokenKind::LBracket => self.parse_array_expression(),
            _ => Err(self.error_here("expected expression")),
        }
    }

    fn parse_expression_postfix(&mut self, lhs: TermExpression) -> Result<TermExpression, ParseError> {
        match self.current().kind {
            TokenKind::LParen => self.parse_call_expression(lhs),
            TokenKind::Dot => self.parse_member_expression(lhs),
            TokenKind::LBracket => self.parse_subscript_expression(lhs, SubscriptKind::Ordinal),
            TokenKind::LOffsetBracket => self.parse_offset_subscript_expression(lhs),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LBracket) => self.parse_offset_alias_subscript_expression(lhs),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LAngle) => self.parse_turbofish_expression(lhs),
            // `::identifier` 路径解析，用于静态方法访问如 `Type::new()`。
            TokenKind::DoubleColon if self.nth_is_identifier(1) => self.parse_path_member_expression(lhs),
            TokenKind::LBrace => self.parse_construct_expression(lhs),
            _ => Err(self.error_here("expected postfix operator")),
        }
    }

    fn parse_call_expression(&mut self, callee: TermExpression) -> Result<TermExpression, ParseError> {
        let start = callee.span().start;
        self.expect_symbol(TokenKind::LParen)?;
        let args = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_expression_bp(0))?;
        let close = self.expect_symbol(TokenKind::RParen)?;
        Ok(TermExpression::Call { callee: Box::new(callee), args, span: span(start, close.span.end) })
    }

    fn parse_member_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::Dot)?;
        let member = self.expect_member_name_text()?;
        let end = self.previous().span.end;
        Ok(TermExpression::MemberAccess { object: Box::new(object), member, span: span(start, end) })
    }

    /// 解析 `::identifier` 路径访问，用于静态方法如 `Type::new()`。
    ///
    /// 在语义上与 `.` 成员访问相同，均生成 `MemberAccess` 节点。
    fn parse_path_member_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::DoubleColon)?;
        let member = self.expect_member_name_text()?;
        let end = self.previous().span.end;
        Ok(TermExpression::MemberAccess { object: Box::new(object), member, span: span(start, end) })
    }

    fn parse_subscript_expression(&mut self, object: TermExpression, kind: SubscriptKind) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::LBracket)?;
        let index = self.parse_expression_bp(0)?;
        let close = self.expect_symbol(TokenKind::RBracket)?;
        Ok(TermExpression::Subscript { object: Box::new(object), index: Box::new(index), kind, span: span(start, close.span.end) })
    }

    fn parse_offset_subscript_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::LOffsetBracket)?;
        let index = self.parse_expression_bp(0)?;
        let close = self.expect_symbol(TokenKind::ROffsetBracket)?;
        Ok(TermExpression::Subscript {
            object: Box::new(object),
            index: Box::new(index),
            kind: SubscriptKind::Offset,
            span: span(start, close.span.end),
        })
    }

    fn parse_offset_alias_subscript_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::DoubleColon)?;
        self.expect_symbol(TokenKind::LBracket)?;
        let index = self.parse_expression_bp(0)?;
        let close = self.expect_symbol(TokenKind::RBracket)?;
        Ok(TermExpression::Subscript {
            object: Box::new(object),
            index: Box::new(index),
            kind: SubscriptKind::Offset,
            span: span(start, close.span.end),
        })
    }

    fn parse_turbofish_expression(&mut self, expr: TermExpression) -> Result<TermExpression, ParseError> {
        let start = expr.span().start;
        self.expect_symbol(TokenKind::DoubleColon)?;
        let arguments = self.parse_type_argument_clause()?;
        let end = self.previous().span.end;
        Ok(TermExpression::Turbofish { expr: Box::new(expr), arguments, span: span(start, end) })
    }

    /// 解析结构体构造表达式 `Type { field: value, field2: value2 }`。
    ///
    /// `lhs` 应为 `TermExpression::Name`（类型路径），`{` 已由 postfix 分发器确认。
    /// 同时支持 `Type::<T> { ... }` 形式（turbofish 后跟结构体构造）。
    fn parse_construct_expression(&mut self, lhs: TermExpression) -> Result<TermExpression, ParseError> {
        let path = match lhs {
            TermExpression::Name { path, .. } => path,
            // 处理 `Type::<T> { ... }` 形式：提取内部类型名，忽略泛型参数（自举阶段不做类型检查）。
            TermExpression::Turbofish { expr, .. } => match *expr {
                TermExpression::Name { path, .. } => path,
                _ => return Err(self.error_here("struct constructor requires a type name")),
            },
            _ => return Err(self.error_here("struct constructor requires a type name")),
        };
        let start = path.span.start;
        self.expect_symbol(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.check_symbol(TokenKind::RBrace) {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated struct constructor"));
            }
            let field_name = self.expect_identifier_text()?.to_string();
            self.expect_symbol(TokenKind::Colon)?;
            let value = self.parse_expression_bp(0)?;
            fields.push((field_name, value));
            if !self.match_symbol(TokenKind::Comma) {
                break;
            }
        }
        let close = self.expect_symbol(TokenKind::RBrace)?;
        Ok(TermExpression::Construct { path, fields, span: span(start, close.span.end) })
    }

    fn parse_parenthesized_expression(&mut self) -> Result<TermExpression, ParseError> {
        let open = self.expect_symbol(TokenKind::LParen)?;
        if self.match_symbol(TokenKind::RParen) {
            return Ok(TermExpression::Literal { literal: LiteralExpression::Unit, span: span(open.span.start, self.previous().span.end) });
        }

        let first = self.parse_expression_bp(0)?;
        if self.match_symbol(TokenKind::Comma) {
            let mut items = vec![first];
            loop {
                if self.check_symbol(TokenKind::RParen) {
                    break;
                }
                items.push(self.parse_expression_bp(0)?);
                if !self.match_symbol(TokenKind::Comma) {
                    break;
                }
            }
            let close = self.expect_symbol(TokenKind::RParen)?;
            return Ok(TermExpression::Tuple { items, span: span(open.span.start, close.span.end) });
        }

        let close = self.expect_symbol(TokenKind::RParen)?;
        Ok(with_term_span(first, span(open.span.start, close.span.end)))
    }

    fn parse_array_expression(&mut self) -> Result<TermExpression, ParseError> {
        let open = self.expect_symbol(TokenKind::LBracket)?;
        let items = self.parse_comma_separated_until(TokenKind::RBracket, |parser| parser.parse_expression_bp(0))?;
        let close = self.expect_symbol(TokenKind::RBracket)?;
        Ok(TermExpression::Array { items, span: span(open.span.start, close.span.end) })
    }

    fn parse_return_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Return)?.span.start;
        let value = if self.is_expression_terminator() { None } else { Some(Box::new(self.parse_expression_bp(0)?)) };
        let end = value.as_ref().map_or(self.previous().span.end, |expr| expr.span().end);
        Ok(TermExpression::Return { value, span: span(start, end) })
    }

    fn parse_break_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Break)?.span.start;
        let label = if self.check_symbol(TokenKind::Apostrophe) { Some(self.parse_label_name()?) } else { None };
        let value = if self.is_expression_terminator() { None } else { Some(Box::new(self.parse_expression_bp(0)?)) };
        let end = value.as_ref().map_or(self.previous().span.end, |expr| expr.span().end);
        Ok(TermExpression::Break { label, value, span: span(start, end) })
    }

    fn parse_continue_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Continue)?.span.start;
        let label = if self.check_symbol(TokenKind::Apostrophe) { Some(self.parse_label_name()?) } else { None };
        let end = self.previous().span.end;
        Ok(TermExpression::Continue { label, span: span(start, end) })
    }

    fn parse_yield_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Yield)?.span.start;
        if self.match_identifier_text_eq("from") {
            let value = Box::new(self.parse_expression_bp(0)?);
            let end = value.span().end;
            return Ok(TermExpression::YieldFrom { value, span: span(start, end) });
        }

        let value = if self.is_expression_terminator() { None } else { Some(Box::new(self.parse_expression_bp(0)?)) };
        let end = value.as_ref().map_or(self.previous().span.end, |expr| expr.span().end);
        Ok(TermExpression::Yield { value, span: span(start, end) })
    }

    fn parse_raise_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Raise)?.span.start;
        let value = Box::new(self.parse_expression_bp(0)?);
        let end = value.span().end;
        Ok(TermExpression::Raise { value, span: span(start, end) })
    }

    fn parse_resume_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Resume)?.span.start;
        let value = Box::new(self.parse_expression_bp(0)?);
        let end = value.span().end;
        Ok(TermExpression::Resume { value, span: span(start, end) })
    }

    fn parse_catch_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Catch)?.span.start;
        self.suppress_struct_constructor = true;
        let expr = Box::new(self.parse_expression_bp(0)?);
        self.suppress_struct_constructor = false;
        self.expect_symbol(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while !self.check_symbol(TokenKind::RBrace) {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated catch body"));
            }
            if self.match_symbol(TokenKind::Semicolon) {
                continue;
            }
            let arm = self.parse_match_arm()?;
            arms.push(arm);
        }
        let close = self.expect_symbol(TokenKind::RBrace)?;
        Ok(TermExpression::Catch { expr, arms, span: span(start, close.span.end) })
    }

    /// 解析 lambda 表达式 `micro(params) -> return_type { body }`。
    ///
    /// 与顶层 `micro name(params) -> T { body }` 函数声明不同，lambda 作为表达式
    /// 出现在调用参数等位置时没有函数名，直接以 `micro(` 起始。
    fn parse_lambda_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Micro)?.span.start;
        let params = self.parse_parameter_list()?;
        let return_type = if self.match_symbol(TokenKind::Arrow) { Some(self.parse_type_expression_bp(0)?) } else { None };
        let body = if self.check_symbol(TokenKind::LBrace) {
            self.parse_block_body()?
        }
        else {
            return Err(self.error_here("expected lambda body '{'"));
        };
        let end = body.span.end;
        Ok(TermExpression::Lambda { params, return_type, body: Box::new(body), span: span(start, end) })
    }

    /// 解析 `unsafe { ... }` 块表达式。
    ///
    /// `unsafe` 关键字后紧跟块体，语义上与普通块相同，
    /// 仅标记其中的操作可能涉及不安全行为。
    fn parse_unsafe_block_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_identifier_text_eq("unsafe")?.span.start;
        let body = self.parse_block_body()?;
        let end = body.span.end;
        Ok(TermExpression::Block { body: Box::new(body), span: span(start, end) })
    }

    fn expression_postfix_binding_power(&self) -> Option<(u8, u8)> {
        match self.current().kind {
            TokenKind::Dot => Some((95, 96)),
            // `::identifier` 路径访问，与 `.` 成员访问具有相同的绑定力。
            TokenKind::DoubleColon if self.nth_is_identifier(1) => Some((95, 96)),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LAngle) => Some((93, 94)),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LBracket) => Some((90, 91)),
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LOffsetBracket => Some((90, 91)),
            // 结构体构造 `Type { field: value }`，在条件解析时被抑制。
            TokenKind::LBrace if !self.suppress_struct_constructor => Some((85, 86)),
            _ => None,
        }
    }

    /// 判断当前 token 是否为新语句的起始。
    ///
    /// 用于换行隐式终止：当表达式后紧跟标识符（含关键字）时，无需 `;` 即可结束当前语句。
    /// 值类型关键字（如 `i32`、`utf8`）也视为新语句起始。
    fn is_statement_start(&self) -> bool {
        matches!(self.current().kind, TokenKind::Identifier | TokenKind::Keyword(_) | TokenKind::Apostrophe)
    }

    fn expression_infix_binding_power(&self) -> Option<(BinaryOperator, u8, u8)> {
        match self.current().kind {
            // `|>` 管道操作符优先级最低（低于逻辑或），左结合。
            TokenKind::PipeGt => Some((BinaryOperator::Pipe, 10, 11)),
            TokenKind::OrOr => Some((BinaryOperator::Or, 20, 21)),
            TokenKind::AndAnd => Some((BinaryOperator::And, 30, 31)),
            TokenKind::Star => Some((BinaryOperator::Mul, 60, 61)),
            TokenKind::Slash => Some((BinaryOperator::Div, 60, 61)),
            TokenKind::Percent => Some((BinaryOperator::Rem, 60, 61)),
            TokenKind::Plus => Some((BinaryOperator::Add, 50, 51)),
            TokenKind::Minus => Some((BinaryOperator::Sub, 50, 51)),
            TokenKind::EqEq => Some((BinaryOperator::Eq, 40, 41)),
            TokenKind::NotEq => Some((BinaryOperator::Ne, 40, 41)),
            TokenKind::LAngle => Some((BinaryOperator::Lt, 40, 41)),
            TokenKind::RAngle => Some((BinaryOperator::Gt, 40, 41)),
            TokenKind::LessEq => Some((BinaryOperator::Le, 40, 41)),
            TokenKind::GreaterEq => Some((BinaryOperator::Ge, 40, 41)),
            // `<<` 和 `>>` 移位运算符，优先级介于算术和比较之间。
            TokenKind::Shl => Some((BinaryOperator::Shl, 45, 46)),
            TokenKind::Shr => Some((BinaryOperator::Shr, 45, 46)),
            // 按位运算符，优先级介于比较和逻辑运算之间（C 语言约定）。
            TokenKind::Ampersand => Some((BinaryOperator::BitAnd, 38, 39)),
            TokenKind::Caret => Some((BinaryOperator::Power, 36, 37)),
            TokenKind::Pipe => Some((BinaryOperator::BitOr, 34, 35)),
            _ => None,
        }
    }

    fn is_expression_terminator(&self) -> bool {
        matches!(self.current().kind, TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Comma | TokenKind::RParen | TokenKind::RBracket)
            || matches!(self.current().kind, TokenKind::Keyword(Keyword::Case | Keyword::Else))
    }

    fn parse_comma_separated_until<T>(
        &mut self,
        end: TokenKind,
        mut parse: impl FnMut(&mut Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        let mut items = Vec::new();
        if self.check_symbol(end) {
            return Ok(items);
        }
        loop {
            items.push(parse(self)?);
            if !self.match_symbol(TokenKind::Comma) {
                break;
            }
            if self.check_symbol(end) {
                break;
            }
        }
        Ok(items)
    }

    fn parse_separated_while<T>(
        &mut self,
        mut parse: impl FnMut(&mut Self) -> Result<T, ParseError>,
        separators: &[TokenKind],
    ) -> Result<Vec<T>, ParseError> {
        let mut items = vec![parse(self)?];
        while separators.iter().any(|separator| self.match_symbol(*separator)) {
            items.push(parse(self)?);
        }
        Ok(items)
    }

    fn parse_generic_parameter_clause(&mut self) -> Result<Vec<String>, ParseError> {
        if !self.check_symbol(TokenKind::LAngle) {
            return Ok(Vec::new());
        }

        let open = self.expect_symbol(TokenKind::LAngle)?;
        let mut depth = 1i32;
        let mut segment_start = open.span.end;
        let mut parameters = Vec::new();

        while !self.is_eof() {
            let token = self.bump().clone();
            match token.kind {
                TokenKind::LAngle => depth += 1,
                TokenKind::RAngle => {
                    depth -= 1;
                    if depth == 0 {
                        let text = self.slice(segment_start..token.span.start).trim();
                        if !text.is_empty() {
                            parameters.push(text.to_string());
                        }
                        return Ok(parameters);
                    }
                }
                TokenKind::Comma if depth == 1 => {
                    let text = self.slice(segment_start..token.span.start).trim();
                    if !text.is_empty() {
                        parameters.push(text.to_string());
                    }
                    segment_start = token.span.end;
                }
                _ => {}
            }
        }

        Err(ParseError::invalid("unterminated generic parameter clause"))
    }

    fn parse_structured_generic_parameter_clause(&mut self) -> Result<Vec<GenericParameterDeclaration>, ParseError> {
        if !self.check_symbol(TokenKind::LAngle) {
            return Ok(Vec::new());
        }

        self.expect_symbol(TokenKind::LAngle)?;
        let parameters = self.parse_comma_separated_until(TokenKind::RAngle, |parser| parser.parse_generic_parameter_declaration())?;
        self.expect_symbol(TokenKind::RAngle)?;
        Ok(parameters)
    }

    fn parse_generic_parameter_declaration(&mut self) -> Result<GenericParameterDeclaration, ParseError> {
        let start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        let bounds = if self.match_symbol(TokenKind::Colon) { self.parse_trait_bound_list()? } else { Vec::new() };
        let default_type = if self.match_symbol(TokenKind::Equal) { Some(self.parse_type_expression_bp(0)?) } else { None };
        Ok(GenericParameterDeclaration {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(start, self.previous().span.end)),
            bounds,
            default_type,
            span: span(start, self.previous().span.end),
        })
    }

    fn parse_where_constraints(&mut self) -> Result<Vec<WhereConstraintDeclaration>, ParseError> {
        if !self.check_token_keyword(Keyword::Where) {
            return Ok(Vec::new());
        }

        self.expect_token_keyword(Keyword::Where)?;
        let constraints = self.parse_comma_separated_until(TokenKind::LBrace, |parser| parser.parse_where_constraint())?;
        Ok(constraints)
    }

    fn parse_where_constraint(&mut self) -> Result<WhereConstraintDeclaration, ParseError> {
        let start = self.current().span.start;
        let target_type = self.parse_type_expression_bp(0)?;
        self.expect_symbol(TokenKind::Colon)?;
        let bounds = self.parse_trait_bound_list()?;
        Ok(WhereConstraintDeclaration { target_type, bounds, span: span(start, self.previous().span.end) })
    }

    fn skip_generic_parameter_clause(&mut self) -> Result<(), ParseError> {
        if !self.check_symbol(TokenKind::LAngle) {
            return Ok(());
        }

        let mut depth = 0i32;
        while !self.is_eof() {
            match self.bump().kind {
                TokenKind::LAngle => depth += 1,
                TokenKind::RAngle => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        Err(ParseError::invalid("unterminated generic parameter clause"))
    }

    fn expect_implicit_or_explicit_terminator(&mut self, next_item_keywords: &[&str]) -> Result<(), ParseError> {
        if self.match_symbol(TokenKind::Semicolon) {
            return Ok(());
        }

        if self.check_symbol(TokenKind::RBrace)
            || self.is_eof()
            || self.check_symbol(TokenKind::LBracket)
            || self.current_keyword_text().is_some_and(|text| DECLARATION_MODIFIERS.contains(&text) || next_item_keywords.contains(&text))
            || self.current_keyword_text().is_some_and(|text| next_item_keywords.contains(&text))
            || self.current_identifier_text().is_some_and(|text| DECLARATION_MODIFIERS.contains(&text) || next_item_keywords.contains(&text))
        {
            return Ok(());
        }

        Err(self.error_here("expected ';' or declaration boundary"))
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    /// 查看下一个 token（不消费当前 token）。
    fn peek(&self) -> &Token {
        let next_index = (self.index + 1).min(self.tokens.len().saturating_sub(1));
        &self.tokens[next_index]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.index.saturating_sub(1)]
    }

    fn bump(&mut self) -> &Token {
        let current = self.index;
        if !matches!(self.tokens[current].kind, TokenKind::Eof) {
            self.index += 1;
        }
        &self.tokens[current]
    }

    fn is_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn current_identifier_text(&self) -> Option<&str> {
        (self.current().kind == TokenKind::Identifier).then(|| self.slice(self.current().span.clone()))
    }

    fn current_keyword_text(&self) -> Option<&'static str> {
        match self.current().kind {
            TokenKind::Keyword(keyword) => Some(keyword.as_str()),
            _ => None,
        }
    }

    fn current_modifier_text(&self) -> Option<&str> {
        self.current_identifier_text().or_else(|| self.current_keyword_text())
    }

    fn check_token_keyword(&self, keyword: Keyword) -> bool {
        self.current().kind == TokenKind::Keyword(keyword)
    }

    fn match_token_keyword(&mut self, keyword: Keyword) -> bool {
        if self.check_token_keyword(keyword) {
            self.bump();
            true
        }
        else {
            false
        }
    }

    fn expect_token_keyword(&mut self, keyword: Keyword) -> Result<Token, ParseError> {
        if self.check_token_keyword(keyword) {
            return Ok(self.bump().clone());
        }
        Err(ParseError::invalid_at(format!("expected keyword `{}`", keyword.as_str()), self.current().span.clone()))
    }

    fn check_identifier_text_eq(&self, text: &str) -> bool {
        self.current_identifier_text().is_some_and(|current| current == text)
    }

    fn match_identifier_text_eq(&mut self, text: &str) -> bool {
        if self.check_identifier_text_eq(text) {
            self.bump();
            true
        }
        else {
            false
        }
    }

    fn expect_identifier_text_eq(&mut self, text: &str) -> Result<Token, ParseError> {
        if self.check_identifier_text_eq(text) {
            return Ok(self.bump().clone());
        }
        Err(self.error_here(format!("expected identifier `{}`", text)))
    }

    #[allow(dead_code)]
    #[deprecated(note = "use tokenized keyword checks instead: TokenKind::Keyword(...), expect_token_keyword, or match_token_keyword")]
    fn check_keyword(&self, text: &str) -> bool {
        Keyword::from_str(text).is_some_and(|keyword| self.current().kind == TokenKind::Keyword(keyword))
            || self.current_identifier_text().is_some_and(|current| current == text)
    }

    #[allow(dead_code)]
    #[deprecated(note = "use tokenized keyword checks instead: TokenKind::Keyword(...), expect_token_keyword, or match_token_keyword")]
    fn match_keyword(&mut self, text: &str) -> bool {
        if Keyword::from_str(text).is_some_and(|keyword| self.current().kind == TokenKind::Keyword(keyword))
            || self.current_identifier_text().is_some_and(|current| current == text)
        {
            self.bump();
            true
        }
        else {
            false
        }
    }

    #[allow(dead_code)]
    #[deprecated(note = "use tokenized keyword checks instead: TokenKind::Keyword(...), expect_token_keyword, or match_token_keyword")]
    fn expect_keyword(&mut self, text: &str) -> Result<Token, ParseError> {
        if Keyword::from_str(text).is_some_and(|keyword| self.current().kind == TokenKind::Keyword(keyword))
            || self.current_identifier_text().is_some_and(|current| current == text)
        {
            return Ok(self.bump().clone());
        }
        Err(self.error_here(format!("expected keyword '{}'", text)))
    }

    fn expect_identifier_text(&mut self) -> Result<&str, ParseError> {
        if matches!(self.current().kind, TokenKind::Identifier) {
            let span = self.bump().span.clone();
            return Ok(self.slice(span));
        }
        Err(self.error_here("expected identifier"))
    }

    fn parse_label_name(&mut self) -> Result<String, ParseError> {
        self.expect_symbol(TokenKind::Apostrophe)?;
        Ok(self.expect_identifier_text()?.to_string())
    }

    #[allow(dead_code)]
    fn expect_keyword_text(&mut self) -> Result<String, ParseError> {
        if let TokenKind::Keyword(keyword) = self.current().kind {
            self.bump();
            return Ok(keyword.as_str().to_string());
        }
        if let Some(text) = self.current_identifier_text() {
            let owned = text.to_string();
            self.bump();
            return Ok(owned);
        }
        Err(self.error_here("expected keyword"))
    }

    fn expect_member_name_text(&mut self) -> Result<String, ParseError> {
        match self.current().kind {
            TokenKind::Identifier => {
                let span = self.bump().span.clone();
                Ok(self.slice(span).to_string())
            }
            TokenKind::Keyword(keyword) => {
                self.bump();
                Ok(keyword.as_str().to_string())
            }
            TokenKind::BacktickSymbol => self.expect_name_text(),
            _ => Err(self.error_here("expected member name")),
        }
    }

    fn expect_type_name_text(&mut self) -> Result<String, ParseError> {
        match self.current().kind {
            TokenKind::Identifier => {
                let span = self.bump().span.clone();
                Ok(self.slice(span).to_string())
            }
            TokenKind::Keyword(keyword) => {
                self.bump();
                Ok(keyword.as_str().to_string())
            }
            TokenKind::BacktickSymbol => self.expect_name_text(),
            _ => Err(self.error_here("expected type name")),
        }
    }

    /// 期望一个名称文本，接受普通标识符或反引号包裹的标识符。
    ///
    /// 反引号包裹的标识符（如 `` `any` ``、`` `bool` ``）允许使用关键字作为名称。
    /// 返回的文本已剥离反引号。
    fn expect_name_text(&mut self) -> Result<String, ParseError> {
        if matches!(self.current().kind, TokenKind::BacktickSymbol) {
            let span = self.bump().span.clone();
            let text = self.slice(span);
            let trimmed = text.trim_start_matches('`').trim_end_matches('`');
            return Ok(trimmed.to_string());
        }
        if matches!(self.current().kind, TokenKind::Identifier) {
            let span = self.bump().span.clone();
            return Ok(self.slice(span).to_string());
        }
        Err(self.error_here("expected identifier or backtick name"))
    }

    fn check_symbol(&self, symbol: TokenKind) -> bool {
        self.current().kind == symbol
            // 嵌套泛型中 `>>` 应视为两个 `>`，此处允许 `Shr` 匹配 `RAngle`。
            || (symbol == TokenKind::RAngle && self.current().kind == TokenKind::Shr)
    }

    fn nth_is_symbol(&self, offset: usize, symbol: TokenKind) -> bool {
        self.tokens.get(self.index + offset).is_some_and(|token| token.kind == symbol)
    }

    fn nth_is_identifier(&self, offset: usize) -> bool {
        self.tokens.get(self.index + offset).is_some_and(|token| token.kind == TokenKind::Identifier)
    }

    fn nth_is_type_name(&self, offset: usize) -> bool {
        self.tokens
            .get(self.index + offset)
            .is_some_and(|token| matches!(token.kind, TokenKind::Identifier | TokenKind::Keyword(_) | TokenKind::BacktickSymbol))
    }

    fn match_symbol(&mut self, symbol: TokenKind) -> bool {
        if self.check_symbol(symbol) {
            self.bump();
            true
        }
        else {
            false
        }
    }

    fn expect_symbol(&mut self, symbol: TokenKind) -> Result<Token, ParseError> {
        if self.check_symbol(symbol) {
            // 嵌套泛型中 `>>` (Shr) 需拆分为两个 `>` (RAngle)，
            // 此处将当前 `>>` token 替换为第一个 `>`，并插入第二个 `>` 供外层泛型消费。
            if symbol == TokenKind::RAngle && self.current().kind == TokenKind::Shr {
                let start = self.current().span.start;
                self.tokens[self.index] = Token { kind: TokenKind::RAngle, span: span(start, start + 1) };
                self.tokens.insert(self.index + 1, Token { kind: TokenKind::RAngle, span: span(start + 1, start + 2) });
            }
            return Ok(self.bump().clone());
        }
        Err(self.error_here(format!("expected symbol {:?}", symbol)))
    }

    fn slice(&self, source_span: Range<usize>) -> &str {
        &self.source[source_span.start as usize..source_span.end as usize]
    }

    fn error_here(&self, message: impl Into<String>) -> ParseError {
        let current = self.current();
        let token_text = if matches!(current.kind, TokenKind::Eof) { "<eof>" } else { self.slice(current.span.clone()) };
        ParseError::invalid_at(format!("{} near '{}'", message.into(), token_text), current.span.clone())
    }
}

/// Entry point that parses source text into `ValkyrieRoot`.
pub struct AstParser;

impl AstParser {
    /// Parses a source file from disk.
    pub fn parse_path(path: &PathBuf) -> Result<ValkyrieRoot, ParseError> {
        let source = std::fs::read_to_string(path)?;
        Self::parse_root(&source)
    }

    /// Parses source text into a parser root.
    pub fn parse_root(source: &str) -> Result<ValkyrieRoot, ParseError> {
        let tokens = Lexer::tokenize(source)?;
        Parser::new(source, tokens).parse_root()
    }
}

/// 解析字符串字面量原始文本，生成结构化的 `StringLiteral`。
///
/// 输入是词法分析器产生的原始切片，包含前缀（如 `r`）、引号和内容。
/// 非原始字符串会处理转义序列与 `{...}` 插值片段。
fn parse_string_literal(raw: &str) -> Result<StringLiteral, ParseError> {
    let mut chars = raw.chars();
    let prefix = if raw.starts_with('r') {
        chars.next();
        Some("r".to_string())
    }
    else {
        None
    };

    let rest: String = chars.collect();
    let quote_count: u8 = if rest.starts_with("\"\"\"") { 3 } else { 1 };
    let quote_len = quote_count as usize;

    if rest.len() < quote_len * 2 {
        return Err(ParseError::invalid("字符串字面量过短"));
    }

    let inner = &rest[quote_len..rest.len() - quote_len];
    let is_raw = prefix.is_some();

    let segments = if is_raw { vec![StringSegment::Text(inner.to_string())] } else { parse_cooked_string_segments(inner)? };

    Ok(StringLiteral { prefix, quote_count, segments })
}

/// 解析普通字符串中的文本与插值片段。
fn parse_cooked_string_segments(input: &str) -> Result<Vec<StringSegment>, ParseError> {
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut cursor = 0;

    while cursor < input.len() {
        if input[cursor..].starts_with('{') {
            // 尝试查找插值结束的 `}`；若找不到（如字面 `{`），视为普通字符。
            match find_interpolation_end(input, cursor + 1) {
                Ok(end) => {
                    let expression_source = input[cursor + 1..end].trim();
                    // 空插值 `{}` 视为字面文本（常用于 `format("{}", x)` 等格式化占位符）。
                    if expression_source.is_empty() {
                        text.push('{');
                        text.push('}');
                        cursor = end + '}'.len_utf8();
                        continue;
                    }
                    push_text_segment(&mut segments, &mut text);
                    let is_fluent = false;
                    let expression = parse_interpolation_expression(expression_source)?;
                    segments.push(StringSegment::Interpolation { expression: Box::new(expression), is_fluent });
                    cursor = end + '}'.len_utf8();
                    continue;
                }
                Err(_) => {
                    // `find_interpolation_end` 失败时，将 `{` 视为字面字符。
                    text.push('{');
                    cursor += '{'.len_utf8();
                    continue;
                }
            }
        }

        if input[cursor..].starts_with('\\') {
            let (decoded, next_cursor) = decode_escape_sequence(input, cursor)?;
            text.push_str(&decoded);
            cursor = next_cursor;
            continue;
        }

        let Some(ch) = input[cursor..].chars().next()
        else {
            break;
        };
        text.push(ch);
        cursor += ch.len_utf8();
    }

    push_text_segment(&mut segments, &mut text);
    if segments.is_empty() {
        segments.push(StringSegment::Text(String::new()));
    }
    Ok(segments)
}

/// 解析插值表达式正文。
fn parse_interpolation_expression(source: &str) -> Result<TermExpression, ParseError> {
    if source.is_empty() {
        return Err(ParseError::invalid("字符串插值表达式不能为空"));
    }

    let tokens = Lexer::tokenize(source)?;
    let mut parser = Parser::new(source, tokens);
    let expression = parser.parse_expression_bp(0)?;
    if !parser.is_eof() {
        return Err(ParseError::invalid(format!("字符串插值表达式未完全解析: {}", source)));
    }
    Ok(expression)
}

/// 查找 `{` 对应的结束 `}`，支持嵌套 `{}` 与表达式中的字符串字面量。
fn find_interpolation_end(source: &str, mut cursor: usize) -> Result<usize, ParseError> {
    let mut brace_depth = 1usize;

    while cursor < source.len() {
        if let Some(next_cursor) = try_skip_nested_string_literal(source, cursor)? {
            cursor = next_cursor;
            continue;
        }

        let Some(ch) = source[cursor..].chars().next()
        else {
            break;
        };

        match ch {
            '{' => {
                brace_depth += 1;
            }
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return Ok(cursor);
                }
            }
            _ => {}
        }

        cursor += ch.len_utf8();
    }

    Err(ParseError::invalid("字符串插值缺少结束的 '}'"))
}

/// 若当前位置是嵌套字符串字面量，则跳过整个字面量并返回新的偏移。
fn try_skip_nested_string_literal(source: &str, cursor: usize) -> Result<Option<usize>, ParseError> {
    if source[cursor..].starts_with("r\"\"\"") {
        return Ok(Some(skip_string_body(source, cursor + 4, true, 3)?));
    }
    if source[cursor..].starts_with("r\"") {
        return Ok(Some(skip_string_body(source, cursor + 2, true, 1)?));
    }
    if source[cursor..].starts_with("\"\"\"") {
        return Ok(Some(skip_string_body(source, cursor + 3, false, 3)?));
    }
    if source[cursor..].starts_with('"') {
        return Ok(Some(skip_string_body(source, cursor + 1, false, 1)?));
    }
    Ok(None)
}

/// 跳过嵌套字符串正文，返回闭合引号之后的偏移。
fn skip_string_body(source: &str, mut cursor: usize, is_raw: bool, quote_count: usize) -> Result<usize, ParseError> {
    if quote_count == 3 {
        while cursor < source.len() {
            if source[cursor..].starts_with("\"\"\"") {
                return Ok(cursor + 3);
            }
            let Some(ch) = source[cursor..].chars().next()
            else {
                break;
            };
            cursor += ch.len_utf8();
        }
        return Err(ParseError::invalid("字符串插值中的三引号字符串未闭合"));
    }

    while cursor < source.len() {
        if !is_raw && source[cursor..].starts_with('\\') {
            let (_, next_cursor) = decode_escape_sequence(source, cursor)?;
            cursor = next_cursor;
            continue;
        }
        if source[cursor..].starts_with('"') {
            return Ok(cursor + 1);
        }
        let Some(ch) = source[cursor..].chars().next()
        else {
            break;
        };
        cursor += ch.len_utf8();
    }

    Err(ParseError::invalid("字符串插值中的字符串字面量未闭合"))
}

/// 解码单个转义序列，返回解码结果与新的偏移。
fn decode_escape_sequence(input: &str, cursor: usize) -> Result<(String, usize), ParseError> {
    let rest = &input[cursor..];
    if !rest.starts_with('\\') {
        return Err(ParseError::invalid("内部错误：转义序列应以反斜杠开头"));
    }

    let mut chars = rest.chars();
    chars.next();
    let Some(escape) = chars.next()
    else {
        return Err(ParseError::invalid("不完整的转义序列"));
    };

    let next_cursor = cursor + '\\'.len_utf8() + escape.len_utf8();
    let decoded = match escape {
        'n' => "\n".to_string(),
        't' => "\t".to_string(),
        'r' => "\r".to_string(),
        '\\' => "\\".to_string(),
        '"' => "\"".to_string(),
        '0' => "\0".to_string(),
        '{' => "{".to_string(),
        '}' => "}".to_string(),
        'u' => {
            let unicode_rest = &input[next_cursor..];
            let Some(hex_body) = unicode_rest.strip_prefix('{').and_then(|value| value.split_once('}').map(|(hex, _)| hex))
            else {
                return Err(ParseError::invalid("Unicode 转义需要使用 \\u{...} 形式"));
            };
            let hex_digits = hex_body.trim();
            let codepoint =
                u32::from_str_radix(hex_digits, 16).map_err(|_| ParseError::invalid(format!("无效的 Unicode 码点: {}", hex_digits)))?;
            let Some(ch) = char::from_u32(codepoint)
            else {
                return Err(ParseError::invalid(format!("无效的 Unicode 标量值: {}", hex_digits)));
            };
            let consumed = 2 + unicode_rest.find('}').unwrap() + 1;
            return Ok((ch.to_string(), cursor + consumed));
        }
        other => {
            return Err(ParseError::invalid(format!("不支持的转义序列 \\{}", other)));
        }
    };

    Ok((decoded, next_cursor))
}

/// 将暂存文本写回片段列表，并自动合并相邻文本片段。
fn push_text_segment(segments: &mut Vec<StringSegment>, text: &mut String) {
    if text.is_empty() {
        return;
    }

    if let Some(StringSegment::Text(existing)) = segments.last_mut() {
        existing.push_str(text);
        text.clear();
        return;
    }

    segments.push(StringSegment::Text(std::mem::take(text)));
}

fn span(start: usize, end: usize) -> Range<usize> {
    start..end
}

fn expression_requires_statement(expression: &TermExpression) -> bool {
    matches!(
        expression,
        TermExpression::Return { .. }
            | TermExpression::Break { .. }
            | TermExpression::Continue { .. }
            | TermExpression::Yield { .. }
            | TermExpression::YieldFrom { .. }
            | TermExpression::Fallthrough { .. }
            | TermExpression::If(_)
            | TermExpression::Loop(_)
            | TermExpression::Case { .. }
            | TermExpression::Match { .. }
    )
}

fn with_term_span(expression: TermExpression, span: Range<usize>) -> TermExpression {
    match expression {
        TermExpression::Name { path, .. } => TermExpression::Name { path, span },
        TermExpression::Literal { literal, .. } => TermExpression::Literal { literal, span },
        TermExpression::Unary(term_unary) => {
            let TermUnaryExpression { operator, base: expr, .. } = *term_unary;
            TermExpression::Unary(Box::new(TermUnaryExpression { operator, base: expr, span }))
        }
        TermExpression::Binary(term_binary) => {
            let TermBinaryExpression { operator, lhs, rhs, .. } = *term_binary;
            TermExpression::Binary(Box::new(TermBinaryExpression { operator, lhs, rhs, span }))
        }
        TermExpression::Call { callee, args, .. } => TermExpression::Call { callee, args, span },
        TermExpression::MemberAccess { object, member, .. } => TermExpression::MemberAccess { object, member, span },
        TermExpression::Subscript { object, index, kind, .. } => TermExpression::Subscript { object, index, kind, span },
        TermExpression::Tuple { items, .. } => TermExpression::Tuple { items, span },
        TermExpression::Array { items, .. } => TermExpression::Array { items, span },
        TermExpression::As(term_as) => {
            let TermAsExpression { base: expr, target: ty, .. } = *term_as;
            TermExpression::As(Box::new(TermAsExpression { base: expr, target: ty, span }))
        }
        TermExpression::Turbofish { expr, arguments, .. } => TermExpression::Turbofish { expr, arguments, span },
        TermExpression::Assign { target, value, .. } => TermExpression::Assign { target, value, span },
        TermExpression::Return { value, .. } => TermExpression::Return { value, span },
        TermExpression::Break { label, value, .. } => TermExpression::Break { label, value, span },
        TermExpression::Continue { label, .. } => TermExpression::Continue { label, span },
        TermExpression::Yield { value, .. } => TermExpression::Yield { value, span },
        TermExpression::YieldFrom { value, .. } => TermExpression::YieldFrom { value, span },
        TermExpression::Raise { value, .. } => TermExpression::Raise { value, span },
        TermExpression::Resume { value, .. } => TermExpression::Resume { value, span },
        TermExpression::Catch { expr, arms, .. } => TermExpression::Catch { expr, arms, span },
        TermExpression::If(if_stmt) => {
            let IfStatement { condition, then_body, else_body, .. } = *if_stmt;
            TermExpression::If(Box::new(IfStatement { condition, then_body, else_body, span }))
        }
        TermExpression::Loop(loop_stmt) => {
            let LoopStatement { label, pattern, iterator, condition, body, .. } = *loop_stmt;
            TermExpression::Loop(Box::new(LoopStatement { label, pattern, iterator, condition, body, span }))
        }
        TermExpression::Match { scrutinee, arms, .. } => TermExpression::Match { scrutinee, arms, span },
        TermExpression::Case { scrutinee, arms, .. } => TermExpression::Case { scrutinee, arms, span },
        TermExpression::Construct { path, fields, .. } => TermExpression::Construct { path, fields, span },
        TermExpression::Lambda { params, return_type, body, .. } => TermExpression::Lambda { params, return_type, body, span },
        TermExpression::Block { body, .. } => TermExpression::Block { body, span },
        TermExpression::Fallthrough { .. } => TermExpression::Fallthrough { span },
    }
}

fn with_type_span(expression: TypeExpression, span: Range<usize>) -> TypeExpression {
    match expression {
        TypeExpression::Path(mut path) => {
            path.span = span;
            TypeExpression::Path(path)
        }
        TypeExpression::Array { item, .. } => TypeExpression::Array { item, span },
        TypeExpression::Tuple { items, .. } => TypeExpression::Tuple { items, span },
        TypeExpression::Associated { name, ty, .. } => TypeExpression::Associated { name, ty, span },
        TypeExpression::Nullable { item, .. } => TypeExpression::Nullable { item, span },
        TypeExpression::Function { params, return_type, .. } => TypeExpression::Function { params, return_type, span },
    }
}
