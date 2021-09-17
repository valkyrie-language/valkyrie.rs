use valkyrie_types::Identifier;
use super::*;

mod loop_each;
mod loop_until;
mod loop_while;
mod normal;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoopRepeat {
    pub label: Identifier,
    pub body: Vec<WasiInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoopEach {
    pub label: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoopWhileBody {
    pub label: Identifier,
    pub condition: Vec<WasiInstruction>,
    pub body: Vec<WasiInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoopUntilBody {
    pub label: Identifier,
    pub condition: Vec<WasiInstruction>,
    pub body: Vec<WasiInstruction>,
}
