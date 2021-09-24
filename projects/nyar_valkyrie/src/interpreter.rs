//! 解释器模块
//!
//! 提供 Nyar 代码的解释执行功能。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::time::timeout;
use parking_lot::RwLock;

use nyar_core::{Position, Value};
use nyar_hir::{HirNode, Module, Function, Statement, Expression};

use crate::{
    config::InterpreterConfig,
    error::{InterpreterError, RuntimeError, RuntimeResult},
};

/// 解释器执行结果
#[derive(Debug, Clone)]
pub struct InterpreterResult {
    /// 返回值
    pub value: Value,
    /// 执行时间
    pub execution_time: Duration,
    /// 内存使用统计
    pub memory_stats: MemoryStats,
    /// 性能分析数据（如果启用）
    pub profiling_data: Option<ProfilingData>,
}

/// 内存使用统计
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// 栈使用量（字节）
    pub stack_used: usize,
    /// 堆使用量（字节）
    pub heap_used: usize,
    /// 最大栈使用量
    pub max_stack_used: usize,
    /// 最大堆使用量
    pub max_heap_used: usize,
}

/// 性能分析数据
#[derive(Debug, Clone)]
pub struct ProfilingData {
    /// 函数调用统计
    pub function_calls: HashMap<String, FunctionStats>,
    /// 总指令数
    pub total_instructions: u64,
    /// 分支预测统计
    pub branch_stats: BranchStats,
}

/// 函数调用统计
#[derive(Debug, Clone)]
pub struct FunctionStats {
    /// 调用次数
    pub call_count: u64,
    /// 总执行时间
    pub total_time: Duration,
    /// 平均执行时间
    pub average_time: Duration,
}

/// 分支预测统计
#[derive(Debug, Clone)]
pub struct BranchStats {
    /// 总分支数
    pub total_branches: u64,
    /// 预测正确数
    pub correct_predictions: u64,
    /// 预测准确率
    pub accuracy: f64,
}

/// 执行栈帧
#[derive(Debug, Clone)]
struct StackFrame {
    /// 函数名
    function_name: String,
    /// 局部变量
    locals: HashMap<String, Value>,
    /// 返回地址
    return_position: Option<Position>,
}

/// 解释器引擎
pub struct InterpreterEngine {
    /// 配置
    config: InterpreterConfig,
    /// 执行栈
    stack: Vec<StackFrame>,
    /// 全局变量
    globals: Arc<RwLock<HashMap<String, Value>>>,
    /// 内存统计
    memory_stats: MemoryStats,
    /// 性能分析器（如果启用）
    profiler: Option<Profiler>,
    /// 执行开始时间
    start_time: Option<Instant>,
}

impl InterpreterEngine {
    /// 创建新的解释器引擎
    pub async fn new(config: InterpreterConfig) -> RuntimeResult<Self> {
        let profiler = if config.enable_profiling {
            Some(Profiler::new())
        } else {
            None
        };
        
        Ok(Self {
            config,
            stack: Vec::new(),
            globals: Arc::new(RwLock::new(HashMap::new())),
            memory_stats: MemoryStats {
                stack_used: 0,
                heap_used: 0,
                max_stack_used: 0,
                max_heap_used: 0,
            },
            profiler,
            start_time: None,
        })
    }
    
    /// 执行 HIR 模块
    pub async fn execute(&mut self, module: &Module) -> RuntimeResult<InterpreterResult> {
        self.start_time = Some(Instant::now());
        
        // 设置执行超时
        let execution_future = self.execute_internal(module);
        
        let result = if let Some(timeout_duration) = self.config.execution_timeout {
            timeout(timeout_duration, execution_future)
                .await
                .map_err(|_| {
                    InterpreterError::ExecutionTimeout {
                        timeout_ms: timeout_duration.as_millis() as u64,
                    }
                })?
        } else {
            execution_future.await
        };
        
        let execution_time = self.start_time.unwrap().elapsed();
        
        match result {
            Ok(value) => Ok(InterpreterResult {
                value,
                execution_time,
                memory_stats: self.memory_stats.clone(),
                profiling_data: self.profiler.as_ref().map(|p| p.get_data()),
            }),
            Err(e) => Err(RuntimeError::InterpreterError(e)),
        }
    }
    
    /// 内部执行逻辑
    async fn execute_internal(&mut self, module: &Module) -> Result<Value, InterpreterError> {
        // 查找 main 函数
        let main_function = module.functions.iter()
            .find(|f| f.name == "main")
            .ok_or_else(|| InterpreterError::RuntimeError {
                position: Position { line: 0, column: 0 },
                message: "No main function found".to_string(),
                source_code: String::new(),
                span: miette::SourceSpan::new(0.into(), 0.into()),
            })?;
        
        // 执行 main 函数
        self.call_function(main_function, vec![]).await
    }
    
    /// 调用函数
    async fn call_function(
        &mut self,
        function: &Function,
        args: Vec<Value>,
    ) -> Result<Value, InterpreterError> {
        // 检查栈溢出
        self.check_stack_overflow()?;
        
        // 创建新的栈帧
        let mut frame = StackFrame {
            function_name: function.name.clone(),
            locals: HashMap::new(),
            return_position: None,
        };
        
        // 设置参数
        for (i, param) in function.parameters.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                frame.locals.insert(param.name.clone(), arg.clone());
            }
        }
        
        // 推入栈帧
        self.stack.push(frame);
        self.update_memory_stats();
        
        // 记录函数调用（如果启用性能分析）
        if let Some(profiler) = &mut self.profiler {
            profiler.record_function_call(&function.name);
        }
        
        // 执行函数体
        let result = self.execute_block(&function.body).await;
        
        // 弹出栈帧
        self.stack.pop();
        self.update_memory_stats();
        
        result
    }
    
    /// 执行语句块
    async fn execute_block(&mut self, statements: &[Statement]) -> Result<Value, InterpreterError> {
        let mut last_value = Value::Unit;
        
        for statement in statements {
            match self.execute_statement(statement).await? {
                StatementResult::Value(value) => last_value = value,
                StatementResult::Return(value) => return Ok(value),
                StatementResult::Break => break,
                StatementResult::Continue => continue,
                StatementResult::None => {},
            }
        }
        
        Ok(last_value)
    }
    
    /// 执行语句
    async fn execute_statement(&mut self, statement: &Statement) -> Result<StatementResult, InterpreterError> {
        match statement {
            Statement::Expression(expr) => {
                let value = self.evaluate_expression(expr).await?;
                Ok(StatementResult::Value(value))
            }
            Statement::Let { name, value, .. } => {
                let val = self.evaluate_expression(value).await?;
                if let Some(frame) = self.stack.last_mut() {
                    frame.locals.insert(name.clone(), val);
                }
                Ok(StatementResult::None)
            }
            Statement::Return(expr) => {
                let value = if let Some(expr) = expr {
                    self.evaluate_expression(expr).await?
                } else {
                    Value::Unit
                };
                Ok(StatementResult::Return(value))
            }
            Statement::If { condition, then_block, else_block } => {
                let condition_value = self.evaluate_expression(condition).await?;
                
                if self.is_truthy(&condition_value) {
                    self.execute_block(then_block).await.map(StatementResult::Value)
                } else if let Some(else_block) = else_block {
                    self.execute_block(else_block).await.map(StatementResult::Value)
                } else {
                    Ok(StatementResult::None)
                }
            }
            Statement::While { condition, body } => {
                while self.is_truthy(&self.evaluate_expression(condition).await?) {
                    match self.execute_block(body).await? {
                        value => return Ok(StatementResult::Value(value)),
                    }
                }
                Ok(StatementResult::None)
            }
            _ => {
                // TODO: 实现其他语句类型
                Err(InterpreterError::RuntimeError {
                    position: Position { line: 0, column: 0 },
                    message: "Statement type not implemented".to_string(),
                    source_code: String::new(),
                    span: miette::SourceSpan::new(0.into(), 0.into()),
                })
            }
        }
    }
    
    /// 语句执行结果
    enum StatementResult {
        Value(Value),
        Return(Value),
        Break,
        Continue,
        None,
    }
    
    /// 计算表达式
    async fn evaluate_expression(&mut self, expression: &Expression) -> Result<Value, InterpreterError> {
        match expression {
            Expression::Literal(literal) => Ok(literal.value.clone()),
            Expression::Identifier(name) => self.resolve_variable(name),
            Expression::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(left).await?;
                let right_val = self.evaluate_expression(right).await?;
                self.apply_binary_operator(&left_val, operator, &right_val)
            }
            Expression::Unary { operator, operand } => {
                let operand_val = self.evaluate_expression(operand).await?;
                self.apply_unary_operator(operator, &operand_val)
            }
            Expression::Call { function, arguments } => {
                // TODO: 实现函数调用
                Err(InterpreterError::RuntimeError {
                    position: Position { line: 0, column: 0 },
                    message: "Function calls not implemented".to_string(),
                    source_code: String::new(),
                    span: miette::SourceSpan::new(0.into(), 0.into()),
                })
            }
            _ => {
                // TODO: 实现其他表达式类型
                Err(InterpreterError::RuntimeError {
                    position: Position { line: 0, column: 0 },
                    message: "Expression type not implemented".to_string(),
                    source_code: String::new(),
                    span: miette::SourceSpan::new(0.into(), 0.into()),
                })
            }
        }
    }
    
    /// 解析变量
    fn resolve_variable(&self, name: &str) -> Result<Value, InterpreterError> {
        // 首先在当前栈帧中查找
        if let Some(frame) = self.stack.last() {
            if let Some(value) = frame.locals.get(name) {
                return Ok(value.clone());
            }
        }
        
        // 然后在全局变量中查找
        let globals = self.globals.read();
        if let Some(value) = globals.get(name) {
            return Ok(value.clone());
        }
        
        Err(InterpreterError::RuntimeError {
            position: Position { line: 0, column: 0 },
            message: format!("Undefined variable: {}", name),
            source_code: String::new(),
            span: miette::SourceSpan::new(0.into(), 0.into()),
        })
    }
    
    /// 应用二元运算符
    fn apply_binary_operator(
        &self,
        left: &Value,
        operator: &str,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        match (left, operator, right) {
            (Value::Integer(a), "+", Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Integer(a), "-", Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Integer(a), "*", Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Integer(a), "/", Value::Integer(b)) => {
                if *b == 0 {
                    Err(InterpreterError::DivisionByZero {
                        position: Position { line: 0, column: 0 },
                        source_code: String::new(),
                        span: miette::SourceSpan::new(0.into(), 0.into()),
                    })
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Value::Integer(a), "==", Value::Integer(b)) => Ok(Value::Boolean(a == b)),
            (Value::Integer(a), "!=", Value::Integer(b)) => Ok(Value::Boolean(a != b)),
            (Value::Integer(a), "<", Value::Integer(b)) => Ok(Value::Boolean(a < b)),
            (Value::Integer(a), ">", Value::Integer(b)) => Ok(Value::Boolean(a > b)),
            (Value::Integer(a), "<=", Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
            (Value::Integer(a), ">=", Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(InterpreterError::RuntimeError {
                position: Position { line: 0, column: 0 },
                message: format!("Unsupported binary operation: {:?} {} {:?}", left, operator, right),
                source_code: String::new(),
                span: miette::SourceSpan::new(0.into(), 0.into()),
            }),
        }
    }
    
    /// 应用一元运算符
    fn apply_unary_operator(
        &self,
        operator: &str,
        operand: &Value,
    ) -> Result<Value, InterpreterError> {
        match (operator, operand) {
            ("-", Value::Integer(n)) => Ok(Value::Integer(-n)),
            ("!", Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            _ => Err(InterpreterError::RuntimeError {
                position: Position { line: 0, column: 0 },
                message: format!("Unsupported unary operation: {} {:?}", operator, operand),
                source_code: String::new(),
                span: miette::SourceSpan::new(0.into(), 0.into()),
            }),
        }
    }
    
    /// 检查值的真假性
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Unit => false,
            _ => true,
        }
    }
    
    /// 检查栈溢出
    fn check_stack_overflow(&self) -> Result<(), InterpreterError> {
        let current_stack_size = self.stack.len() * std::mem::size_of::<StackFrame>();
        if current_stack_size > self.config.stack_size_limit {
            Err(InterpreterError::StackOverflow {
                max_size: self.config.stack_size_limit,
            })
        } else {
            Ok(())
        }
    }
    
    /// 更新内存统计
    fn update_memory_stats(&mut self) {
        let stack_used = self.stack.len() * std::mem::size_of::<StackFrame>();
        self.memory_stats.stack_used = stack_used;
        self.memory_stats.max_stack_used = self.memory_stats.max_stack_used.max(stack_used);
        
        // TODO: 实际的堆内存统计
        // 这里需要与内存分配器集成
    }
}

/// 性能分析器
struct Profiler {
    function_stats: HashMap<String, FunctionStats>,
    total_instructions: u64,
    branch_stats: BranchStats,
}

impl Profiler {
    fn new() -> Self {
        Self {
            function_stats: HashMap::new(),
            total_instructions: 0,
            branch_stats: BranchStats {
                total_branches: 0,
                correct_predictions: 0,
                accuracy: 0.0,
            },
        }
    }
    
    fn record_function_call(&mut self, function_name: &str) {
        let stats = self.function_stats.entry(function_name.to_string())
            .or_insert_with(|| FunctionStats {
                call_count: 0,
                total_time: Duration::ZERO,
                average_time: Duration::ZERO,
            });
        
        stats.call_count += 1;
        // TODO: 记录实际的执行时间
    }
    
    fn get_data(&self) -> ProfilingData {
        ProfilingData {
            function_calls: self.function_stats.clone(),
            total_instructions: self.total_instructions,
            branch_stats: self.branch_stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InterpreterConfig;

    #[tokio::test]
    async fn test_interpreter_creation() {
        let config = InterpreterConfig::default();
        let interpreter = InterpreterEngine::new(config).await;
        assert!(interpreter.is_ok());
    }

    #[tokio::test]
    async fn test_memory_stats() {
        let config = InterpreterConfig::default();
        let interpreter = InterpreterEngine::new(config).await.unwrap();
        
        assert_eq!(interpreter.memory_stats.stack_used, 0);
        assert_eq!(interpreter.memory_stats.heap_used, 0);
    }

    #[test]
    fn test_profiler() {
        let mut profiler = Profiler::new();
        profiler.record_function_call("main");
        profiler.record_function_call("main");
        
        let data = profiler.get_data();
        assert_eq!(data.function_calls.get("main").unwrap().call_count, 2);
    }
}