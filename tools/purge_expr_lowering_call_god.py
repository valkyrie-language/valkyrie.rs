from pathlib import Path

p = Path("projects/nyar-language/src/valkyrie/mir/ssa/expr_lowering.rs")
t = p.read_text(encoding="utf-8")

# Drop God Call minting for method receivers.
start = "                    let mut parameter_types = resolved.as_ref().map(|call| {"
end = "                    let callee = MirOperand::Symbol(callee_symbol);"
i = t.find(start)
j = t.find(end)
if i < 0 or j < 0 or j < i:
    raise SystemExit(f"method block markers missing i={i} j={j}")
replacement = """                    // ADR 0010: no dispatch/witness/evidence/intrinsic/parameter_types on Call.
                    let (callee_symbol, return_type) = match resolved.as_ref() {
                        Some(call) => (call.symbol.clone(), Some(call.return_type.clone())),
                        None => {
                            eprintln!(
                                "[mir] unresolved receiver call; lowering `{}` as diagnostic static symbol (ADR 0008)",
                                method_name.as_str()
                            );
                            (NamePath::new(vec![method_name.clone()]), None)
                        }
                    };
"""
t = t[:i] + replacement + t[j:]

# Neutralize remaining helper references by commenting-out lines (fail compile intentionally if missed).
for dead in (
    "receiver_uses_witness_dispatch",
    "mint_receiver_call_evidence",
    "generic_call_facts",
):
    if dead in t:
        print("still present:", dead, t.count(dead))

p.write_text(t, encoding="utf-8", newline="\n")
print("rewrote method-call minting block")
