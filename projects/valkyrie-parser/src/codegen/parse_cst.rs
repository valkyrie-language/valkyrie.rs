use super::*;

pub(super) fn parse_cst(input: &str, rule: ValkyrieRule) -> OutputResult<ValkyrieRule> {
    state(input, |state| match rule {
        ValkyrieRule::Program => parse_program(state),
        ValkyrieRule::Statement => parse_statement(state),
        ValkyrieRule::EOS => parse_eos(state),
        ValkyrieRule::EOS_FREE => parse_eos_free(state),
        ValkyrieRule::DefineNamespace => parse_define_namespace(state),
        ValkyrieRule::OP_NAMESPACE => parse_op_namespace(state),
        ValkyrieRule::DefineImport => parse_define_import(state),
        ValkyrieRule::ImportBlock => parse_import_block(state),
        ValkyrieRule::ImportTerm => parse_import_term(state),
        ValkyrieRule::ImportAll => parse_import_all(state),
        ValkyrieRule::ImportSpace => parse_import_space(state),
        ValkyrieRule::ImportName => parse_import_name(state),
        ValkyrieRule::ImportAs => parse_import_as(state),
        ValkyrieRule::ImportNameItem => parse_import_name_item(state),
        ValkyrieRule::DefineConstraint => parse_define_constraint(state),
        ValkyrieRule::ConstraintParameters => parse_constraint_parameters(state),
        ValkyrieRule::ConstraintBlock => parse_constraint_block(state),
        ValkyrieRule::ConstraintStatement => parse_constraint_statement(state),
        ValkyrieRule::ConstraintImplements => parse_constraint_implements(state),
        ValkyrieRule::WhereBlock => parse_where_block(state),
        ValkyrieRule::WhereBound => parse_where_bound(state),
        ValkyrieRule::DefineClass => parse_define_class(state),
        ValkyrieRule::ClassBlock => parse_class_block(state),
        ValkyrieRule::ClassTerm => parse_class_term(state),
        ValkyrieRule::KW_CLASS => parse_kw_class(state),
        ValkyrieRule::DefineField => parse_define_field(state),
        ValkyrieRule::ParameterDefault => parse_parameter_default(state),
        ValkyrieRule::DefineMethod => parse_define_method(state),
        ValkyrieRule::DefineDomain => parse_define_domain(state),
        ValkyrieRule::DomainTerm => parse_domain_term(state),
        ValkyrieRule::DefineInherit => parse_define_inherit(state),
        ValkyrieRule::InheritTerm => parse_inherit_term(state),
        ValkyrieRule::ObjectStatement => parse_object_statement(state),
        ValkyrieRule::DefineEnumerate => parse_define_enumerate(state),
        ValkyrieRule::FlagTerm => parse_flag_term(state),
        ValkyrieRule::FlagField => parse_flag_field(state),
        ValkyrieRule::KW_FLAGS => parse_kw_flags(state),
        ValkyrieRule::DefineUnion => parse_define_union(state),
        ValkyrieRule::UnionTerm => parse_union_term(state),
        ValkyrieRule::DefineVariant => parse_define_variant(state),
        ValkyrieRule::KW_UNION => parse_kw_union(state),
        ValkyrieRule::DefineTrait => parse_define_trait(state),
        ValkyrieRule::DefineExtends => parse_define_extends(state),
        ValkyrieRule::TraitBlock => parse_trait_block(state),
        ValkyrieRule::TraitTerm => parse_trait_term(state),
        ValkyrieRule::KW_TRAIT => parse_kw_trait(state),
        ValkyrieRule::DefineFunction => parse_define_function(state),
        ValkyrieRule::DefineLambda => parse_define_lambda(state),
        ValkyrieRule::FunctionMiddle => parse_function_middle(state),
        ValkyrieRule::TypeHint => parse_type_hint(state),
        ValkyrieRule::TypeReturn => parse_type_return(state),
        ValkyrieRule::TypeEffect => parse_type_effect(state),
        ValkyrieRule::FunctionParameters => parse_function_parameters(state),
        ValkyrieRule::ParameterItem => parse_parameter_item(state),
        ValkyrieRule::ParameterItemControl => parse_parameter_item_control(state),
        ValkyrieRule::ParameterPair => parse_parameter_pair(state),
        ValkyrieRule::ParameterHint => parse_parameter_hint(state),
        ValkyrieRule::Continuation => parse_continuation(state),
        ValkyrieRule::KW_FUNCTION => parse_kw_function(state),
        ValkyrieRule::DefineVariable => parse_define_variable(state),
        ValkyrieRule::LetPattern => parse_let_pattern(state),
        ValkyrieRule::StandardPattern => parse_standard_pattern(state),
        ValkyrieRule::BarePattern => parse_bare_pattern(state),
        ValkyrieRule::BarePatternItem => parse_bare_pattern_item(state),
        ValkyrieRule::TuplePattern => parse_tuple_pattern(state),
        ValkyrieRule::PatternItem => parse_pattern_item(state),
        ValkyrieRule::TuplePatternItem => parse_tuple_pattern_item(state),
        ValkyrieRule::WhileStatement => parse_while_statement(state),
        ValkyrieRule::KW_WHILE => parse_kw_while(state),
        ValkyrieRule::ForStatement => parse_for_statement(state),
        ValkyrieRule::IfGuard => parse_if_guard(state),
        ValkyrieRule::ControlFlow => parse_control_flow(state),
        ValkyrieRule::JumpLabel => parse_jump_label(state),
        ValkyrieRule::ExpressionRoot => parse_expression_root(state),
        ValkyrieRule::MatchExpression => parse_match_expression(state),
        ValkyrieRule::SwitchStatement => parse_switch_statement(state),
        ValkyrieRule::MatchBlock => parse_match_block(state),
        ValkyrieRule::MatchTerms => parse_match_terms(state),
        ValkyrieRule::MatchType => parse_match_type(state),
        ValkyrieRule::MatchCase => parse_match_case(state),
        ValkyrieRule::CasePattern => parse_case_pattern(state),
        ValkyrieRule::MatchWhen => parse_match_when(state),
        ValkyrieRule::MatchElse => parse_match_else(state),
        ValkyrieRule::MatchStatement => parse_match_statement(state),
        ValkyrieRule::KW_MATCH => parse_kw_match(state),
        ValkyrieRule::BIND_L => parse_bind_l(state),
        ValkyrieRule::BIND_R => parse_bind_r(state),
        ValkyrieRule::DotMatchCall => parse_dot_match_call(state),
        ValkyrieRule::MainExpression => parse_main_expression(state),
        ValkyrieRule::MainTerm => parse_main_term(state),
        ValkyrieRule::MainFactor => parse_main_factor(state),
        ValkyrieRule::GroupFactor => parse_group_factor(state),
        ValkyrieRule::Leading => parse_leading(state),
        ValkyrieRule::MainSuffixTerm => parse_main_suffix_term(state),
        ValkyrieRule::MainPrefix => parse_main_prefix(state),
        ValkyrieRule::TypePrefix => parse_type_prefix(state),
        ValkyrieRule::MainInfix => parse_main_infix(state),
        ValkyrieRule::TypeInfix => parse_type_infix(state),
        ValkyrieRule::MainSuffix => parse_main_suffix(state),
        ValkyrieRule::TypeSuffix => parse_type_suffix(state),
        ValkyrieRule::InlineExpression => parse_inline_expression(state),
        ValkyrieRule::InlineTerm => parse_inline_term(state),
        ValkyrieRule::InlineSuffixTerm => parse_inline_suffix_term(state),
        ValkyrieRule::TypeExpression => parse_type_expression(state),
        ValkyrieRule::TypeTerm => parse_type_term(state),
        ValkyrieRule::TypeFactor => parse_type_factor(state),
        ValkyrieRule::TypeSuffixTerm => parse_type_suffix_term(state),
        ValkyrieRule::TryStatement => parse_try_statement(state),
        ValkyrieRule::NewStatement => parse_new_statement(state),
        ValkyrieRule::NewBlock => parse_new_block(state),
        ValkyrieRule::NewPair => parse_new_pair(state),
        ValkyrieRule::NewPairKey => parse_new_pair_key(state),
        ValkyrieRule::DotCall => parse_dot_call(state),
        ValkyrieRule::DotCallItem => parse_dot_call_item(state),
        ValkyrieRule::DotClosureCall => parse_dot_closure_call(state),
        ValkyrieRule::InlineTupleCall => parse_inline_tuple_call(state),
        ValkyrieRule::TupleCall => parse_tuple_call(state),
        ValkyrieRule::TupleLiteral => parse_tuple_literal(state),
        ValkyrieRule::TupleLiteralStrict => parse_tuple_literal_strict(state),
        ValkyrieRule::TupleTerms => parse_tuple_terms(state),
        ValkyrieRule::TuplePair => parse_tuple_pair(state),
        ValkyrieRule::TupleKey => parse_tuple_key(state),
        ValkyrieRule::RangeCall => parse_range_call(state),
        ValkyrieRule::RangeLiteral => parse_range_literal(state),
        ValkyrieRule::RangeLiteralIndex0 => parse_range_literal_index_0(state),
        ValkyrieRule::RangeLiteralIndex1 => parse_range_literal_index_1(state),
        ValkyrieRule::SubscriptAxis => parse_subscript_axis(state),
        ValkyrieRule::SubscriptOnly => parse_subscript_only(state),
        ValkyrieRule::SubscriptRange => parse_subscript_range(state),
        ValkyrieRule::RangeOmit => parse_range_omit(state),
        ValkyrieRule::DefineGeneric => parse_define_generic(state),
        ValkyrieRule::GenericParameter => parse_generic_parameter(state),
        ValkyrieRule::GenericParameterPair => parse_generic_parameter_pair(state),
        ValkyrieRule::GenericCall => parse_generic_call(state),
        ValkyrieRule::GenericHide => parse_generic_hide(state),
        ValkyrieRule::GenericTerms => parse_generic_terms(state),
        ValkyrieRule::GenericPair => parse_generic_pair(state),
        ValkyrieRule::AnnotationHead => parse_annotation_head(state),
        ValkyrieRule::AnnotationMix => parse_annotation_mix(state),
        ValkyrieRule::AnnotationTerm => parse_annotation_term(state),
        ValkyrieRule::AnnotationTermMix => parse_annotation_term_mix(state),
        ValkyrieRule::AttributeList => parse_attribute_list(state),
        ValkyrieRule::AttributeCall => parse_attribute_call(state),
        ValkyrieRule::AttributeItem => parse_attribute_item(state),
        ValkyrieRule::AttributeName => parse_attribute_name(state),
        ValkyrieRule::ProceduralCall => parse_procedural_call(state),
        ValkyrieRule::ProceduralName => parse_procedural_name(state),
        ValkyrieRule::TextLiteral => parse_text_literal(state),
        ValkyrieRule::TextRaw => parse_text_raw(state),
        ValkyrieRule::Text_L => parse_text_l(state),
        ValkyrieRule::Text_R => parse_text_r(state),
        ValkyrieRule::Text_X => parse_text_x(state),
        ValkyrieRule::TEXT_CONTENT1 => parse_text_content_1(state),
        ValkyrieRule::TEXT_CONTENT2 => parse_text_content_2(state),
        ValkyrieRule::TEXT_CONTENT3 => parse_text_content_3(state),
        ValkyrieRule::TEXT_CONTENT4 => parse_text_content_4(state),
        ValkyrieRule::TEXT_CONTENT5 => parse_text_content_5(state),
        ValkyrieRule::TEXT_CONTENT6 => parse_text_content_6(state),
        ValkyrieRule::ModifierCall => parse_modifier_call(state),
        ValkyrieRule::ModifierAhead => parse_modifier_ahead(state),
        ValkyrieRule::KEYWORDS_STOP => parse_keywords_stop(state),
        ValkyrieRule::IDENTIFIER_STOP => parse_identifier_stop(state),
        ValkyrieRule::Slot => parse_slot(state),
        ValkyrieRule::SlotItem => parse_slot_item(state),
        ValkyrieRule::NamepathFree => parse_namepath_free(state),
        ValkyrieRule::Namepath => parse_namepath(state),
        ValkyrieRule::Identifier => parse_identifier(state),
        ValkyrieRule::IdentifierBare => parse_identifier_bare(state),
        ValkyrieRule::IdentifierRaw => parse_identifier_raw(state),
        ValkyrieRule::IdentifierRawText => parse_identifier_raw_text(state),
        ValkyrieRule::Special => parse_special(state),
        ValkyrieRule::Number => parse_number(state),
        ValkyrieRule::Sign => parse_sign(state),
        ValkyrieRule::Integer => parse_integer(state),
        ValkyrieRule::DigitsX => parse_digits_x(state),
        ValkyrieRule::Decimal => parse_decimal(state),
        ValkyrieRule::DecimalX => parse_decimal_x(state),
        ValkyrieRule::PROPORTION => parse_proportion(state),
        ValkyrieRule::NS_CONCAT => parse_ns_concat(state),
        ValkyrieRule::COLON => parse_colon(state),
        ValkyrieRule::ARROW1 => parse_arrow_1(state),
        ValkyrieRule::COMMA => parse_comma(state),
        ValkyrieRule::DOT => parse_dot(state),
        ValkyrieRule::OP_SLOT => parse_op_slot(state),
        ValkyrieRule::OFFSET_L => parse_offset_l(state),
        ValkyrieRule::OFFSET_R => parse_offset_r(state),
        ValkyrieRule::PROPORTION2 => parse_proportion_2(state),
        ValkyrieRule::OP_IMPORT_ALL => parse_op_import_all(state),
        ValkyrieRule::OP_AND_THEN => parse_op_and_then(state),
        ValkyrieRule::OP_BIND => parse_op_bind(state),
        ValkyrieRule::KW_CONTROL => parse_kw_control(state),
        ValkyrieRule::KW_NAMESPACE => parse_kw_namespace(state),
        ValkyrieRule::KW_IMPORT => parse_kw_import(state),
        ValkyrieRule::KW_CONSTRAINT => parse_kw_constraint(state),
        ValkyrieRule::KW_WHERE => parse_kw_where(state),
        ValkyrieRule::KW_IMPLEMENTS => parse_kw_implements(state),
        ValkyrieRule::KW_EXTENDS => parse_kw_extends(state),
        ValkyrieRule::KW_INHERITS => parse_kw_inherits(state),
        ValkyrieRule::KW_FOR => parse_kw_for(state),
        ValkyrieRule::KW_END => parse_kw_end(state),
        ValkyrieRule::KW_LET => parse_kw_let(state),
        ValkyrieRule::KW_NEW => parse_kw_new(state),
        ValkyrieRule::KW_OBJECT => parse_kw_object(state),
        ValkyrieRule::KW_LAMBDA => parse_kw_lambda(state),
        ValkyrieRule::KW_IF => parse_kw_if(state),
        ValkyrieRule::KW_SWITCH => parse_kw_switch(state),
        ValkyrieRule::KW_TRY => parse_kw_try(state),
        ValkyrieRule::KW_TYPE => parse_kw_type(state),
        ValkyrieRule::KW_CASE => parse_kw_case(state),
        ValkyrieRule::KW_WHEN => parse_kw_when(state),
        ValkyrieRule::KW_ELSE => parse_kw_else(state),
        ValkyrieRule::KW_NOT => parse_kw_not(state),
        ValkyrieRule::KW_IN => parse_kw_in(state),
        ValkyrieRule::KW_IS => parse_kw_is(state),
        ValkyrieRule::KW_AS => parse_kw_as(state),
        ValkyrieRule::Shebang => parse_shebang(state),
        ValkyrieRule::WhiteSpace => parse_white_space(state),
        ValkyrieRule::SkipSpace => parse_skip_space(state),
        ValkyrieRule::Comment => parse_comment(state),
        ValkyrieRule::StringInterpolations => parse_string_interpolations(state),
        ValkyrieRule::StringInterpolationTerm => parse_string_interpolation_term(state),
        ValkyrieRule::EscapeCharacter => parse_escape_character(state),
        ValkyrieRule::EscapeUnicode => parse_escape_unicode(state),
        ValkyrieRule::EscapeUnicodeCode => parse_escape_unicode_code(state),
        ValkyrieRule::StringInterpolationSimple => parse_string_interpolation_simple(state),
        ValkyrieRule::StringInterpolationText => parse_string_interpolation_text(state),
        ValkyrieRule::StringFormatter => parse_string_formatter(state),
        ValkyrieRule::StringInterpolationComplex => parse_string_interpolation_complex(state),
        ValkyrieRule::StringTemplates => parse_string_templates(state),
        ValkyrieRule::StringTemplateTerm => parse_string_template_term(state),
        ValkyrieRule::ExpressionTemplate => parse_expression_template(state),
        ValkyrieRule::ForTemplate => parse_for_template(state),
        ValkyrieRule::ForTemplateBegin => parse_for_template_begin(state),
        ValkyrieRule::ForTemplateElse => parse_for_template_else(state),
        ValkyrieRule::ForTemplateEnd => parse_for_template_end(state),
        ValkyrieRule::TEMPLATE_S => parse_template_s(state),
        ValkyrieRule::TEMPLATE_E => parse_template_e(state),
        ValkyrieRule::TEMPLATE_L => parse_template_l(state),
        ValkyrieRule::TEMPLATE_R => parse_template_r(state),
        ValkyrieRule::TEMPLATE_M => parse_template_m(state),
        ValkyrieRule::HiddenText => unreachable!(),
    })
}
#[inline]
fn parse_program(state: Input) -> Output {
    state.rule(ValkyrieRule::Program, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.start_of_input())
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_shebang(s).and_then(|s| s.tag_node("shebang"))))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_statement(s).and_then(|s| s.tag_node("statement")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.end_of_input())
        })
    })
}
#[inline]
fn parse_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::Statement, |s| {
        Err(s)
            .or_else(|s| parse_define_namespace(s).and_then(|s| s.tag_node("define_namespace")))
            .or_else(|s| parse_define_class(s).and_then(|s| s.tag_node("define_class")))
            .or_else(|s| parse_define_union(s).and_then(|s| s.tag_node("define_union")))
            .or_else(|s| parse_define_enumerate(s).and_then(|s| s.tag_node("define_enumerate")))
            .or_else(|s| parse_define_trait(s).and_then(|s| s.tag_node("define_trait")))
            .or_else(|s| parse_define_extends(s).and_then(|s| s.tag_node("define_extends")))
            .or_else(|s| parse_define_function(s).and_then(|s| s.tag_node("define_function")))
            .or_else(|s| parse_define_variable(s).and_then(|s| s.tag_node("define_variable")))
            .or_else(|s| parse_define_import(s).and_then(|s| s.tag_node("define_import")))
            .or_else(|s| parse_control_flow(s).and_then(|s| s.tag_node("control_flow")))
            .or_else(|s| parse_while_statement(s).and_then(|s| s.tag_node("while_statement")))
            .or_else(|s| parse_for_statement(s).and_then(|s| s.tag_node("for_statement")))
            .or_else(|s| parse_expression_root(s).and_then(|s| s.tag_node("expression_root")))
            .or_else(|s| parse_eos(s).and_then(|s| s.tag_node("eos")))
    })
}
#[inline]
fn parse_eos(state: Input) -> Output {
    state.rule(ValkyrieRule::EOS, |s| {
        Err(s)
            .or_else(|s| {
                builtin_regex(s, {
                    static REGEX: OnceLock<Regex> = OnceLock::new();
                    REGEX.get_or_init(|| Regex::new("^(?x)([;；])").unwrap())
                })
                .and_then(|s| s.tag_node("omit"))
            })
            .or_else(|s| {
                builtin_regex(s, {
                    static REGEX: OnceLock<Regex> = OnceLock::new();
                    REGEX.get_or_init(|| Regex::new("^(?x)(⁏|;;)").unwrap())
                })
                .and_then(|s| s.tag_node("show"))
            })
    })
}
#[inline]
fn parse_eos_free(state: Input) -> Output {
    state.rule(ValkyrieRule::EOS_FREE, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([,，;；⁏])").unwrap())
        })
    })
}
#[inline]
fn parse_define_namespace(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineNamespace, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_namespace(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_op_namespace(s).and_then(|s| s.tag_node("op_namespace"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_namepath_free(s).and_then(|s| s.tag_node("namepath_free")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_op_namespace(state: Input) -> Output {
    state.rule(ValkyrieRule::OP_NAMESPACE, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, "!", false).and_then(|s| s.tag_node("main")))
            .or_else(|s| builtin_text(s, "?", false).and_then(|s| s.tag_node("test")))
            .or_else(|s| builtin_text(s, "*", false).and_then(|s| s.tag_node("hide")))
    })
}
#[inline]
fn parse_define_import(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineImport, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term(s).and_then(|s| s.tag_node("annotation_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_import(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    Err(s)
                        .or_else(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| parse_import_block(s).and_then(|s| s.tag_node("import_block")))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| s.optional(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free"))))
                            })
                        })
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| parse_import_term(s).and_then(|s| s.tag_node("import_term")))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| s.optional(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free"))))
                            })
                        })
                })
        })
    })
}
#[inline]
fn parse_import_block(state: Input) -> Output {
    state.rule(ValkyrieRule::ImportBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_import_term(s).and_then(|s| s.tag_node("import_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_import_term(state: Input) -> Output {
    state.rule(ValkyrieRule::ImportTerm, |s| {
        Err(s)
            .or_else(|s| parse_import_all(s).and_then(|s| s.tag_node("import_all")))
            .or_else(|s| parse_import_space(s).and_then(|s| s.tag_node("import_space")))
            .or_else(|s| parse_import_name(s).and_then(|s| s.tag_node("import_name")))
            .or_else(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
    })
}
#[inline]
fn parse_import_all(state: Input) -> Output {
    state.rule(ValkyrieRule::ImportAll, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(1..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("path")))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| parse_ns_concat(s))
                                })
                            })
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_op_import_all(s))
        })
    })
}
#[inline]
fn parse_import_space(state: Input) -> Output {
    state.rule(ValkyrieRule::ImportSpace, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("path")))
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| {
                                s.repeat(0..4294967295, |s| {
                                    s.sequence(|s| {
                                        Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                            s.sequence(|s| {
                                                Ok(s)
                                                    .and_then(|s| {
                                                        s.sequence(|s| {
                                                            Ok(s)
                                                                .and_then(|s| parse_ns_concat(s))
                                                                .and_then(|s| builtin_ignore(s))
                                                        })
                                                    })
                                                    .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("path")))
                                            })
                                        })
                                    })
                                })
                            })
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| s.optional(|s| parse_ns_concat(s)))
                            .and_then(|s| builtin_ignore(s))
                    })
                })
                .and_then(|s| parse_import_block(s).and_then(|s| s.tag_node("body")))
        })
    })
}
#[inline]
fn parse_import_name(state: Input) -> Output {
    state.rule(ValkyrieRule::ImportName, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| {
                                            s.repeat(0..4294967295, |s| {
                                                s.sequence(|s| {
                                                    Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                        s.sequence(|s| {
                                                            Ok(s)
                                                                .and_then(|s| {
                                                                    parse_identifier(s).and_then(|s| s.tag_node("path"))
                                                                })
                                                                .and_then(|s| builtin_ignore(s))
                                                                .and_then(|s| parse_ns_concat(s))
                                                        })
                                                    })
                                                })
                                            })
                                        })
                                        .and_then(|s| builtin_ignore(s))
                                })
                            })
                            .and_then(|s| parse_import_name_item(s).and_then(|s| s.tag_node("item")))
                            .and_then(|s| builtin_ignore(s))
                    })
                })
                .and_then(|s| parse_import_as(s).and_then(|s| s.tag_node("alias")))
        })
    })
}
#[inline]
fn parse_import_as(state: Input) -> Output {
    state.rule(ValkyrieRule::ImportAs, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| s.sequence(|s| Ok(s).and_then(|s| parse_kw_as(s)).and_then(|s| builtin_ignore(s))))
                    .and_then(|s| parse_import_name_item(s).and_then(|s| s.tag_node("alias")))
            })
        })
    })
}
#[inline]
fn parse_import_name_item(state: Input) -> Output {
    state.rule(ValkyrieRule::ImportNameItem, |s| {
        Err(s)
            .or_else(|s| parse_procedural_name(s).and_then(|s| s.tag_node("procedural_name")))
            .or_else(|s| parse_attribute_name(s).and_then(|s| s.tag_node("attribute_name")))
            .or_else(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
    })
}
#[inline]
fn parse_define_constraint(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineConstraint, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_head(s).and_then(|s| s.tag_node("annotation_head")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_constraint(s).and_then(|s| s.tag_node("kw_constraint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_constraint_parameters(s).and_then(|s| s.tag_node("constraint_parameters"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_constraint_block(s).and_then(|s| s.tag_node("constraint_block")))
        })
    })
}
#[inline]
fn parse_constraint_parameters(state: Input) -> Output {
    state.rule(ValkyrieRule::ConstraintParameters, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| {
                            s.repeat(0..4294967295, |s| {
                                s.sequence(|s| {
                                    Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                        s.sequence(|s| {
                                            Ok(s)
                                                .and_then(|s| parse_comma(s))
                                                .and_then(|s| builtin_ignore(s))
                                                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                                        })
                                    })
                                })
                            })
                        })
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| s.optional(|s| parse_comma(s)))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "<", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| {
                            s.repeat(0..4294967295, |s| {
                                s.sequence(|s| {
                                    Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                        s.sequence(|s| {
                                            Ok(s)
                                                .and_then(|s| parse_comma(s))
                                                .and_then(|s| builtin_ignore(s))
                                                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                                        })
                                    })
                                })
                            })
                        })
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| s.optional(|s| parse_comma(s)))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ">", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "⟨", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| {
                            s.repeat(0..4294967295, |s| {
                                s.sequence(|s| {
                                    Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                        s.sequence(|s| {
                                            Ok(s)
                                                .and_then(|s| parse_comma(s))
                                                .and_then(|s| builtin_ignore(s))
                                                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                                        })
                                    })
                                })
                            })
                        })
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| s.optional(|s| parse_comma(s)))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "⟩", false))
                })
            })
    })
}
#[inline]
fn parse_constraint_block(state: Input) -> Output {
    state.rule(ValkyrieRule::ConstraintBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                Err(s)
                                    .or_else(|s| parse_constraint_statement(s).and_then(|s| s.tag_node("constraint_statement")))
                                    .or_else(|s| {
                                        parse_constraint_implements(s).and_then(|s| s.tag_node("constraint_implements"))
                                    })
                                    .or_else(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
                            })
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_constraint_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::ConstraintStatement, |s| parse_where_block(s).and_then(|s| s.tag_node("where_block")))
}
#[inline]
fn parse_constraint_implements(state: Input) -> Output {
    state.rule(ValkyrieRule::ConstraintImplements, |s| parse_kw_implements(s).and_then(|s| s.tag_node("kw_implements")))
}
#[inline]
fn parse_where_block(state: Input) -> Output {
    state.rule(ValkyrieRule::WhereBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_where(s).and_then(|s| s.tag_node("kw_where")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_where_bound(s).and_then(|s| s.tag_node("where_bound")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_where_bound(state: Input) -> Output {
    state.rule(ValkyrieRule::WhereBound, |s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
}
#[inline]
fn parse_define_class(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineClass, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_define_constraint(s).and_then(|s| s.tag_node("define_constraint"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_annotation_head(s).and_then(|s| s.tag_node("annotation_head")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_class(s).and_then(|s| s.tag_node("kw_class")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_generic(s).and_then(|s| s.tag_node("define_generic"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_inherit(s).and_then(|s| s.tag_node("define_inherit"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_class_block(s).and_then(|s| s.tag_node("class_block")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_class_block(state: Input) -> Output {
    state.rule(ValkyrieRule::ClassBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_class_term(s).and_then(|s| s.tag_node("class_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_class_term(state: Input) -> Output {
    state.rule(ValkyrieRule::ClassTerm, |s| {
        Err(s)
            .or_else(|s| parse_procedural_call(s).and_then(|s| s.tag_node("procedural_call")))
            .or_else(|s| parse_define_method(s).and_then(|s| s.tag_node("define_method")))
            .or_else(|s| parse_define_domain(s).and_then(|s| s.tag_node("define_domain")))
            .or_else(|s| parse_define_field(s).and_then(|s| s.tag_node("define_field")))
            .or_else(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
    })
}
#[inline]
fn parse_kw_class(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_CLASS, |s| {
        Err(s).or_else(|s| builtin_text(s, "class", false)).or_else(|s| builtin_text(s, "structure", false))
    })
}
#[inline]
fn parse_define_field(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineField, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_mix(s).and_then(|s| s.tag_node("annotation_mix")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_parameter_default(s).and_then(|s| s.tag_node("parameter_default")))
        })
    })
}
#[inline]
fn parse_parameter_default(state: Input) -> Output {
    state.rule(ValkyrieRule::ParameterDefault, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| builtin_text(s, "=", false))
                    .and_then(|s| builtin_ignore(s))
                    .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
            })
        })
    })
}
#[inline]
fn parse_define_method(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineMethod, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_mix(s).and_then(|s| s.tag_node("annotation_mix")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_function_middle(s).and_then(|s| s.tag_node("function_middle")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation"))))
        })
    })
}
#[inline]
fn parse_define_domain(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineDomain, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_mix(s).and_then(|s| s.tag_node("annotation_mix")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_domain_term(s).and_then(|s| s.tag_node("domain_term")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_statement(s).and_then(|s| s.tag_node("statement")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_domain_term(state: Input) -> Output {
    state.rule(ValkyrieRule::DomainTerm, |s| Err(s).or_else(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier"))))
}
#[inline]
fn parse_define_inherit(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineInherit, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "(", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_inherit_term(s).and_then(|s| s.tag_node("inherit_term")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| {
                                    s.repeat(0..4294967295, |s| {
                                        s.sequence(|s| {
                                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                s.sequence(|s| {
                                                    Ok(s)
                                                        .and_then(|s| builtin_text(s, ",", false))
                                                        .and_then(|s| builtin_ignore(s))
                                                        .and_then(|s| {
                                                            parse_inherit_term(s).and_then(|s| s.tag_node("inherit_term"))
                                                        })
                                                })
                                            })
                                        })
                                    })
                                })
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| s.optional(|s| builtin_text(s, ",", false)))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, ")", false))
        })
    })
}
#[inline]
fn parse_inherit_term(state: Input) -> Output {
    state.rule(ValkyrieRule::InheritTerm, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_mix(s).and_then(|s| s.tag_node("annotation_mix")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression")))
        })
    })
}
#[inline]
fn parse_object_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::ObjectStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_object(s).and_then(|s| s.tag_node("kw_object")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_inherit(s).and_then(|s| s.tag_node("define_inherit"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_class_block(s).and_then(|s| s.tag_node("class_block")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_define_enumerate(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineEnumerate, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_head(s).and_then(|s| s.tag_node("annotation_head")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_flags(s).and_then(|s| s.tag_node("kw_flags")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_inherit(s).and_then(|s| s.tag_node("define_inherit"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_flag_term(s).and_then(|s| s.tag_node("flag_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_flag_term(state: Input) -> Output {
    state.rule(ValkyrieRule::FlagTerm, |s| {
        Err(s)
            .or_else(|s| parse_procedural_call(s).and_then(|s| s.tag_node("procedural_call")))
            .or_else(|s| parse_define_method(s).and_then(|s| s.tag_node("define_method")))
            .or_else(|s| parse_flag_field(s).and_then(|s| s.tag_node("flag_field")))
            .or_else(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
    })
}
#[inline]
fn parse_flag_field(state: Input) -> Output {
    state.rule(ValkyrieRule::FlagField, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_parameter_default(s).and_then(|s| s.tag_node("parameter_default")))
        })
    })
}
#[inline]
fn parse_kw_flags(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_FLAGS, |s| {
        Err(s).or_else(|s| builtin_text(s, "enumerate", false)).or_else(|s| builtin_text(s, "flags", false))
    })
}
#[inline]
fn parse_define_union(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineUnion, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_define_constraint(s).and_then(|s| s.tag_node("define_constraint"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_annotation_head(s).and_then(|s| s.tag_node("annotation_head")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_union(s).and_then(|s| s.tag_node("kw_union")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_generic(s).and_then(|s| s.tag_node("define_generic"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_inherit(s).and_then(|s| s.tag_node("define_inherit"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_union_term(s).and_then(|s| s.tag_node("union_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_union_term(state: Input) -> Output {
    state.rule(ValkyrieRule::UnionTerm, |s| {
        Err(s)
            .or_else(|s| parse_procedural_call(s).and_then(|s| s.tag_node("procedural_call")))
            .or_else(|s| parse_define_method(s).and_then(|s| s.tag_node("define_method")))
            .or_else(|s| parse_define_variant(s).and_then(|s| s.tag_node("define_variant")))
            .or_else(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
    })
}
#[inline]
fn parse_define_variant(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineVariant, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term(s).and_then(|s| s.tag_node("annotation_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_class_block(s).and_then(|s| s.tag_node("class_block"))))
        })
    })
}
#[inline]
fn parse_kw_union(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_UNION, |s| s.match_string("union", false))
}
#[inline]
fn parse_define_trait(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineTrait, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_define_constraint(s).and_then(|s| s.tag_node("define_constraint"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_annotation_head(s).and_then(|s| s.tag_node("annotation_head")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_trait(s).and_then(|s| s.tag_node("kw_trait")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_generic(s).and_then(|s| s.tag_node("define_generic"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_define_inherit(s).and_then(|s| s.tag_node("define_inherit"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_trait_block(s).and_then(|s| s.tag_node("trait_block")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_define_extends(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineExtends, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_define_constraint(s).and_then(|s| s.tag_node("define_constraint"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_annotation_head(s).and_then(|s| s.tag_node("annotation_head")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_extends(s).and_then(|s| s.tag_node("kw_extends")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_trait_block(s).and_then(|s| s.tag_node("trait_block")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_trait_block(state: Input) -> Output {
    state.rule(ValkyrieRule::TraitBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_trait_term(s).and_then(|s| s.tag_node("trait_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_trait_term(state: Input) -> Output {
    state.rule(ValkyrieRule::TraitTerm, |s| {
        Err(s)
            .or_else(|s| parse_procedural_call(s).and_then(|s| s.tag_node("procedural_call")))
            .or_else(|s| parse_define_method(s).and_then(|s| s.tag_node("define_method")))
            .or_else(|s| parse_define_field(s).and_then(|s| s.tag_node("define_field")))
            .or_else(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
    })
}
#[inline]
fn parse_kw_trait(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_TRAIT, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, "trait", false).and_then(|s| s.tag_node("trait")))
            .or_else(|s| builtin_text(s, "interface", false).and_then(|s| s.tag_node("interface")))
    })
}
#[inline]
fn parse_define_function(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineFunction, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_head(s).and_then(|s| s.tag_node("annotation_head")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_function(s).and_then(|s| s.tag_node("kw_function")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_function_middle(s).and_then(|s| s.tag_node("function_middle")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation")))
        })
    })
}
#[inline]
fn parse_define_lambda(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineLambda, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term(s).and_then(|s| s.tag_node("annotation_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_lambda(s).and_then(|s| s.tag_node("kw_lambda")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_function_middle(s).and_then(|s| s.tag_node("function_middle")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation")))
        })
    })
}
#[inline]
fn parse_function_middle(state: Input) -> Output {
    state.rule(ValkyrieRule::FunctionMiddle, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_define_generic(s).and_then(|s| s.tag_node("define_generic"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "(", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_function_parameters(s).and_then(|s| s.tag_node("function_parameters")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, ")", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_type_return(s).and_then(|s| s.tag_node("type_return"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_type_effect(s).and_then(|s| s.tag_node("type_effect"))))
        })
    })
}
#[inline]
fn parse_type_hint(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeHint, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| s.sequence(|s| Ok(s).and_then(|s| parse_colon(s)).and_then(|s| builtin_ignore(s))))
                    .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("hint")))
            })
        })
    })
}
#[inline]
fn parse_type_return(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeReturn, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_arrow_1(s).and_then(|s| s.tag_node("arrow_1")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression")))
        })
    })
}
#[inline]
fn parse_type_effect(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeEffect, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "/", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression")))
        })
    })
}
#[inline]
fn parse_function_parameters(state: Input) -> Output {
    state.rule(ValkyrieRule::FunctionParameters, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| parse_parameter_item(s).and_then(|s| s.tag_node("parameter_item")))
                    .and_then(|s| {
                        s.repeat(0..4294967295, |s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_comma(s))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_parameter_item(s).and_then(|s| s.tag_node("parameter_item")))
                            })
                        })
                    })
                    .and_then(|s| {
                        s.optional(|s| s.sequence(|s| Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| parse_comma(s))))
                    })
            })
        })
    })
}
#[inline]
fn parse_parameter_item(state: Input) -> Output {
    state.rule(ValkyrieRule::ParameterItem, |s| {
        Err(s)
            .or_else(|s| parse_parameter_item_control(s).and_then(|s| s.tag_node("parameter_item_control")))
            .or_else(|s| parse_parameter_pair(s).and_then(|s| s.tag_node("parameter_pair")))
    })
}
#[inline]
fn parse_parameter_item_control(state: Input) -> Output {
    state.rule(ValkyrieRule::ParameterItemControl, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([<「」>]|\\.{2,3})").unwrap())
        })
    })
}
#[inline]
fn parse_parameter_pair(state: Input) -> Output {
    state.rule(ValkyrieRule::ParameterPair, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_modifier_ahead(s).and_then(|s| s.tag_node("modifier_ahead")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_parameter_hint(s).and_then(|s| s.tag_node("parameter_hint"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_parameter_default(s).and_then(|s| s.tag_node("parameter_default")))
        })
    })
}
#[inline]
fn parse_parameter_hint(state: Input) -> Output {
    state.rule(ValkyrieRule::ParameterHint, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([.]{2,3}|[%^])").unwrap())
        })
    })
}
#[inline]
fn parse_continuation(state: Input) -> Output {
    state.rule(ValkyrieRule::Continuation, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_statement(s).and_then(|s| s.tag_node("statement")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_kw_function(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_FUNCTION, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(micro|macro|function|func|fun|fn)").unwrap())
        })
    })
}
#[inline]
fn parse_define_variable(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineVariable, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term(s).and_then(|s| s.tag_node("annotation_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_let(s).and_then(|s| s.tag_node("kw_let")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_let_pattern(s).and_then(|s| s.tag_node("let_pattern")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_hint(s).and_then(|s| s.tag_node("type_hint")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_parameter_default(s).and_then(|s| s.tag_node("parameter_default")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_let_pattern(state: Input) -> Output {
    state.rule(ValkyrieRule::LetPattern, |s| {
        Err(s)
            .or_else(|s| parse_standard_pattern(s).and_then(|s| s.tag_node("standard_pattern")))
            .or_else(|s| parse_bare_pattern(s).and_then(|s| s.tag_node("bare_pattern")))
    })
}
#[inline]
fn parse_standard_pattern(state: Input) -> Output {
    state.rule(ValkyrieRule::StandardPattern, |s| {
        Err(s).or_else(|s| parse_tuple_pattern(s).and_then(|s| s.tag_node("tuple_pattern")))
    })
}
#[inline]
fn parse_bare_pattern(state: Input) -> Output {
    state.rule(ValkyrieRule::BarePattern, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_bare_pattern_item(s).and_then(|s| s.tag_node("bare_pattern_item")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| parse_comma(s))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| parse_bare_pattern_item(s).and_then(|s| s.tag_node("bare_pattern_item")))
                                })
                            })
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_comma(s)))
        })
    })
}
#[inline]
fn parse_bare_pattern_item(state: Input) -> Output {
    state.rule(ValkyrieRule::BarePatternItem, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_modifier_ahead(s).and_then(|s| s.tag_node("modifier_ahead")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
        })
    })
}
#[inline]
fn parse_tuple_pattern(state: Input) -> Output {
    state.rule(ValkyrieRule::TuplePattern, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| s.optional(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath"))))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "(", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| {
                            s.optional(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| parse_pattern_item(s).and_then(|s| s.tag_node("pattern_item")))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| {
                                            s.repeat(0..4294967295, |s| {
                                                s.sequence(|s| {
                                                    Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                        s.sequence(|s| {
                                                            Ok(s)
                                                                .and_then(|s| parse_comma(s))
                                                                .and_then(|s| builtin_ignore(s))
                                                                .and_then(|s| {
                                                                    parse_pattern_item(s)
                                                                        .and_then(|s| s.tag_node("pattern_item"))
                                                                })
                                                        })
                                                    })
                                                })
                                            })
                                        })
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| s.optional(|s| parse_comma(s)))
                                })
                            })
                        })
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ")", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| s.optional(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath"))))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "{", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| {
                            s.optional(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| parse_pattern_item(s).and_then(|s| s.tag_node("pattern_item")))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| {
                                            s.repeat(0..4294967295, |s| {
                                                s.sequence(|s| {
                                                    Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                        s.sequence(|s| {
                                                            Ok(s)
                                                                .and_then(|s| parse_comma(s))
                                                                .and_then(|s| builtin_ignore(s))
                                                                .and_then(|s| {
                                                                    parse_pattern_item(s)
                                                                        .and_then(|s| s.tag_node("pattern_item"))
                                                                })
                                                        })
                                                    })
                                                })
                                            })
                                        })
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| s.optional(|s| parse_comma(s)))
                                })
                            })
                        })
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "}", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "[", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| {
                            s.optional(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| parse_pattern_item(s).and_then(|s| s.tag_node("pattern_item")))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| {
                                            s.repeat(0..4294967295, |s| {
                                                s.sequence(|s| {
                                                    Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                        s.sequence(|s| {
                                                            Ok(s)
                                                                .and_then(|s| parse_comma(s))
                                                                .and_then(|s| builtin_ignore(s))
                                                                .and_then(|s| {
                                                                    parse_pattern_item(s)
                                                                        .and_then(|s| s.tag_node("pattern_item"))
                                                                })
                                                        })
                                                    })
                                                })
                                            })
                                        })
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| s.optional(|s| parse_comma(s)))
                                })
                            })
                        })
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "]", false))
                })
            })
    })
}
#[inline]
fn parse_pattern_item(state: Input) -> Output {
    state.rule(ValkyrieRule::PatternItem, |s| {
        Err(s)
            .or_else(|s| parse_tuple_pattern_item(s).and_then(|s| s.tag_node("tuple_pattern_item")))
            .or_else(|s| builtin_text(s, "...", false).and_then(|s| s.tag_node("omit_dict")))
            .or_else(|s| builtin_text(s, "..", false).and_then(|s| s.tag_node("omit_list")))
    })
}
#[inline]
fn parse_tuple_pattern_item(state: Input) -> Output {
    state.rule(ValkyrieRule::TuplePatternItem, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_annotation_mix(s).and_then(|s| s.tag_node("annotation_mix")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_parameter_hint(s).and_then(|s| s.tag_node("parameter_hint"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_colon(s).and_then(|s| s.tag_node("colon")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_standard_pattern(s).and_then(|s| s.tag_node("standard_pattern")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_while_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::WhileStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_while(s).and_then(|s| s.tag_node("kw_while")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_inline_expression(s).and_then(|s| s.tag_node("inline_expression"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_kw_while(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_WHILE, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, "while", false).and_then(|s| s.tag_node("while")))
            .or_else(|s| builtin_text(s, "until", false).and_then(|s| s.tag_node("until")))
    })
}
#[inline]
fn parse_for_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::ForStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_for(s).and_then(|s| s.tag_node("kw_for")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_let_pattern(s).and_then(|s| s.tag_node("let_pattern")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_in(s).and_then(|s| s.tag_node("kw_in")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_inline_expression(s).and_then(|s| s.tag_node("inline_expression"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_if_guard(s).and_then(|s| s.tag_node("if_guard")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_if_guard(state: Input) -> Output {
    state.rule(ValkyrieRule::IfGuard, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| s.sequence(|s| Ok(s).and_then(|s| parse_kw_if(s)).and_then(|s| builtin_ignore(s))))
                    .and_then(|s| parse_inline_expression(s).and_then(|s| s.tag_node("condition")))
            })
        })
    })
}
#[inline]
fn parse_control_flow(state: Input) -> Output {
    state.rule(ValkyrieRule::ControlFlow, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term(s).and_then(|s| s.tag_node("annotation_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_control(s).and_then(|s| s.tag_node("kw_control")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_jump_label(s).and_then(|s| s.tag_node("jump_label")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression"))))
        })
    })
}
#[inline]
fn parse_jump_label(state: Input) -> Output {
    state.rule(ValkyrieRule::JumpLabel, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| builtin_text(s, "^", false))
                    .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
            })
        })
    })
}
#[inline]
fn parse_expression_root(state: Input) -> Output {
    state.rule(ValkyrieRule::ExpressionRoot, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term(s).and_then(|s| s.tag_node("annotation_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s).and_then(|s| s.tag_node("eos"))))
        })
    })
}
#[inline]
fn parse_match_expression(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchExpression, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_match(s).and_then(|s| s.tag_node("kw_match")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    Err(s)
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| {
                                        s.optional(|s| {
                                            s.sequence(|s| {
                                                Ok(s)
                                                    .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                                                    .and_then(|s| builtin_ignore(s))
                                                    .and_then(|s| parse_bind_l(s).and_then(|s| s.tag_node("bind_l")))
                                            })
                                        })
                                    })
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_inline_expression(s).and_then(|s| s.tag_node("inline_expression")))
                            })
                        })
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| parse_inline_expression(s).and_then(|s| s.tag_node("inline_expression")))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| {
                                        s.optional(|s| {
                                            s.sequence(|s| {
                                                Ok(s)
                                                    .and_then(|s| parse_bind_r(s).and_then(|s| s.tag_node("bind_r")))
                                                    .and_then(|s| builtin_ignore(s))
                                                    .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                                            })
                                        })
                                    })
                            })
                        })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_match_block(s).and_then(|s| s.tag_node("match_block")))
        })
    })
}
#[inline]
fn parse_switch_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::SwitchStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_switch(s).and_then(|s| s.tag_node("kw_switch")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_match_block(s).and_then(|s| s.tag_node("match_block")))
        })
    })
}
#[inline]
fn parse_match_block(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_match_terms(s).and_then(|s| s.tag_node("match_terms")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_match_terms(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchTerms, |s| {
        Err(s)
            .or_else(|s| parse_match_type(s).and_then(|s| s.tag_node("match_type")))
            .or_else(|s| parse_match_case(s).and_then(|s| s.tag_node("match_case")))
            .or_else(|s| parse_match_when(s).and_then(|s| s.tag_node("match_when")))
            .or_else(|s| parse_match_else(s).and_then(|s| s.tag_node("match_else")))
            .or_else(|s| parse_comma(s).and_then(|s| s.tag_node("comma")))
    })
}
#[inline]
fn parse_match_type(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchType, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_type(s).and_then(|s| s.tag_node("kw_type")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_if_guard(s).and_then(|s| s.tag_node("if_guard")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_colon(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_match_statement(s).and_then(|s| s.tag_node("match_statement")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_match_case(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchCase, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_case(s).and_then(|s| s.tag_node("kw_case")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_case_pattern(s).and_then(|s| s.tag_node("case_pattern")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_if_guard(s).and_then(|s| s.tag_node("if_guard")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_colon(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_match_statement(s).and_then(|s| s.tag_node("match_statement")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_case_pattern(state: Input) -> Output {
    state.rule(ValkyrieRule::CasePattern, |s| {
        Err(s)
            .or_else(|s| parse_standard_pattern(s).and_then(|s| s.tag_node("standard_pattern")))
            .or_else(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath")))
    })
}
#[inline]
fn parse_match_when(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchWhen, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_when(s).and_then(|s| s.tag_node("kw_when")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_inline_expression(s).and_then(|s| s.tag_node("inline_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_colon(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_match_statement(s).and_then(|s| s.tag_node("match_statement")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_match_else(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchElse, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_else(s).and_then(|s| s.tag_node("kw_else")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_colon(s))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_match_statement(s).and_then(|s| s.tag_node("match_statement")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_match_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::MatchStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.lookahead(false, |s| {
                        builtin_regex(s, {
                            static REGEX: OnceLock<Regex> = OnceLock::new();
                            REGEX.get_or_init(|| Regex::new("^(?x)(type|case|when|else|[,，])").unwrap())
                        })
                    })
                })
                .and_then(|s| parse_statement(s).and_then(|s| s.tag_node("statement")))
        })
    })
}
#[inline]
fn parse_kw_match(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_MATCH, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, "match", false).and_then(|s| s.tag_node("match")))
            .or_else(|s| builtin_text(s, "catch", false).and_then(|s| s.tag_node("catch")))
    })
}
#[inline]
fn parse_bind_l(state: Input) -> Output {
    state.rule(ValkyrieRule::BIND_L, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(≔|:=)").unwrap())
        })
    })
}
#[inline]
fn parse_bind_r(state: Input) -> Output {
    state.rule(ValkyrieRule::BIND_R, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(≕|=:)").unwrap())
        })
    })
}
#[inline]
fn parse_dot_match_call(state: Input) -> Output {
    state.rule(ValkyrieRule::DotMatchCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_dot(s))
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| parse_kw_match(s).and_then(|s| s.tag_node("kw_match")))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| s.optional(|s| parse_white_space(s)))
                                .and_then(|s| parse_bind_r(s).and_then(|s| s.tag_node("bind_r")))
                                .and_then(|s| s.optional(|s| parse_white_space(s)))
                                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                        })
                    })
                })
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| parse_match_block(s).and_then(|s| s.tag_node("match_block")))
        })
    })
}
#[inline]
fn parse_main_expression(state: Input) -> Output {
    state.rule(ValkyrieRule::MainExpression, |s| {
        s.sequence(|s| {
            Ok(s).and_then(|s| parse_main_term(s).and_then(|s| s.tag_node("main_term"))).and_then(|s| {
                s.repeat(0..4294967295, |s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_main_infix(s).and_then(|s| s.tag_node("main_infix")))
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_main_term(s).and_then(|s| s.tag_node("main_term")))
                    })
                })
            })
        })
    })
}
#[inline]
fn parse_main_term(state: Input) -> Output {
    state.rule(ValkyrieRule::MainTerm, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_main_prefix(s).and_then(|s| s.tag_node("main_prefix")))
                                .and_then(|s| builtin_ignore(s))
                        })
                    })
                })
                .and_then(|s| parse_main_factor(s).and_then(|s| s.tag_node("main_factor")))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| parse_main_suffix_term(s).and_then(|s| s.tag_node("main_suffix_term")))
                })
        })
    })
}
#[inline]
fn parse_main_factor(state: Input) -> Output {
    state.rule(ValkyrieRule::MainFactor, |s| {
        Err(s)
            .or_else(|s| parse_switch_statement(s).and_then(|s| s.tag_node("switch_statement")))
            .or_else(|s| parse_try_statement(s).and_then(|s| s.tag_node("try_statement")))
            .or_else(|s| parse_match_expression(s).and_then(|s| s.tag_node("match_expression")))
            .or_else(|s| parse_define_lambda(s).and_then(|s| s.tag_node("define_lambda")))
            .or_else(|s| parse_object_statement(s).and_then(|s| s.tag_node("object_statement")))
            .or_else(|s| parse_new_statement(s).and_then(|s| s.tag_node("new_statement")))
            .or_else(|s| parse_group_factor(s).and_then(|s| s.tag_node("group_factor")))
            .or_else(|s| parse_leading(s).and_then(|s| s.tag_node("leading")))
    })
}
#[inline]
fn parse_group_factor(state: Input) -> Output {
    state.rule(ValkyrieRule::GroupFactor, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "(", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, ")", false))
        })
    })
}
#[inline]
fn parse_leading(state: Input) -> Output {
    state.rule(ValkyrieRule::Leading, |s| {
        Err(s)
            .or_else(|s| parse_procedural_call(s).and_then(|s| s.tag_node("procedural_call")))
            .or_else(|s| parse_tuple_literal_strict(s).and_then(|s| s.tag_node("tuple_literal_strict")))
            .or_else(|s| parse_range_literal(s).and_then(|s| s.tag_node("range_literal")))
            .or_else(|s| parse_text_literal(s).and_then(|s| s.tag_node("text_literal")))
            .or_else(|s| parse_slot(s).and_then(|s| s.tag_node("slot")))
            .or_else(|s| parse_number(s).and_then(|s| s.tag_node("number")))
            .or_else(|s| parse_special(s).and_then(|s| s.tag_node("special")))
            .or_else(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath")))
    })
}
#[inline]
fn parse_main_suffix_term(state: Input) -> Output {
    state.rule(ValkyrieRule::MainSuffixTerm, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_dot_match_call(s).and_then(|s| s.tag_node("dot_match_call")))
                })
                .and_then(|s| s.tag_node("dot_match_call"))
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_dot_closure_call(s).and_then(|s| s.tag_node("dot_closure_call")))
                })
                .and_then(|s| s.tag_node("dot_closure_call"))
            })
            .or_else(|s| parse_tuple_call(s).and_then(|s| s.tag_node("tuple_call")))
            .or_else(|s| parse_inline_suffix_term(s).and_then(|s| s.tag_node("inline_suffix_term")))
    })
}
#[inline]
fn parse_main_prefix(state: Input) -> Output {
    state.rule(ValkyrieRule::MainPrefix, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(
                    "^(?x)([¬!+]
    | [-]
    | [.]{2,3}
    | [⅟]
    | [√∛∜]
    | [&*])",
                )
                .unwrap()
            })
        })
    })
}
#[inline]
fn parse_type_prefix(state: Input) -> Output {
    state.rule(ValkyrieRule::TypePrefix, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(
                    "^(?x)([-+¬]
    | [&^])",
                )
                .unwrap()
            })
        })
    })
}
#[inline]
fn parse_main_infix(state: Input) -> Output {
    state.rule(ValkyrieRule::MainInfix, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(
                    "^(?x)([+\\-*٪⁒÷/%]=?
    | /%=? | %%=?
    | [√^]
    # start with ?, !, =
    | [?]=
    | !==|=!=|===|==|!=|=|[!≢≠≡]
    # start with `<, >`
    | <<<|<<=|<<|<=|[⋘≪⩽≤<]
    | >>>|>>=|>>|>=|[⋙≫⩾≥>]
    # start with &, |
    | [&|]{1,3}
    | [∧⊼⩟∨⊽⊻]
    # start with .
    | [.]{1,2}[<=]
    | [.]=
    | [∈∊∉∋∍∌]
    | (not\\s+)?in
    | is(\\s+not)?
    | as[*!?]?
    # map, apply
    | /@ | [⇴⨵⊕⟴] | @{2,3})",
                )
                .unwrap()
            })
        })
    })
}
#[inline]
fn parse_type_infix(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeInfix, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(
                    "^(?x)([⟶]
    | ->
    | [-+&|∧∨])",
                )
                .unwrap()
            })
        })
    })
}
#[inline]
fn parse_main_suffix(state: Input) -> Output {
    state.rule(ValkyrieRule::MainSuffix, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(
                    "^(?x)([!]
    | [٪⁒%‰‱]
    | [′″‴⁗]
    | [℃℉])",
                )
                .unwrap()
            })
        })
    })
}
#[inline]
fn parse_type_suffix(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeSuffix, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([!?])").unwrap())
        })
    })
}
#[inline]
fn parse_inline_expression(state: Input) -> Output {
    state.rule(ValkyrieRule::InlineExpression, |s| {
        s.sequence(|s| {
            Ok(s).and_then(|s| parse_inline_term(s).and_then(|s| s.tag_node("inline_term"))).and_then(|s| {
                s.repeat(0..4294967295, |s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_main_infix(s).and_then(|s| s.tag_node("main_infix")))
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_inline_term(s).and_then(|s| s.tag_node("inline_term")))
                    })
                })
            })
        })
    })
}
#[inline]
fn parse_inline_term(state: Input) -> Output {
    state.rule(ValkyrieRule::InlineTerm, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_main_prefix(s).and_then(|s| s.tag_node("main_prefix")))
                                .and_then(|s| builtin_ignore(s))
                        })
                    })
                })
                .and_then(|s| parse_main_factor(s).and_then(|s| s.tag_node("main_factor")))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| parse_inline_suffix_term(s).and_then(|s| s.tag_node("inline_suffix_term")))
                })
        })
    })
}
#[inline]
fn parse_inline_suffix_term(state: Input) -> Output {
    state.rule(ValkyrieRule::InlineSuffixTerm, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_main_suffix(s).and_then(|s| s.tag_node("main_suffix")))
                })
                .and_then(|s| s.tag_node("main_suffix"))
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_dot_call(s).and_then(|s| s.tag_node("dot_call")))
                })
                .and_then(|s| s.tag_node("dot_call"))
            })
            .or_else(|s| parse_inline_tuple_call(s).and_then(|s| s.tag_node("inline_tuple_call")))
            .or_else(|s| parse_range_call(s).and_then(|s| s.tag_node("range_call")))
            .or_else(|s| parse_generic_call(s).and_then(|s| s.tag_node("generic_call")))
    })
}
#[inline]
fn parse_type_expression(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeExpression, |s| {
        s.sequence(|s| {
            Ok(s).and_then(|s| parse_type_term(s).and_then(|s| s.tag_node("type_term"))).and_then(|s| {
                s.repeat(0..4294967295, |s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_type_infix(s).and_then(|s| s.tag_node("type_infix")))
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_type_term(s).and_then(|s| s.tag_node("type_term")))
                    })
                })
            })
        })
    })
}
#[inline]
fn parse_type_term(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeTerm, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_type_prefix(s).and_then(|s| s.tag_node("type_prefix")))
                                .and_then(|s| builtin_ignore(s))
                        })
                    })
                })
                .and_then(|s| parse_main_factor(s).and_then(|s| s.tag_node("main_factor")))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| parse_type_suffix_term(s).and_then(|s| s.tag_node("type_suffix_term")))
                })
        })
    })
}
#[inline]
fn parse_type_factor(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeFactor, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "(", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ")", false))
                })
                .and_then(|s| s.tag_node("type_expression"))
            })
            .or_else(|s| parse_leading(s).and_then(|s| s.tag_node("leading")))
    })
}
#[inline]
fn parse_type_suffix_term(state: Input) -> Output {
    state.rule(ValkyrieRule::TypeSuffixTerm, |s| {
        Err(s)
            .or_else(|s| parse_generic_hide(s).and_then(|s| s.tag_node("generic_hide")))
            .or_else(|s| parse_type_suffix(s).and_then(|s| s.tag_node("type_suffix")))
    })
}
#[inline]
fn parse_try_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::TryStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_try(s).and_then(|s| s.tag_node("kw_try")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation")))
        })
    })
}
#[inline]
fn parse_new_statement(state: Input) -> Output {
    state.rule(ValkyrieRule::NewStatement, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_kw_new(s).and_then(|s| s.tag_node("kw_new")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_modifier_ahead(s).and_then(|s| s.tag_node("modifier_ahead")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_generic_hide(s).and_then(|s| s.tag_node("generic_hide"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_tuple_literal(s).and_then(|s| s.tag_node("tuple_literal"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_new_block(s).and_then(|s| s.tag_node("new_block"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_eos(s)))
        })
    })
}
#[inline]
fn parse_new_block(state: Input) -> Output {
    state.rule(ValkyrieRule::NewBlock, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_new_pair(s).and_then(|s| s.tag_node("new_pair")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| {
                                    s.repeat(0..4294967295, |s| {
                                        s.sequence(|s| {
                                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                s.sequence(|s| {
                                                    Ok(s)
                                                        .and_then(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free")))
                                                        .and_then(|s| builtin_ignore(s))
                                                        .and_then(|s| parse_new_pair(s).and_then(|s| s.tag_node("new_pair")))
                                                })
                                            })
                                        })
                                    })
                                })
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| s.optional(|s| parse_eos_free(s).and_then(|s| s.tag_node("eos_free"))))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_new_pair(state: Input) -> Output {
    state.rule(ValkyrieRule::NewPair, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_new_pair_key(s).and_then(|s| s.tag_node("new_pair_key")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_colon(s).and_then(|s| s.tag_node("colon")))
                                .and_then(|s| builtin_ignore(s))
                        })
                    })
                })
                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
        })
    })
}
#[inline]
fn parse_new_pair_key(state: Input) -> Output {
    state.rule(ValkyrieRule::NewPairKey, |s| {
        Err(s)
            .or_else(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
            .or_else(|s| parse_text_raw(s).and_then(|s| s.tag_node("text_raw")))
            .or_else(|s| parse_range_literal(s).and_then(|s| s.tag_node("range_literal")))
    })
}
#[inline]
fn parse_dot_call(state: Input) -> Output {
    state.rule(ValkyrieRule::DotCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_dot(s))
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| parse_dot_call_item(s).and_then(|s| s.tag_node("dot_call_item")))
        })
    })
}
#[inline]
fn parse_dot_call_item(state: Input) -> Output {
    state.rule(ValkyrieRule::DotCallItem, |s| {
        Err(s)
            .or_else(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath")))
            .or_else(|s| parse_integer(s).and_then(|s| s.tag_node("integer")))
    })
}
#[inline]
fn parse_dot_closure_call(state: Input) -> Output {
    state.rule(ValkyrieRule::DotClosureCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_dot(s))
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation")))
        })
    })
}
#[inline]
fn parse_inline_tuple_call(state: Input) -> Output {
    state.rule(ValkyrieRule::InlineTupleCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| parse_tuple_literal(s).and_then(|s| s.tag_node("tuple_literal")))
        })
    })
}
#[inline]
fn parse_tuple_call(state: Input) -> Output {
    state.rule(ValkyrieRule::TupleCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| {
                    Err(s)
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s).and_then(|s| parse_tuple_literal(s).and_then(|s| s.tag_node("tuple_literal"))).and_then(
                                    |s| {
                                        s.optional(|s| {
                                            s.sequence(|s| {
                                                Ok(s).and_then(|s| s.optional(|s| parse_white_space(s))).and_then(|s| {
                                                    parse_continuation(s).and_then(|s| s.tag_node("continuation"))
                                                })
                                            })
                                        })
                                    },
                                )
                            })
                        })
                        .or_else(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation")))
                })
        })
    })
}
#[inline]
fn parse_tuple_literal(state: Input) -> Output {
    state.rule(ValkyrieRule::TupleLiteral, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "(", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_tuple_terms(s).and_then(|s| s.tag_node("tuple_terms")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, ")", false))
        })
    })
}
#[inline]
fn parse_tuple_literal_strict(state: Input) -> Output {
    state.rule(ValkyrieRule::TupleLiteralStrict, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "(", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ")", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "(", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_tuple_pair(s).and_then(|s| s.tag_node("tuple_pair")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_comma(s))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ")", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "(", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_tuple_pair(s).and_then(|s| s.tag_node("tuple_pair")))
                        .and_then(|s| {
                            s.repeat(0..4294967295, |s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| parse_comma(s))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| parse_tuple_pair(s).and_then(|s| s.tag_node("tuple_pair")))
                                })
                            })
                        })
                        .and_then(|s| {
                            s.optional(|s| s.sequence(|s| Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| parse_comma(s))))
                        })
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ")", false))
                })
            })
    })
}
#[inline]
fn parse_tuple_terms(state: Input) -> Output {
    state.rule(ValkyrieRule::TupleTerms, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| parse_tuple_pair(s).and_then(|s| s.tag_node("tuple_pair")))
                    .and_then(|s| {
                        s.repeat(0..4294967295, |s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_comma(s))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_tuple_pair(s).and_then(|s| s.tag_node("tuple_pair")))
                            })
                        })
                    })
                    .and_then(|s| {
                        s.optional(|s| s.sequence(|s| Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| parse_comma(s))))
                    })
            })
        })
    })
}
#[inline]
fn parse_tuple_pair(state: Input) -> Output {
    state.rule(ValkyrieRule::TuplePair, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_tuple_key(s).and_then(|s| s.tag_node("tuple_key")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_colon(s).and_then(|s| s.tag_node("colon")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
        })
    })
}
#[inline]
fn parse_tuple_key(state: Input) -> Output {
    state.rule(ValkyrieRule::TupleKey, |s| {
        Err(s)
            .or_else(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
            .or_else(|s| parse_integer(s).and_then(|s| s.tag_node("integer")))
            .or_else(|s| parse_text_raw(s).and_then(|s| s.tag_node("text_raw")))
    })
}
#[inline]
fn parse_range_call(state: Input) -> Output {
    state.rule(ValkyrieRule::RangeCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| s.optional(|s| parse_white_space(s)))
                .and_then(|s| parse_range_literal(s).and_then(|s| s.tag_node("range_literal")))
        })
    })
}
#[inline]
fn parse_range_literal(state: Input) -> Output {
    state.rule(ValkyrieRule::RangeLiteral, |s| {
        Err(s)
            .or_else(|s| parse_range_literal_index_0(s).and_then(|s| s.tag_node("range_literal_index_0")))
            .or_else(|s| parse_range_literal_index_1(s).and_then(|s| s.tag_node("range_literal_index_1")))
    })
}
#[inline]
fn parse_range_literal_index_0(state: Input) -> Output {
    state.rule(ValkyrieRule::RangeLiteralIndex0, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "⁅", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_subscript_axis(s).and_then(|s| s.tag_node("subscript_axis")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| {
                                    s.repeat(0..4294967295, |s| {
                                        s.sequence(|s| {
                                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                s.sequence(|s| {
                                                    Ok(s).and_then(|s| parse_comma(s)).and_then(|s| builtin_ignore(s)).and_then(
                                                        |s| parse_subscript_axis(s).and_then(|s| s.tag_node("subscript_axis")),
                                                    )
                                                })
                                            })
                                        })
                                    })
                                })
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| s.optional(|s| parse_comma(s)))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "⁆", false))
        })
    })
}
#[inline]
fn parse_range_literal_index_1(state: Input) -> Output {
    state.rule(ValkyrieRule::RangeLiteralIndex1, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "[", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_subscript_axis(s).and_then(|s| s.tag_node("subscript_axis")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| {
                                    s.repeat(0..4294967295, |s| {
                                        s.sequence(|s| {
                                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                s.sequence(|s| {
                                                    Ok(s).and_then(|s| parse_comma(s)).and_then(|s| builtin_ignore(s)).and_then(
                                                        |s| parse_subscript_axis(s).and_then(|s| s.tag_node("subscript_axis")),
                                                    )
                                                })
                                            })
                                        })
                                    })
                                })
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| s.optional(|s| parse_comma(s)))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "]", false))
        })
    })
}
#[inline]
fn parse_subscript_axis(state: Input) -> Output {
    state.rule(ValkyrieRule::SubscriptAxis, |s| {
        Err(s)
            .or_else(|s| parse_subscript_range(s).and_then(|s| s.tag_node("subscript_range")))
            .or_else(|s| parse_subscript_only(s).and_then(|s| s.tag_node("subscript_only")))
    })
}
#[inline]
fn parse_subscript_only(state: Input) -> Output {
    state.rule(ValkyrieRule::SubscriptOnly, |s| parse_main_expression(s).and_then(|s| s.tag_node("index")))
}
#[inline]
fn parse_subscript_range(state: Input) -> Output {
    state.rule(ValkyrieRule::SubscriptRange, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("head")))
                                .and_then(|s| builtin_ignore(s))
                        })
                    })
                })
                .and_then(|s| {
                    Err(s)
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s).and_then(|s| parse_range_omit(s)).and_then(|s| {
                                    s.optional(|s| {
                                        s.sequence(|s| {
                                            Ok(s)
                                                .and_then(|s| builtin_ignore(s))
                                                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("step")))
                                        })
                                    })
                                })
                            })
                        })
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s).and_then(|s| parse_colon(s)).and_then(|s| {
                                    s.optional(|s| {
                                        s.sequence(|s| {
                                            Ok(s)
                                                .and_then(|s| builtin_ignore(s))
                                                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("tail")))
                                                .and_then(|s| {
                                                    s.optional(|s| {
                                                        s.sequence(|s| {
                                                            Ok(s)
                                                                .and_then(|s| builtin_ignore(s))
                                                                .and_then(|s| parse_colon(s))
                                                                .and_then(|s| {
                                                                    s.optional(|s| {
                                                                        s.sequence(|s| {
                                                                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(
                                                                                |s| {
                                                                                    parse_main_expression(s)
                                                                                        .and_then(|s| s.tag_node("step"))
                                                                                },
                                                                            )
                                                                        })
                                                                    })
                                                                })
                                                        })
                                                    })
                                                })
                                        })
                                    })
                                })
                            })
                        })
                })
        })
    })
}
#[inline]
fn parse_range_omit(state: Input) -> Output {
    state.rule(ValkyrieRule::RangeOmit, |s| {
        Err(s).or_else(|s| parse_proportion(s).and_then(|s| s.tag_node("proportion"))).or_else(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| parse_colon(s).and_then(|s| s.tag_node("colon")))
                    .and_then(|s| builtin_ignore(s))
                    .and_then(|s| parse_colon(s).and_then(|s| s.tag_node("colon")))
            })
        })
    })
}
#[inline]
fn parse_define_generic(state: Input) -> Output {
    state.rule(ValkyrieRule::DefineGeneric, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| s.optional(|s| parse_proportion(s).and_then(|s| s.tag_node("proportion"))))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "<", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_generic_parameter(s).and_then(|s| s.tag_node("generic_parameter")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ">", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "⟨", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_generic_parameter(s).and_then(|s| s.tag_node("generic_parameter")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "⟩", false))
                })
            })
    })
}
#[inline]
fn parse_generic_parameter(state: Input) -> Output {
    state.rule(ValkyrieRule::GenericParameter, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| parse_generic_parameter_pair(s).and_then(|s| s.tag_node("generic_parameter_pair")))
                    .and_then(|s| {
                        s.repeat(0..4294967295, |s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_comma(s))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| {
                                        parse_generic_parameter_pair(s).and_then(|s| s.tag_node("generic_parameter_pair"))
                                    })
                            })
                        })
                    })
                    .and_then(|s| {
                        s.optional(|s| s.sequence(|s| Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| parse_comma(s))))
                    })
            })
        })
    })
}
#[inline]
fn parse_generic_parameter_pair(state: Input) -> Output {
    state.rule(ValkyrieRule::GenericParameterPair, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| {
                                    s.sequence(|s| Ok(s).and_then(|s| parse_colon(s)).and_then(|s| builtin_ignore(s)))
                                })
                                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("bound")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| {
                                    s.sequence(|s| {
                                        Ok(s).and_then(|s| builtin_text(s, "=", false)).and_then(|s| builtin_ignore(s))
                                    })
                                })
                                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("default")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_generic_call(state: Input) -> Output {
    state.rule(ValkyrieRule::GenericCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_op_and_then(s).and_then(|s| s.tag_node("op_and_then"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    Err(s)
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| parse_proportion(s).and_then(|s| s.tag_node("proportion")))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| builtin_text(s, "<", false))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_generic_terms(s).and_then(|s| s.tag_node("generic_terms")))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| builtin_text(s, ">", false))
                            })
                        })
                        .or_else(|s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| builtin_text(s, "⟨", false))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_generic_terms(s).and_then(|s| s.tag_node("generic_terms")))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| builtin_text(s, "⟩", false))
                            })
                        })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_proportion(s).and_then(|s| s.tag_node("proportion")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_generic_hide(state: Input) -> Output {
    state.rule(ValkyrieRule::GenericHide, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| s.optional(|s| parse_proportion(s).and_then(|s| s.tag_node("proportion"))))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "<", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_generic_terms(s).and_then(|s| s.tag_node("generic_terms")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, ">", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "⟨", false))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| parse_generic_terms(s).and_then(|s| s.tag_node("generic_terms")))
                        .and_then(|s| builtin_ignore(s))
                        .and_then(|s| builtin_text(s, "⟩", false))
                })
            })
    })
}
#[inline]
fn parse_generic_terms(state: Input) -> Output {
    state.rule(ValkyrieRule::GenericTerms, |s| {
        s.optional(|s| {
            s.sequence(|s| {
                Ok(s)
                    .and_then(|s| parse_generic_pair(s).and_then(|s| s.tag_node("generic_pair")))
                    .and_then(|s| {
                        s.repeat(0..4294967295, |s| {
                            s.sequence(|s| {
                                Ok(s)
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_comma(s))
                                    .and_then(|s| builtin_ignore(s))
                                    .and_then(|s| parse_generic_pair(s).and_then(|s| s.tag_node("generic_pair")))
                            })
                        })
                    })
                    .and_then(|s| {
                        s.optional(|s| s.sequence(|s| Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| parse_comma(s))))
                    })
            })
        })
    })
}
#[inline]
fn parse_generic_pair(state: Input) -> Output {
    state.rule(ValkyrieRule::GenericPair, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_colon(s).and_then(|s| s.tag_node("colon")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_type_expression(s).and_then(|s| s.tag_node("type_expression")))
        })
    })
}
#[inline]
fn parse_annotation_head(state: Input) -> Output {
    state.rule(ValkyrieRule::AnnotationHead, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term(s).and_then(|s| s.tag_node("annotation_term")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_modifier_call(s).and_then(|s| s.tag_node("modifier_call")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_annotation_mix(state: Input) -> Output {
    state.rule(ValkyrieRule::AnnotationMix, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_annotation_term_mix(s).and_then(|s| s.tag_node("annotation_term_mix")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_modifier_ahead(s).and_then(|s| s.tag_node("modifier_ahead")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_annotation_term(state: Input) -> Output {
    state.rule(ValkyrieRule::AnnotationTerm, |s| {
        Err(s)
            .or_else(|s| parse_attribute_list(s).and_then(|s| s.tag_node("attribute_list")))
            .or_else(|s| parse_attribute_call(s).and_then(|s| s.tag_node("attribute_call")))
    })
}
#[inline]
fn parse_annotation_term_mix(state: Input) -> Output {
    state.rule(ValkyrieRule::AnnotationTermMix, |s| {
        Err(s)
            .or_else(|s| parse_attribute_list(s).and_then(|s| s.tag_node("attribute_list")))
            .or_else(|s| parse_attribute_call(s).and_then(|s| s.tag_node("attribute_call")))
            .or_else(|s| parse_procedural_call(s).and_then(|s| s.tag_node("procedural_call")))
    })
}
#[inline]
fn parse_attribute_list(state: Input) -> Output {
    state.rule(ValkyrieRule::AttributeList, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "#", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "[", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_attribute_item(s).and_then(|s| s.tag_node("attribute_item")))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| {
                                    s.repeat(0..4294967295, |s| {
                                        s.sequence(|s| {
                                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                                s.sequence(|s| {
                                                    Ok(s)
                                                        .and_then(|s| builtin_ignore(s))
                                                        .and_then(|s| builtin_ignore(s))
                                                        .and_then(|s| parse_eos_free(s))
                                                        .and_then(|s| builtin_ignore(s))
                                                        .and_then(|s| builtin_ignore(s))
                                                        .and_then(|s| builtin_ignore(s))
                                                        .and_then(|s| {
                                                            parse_attribute_item(s).and_then(|s| s.tag_node("attribute_item"))
                                                        })
                                                })
                                            })
                                        })
                                    })
                                })
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| {
                                    s.optional(|s| {
                                        s.sequence(|s| {
                                            Ok(s)
                                                .and_then(|s| builtin_ignore(s))
                                                .and_then(|s| builtin_ignore(s))
                                                .and_then(|s| parse_eos_free(s))
                                        })
                                    })
                                })
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "]", false))
        })
    })
}
#[inline]
fn parse_attribute_call(state: Input) -> Output {
    state.rule(ValkyrieRule::AttributeCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "#", false))
                .and_then(|s| parse_attribute_item(s).and_then(|s| s.tag_node("attribute_item")))
        })
    })
}
#[inline]
fn parse_attribute_item(state: Input) -> Output {
    state.rule(ValkyrieRule::AttributeItem, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| parse_dot(s))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                                })
                            })
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_tuple_literal(s).and_then(|s| s.tag_node("tuple_literal"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation"))))
        })
    })
}
#[inline]
fn parse_attribute_name(state: Input) -> Output {
    state.rule(ValkyrieRule::AttributeName, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "#", false))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
        })
    })
}
#[inline]
fn parse_procedural_call(state: Input) -> Output {
    state.rule(ValkyrieRule::ProceduralCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "@", false))
                .and_then(|s| parse_namepath(s).and_then(|s| s.tag_node("namepath")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_tuple_literal(s).and_then(|s| s.tag_node("tuple_literal"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_continuation(s).and_then(|s| s.tag_node("continuation"))))
        })
    })
}
#[inline]
fn parse_procedural_name(state: Input) -> Output {
    state.rule(ValkyrieRule::ProceduralName, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "@", false))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
        })
    })
}
#[inline]
fn parse_text_literal(state: Input) -> Output {
    state.rule(ValkyrieRule::TextLiteral, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier"))))
                .and_then(|s| parse_text_raw(s).and_then(|s| s.tag_node("text_raw")))
        })
    })
}
#[inline]
fn parse_text_raw(state: Input) -> Output {
    state.rule(ValkyrieRule::TextRaw, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "\"\"\"\"", false))
                        .and_then(|s| parse_text_content_5(s).and_then(|s| s.tag_node("text_content_5")))
                        .and_then(|s| builtin_text(s, "\"\"\"\"", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "''''", false))
                        .and_then(|s| parse_text_content_6(s).and_then(|s| s.tag_node("text_content_6")))
                        .and_then(|s| builtin_text(s, "''''", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "\"\"\"", false))
                        .and_then(|s| parse_text_content_3(s).and_then(|s| s.tag_node("text_content_3")))
                        .and_then(|s| builtin_text(s, "\"\"\"", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "'''", false))
                        .and_then(|s| parse_text_content_4(s).and_then(|s| s.tag_node("text_content_4")))
                        .and_then(|s| builtin_text(s, "'''", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "\"", false))
                        .and_then(|s| parse_text_content_1(s).and_then(|s| s.tag_node("text_content_1")))
                        .and_then(|s| builtin_text(s, "\"", false))
                })
            })
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| builtin_text(s, "'", false))
                        .and_then(|s| parse_text_content_2(s).and_then(|s| s.tag_node("text_content_2")))
                        .and_then(|s| builtin_text(s, "'", false))
                })
            })
    })
}
#[inline]
fn parse_text_l(state: Input) -> Output {
    state.rule(ValkyrieRule::Text_L, |s| builtin_ignore(s))
}
#[inline]
fn parse_text_r(state: Input) -> Output {
    state.rule(ValkyrieRule::Text_R, |s| builtin_ignore(s))
}
#[inline]
fn parse_text_x(state: Input) -> Output {
    state.rule(ValkyrieRule::Text_X, |s| builtin_ignore(s))
}
#[inline]
fn parse_text_content_1(state: Input) -> Output {
    state.rule(ValkyrieRule::TEXT_CONTENT1, |s| {
        s.repeat(0..4294967295, |s| {
            builtin_regex(s, {
                static REGEX: OnceLock<Regex> = OnceLock::new();
                REGEX.get_or_init(|| Regex::new("^(?x)([^\"])").unwrap())
            })
        })
    })
}
#[inline]
fn parse_text_content_2(state: Input) -> Output {
    state.rule(ValkyrieRule::TEXT_CONTENT2, |s| {
        s.repeat(0..4294967295, |s| {
            builtin_regex(s, {
                static REGEX: OnceLock<Regex> = OnceLock::new();
                REGEX.get_or_init(|| Regex::new("^(?x)([^'])").unwrap())
            })
        })
    })
}
#[inline]
fn parse_text_content_3(state: Input) -> Output {
    state.rule(ValkyrieRule::TEXT_CONTENT3, |s| {
        s.repeat(1..4294967295, |s| {
            s.sequence(|s| {
                Ok(s).and_then(|s| s.lookahead(false, |s| builtin_text(s, "\"\"\"", false))).and_then(|s| builtin_any(s))
            })
        })
    })
}
#[inline]
fn parse_text_content_4(state: Input) -> Output {
    state.rule(ValkyrieRule::TEXT_CONTENT4, |s| {
        s.repeat(1..4294967295, |s| {
            s.sequence(|s| {
                Ok(s).and_then(|s| s.lookahead(false, |s| builtin_text(s, "'''", false))).and_then(|s| builtin_any(s))
            })
        })
    })
}
#[inline]
fn parse_text_content_5(state: Input) -> Output {
    state.rule(ValkyrieRule::TEXT_CONTENT5, |s| {
        s.repeat(1..4294967295, |s| {
            s.sequence(|s| {
                Ok(s).and_then(|s| s.lookahead(false, |s| builtin_text(s, "\"\"\"\"", false))).and_then(|s| builtin_any(s))
            })
        })
    })
}
#[inline]
fn parse_text_content_6(state: Input) -> Output {
    state.rule(ValkyrieRule::TEXT_CONTENT6, |s| {
        s.repeat(1..4294967295, |s| {
            s.sequence(|s| {
                Ok(s).and_then(|s| s.lookahead(false, |s| builtin_text(s, "''''", false))).and_then(|s| builtin_any(s))
            })
        })
    })
}
#[inline]
fn parse_modifier_call(state: Input) -> Output {
    state.rule(ValkyrieRule::ModifierCall, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.lookahead(false, |s| parse_keywords_stop(s)))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
        })
    })
}
#[inline]
fn parse_modifier_ahead(state: Input) -> Output {
    state.rule(ValkyrieRule::ModifierAhead, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.lookahead(false, |s| parse_identifier_stop(s)))
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
        })
    })
}
#[inline]
fn parse_keywords_stop(state: Input) -> Output {
    state.rule(ValkyrieRule::KEYWORDS_STOP, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(
                    "^(?x)(template | generic | constraint
    | class | structure
    | enumerate | flags | union
    | function | micro | macro
    | trait | interface
    | extends?)",
                )
                .unwrap()
            })
        })
    })
}
#[inline]
fn parse_identifier_stop(state: Input) -> Output {
    state.rule(ValkyrieRule::IDENTIFIER_STOP, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    builtin_regex(s, {
                        static REGEX: OnceLock<Regex> = OnceLock::new();
                        REGEX.get_or_init(|| Regex::new("^(?x)([\\[\\](){}<>⟨:∷,.;∈=]|in|is)").unwrap())
                    })
                })
        })
    })
}
#[inline]
fn parse_slot(state: Input) -> Output {
    state.rule(ValkyrieRule::Slot, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_op_slot(s).and_then(|s| s.tag_node("op_slot")))
                .and_then(|s| s.optional(|s| parse_slot_item(s).and_then(|s| s.tag_node("slot_item"))))
        })
    })
}
#[inline]
fn parse_slot_item(state: Input) -> Output {
    state.rule(ValkyrieRule::SlotItem, |s| {
        Err(s)
            .or_else(|s| parse_integer(s).and_then(|s| s.tag_node("integer")))
            .or_else(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
    })
}
#[inline]
fn parse_namepath_free(state: Input) -> Output {
    state.rule(ValkyrieRule::NamepathFree, |s| {
        s.sequence(|s| {
            Ok(s).and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier"))).and_then(|s| {
                s.repeat(0..4294967295, |s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_proportion_2(s))
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                    })
                })
            })
        })
    })
}
#[inline]
fn parse_namepath(state: Input) -> Output {
    state.rule(ValkyrieRule::Namepath, |s| {
        s.sequence(|s| {
            Ok(s).and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier"))).and_then(|s| {
                s.repeat(0..4294967295, |s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_proportion(s))
                            .and_then(|s| builtin_ignore(s))
                            .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("identifier")))
                    })
                })
            })
        })
    })
}
#[inline]
fn parse_identifier(state: Input) -> Output {
    state.rule(ValkyrieRule::Identifier, |s| {
        Err(s)
            .or_else(|s| parse_identifier_bare(s).and_then(|s| s.tag_node("identifier_bare")))
            .or_else(|s| parse_identifier_raw(s).and_then(|s| s.tag_node("identifier_raw")))
    })
}
#[inline]
fn parse_identifier_bare(state: Input) -> Output {
    state.rule(ValkyrieRule::IdentifierBare, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([_\\p{XID_start}]\\p{XID_continue}*)").unwrap())
        })
    })
}
#[inline]
fn parse_identifier_raw(state: Input) -> Output {
    state.rule(ValkyrieRule::IdentifierRaw, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "`", false))
                .and_then(|s| parse_identifier_raw_text(s).and_then(|s| s.tag_node("identifier_raw_text")))
                .and_then(|s| builtin_text(s, "`", false))
        })
    })
}
#[inline]
fn parse_identifier_raw_text(state: Input) -> Output {
    state.rule(ValkyrieRule::IdentifierRawText, |s| {
        s.repeat(1..4294967295, |s| {
            s.sequence(|s| {
                Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                    builtin_regex(s, {
                        static REGEX: OnceLock<Regex> = OnceLock::new();
                        REGEX.get_or_init(|| Regex::new("^(?x)([^`])").unwrap())
                    })
                })
            })
        })
    })
}
#[inline]
fn parse_special(state: Input) -> Output {
    state.rule(ValkyrieRule::Special, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([∅∞]|true|false|[?]{3})").unwrap())
        })
    })
}
#[inline]
fn parse_number(state: Input) -> Output {
    state.rule(ValkyrieRule::Number, |s| {
        Err(s)
            .or_else(|s| parse_decimal_x(s).and_then(|s| s.tag_node("decimal_x")))
            .or_else(|s| parse_decimal(s).and_then(|s| s.tag_node("decimal")))
    })
}
#[inline]
fn parse_sign(state: Input) -> Output {
    state.rule(ValkyrieRule::Sign, |s| {
        Err(s)
            .or_else(|s| builtin_text(s, "+", false).and_then(|s| s.tag_node("positive")))
            .or_else(|s| builtin_text(s, "-", false).and_then(|s| s.tag_node("netative")))
    })
}
#[inline]
fn parse_integer(state: Input) -> Output {
    state.rule(ValkyrieRule::Integer, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([0-9](_*[0-9])*)").unwrap())
        })
    })
}
#[inline]
fn parse_digits_x(state: Input) -> Output {
    state.rule(ValkyrieRule::DigitsX, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([0-9a-zA-Z](_*[0-9a-zA-Z])*)").unwrap())
        })
    })
}
#[inline]
fn parse_decimal(state: Input) -> Output {
    state.rule(ValkyrieRule::Decimal, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_integer(s).and_then(|s| s.tag_node("lhs")))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_dot(s).and_then(|s| s.tag_node("dot")))
                                .and_then(|s| parse_integer(s).and_then(|s| s.tag_node("rhs")))
                        })
                    })
                })
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| {
                                    s.sequence(|s| {
                                        Ok(s)
                                            .and_then(|s| {
                                                builtin_regex(s, {
                                                    static REGEX: OnceLock<Regex> = OnceLock::new();
                                                    REGEX.get_or_init(|| Regex::new("^(?x)([⁑]|[*]{2})").unwrap())
                                                })
                                            })
                                            .and_then(|s| s.optional(|s| parse_sign(s).and_then(|s| s.tag_node("sign"))))
                                    })
                                })
                                .and_then(|s| parse_integer(s).and_then(|s| s.tag_node("shift")))
                        })
                    })
                })
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| {
                                    builtin_regex(s, {
                                        static REGEX: OnceLock<Regex> = OnceLock::new();
                                        REGEX.get_or_init(|| Regex::new("^(?x)([_]*)").unwrap())
                                    })
                                })
                                .and_then(|s| parse_identifier(s).and_then(|s| s.tag_node("unit")))
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_decimal_x(state: Input) -> Output {
    state.rule(ValkyrieRule::DecimalX, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.sequence(|s| {
                        Ok(s).and_then(|s| parse_integer(s).and_then(|s| s.tag_node("base"))).and_then(|s| {
                            builtin_regex(s, {
                                static REGEX: OnceLock<Regex> = OnceLock::new();
                                REGEX.get_or_init(|| Regex::new("^(?x)([⁂]|[*]{3})").unwrap())
                            })
                        })
                    })
                })
                .and_then(|s| parse_digits_x(s).and_then(|s| s.tag_node("lhs")))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_dot(s).and_then(|s| s.tag_node("dot")))
                                .and_then(|s| parse_digits_x(s).and_then(|s| s.tag_node("rhs")))
                        })
                    })
                })
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| {
                                    builtin_regex(s, {
                                        static REGEX: OnceLock<Regex> = OnceLock::new();
                                        REGEX.get_or_init(|| Regex::new("^(?x)([⁑]|[*]{2})").unwrap())
                                    })
                                })
                                .and_then(|s| {
                                    Err(s)
                                        .or_else(|s| {
                                            s.sequence(|s| {
                                                Ok(s)
                                                    .and_then(|s| {
                                                        s.optional(|s| parse_sign(s).and_then(|s| s.tag_node("sign")))
                                                    })
                                                    .and_then(|s| parse_integer(s).and_then(|s| s.tag_node("shift")))
                                                    .and_then(|s| {
                                                        s.optional(|s| {
                                                            s.sequence(|s| {
                                                                Ok(s)
                                                                    .and_then(|s| {
                                                                        builtin_regex(s, {
                                                                            static REGEX: OnceLock<Regex> = OnceLock::new();
                                                                            REGEX.get_or_init(|| {
                                                                                Regex::new("^(?x)([_]*)").unwrap()
                                                                            })
                                                                        })
                                                                    })
                                                                    .and_then(|s| {
                                                                        parse_identifier(s).and_then(|s| s.tag_node("unit"))
                                                                    })
                                                            })
                                                        })
                                                    })
                                            })
                                        })
                                        .or_else(|s| parse_identifier(s).and_then(|s| s.tag_node("unit")))
                                })
                        })
                    })
                })
        })
    })
}
#[inline]
fn parse_proportion(state: Input) -> Output {
    state.rule(ValkyrieRule::PROPORTION, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(∷|::)").unwrap())
        })
    })
}
#[inline]
fn parse_ns_concat(state: Input) -> Output {
    state.rule(ValkyrieRule::NS_CONCAT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([.∷]|::)").unwrap())
        })
    })
}
#[inline]
fn parse_colon(state: Input) -> Output {
    state.rule(ValkyrieRule::COLON, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([:：])").unwrap())
        })
    })
}
#[inline]
fn parse_arrow_1(state: Input) -> Output {
    state.rule(ValkyrieRule::ARROW1, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([:：⟶]|->)").unwrap())
        })
    })
}
#[inline]
fn parse_comma(state: Input) -> Output {
    state.rule(ValkyrieRule::COMMA, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([,，])").unwrap())
        })
    })
}
#[inline]
fn parse_dot(state: Input) -> Output {
    state.rule(ValkyrieRule::DOT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([.．])").unwrap())
        })
    })
}
#[inline]
fn parse_op_slot(state: Input) -> Output {
    state.rule(ValkyrieRule::OP_SLOT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([$]{1,3})").unwrap())
        })
    })
}
#[inline]
fn parse_offset_l(state: Input) -> Output {
    state.rule(ValkyrieRule::OFFSET_L, |s| s.match_string("⁅", false))
}
#[inline]
fn parse_offset_r(state: Input) -> Output {
    state.rule(ValkyrieRule::OFFSET_R, |s| s.match_string("⁆", false))
}
#[inline]
fn parse_proportion_2(state: Input) -> Output {
    state.rule(ValkyrieRule::PROPORTION2, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([.．∷]|::)").unwrap())
        })
    })
}
#[inline]
fn parse_op_import_all(state: Input) -> Output {
    state.rule(ValkyrieRule::OP_IMPORT_ALL, |s| s.match_string("*", false))
}
#[inline]
fn parse_op_and_then(state: Input) -> Output {
    state.rule(ValkyrieRule::OP_AND_THEN, |s| s.match_string("?", false))
}
#[inline]
fn parse_op_bind(state: Input) -> Output {
    state.rule(ValkyrieRule::OP_BIND, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(≔|:=)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_control(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_CONTROL, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(
                    "^(?x)(continue
    | break
    | fallthrough!?
    | raise | throw
    | return
    | resume
    | yield\\s+break
    | yield\\s+from
    | yield\\s+wait
    | yield(\\s+return)?)",
                )
                .unwrap()
            })
        })
    })
}
#[inline]
fn parse_kw_namespace(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_NAMESPACE, |s| s.match_string("namespace", false))
}
#[inline]
fn parse_kw_import(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_IMPORT, |s| s.match_string("using", false))
}
#[inline]
fn parse_kw_constraint(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_CONSTRAINT, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(template|generic|constraint|∀)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_where(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_WHERE, |s| s.match_string("where", false))
}
#[inline]
fn parse_kw_implements(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_IMPLEMENTS, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(implements?)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_extends(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_EXTENDS, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(extends?)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_inherits(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_INHERITS, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(inherits?)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_for(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_FOR, |s| s.match_string("for", false))
}
#[inline]
fn parse_kw_end(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_END, |s| s.match_string("end", false))
}
#[inline]
fn parse_kw_let(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_LET, |s| s.match_string("let", false))
}
#[inline]
fn parse_kw_new(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_NEW, |s| s.match_string("new", false))
}
#[inline]
fn parse_kw_object(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_OBJECT, |s| s.match_string("object", false))
}
#[inline]
fn parse_kw_lambda(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_LAMBDA, |s| s.match_string("lambda", false))
}
#[inline]
fn parse_kw_if(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_IF, |s| s.match_string("if", false))
}
#[inline]
fn parse_kw_switch(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_SWITCH, |s| s.match_string("switch", false))
}
#[inline]
fn parse_kw_try(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_TRY, |s| s.match_string("try", false))
}
#[inline]
fn parse_kw_type(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_TYPE, |s| s.match_string("type", false))
}
#[inline]
fn parse_kw_case(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_CASE, |s| s.match_string("case", false))
}
#[inline]
fn parse_kw_when(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_WHEN, |s| s.match_string("when", false))
}
#[inline]
fn parse_kw_else(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_ELSE, |s| s.match_string("else", false))
}
#[inline]
fn parse_kw_not(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_NOT, |s| s.match_string("not", false))
}
#[inline]
fn parse_kw_in(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_IN, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(in|∈)").unwrap())
        })
    })
}
#[inline]
fn parse_kw_is(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_IS, |s| s.match_string("is", false))
}
#[inline]
fn parse_kw_as(state: Input) -> Output {
    state.rule(ValkyrieRule::KW_AS, |s| s.match_string("as", false))
}
#[inline]
fn parse_shebang(state: Input) -> Output {
    state.rule(ValkyrieRule::Shebang, |s| {
        s.sequence(|s| Ok(s).and_then(|s| builtin_text(s, "#!", false)).and_then(|s| s.rest_of_line()))
    })
}
#[inline]
fn parse_white_space(state: Input) -> Output {
    state.rule(ValkyrieRule::WhiteSpace, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([^\\S\\r\\n]+)").unwrap())
        })
    })
}
#[inline]
fn parse_skip_space(state: Input) -> Output {
    state.rule(ValkyrieRule::SkipSpace, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(\\p{White_Space}+)").unwrap())
        })
    })
}
#[inline]
fn parse_comment(state: Input) -> Output {
    state.rule(ValkyrieRule::Comment, |s| {
        Err(s)
            .or_else(|s| {
                s.sequence(|s| {
                    Ok(s)
                        .and_then(|s| {
                            builtin_regex(s, {
                                static REGEX: OnceLock<Regex> = OnceLock::new();
                                REGEX.get_or_init(|| Regex::new("^(?x)([~⍝🗨])").unwrap())
                            })
                        })
                        .and_then(|s| s.rest_of_line())
                })
            })
            .or_else(|s| {
                s.sequence(|s| Ok(s).and_then(|s| builtin_text(s, "/*", false)).and_then(|s| builtin_text(s, "*/", false)))
            })
    })
}
#[inline]
fn parse_string_interpolations(state: Input) -> Output {
    state.rule(ValkyrieRule::StringInterpolations, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        parse_string_interpolation_term(s).and_then(|s| s.tag_node("string_interpolation_term"))
                    })
                })
                .and_then(|s| s.end_of_input())
        })
    })
}
#[inline]
fn parse_string_interpolation_term(state: Input) -> Output {
    state.rule(ValkyrieRule::StringInterpolationTerm, |s| {
        Err(s)
            .or_else(|s| parse_escape_unicode(s).and_then(|s| s.tag_node("escape_unicode")))
            .or_else(|s| parse_escape_character(s).and_then(|s| s.tag_node("escape_character")))
            .or_else(|s| parse_string_interpolation_simple(s).and_then(|s| s.tag_node("string_interpolation_simple")))
            .or_else(|s| parse_string_interpolation_complex(s).and_then(|s| s.tag_node("string_interpolation_complex")))
            .or_else(|s| parse_string_interpolation_text(s).and_then(|s| s.tag_node("string_interpolation_text")))
    })
}
#[inline]
fn parse_escape_character(state: Input) -> Output {
    state.rule(ValkyrieRule::EscapeCharacter, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)(\\\\.|\\{\\{|\\}\\})").unwrap())
        })
    })
}
#[inline]
fn parse_escape_unicode(state: Input) -> Output {
    state.rule(ValkyrieRule::EscapeUnicode, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.sequence(|s| {
                        Ok(s)
                            .and_then(|s| builtin_text(s, "\\u", false))
                            .and_then(|s| builtin_text(s, "{", false))
                            .and_then(|s| builtin_ignore(s))
                    })
                })
                .and_then(|s| parse_escape_unicode_code(s).and_then(|s| s.tag_node("code")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_escape_unicode_code(state: Input) -> Output {
    state.rule(ValkyrieRule::EscapeUnicodeCode, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([0-9a-zA-Z]*)").unwrap())
        })
    })
}
#[inline]
fn parse_string_interpolation_simple(state: Input) -> Output {
    state.rule(ValkyrieRule::StringInterpolationSimple, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.optional(|s| {
                        s.sequence(|s| {
                            Ok(s)
                                .and_then(|s| parse_colon(s))
                                .and_then(|s| builtin_ignore(s))
                                .and_then(|s| parse_string_formatter(s).and_then(|s| s.tag_node("string_formatter")))
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_string_interpolation_text(state: Input) -> Output {
    state.rule(ValkyrieRule::StringInterpolationText, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([^{}\\\\]+)").unwrap())
        })
    })
}
#[inline]
fn parse_string_formatter(state: Input) -> Output {
    state.rule(ValkyrieRule::StringFormatter, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([^}]+)").unwrap())
        })
    })
}
#[inline]
fn parse_string_interpolation_complex(state: Input) -> Output {
    state.rule(ValkyrieRule::StringInterpolationComplex, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| builtin_text(s, "{", false))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| {
                        s.sequence(|s| {
                            Ok(s).and_then(|s| builtin_ignore(s)).and_then(|s| {
                                s.sequence(|s| {
                                    Ok(s)
                                        .and_then(|s| parse_comma(s))
                                        .and_then(|s| builtin_ignore(s))
                                        .and_then(|s| parse_tuple_pair(s).and_then(|s| s.tag_node("tuple_pair")))
                                })
                            })
                        })
                    })
                })
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_comma(s)))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| builtin_text(s, "}", false))
        })
    })
}
#[inline]
fn parse_string_templates(state: Input) -> Output {
    state.rule(ValkyrieRule::StringTemplates, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| {
                    s.repeat(0..4294967295, |s| parse_string_template_term(s).and_then(|s| s.tag_node("string_template_term")))
                })
                .and_then(|s| s.end_of_input())
        })
    })
}
#[inline]
fn parse_string_template_term(state: Input) -> Output {
    state.rule(ValkyrieRule::StringTemplateTerm, |s| {
        Err(s)
            .or_else(|s| parse_for_template(s).and_then(|s| s.tag_node("for_template")))
            .or_else(|s| parse_expression_template(s).and_then(|s| s.tag_node("expression_template")))
    })
}
#[inline]
fn parse_expression_template(state: Input) -> Output {
    state.rule(ValkyrieRule::ExpressionTemplate, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_template_s(s).and_then(|s| s.tag_node("template_s")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_main_expression(s).and_then(|s| s.tag_node("main_expression")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_template_e(s).and_then(|s| s.tag_node("template_e")))
        })
    })
}
#[inline]
fn parse_for_template(state: Input) -> Output {
    state.rule(ValkyrieRule::ForTemplate, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_for_template_begin(s).and_then(|s| s.tag_node("for_template_begin")))
                .and_then(|s| s.optional(|s| parse_for_template_else(s).and_then(|s| s.tag_node("for_template_else"))))
                .and_then(|s| parse_for_template_end(s).and_then(|s| s.tag_node("for_template_end")))
        })
    })
}
#[inline]
fn parse_for_template_begin(state: Input) -> Output {
    state.rule(ValkyrieRule::ForTemplateBegin, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_template_s(s).and_then(|s| s.tag_node("template_s")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_for(s).and_then(|s| s.tag_node("kw_for")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_let_pattern(s).and_then(|s| s.tag_node("let_pattern")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_in(s).and_then(|s| s.tag_node("kw_in")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_inline_expression(s).and_then(|s| s.tag_node("inline_expression"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_if_guard(s).and_then(|s| s.tag_node("if_guard")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_template_e(s).and_then(|s| s.tag_node("template_e")))
        })
    })
}
#[inline]
fn parse_for_template_else(state: Input) -> Output {
    state.rule(ValkyrieRule::ForTemplateElse, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_template_s(s).and_then(|s| s.tag_node("template_s")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_else(s).and_then(|s| s.tag_node("kw_else")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_template_e(s).and_then(|s| s.tag_node("template_e")))
        })
    })
}
#[inline]
fn parse_for_template_end(state: Input) -> Output {
    state.rule(ValkyrieRule::ForTemplateEnd, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_template_s(s).and_then(|s| s.tag_node("template_s")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_kw_end(s).and_then(|s| s.tag_node("kw_end")))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| s.optional(|s| parse_kw_for(s).and_then(|s| s.tag_node("kw_for"))))
                .and_then(|s| builtin_ignore(s))
                .and_then(|s| parse_template_e(s).and_then(|s| s.tag_node("template_e")))
        })
    })
}
#[inline]
fn parse_template_s(state: Input) -> Output {
    state.rule(ValkyrieRule::TEMPLATE_S, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| parse_template_l(s))
                .and_then(|s| s.optional(|s| parse_template_m(s).and_then(|s| s.tag_node("template_m"))))
        })
    })
}
#[inline]
fn parse_template_e(state: Input) -> Output {
    state.rule(ValkyrieRule::TEMPLATE_E, |s| {
        s.sequence(|s| {
            Ok(s)
                .and_then(|s| s.optional(|s| parse_template_m(s).and_then(|s| s.tag_node("template_m"))))
                .and_then(|s| parse_template_r(s))
        })
    })
}
#[inline]
fn parse_template_l(state: Input) -> Output {
    state.rule(ValkyrieRule::TEMPLATE_L, |s| s.match_string("<%", false))
}
#[inline]
fn parse_template_r(state: Input) -> Output {
    state.rule(ValkyrieRule::TEMPLATE_R, |s| s.match_string("%>", false))
}
#[inline]
fn parse_template_m(state: Input) -> Output {
    state.rule(ValkyrieRule::TEMPLATE_M, |s| {
        s.match_regex({
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| Regex::new("^(?x)([-_~.=])").unwrap())
        })
    })
}

/// All rules ignored in ast mode, inline is not recommended
fn builtin_ignore(state: Input) -> Output {
    state.repeat(0..u32::MAX, |s| parse_skip_space(s).or_else(|s| parse_comment(s)))
}

fn builtin_any(state: Input) -> Output {
    state.rule(ValkyrieRule::HiddenText, |s| s.match_char_if(|_| true))
}

fn builtin_text<'i>(state: Input<'i>, text: &'static str, case: bool) -> Output<'i> {
    state.rule(ValkyrieRule::HiddenText, |s| s.match_string(text, case))
}

fn builtin_regex<'i, 'r>(state: Input<'i>, regex: &'r Regex) -> Output<'i> {
    state.rule(ValkyrieRule::HiddenText, |s| s.match_regex(regex))
}
