# Theorem Proving

Valkyrie supports formal verification and theorem proving through its powerful type system and proof tactics.

## Overview

Valkyrie's approach to theorem proving combines:
- **Dependent Types**: Types that depend on values
- **Proof Tactics**: Strategies for constructing proofs
- **Automation**: SMT solver integration for automatic proofs

## Propositions as Types

Following the Curry-Howard correspondence, Valkyrie treats propositions as types and proofs as programs:

```valkyrie
# A proposition is a type
type Proposition = Type

# A proof is a value of that type
type Proof⟨P: Proposition⟩ = P

# True proposition has an inhabitant
unite True {
    intro
}

# False proposition has no inhabitants (empty type)
unite False { }

# Logical AND (conjunction)
unite And⟨A, B⟩ {
    intro { left: A, right: B }
}

# Logical OR (disjunction)
unite Or⟨A, B⟩ {
    inl { value: A },
    inr { value: B }
}
```

## Natural Number Proofs

### Peano Arithmetic

```valkyrie
unite Nat {
    zero,
    succ { pred: Nat }
}

# Equality type
unite Equal⟨A, B⟩ {
    refl { }
}

# Proof that n + 0 = n
micro plus_zero_identity(n: Nat) -> Proof⟨Equal⟨Nat.add(n, Nat.zero), n⟩⟩ {
    match n {
        case Nat.zero => Equal.refl
        case Nat.succ { pred } => 
            # Inductive step
            let ih = plus_zero_identity(pred)
            Equal.cong_succ(ih)
    }
}
```

## Proof Tactics

Valkyrie provides built-in tactics for constructing proofs:

```valkyrie
using std::proof::{tactic, auto, simp}

# Automatic proof using SMT
micro auto_proof() -> Proof⟨SomeProposition⟩ {
    auto.solve()
}

# Simplification tactic
micro simplify_proof() -> Proof⟨AnotherProposition⟩ {
    simp.then(auto.solve())
}

# Interactive proof construction
micro interactive_proof() -> Proof⟨ComplexProposition⟩ {
    tactic {
        intro x
        intro y
        apply lemma1
        assumption
    }
}
```

## Induction Principles

### Structural Induction

```valkyrie
# Induction principle for natural numbers
micro nat_induction⟨P: Nat -> Proposition⟩(
    base: Proof⟨P(Nat.zero)⟩,
    step: micro(n: Nat) -> Proof⟨P(n)⟩ -> Proof⟨P(Nat.succ(n))⟩,
    n: Nat
) -> Proof⟨P(n)⟩ {
    match n {
        case Nat.zero => base
        case Nat.succ { pred } =>
            let ih = nat_induction(base, step, pred)
            step(pred)(ih)
    }
}
```

### List Induction

```valkyrie
unite List⟨T⟩ {
    nil,
    cons { head: T, tail: List⟨T⟩ }
}

# Prove that reverse(reverse(xs)) = xs
micro reverse_involutive⟨T⟩(xs: List⟨T⟩) 
    -> Proof⟨Equal⟨List.reverse(List.reverse(xs)), xs⟩⟩ {
    list_induction(xs) {
        base => Equal.refl,
        step => { head, tail, ih =>
            # Use lemmas about reverse and append
            calc {
                List.reverse(List.reverse(cons(head, tail)))
                == List.reverse(cons(head, List.reverse(tail))) by reverse_cons
                == List.append(List.reverse(List.reverse(tail), List.singleton(head))) by reverse_append
                == List.append(tail, List.singleton(head)) by { cong_append_left(ih) }
                == cons(head, tail) by append_singleton
            }
        }
    }
}
```

## Dependent Types

### Length-Indexed Vectors

```valkyrie
unite Vec⟨T, n: Nat⟩ {
    nil,
    cons { head: T, tail: Vec⟨T, n.pred⟩ }
}

# Safe head - type guarantees non-empty vector
micro safe_head⟨T, n: Nat⟩(v: Vec⟨T, Nat.succ(n)⟩) -> T {
    match v {
        case Vec.cons { head, .. } => head
    }
}

# Append preserves length
micro vec_append⟨T, m: Nat, n: Nat⟩(
    v1: Vec⟨T, m⟩,
    v2: Vec⟨T, n⟩
) -> Vec⟨T, Nat.add(m, n)⟩ {
    match v1 {
        case Vec.nil => v2
        case Vec.cons { head, tail } =>
            Vec.cons(head, vec_append(tail, v2))
    }
}
```

## Proof Automation

### SMT Integration

```valkyrie
using std::proof::smt

# Automatically prove arithmetic properties
micro arithmetic_proof() -> Proof⟨∀n: Nat. n + 1 > n⟩ {
    smt.solve()
}

# Verify program correctness
micro verify_sort⟨T: Ordered⟩(input: [T], output: [T]) 
    -> Option⟨Proof⟨IsSorted(output) ∧ IsPermutation(input, output)⟩⟩ {
    
    smt.verify {
        precondition: True
        postcondition: IsSorted(output) ∧ IsPermutation(input, output)
        program: sort_algorithm(input)
    }
}
```

## Refinement Types

```valkyrie
# Type with runtime invariant
type NonZero = { x: i32 | x ≠ 0 }

# Safe division - type system ensures non-zero divisor
micro safe_div(a: i32, b: NonZero) -> i32 {
    a / b.value
}

# Refinement type construction requires proof
micro make_non_zero(x: i32) -> Option⟨NonZero⟩ {
    if x ≠ 0 {
        Some(NonZero { value: x, proof: auto.solve() })
    } else {
        None
    }
}
```

## Program Verification

### Hoare Logic

```valkyrie
using std::verification::hoare

# Verify imperative code with pre/post conditions
micro verified_binary_search(arr: [i32], target: i32) -> Option⟨usize⟩
    requires { IsSorted(arr) }
    ensures { 
        match result {
            Some(idx) => arr[idx] = target,
            None => ∀i. arr[i] ≠ target
        }
    }
{
    var lo = 0
    var hi = arr.length
    
    while lo < hi
        invariant { 
            ∀i. i < lo ⇒ arr[i] < target
            ∧ ∀i. i ≥ hi ⇒ arr[i] > target
        }
    {
        let mid = lo + (hi - lo) / 2
        if arr[mid] = target {
            return Some(mid)
        } else if arr[mid] < target {
            lo = mid + 1
        } else {
            hi = mid
        }
    }
    
    None
}
```

## Best Practices

1. **Start Simple**: Begin with small, tractable proofs
2. **Use Automation**: Let SMT solvers handle routine reasoning
3. **Build Libraries**: Create reusable lemmas and tactics
4. **Document Proofs**: Explain key insights in proof terms
5. **Test Specifications**: Verify that specifications capture intended behavior

## Integration with Development

Valkyrie's proof system integrates seamlessly with regular development:

```valkyrie
# Production code with verified correctness
micro production_sort⟨T: Ordered⟩(data: [T]) -> [T]
    ensures { IsSorted(result) ∧ IsPermutation(data, result) }
{
    # Implementation can use efficient algorithms
    # Correctness is verified at compile time
    quicksort(data)
}
```

This enables building high-assurance software where critical properties are mathematically proven correct.
