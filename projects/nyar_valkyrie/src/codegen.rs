//! 代码生成模块
//!
//! 提供多种后端的代码生成功能，包括 JavaScript、WebAssembly 等。

use std::{
    collections::HashMap,
    fmt,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::fs;

use nyar_hir::{HirNode, Module};

use crate::{
    config::CodegenConfig,
    error::{CodegenError, RuntimeResult},
};

/// 代码生成目标
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodegenTarget {
    /// JavaScript (ES2020+)
    JavaScript,
    /// TypeScript 定义文件
    TypeScript,
    /// WebAssembly 文本格式
    WebAssembly,
    /// LLVM IR
    LLVM,
    /// 自定义目标
    Custom(String),
}

impl fmt::Display for CodegenTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenTarget::JavaScript => write!(f, "javascript"),
            CodegenTarget::TypeScript => write!(f, "typescript"),
            CodegenTarget::WebAssembly => write!(f, "webassembly"),
            CodegenTarget::LLVM => write!(f, "llvm"),
            CodegenTarget::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// 代码生成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodegenOptions {
    /// 目标平台
    pub target: CodegenTarget,
    /// 优化级别
    pub optimization_level: OptimizationLevel,
    /// 输出目录
    pub output_dir: String,
    /// 是否生成调试信息
    pub debug_info: bool,
    /// 是否生成 source map
    pub source_map: bool,
    /// 模块系统
    pub module_system: ModuleSystem,
    /// 自定义选项
    pub custom_options: HashMap<String, String>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            target: CodegenTarget::JavaScript,
            optimization_level: OptimizationLevel::None,
            output_dir: "./dist".to_string(),
            debug_info: true,
            source_map: true,
            module_system: ModuleSystem::ES6,
            custom_options: HashMap::new(),
        }
    }
}

/// 优化级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// 无优化
    None,
    /// 基本优化
    Basic,
    /// 完全优化
    Full,
    /// 大小优化
    Size,
    /// 速度优化
    Speed,
}

/// 模块系统
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleSystem {
    /// ES6 模块
    ES6,
    /// CommonJS
    CommonJS,
    /// AMD
    AMD,
    /// UMD
    UMD,
    /// 无模块系统
    None,
}

/// 代码生成结果
#[derive(Debug, Clone)]
pub struct CodegenResult {
    /// 生成的代码
    pub code: String,
    /// Source map（如果启用）
    pub source_map: Option<String>,
    /// 类型定义（如果适用）
    pub type_definitions: Option<String>,
    /// 生成的文件路径
    pub output_files: Vec<String>,
    /// 生成耗时
    pub duration: Duration,
    /// 统计信息
    pub stats: CodegenStats,
}

/// 代码生成统计
#[derive(Debug, Clone, Default)]
pub struct CodegenStats {
    /// 生成的代码行数
    pub lines_generated: usize,
    /// 生成的字节数
    pub bytes_generated: usize,
    /// 处理的函数数量
    pub functions_processed: usize,
    /// 处理的类型数量
    pub types_processed: usize,
    /// 应用的优化数量
    pub optimizations_applied: usize,
}

/// 代码生成器 trait
pub trait CodeGenerator: Send + Sync {
    /// 生成代码
    async fn generate(&self, module: &Module, options: &CodegenOptions) -> RuntimeResult<CodegenResult>;

    /// 获取支持的目标
    fn supported_targets(&self) -> Vec<CodegenTarget>;

    /// 获取生成器名称
    fn name(&self) -> &str;

    /// 获取生成器版本
    fn version(&self) -> &str;
}

/// JavaScript 代码生成器
pub struct JavaScriptGenerator {
    config: CodegenConfig,
}

impl JavaScriptGenerator {
    pub fn new(config: CodegenConfig) -> Self {
        Self { config }
    }

    /// 生成函数代码
    fn generate_function(&self, function: &nyar_hir::Function) -> RuntimeResult<String> {
        let mut code = String::new();

        // 生成函数签名
        code.push_str(&format!("function {}(", function.name));

        // 生成参数
        for (i, param) in function.parameters.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            code.push_str(&param.name);
        }

        code.push_str(") {\n");

        // 生成函数体
        if let Some(body) = &function.body {
            let body_code = self.generate_block(body)?;
            code.push_str(&body_code);
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// 生成代码块
    fn generate_block(&self, block: &nyar_hir::Block) -> RuntimeResult<String> {
        let mut code = String::new();

        for statement in &block.statements {
            let stmt_code = self.generate_statement(statement)?;
            code.push_str(&format!("  {}\n", stmt_code));
        }

        if let Some(expr) = &block.expression {
            let expr_code = self.generate_expression(expr)?;
            code.push_str(&format!("  return {};\n", expr_code));
        }

        Ok(code)
    }

    /// 生成语句
    fn generate_statement(&self, statement: &nyar_hir::Statement) -> RuntimeResult<String> {
        match statement {
            nyar_hir::Statement::Let { name, value, .. } => {
                let value_code =
                    if let Some(value) = value { self.generate_expression(value)? } else { "undefined".to_string() };
                Ok(format!("let {} = {};", name, value_code))
            }
            nyar_hir::Statement::Expression(expr) => {
                let expr_code = self.generate_expression(expr)?;
                Ok(format!("{};", expr_code))
            }
            _ => {
                // TODO: 实现其他语句类型
                Ok("// TODO: implement statement".to_string())
            }
        }
    }

    /// 生成表达式
    fn generate_expression(&self, expression: &nyar_hir::Expression) -> RuntimeResult<String> {
        match expression {
            nyar_hir::Expression::Literal(literal) => Ok(self.generate_literal(literal)),
            nyar_hir::Expression::Identifier(name) => Ok(name.clone()),
            nyar_hir::Expression::Call { function, arguments } => {
                let func_code = self.generate_expression(function)?;
                let mut args_code = String::new();

                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        args_code.push_str(", ");
                    }
                    args_code.push_str(&self.generate_expression(arg)?);
                }

                Ok(format!("{}({})", func_code, args_code))
            }
            nyar_hir::Expression::Binary { left, operator, right } => {
                let left_code = self.generate_expression(left)?;
                let right_code = self.generate_expression(right)?;
                let op_code = self.generate_binary_operator(operator);

                Ok(format!("({} {} {})", left_code, op_code, right_code))
            }
            _ => {
                // TODO: 实现其他表达式类型
                Ok("/* TODO: implement expression */".to_string())
            }
        }
    }

    /// 生成字面量
    fn generate_literal(&self, literal: &nyar_hir::Literal) -> String {
        match literal {
            nyar_hir::Literal::Integer(n) => n.to_string(),
            nyar_hir::Literal::Float(f) => f.to_string(),
            nyar_hir::Literal::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            nyar_hir::Literal::Boolean(b) => b.to_string(),
            nyar_hir::Literal::Null => "null".to_string(),
        }
    }

    /// 生成二元运算符
    fn generate_binary_operator(&self, operator: &nyar_hir::BinaryOperator) -> &'static str {
        match operator {
            nyar_hir::BinaryOperator::Add => "+",
            nyar_hir::BinaryOperator::Subtract => "-",
            nyar_hir::BinaryOperator::Multiply => "*",
            nyar_hir::BinaryOperator::Divide => "/",
            nyar_hir::BinaryOperator::Modulo => "%",
            nyar_hir::BinaryOperator::Equal => "===",
            nyar_hir::BinaryOperator::NotEqual => "!==",
            nyar_hir::BinaryOperator::Less => "<",
            nyar_hir::BinaryOperator::LessEqual => "<=",
            nyar_hir::BinaryOperator::Greater => ">",
            nyar_hir::BinaryOperator::GreaterEqual => ">=",
            nyar_hir::BinaryOperator::And => "&&",
            nyar_hir::BinaryOperator::Or => "||",
        }
    }

    /// 生成 source map
    fn generate_source_map(&self, _module: &Module) -> RuntimeResult<String> {
        // TODO: 实现 source map 生成
        Ok(r#"{
  "version": 3,
  "sources": [],
  "names": [],
  "mappings": ""
}"#
        .to_string())
    }
}

#[async_trait::async_trait]
impl CodeGenerator for JavaScriptGenerator {
    async fn generate(&self, module: &Module, options: &CodegenOptions) -> RuntimeResult<CodegenResult> {
        let start_time = Instant::now();
        let mut code = String::new();
        let mut stats = CodegenStats::default();

        // 生成模块头部
        code.push_str(&format!("// Generated from module: {}\n", module.name));
        code.push_str("// This file was automatically generated. Do not edit.\n\n");

        // 生成导入语句
        for import in &module.imports {
            code.push_str(&format!("import {{ {} }} from '{}';\n", import.items.join(", "), import.path));
        }

        if !module.imports.is_empty() {
            code.push('\n');
        }

        // 生成函数
        for function in &module.functions {
            let func_code = self.generate_function(function)?;
            code.push_str(&func_code);
            code.push('\n');
            stats.functions_processed += 1;
        }

        // 生成导出语句
        if !module.exports.is_empty() {
            code.push_str("export {\n");
            for (i, export) in module.exports.iter().enumerate() {
                if i > 0 {
                    code.push_str(",\n");
                }
                code.push_str(&format!("  {}", export));
            }
            code.push_str("\n};\n");
        }

        // 生成 source map（如果启用）
        let source_map = if options.source_map { Some(self.generate_source_map(module)?) } else { None };

        // 更新统计信息
        stats.lines_generated = code.lines().count();
        stats.bytes_generated = code.len();

        Ok(CodegenResult {
            code,
            source_map,
            type_definitions: None,
            output_files: vec![],
            duration: start_time.elapsed(),
            stats,
        })
    }

    fn supported_targets(&self) -> Vec<CodegenTarget> {
        vec![CodegenTarget::JavaScript]
    }

    fn name(&self) -> &str {
        "JavaScript Generator"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

/// TypeScript 定义生成器
pub struct TypeScriptGenerator {
    config: CodegenConfig,
}

impl TypeScriptGenerator {
    pub fn new(config: CodegenConfig) -> Self {
        Self { config }
    }

    /// 生成类型定义
    fn generate_type_definitions(&self, module: &Module) -> RuntimeResult<String> {
        let mut code = String::new();

        // 生成模块声明
        code.push_str(&format!("declare module '{}' {{\n", module.name));

        // 生成函数声明
        for function in &module.functions {
            code.push_str(&format!("  export function {}(", function.name));

            // 生成参数类型
            for (i, param) in function.parameters.iter().enumerate() {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&format!("{}: {}", param.name, self.convert_type(&param.type_name)));
            }

            code.push_str(&format!(
                ") : {};\n",
                self.convert_type(&function.return_type.clone().unwrap_or_else(|| "void".to_string()))
            ));
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// 转换类型名称
    fn convert_type(&self, type_name: &str) -> &str {
        match type_name {
            "i32" | "i64" | "f32" | "f64" => "number",
            "bool" => "boolean",
            "str" => "string",
            "()" => "void",
            _ => "any",
        }
    }
}

#[async_trait::async_trait]
impl CodeGenerator for TypeScriptGenerator {
    async fn generate(&self, module: &Module, _options: &CodegenOptions) -> RuntimeResult<CodegenResult> {
        let start_time = Instant::now();
        let code = self.generate_type_definitions(module)?;
        let stats = CodegenStats {
            lines_generated: code.lines().count(),
            bytes_generated: code.len(),
            functions_processed: module.functions.len(),
            types_processed: 0, // TODO: 计算实际类型数量
            optimizations_applied: 0,
        };

        Ok(CodegenResult {
            code: String::new(),
            source_map: None,
            type_definitions: Some(code),
            output_files: vec![],
            duration: start_time.elapsed(),
            stats,
        })
    }

    fn supported_targets(&self) -> Vec<CodegenTarget> {
        vec![CodegenTarget::TypeScript]
    }

    fn name(&self) -> &str {
        "TypeScript Definition Generator"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

/// 代码生成管理器
pub struct CodegenManager {
    /// 配置
    config: CodegenConfig,
    /// 注册的生成器
    generators: HashMap<CodegenTarget, Box<dyn CodeGenerator>>,
}

impl CodegenManager {
    /// 创建新的代码生成管理器
    pub fn new(config: CodegenConfig) -> Self {
        let mut manager = Self { config: config.clone(), generators: HashMap::new() };

        // 注册默认生成器
        manager.register_generator(CodegenTarget::JavaScript, Box::new(JavaScriptGenerator::new(config.clone())));
        manager.register_generator(CodegenTarget::TypeScript, Box::new(TypeScriptGenerator::new(config)));

        manager
    }

    /// 注册代码生成器
    pub fn register_generator(&mut self, target: CodegenTarget, generator: Box<dyn CodeGenerator>) {
        self.generators.insert(target, generator);
    }

    /// 生成代码
    pub async fn generate_code(&self, module: &Module, options: &CodegenOptions) -> RuntimeResult<CodegenResult> {
        let generator = self
            .generators
            .get(&options.target)
            .ok_or_else(|| CodegenError::UnsupportedTarget { target: options.target.to_string() })?;

        generator.generate(module, options).await
    }

    /// 写入生成的代码到文件
    pub async fn write_to_files(
        &self,
        result: &CodegenResult,
        options: &CodegenOptions,
        module_name: &str,
    ) -> RuntimeResult<Vec<String>> {
        let mut output_files = Vec::new();
        let output_dir = Path::new(&options.output_dir);

        // 确保输出目录存在
        fs::create_dir_all(output_dir)
            .await
            .map_err(|e| CodegenError::IoError { message: format!("Failed to create output directory: {}", e) })?;

        // 写入主代码文件
        if !result.code.is_empty() {
            let extension = match options.target {
                CodegenTarget::JavaScript => "js",
                CodegenTarget::WebAssembly => "wat",
                CodegenTarget::LLVM => "ll",
                _ => "txt",
            };

            let code_file = output_dir.join(format!("{}.{}", module_name, extension));
            fs::write(&code_file, &result.code)
                .await
                .map_err(|e| CodegenError::IoError { message: format!("Failed to write code file: {}", e) })?;

            output_files.push(code_file.to_string_lossy().to_string());
        }

        // 写入 source map 文件
        if let Some(source_map) = &result.source_map {
            let map_file = output_dir.join(format!("{}.js.map", module_name));
            fs::write(&map_file, source_map)
                .await
                .map_err(|e| CodegenError::IoError { message: format!("Failed to write source map file: {}", e) })?;

            output_files.push(map_file.to_string_lossy().to_string());
        }

        // 写入类型定义文件
        if let Some(type_definitions) = &result.type_definitions {
            let types_file = output_dir.join(format!("{}.d.ts", module_name));
            fs::write(&types_file, type_definitions)
                .await
                .map_err(|e| CodegenError::IoError { message: format!("Failed to write type definitions file: {}", e) })?;

            output_files.push(types_file.to_string_lossy().to_string());
        }

        Ok(output_files)
    }

    /// 获取支持的目标列表
    pub fn supported_targets(&self) -> Vec<CodegenTarget> {
        self.generators.keys().cloned().collect()
    }

    /// 获取生成器信息
    pub fn generator_info(&self, target: &CodegenTarget) -> Option<(String, String)> {
        self.generators.get(target).map(|gen| (gen.name().to_string(), gen.version().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CodegenConfig;
    use nyar_hir::*;

    fn create_test_module() -> Module {
        Module {
            name: "test".to_string(),
            functions: vec![Function {
                name: "add".to_string(),
                parameters: vec![
                    Parameter { name: "a".to_string(), type_name: "i32".to_string(), span: SourceSpan::default() },
                    Parameter { name: "b".to_string(), type_name: "i32".to_string(), span: SourceSpan::default() },
                ],
                return_type: Some("i32".to_string()),
                body: None,
                span: SourceSpan::default(),
                documentation: None,
            }],
            imports: vec![],
            exports: vec!["add".to_string()],
            span: SourceSpan::default(),
        }
    }

    #[tokio::test]
    async fn test_javascript_generator() {
        let config = CodegenConfig::default();
        let generator = JavaScriptGenerator::new(config);
        let module = create_test_module();
        let options = CodegenOptions::default();

        let result = generator.generate(&module, &options).await.unwrap();

        assert!(result.code.contains("function add"));
        assert!(result.stats.functions_processed > 0);
    }

    #[tokio::test]
    async fn test_typescript_generator() {
        let config = CodegenConfig::default();
        let generator = TypeScriptGenerator::new(config);
        let module = create_test_module();
        let options = CodegenOptions { target: CodegenTarget::TypeScript, ..Default::default() };

        let result = generator.generate(&module, &options).await.unwrap();

        assert!(result.type_definitions.is_some());
        let type_defs = result.type_definitions.unwrap();
        assert!(type_defs.contains("export function add"));
        assert!(type_defs.contains("number"));
    }

    #[tokio::test]
    async fn test_codegen_manager() {
        let config = CodegenConfig::default();
        let manager = CodegenManager::new(config);
        let module = create_test_module();

        // 测试 JavaScript 生成
        let js_options = CodegenOptions { target: CodegenTarget::JavaScript, ..Default::default() };
        let js_result = manager.generate_code(&module, &js_options).await.unwrap();
        assert!(js_result.code.contains("function add"));

        // 测试 TypeScript 生成
        let ts_options = CodegenOptions { target: CodegenTarget::TypeScript, ..Default::default() };
        let ts_result = manager.generate_code(&module, &ts_options).await.unwrap();
        assert!(ts_result.type_definitions.is_some());

        // 测试支持的目标
        let targets = manager.supported_targets();
        assert!(targets.contains(&CodegenTarget::JavaScript));
        assert!(targets.contains(&CodegenTarget::TypeScript));
    }

    #[test]
    fn test_codegen_target_display() {
        assert_eq!(CodegenTarget::JavaScript.to_string(), "javascript");
        assert_eq!(CodegenTarget::TypeScript.to_string(), "typescript");
        assert_eq!(CodegenTarget::WebAssembly.to_string(), "webassembly");
        assert_eq!(CodegenTarget::Custom("rust".to_string()).to_string(), "custom:rust");
    }

    #[test]
    fn test_codegen_options_default() {
        let options = CodegenOptions::default();
        assert_eq!(options.target, CodegenTarget::JavaScript);
        assert_eq!(options.optimization_level, OptimizationLevel::None);
        assert!(options.debug_info);
        assert!(options.source_map);
    }
}
