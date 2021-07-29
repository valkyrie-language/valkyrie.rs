use crate::{
    backends::ConvertTo,
    functions::externals::ValkyrieExternalFunction,
    modifiers::{AccessType, CurryType, InlineType},
    modules::Hir2Mir,
    types::ValkyrieMetaType,
    values::symbols::AsSymbol,
    ModuleItem, ModuleResolver, ValkyrieResult, ValkyrieSymbol, ValkyrieValue,
};
use indexmap::IndexMap;
use nyar_error::{Result, RuntimeError};
use nyar_wasm::{ExternalType, FunctionType};
use shredder::Gc;
use std::{collections::BTreeMap, sync::Arc};
use valkyrie_ast::{FunctionDeclaration, StatementBlock};

mod codegen;
pub mod externals;
pub mod operators;

pub trait ValkyrieFunctionType {
    fn boxed(self) -> ValkyrieValue;
    fn type_info(&self) -> Gc<ValkyrieMetaType>;
}

/// A function that can be called from Valkyrie.
///
/// ```vk
/// fun f(self, a, <, b, c, >, @a mut **a): Return {
///
/// }
/// ```
///
///
/// ```stack
/// 0: ret pointer
/// 1: env pointer
/// ```
#[derive(Clone, Debug)]
pub struct ValkyrieFunction {
    name: ValkyrieSymbol,
}

impl ValkyrieFunction {
    pub fn new(name: ValkyrieSymbol) -> Self {
        Self { name }
    }
}

/// Actual function definition, non-overloading, non-overriding
///
///
///
/// ## Curry Function
///
/// - All mixed arguments and named arguments must be filled.
/// - Named arguments can be filled in any order.
/// - Named arguments can not be override.
///
///
///
/// ```vk
/// curry add(< , a: int, b: int, >) { a + b }
///
/// add(1)(2)          -> 3
/// add(1)(a: 2)(b: 3) -> Cannot override named a argument
///
/// curry function(self, **sum: Array<int>, a: int, b: int, c:int)
///
/// self.function(0)(0, a: 1)(0, a: 2)(0, a: 3)(b: 4, c: 5)
/// ```
///
/// ## Delay Function
///
/// - All mixed arguments and named arguments must be filled.
/// - Named arguments can be filled in any order.
/// - Named arguments can be override.
#[derive(Clone, Debug)]
pub struct ValkyrieMonomorphicFunction {
    /// Behavior when input arguments are incomplete
    pub curry: CurryType,
    /// Threshold at which to inline this function
    pub inline: InlineType,
    /// Access level under normal circumstances
    pub access: AccessType,
    /// The arguments that must be passed without a name
    pub positional: Vec<ValkyrieValue>,
    /// The arguments that can be passed with a name
    pub mixed: IndexMap<String, ValkyrieValue>,
    /// The arguments that must be passed with a name
    pub named: BTreeMap<String, ValkyrieValue>,
    /// The return type of this function, auto if not specified
    pub return_type: ValkyrieValue,
    /// The effect type of this function, auto if not specified
    pub effect_type: ValkyrieValue,
}

/// Actual function definition, overloading, non-overriding
#[derive(Clone, Debug)]
pub struct ValkyriePartialFunction {
    pub prototype: Arc<ValkyrieFunction>,
    pub generic: Vec<ValkyrieValue>,
    pub positional: Vec<ValkyrieValue>,
    pub named: BTreeMap<String, ValkyrieValue>,
    pub continuation: Option<StatementBlock>,
}

pub struct ValkyrieContinuation {
    pub scope: ValkyrieMetaType,
    pub block: StatementBlock,
}

/// if the code contains loop, it will be compiled into a state machine
#[derive(Clone, Debug)]
pub struct ValkyrieFunctionInstance {
    pub return_type: ValkyrieValue,
}

impl ValkyrieFunction {
    pub fn instance(self: Arc<Self>) -> ValkyriePartialFunction {
        ValkyriePartialFunction {
            prototype: self.clone(),
            generic: vec![],
            positional: vec![],
            named: Default::default(),
            continuation: None,
        }
    }
}

impl ValkyrieMonomorphicFunction {
    pub fn can_invoke(partial: &ValkyriePartialFunction) {
        todo!()
    }
}

impl ValkyriePartialFunction {
    pub fn can_invoke(&self) -> bool {
        todo!()
    }
    pub fn add_continuation(&mut self, continuation: StatementBlock) -> ValkyrieResult<()> {
        match self.continuation {
            Some(_) => Err(RuntimeError::new("continuation already exists"))?,
            None => {
                self.continuation = Some(continuation);
                Ok(())
            }
        }
    }
    pub fn add_positional(&mut self, value: ValkyrieValue) -> ValkyrieResult<()> {
        todo!()
        // match self.positional.len() {
        //     len if len < self.prototype.return_type.positional.len() => {
        //         self.positional.push(value);
        //         Ok(())
        //     }
        //     _ => Err(RuntimeError::new("too many arguments"))?,
        // }
    }
    pub fn add_named(&mut self, name: String, value: ValkyrieValue) -> ValkyrieResult<()> {
        match self.named.get(&name) {
            Some(_) => Err(RuntimeError::new("named argument already exists"))?,
            None => {
                self.named.insert(name, value);
                Ok(())
            }
        }
    }
}
