use std::collections::{BTreeMap, BTreeSet};

use super::*;

pub(super) fn analyze_suspend_points(function: &mut MirFunction) {
    if function.suspend_points.is_empty() {
        return;
    }

    let block_use_def = function.blocks.iter().map(|block| (block.id, collect_block_use_def(block))).collect::<BTreeMap<_, _>>();
    let successors = function.blocks.iter().map(|block| (block.id, collect_successors(block))).collect::<BTreeMap<_, _>>();
    let mut live_in = BTreeMap::<MirBlockRef, BTreeSet<MirValueRef>>::new();
    let mut live_out = BTreeMap::<MirBlockRef, BTreeSet<MirValueRef>>::new();

    let mut changed = true;
    while changed {
        changed = false;
        for block in function.blocks.iter().rev() {
            let successors_live_in = successors.get(&block.id).into_iter().flatten().filter_map(|successor| live_in.get(successor)).fold(
                BTreeSet::new(),
                |mut merged, values| {
                    merged.extend(values.iter().copied());
                    merged
                },
            );

            let (block_use, block_def) = block_use_def.get(&block.id).cloned().unwrap_or_default();
            let mut next_live_in = block_use;
            next_live_in.extend(successors_live_in.iter().filter(|value| !block_def.contains(value)).copied());

            if live_out.get(&block.id) != Some(&successors_live_in) {
                live_out.insert(block.id, successors_live_in);
                changed = true;
            }
            if live_in.get(&block.id) != Some(&next_live_in) {
                live_in.insert(block.id, next_live_in);
                changed = true;
            }
        }
    }

    for suspend_point in &mut function.suspend_points {
        suspend_point.spill_candidates = live_out.get(&suspend_point.suspend_block).cloned().unwrap_or_default().into_iter().collect();
    }
}

fn collect_successors(block: &MirBlock) -> Vec<MirBlockRef> {
    match &block.terminator {
        MirTerminator::Return { .. } | MirTerminator::Unreachable => Vec::new(),
        MirTerminator::Jump { target, .. } => vec![*target],
        MirTerminator::Branch { then_target, else_target, .. } => vec![*then_target, *else_target],
        MirTerminator::PerformEffect { resume_target, .. } => vec![*resume_target],
    }
}

fn collect_block_use_def(block: &MirBlock) -> (BTreeSet<MirValueRef>, BTreeSet<MirValueRef>) {
    let mut block_use = BTreeSet::new();
    let mut block_def = block.parameters.iter().copied().collect::<BTreeSet<_>>();

    for instruction in &block.instructions {
        for used_value in collect_instruction_uses(&instruction.kind) {
            if !block_def.contains(&used_value) {
                block_use.insert(used_value);
            }
        }
        if let Some(output) = instruction.output {
            block_def.insert(output);
        }
    }

    for used_value in collect_terminator_uses(&block.terminator) {
        if !block_def.contains(&used_value) {
            block_use.insert(used_value);
        }
    }

    (block_use, block_def)
}

fn collect_instruction_uses(kind: &MirInstructionKind) -> Vec<MirValueRef> {
    match kind {
        MirInstructionKind::LoadConstant { .. } | MirInstructionKind::LoadSymbol { .. } => Vec::new(),
        MirInstructionKind::Copy { source } => collect_operand_uses(source),
        MirInstructionKind::StoreVar { value, .. } => collect_operand_uses(value),
        MirInstructionKind::Call { callee, arguments, witness, effect, .. } => {
            let mut used_values = collect_operand_uses(callee);
            for argument in arguments {
                used_values.extend(collect_operand_uses(argument));
            }
            if let Some(witness) = witness {
                used_values.extend(collect_operand_uses(witness));
            }
            if let Some(effect) = effect {
                used_values.extend(collect_operand_uses(effect));
            }
            used_values
        }
        MirInstructionKind::StructNew { fields, .. } => fields.iter().flat_map(|(_, value)| collect_operand_uses(value)).collect(),
        MirInstructionKind::FieldGet { object, .. } => collect_operand_uses(object),
        MirInstructionKind::FieldSet { object, value, .. } => {
            let mut used_values = collect_operand_uses(object);
            used_values.extend(collect_operand_uses(value));
            used_values
        }
        MirInstructionKind::PatternMatch { value, .. } => collect_operand_uses(value),
        MirInstructionKind::ArrayNew { length, .. } => collect_operand_uses(length),
        MirInstructionKind::ArrayLiteral { items, .. } => items.iter().flat_map(collect_operand_uses).collect(),
    }
}

fn collect_terminator_uses(terminator: &MirTerminator) -> Vec<MirValueRef> {
    match terminator {
        MirTerminator::Return { value } => value.as_ref().map(collect_operand_uses).unwrap_or_default(),
        MirTerminator::Jump { arguments, .. } => arguments.iter().flat_map(collect_operand_uses).collect(),
        MirTerminator::Branch { condition, .. } => collect_operand_uses(condition),
        MirTerminator::PerformEffect { payload, .. } => payload.as_ref().map(collect_operand_uses).unwrap_or_default(),
        MirTerminator::Unreachable => Vec::new(),
    }
}

fn collect_operand_uses(operand: &MirOperand) -> Vec<MirValueRef> {
    match operand {
        MirOperand::Value(value) => vec![*value],
        MirOperand::Constant(_) | MirOperand::Symbol(_) => Vec::new(),
    }
}
