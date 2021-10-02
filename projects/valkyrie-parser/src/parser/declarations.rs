use crate::{
    ast::{
        Annotations, AttributeArgument, AttributeDeclaration, AttributeItem, AttributeList, ClassDeclaration, DeclarationBody,
        FunctionDeclaration, FunctionParameter, FunctionStatement, IdentifierNode, ImplyAssociatedConstBinding, ImplyAssociatedTypeBinding,
        ImplyDeclaration, InheritanceItem, NamePath, NamespaceDeclaration, ObjectBody, ObjectFieldDeclaration, ObjectMethodDeclaration,
        ParameterPassingKind, RootStatement, TraitAssociatedConstDeclaration, TraitAssociatedTypeDeclaration, TraitDeclaration,
        TypeAliasDeclaration, UniteDeclaration, UniteVariantDeclaration, UsingStatement,
    },
    lexer::{Keyword, TokenKind},
};

use super::{span, ParseError, Parser, DECLARATION_MODIFIERS};

impl<'a> Parser<'a> {
    pub(super) fn parse_annotations(&mut self) -> Result<Annotations, ParseError> {
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

    pub(super) fn parse_declaration(&mut self, annotations: Annotations) -> Result<RootStatement, ParseError> {
        if self.check_token_keyword(Keyword::Namespace) {
            return Ok(RootStatement::Namespace(self.parse_namespace()?));
        }
        if self.check_token_keyword(Keyword::Using) {
            return Ok(RootStatement::Using(self.parse_using()?));
        }
        if self.check_token_keyword(Keyword::Micro) {
            return Ok(RootStatement::Function(self.parse_function_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Class) || self.check_token_keyword(Keyword::Structure) {
            return Ok(RootStatement::Class(self.parse_class_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Trait) {
            return Ok(RootStatement::Trait(self.parse_trait_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Imply) {
            return Ok(RootStatement::Imply(self.parse_imply_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Unite) {
            return Ok(RootStatement::Unite(self.parse_unite_declaration(annotations)?));
        }
        if self.check_identifier_text_eq("attribute") {
            return Ok(RootStatement::Attribute(self.parse_attribute_declaration(annotations)?));
        }
        if self.check_token_keyword(Keyword::Type) {
            return Ok(RootStatement::TypeAlias(self.parse_type_alias_declaration(annotations)?));
        }
        if self.check_identifier_text_eq("extern") {
            return Ok(RootStatement::Function(self.parse_extern_function_declaration(annotations)?));
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
                    RootStatement::Function(func) => {
                        let path = NamePath { parts: vec![func.name.as_str().to_string()], span: func.span.clone() };
                        statements.push(FunctionStatement::Term {
                            span: func.span.clone(),
                            expression: crate::ast::TermExpression::Name { path, span: func.span.clone() },
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

    pub(super) fn parse_parameter_list(&mut self) -> Result<Vec<FunctionParameter>, ParseError> {
        self.expect_symbol(TokenKind::LParen)?;
        let params = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_parameter())?;
        self.expect_symbol(TokenKind::RParen)?;
        Ok(params)
    }

    fn parse_parameter(&mut self) -> Result<FunctionParameter, ParseError> {
        let start = self.current().span.start;
        let passing = if self.match_token_keyword(Keyword::Own) {
            ParameterPassingKind::Own
        }
        else if self.match_token_keyword(Keyword::Mut) {
            ParameterPassingKind::Mut
        }
        else {
            self.match_token_keyword(Keyword::Ref);
            ParameterPassingKind::Ref
        };
        let is_mutable = matches!(passing, ParameterPassingKind::Mut);
        let name_start = self.current().span.start;
        let name = self.expect_identifier_text()?.to_string();
        let parameter_type = if self.match_symbol(TokenKind::Colon) { Some(self.parse_type_expression_bp(0)?) } else { None };

        Ok(FunctionParameter {
            name: IdentifierNode::new(valkyrie_types::Identifier::new(&name), span(name_start, self.previous().span.end)),
            parameter_type,
            is_mutable,
            passing,
            span: span(start, self.previous().span.end),
        })
    }

    pub(super) fn parse_inheritance_item(&mut self) -> Result<InheritanceItem, ParseError> {
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
}
