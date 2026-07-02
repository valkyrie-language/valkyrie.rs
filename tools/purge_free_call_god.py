from pathlib import Path

p = Path("projects/nyar-language/src/valkyrie/mir/ssa/expr_lowering.rs")
t = p.read_text(encoding="utf-8")
start = "                let function_ty = self.function_type_of_callee(&callee);"
# Cut until just before ArrayNew arm.
marker = "            HirExprKind::ArrayNew { element_type, length } => {"
i = t.find(start)
j = t.find(marker)
if i < 0 or j < 0 or j < i:
    raise SystemExit(f"markers missing i={i} j={j}")
replacement = '''                let function_ty = self.function_type_of_callee(&callee);
                // ADR 0010: Call is only { callee, arguments }. No intrinsic/dispatch/generic side-channels.
                let value = self.next_value(MirValueOrigin::CallResult);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::Call {
                        callee: callee.clone(),
                        arguments: arguments.clone(),
                    },
                });
                if let Some(ty) = array_index_call_output_type(&callee, &arguments, &self.value_types)
                    .or_else(|| function_ty.map(|func| func.return_type))
                    .or_else(|| resolved.as_ref().map(|call| call.return_type.clone()))
                    .or_else(|| match &callee {
                        MirOperand::Symbol(path) => self
                            .return_types
                            .get(&path.to_string())
                            .cloned()
                            .or_else(|| path.parts().last().and_then(|name| self.return_types.get(name.as_str()).cloned())),
                        _ => None,
                    })
                    .or_else(|| expected_type.cloned())
                {
                    self.value_types.insert(value, ty);
                }
                MirOperand::Value(value)
            }
'''
t = t[:i] + replacement + t[j:]
p.write_text(t, encoding="utf-8", newline="\n")
print("ok")
for k in ["generic_call_facts", "mint_receiver", "receiver_uses_witness", "MirDispatchKind", "intrinsic_opcode"]:
    print(k, t.count(k))
