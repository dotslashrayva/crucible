# Crucible

A C compiler, handwritten in Rust.

Crucible compiles a subset of C down to x86-64 assembly (Intel syntax), performing lexical analysis, parsing, semantic analysis, IR generation, and code generation. No LLVM, no parser generators, no shortcuts.

## Quick Start

```bash
git clone https://github.com/dotslashrayva/crucible
cd crucible
cargo build --release
```

## What It Compiles

```c
int main(void) {
    int sum = 0;
    for (int i = 1; i <= 10; i++) {
        if (i % 2 == 0)
            continue;
        sum += i;
    }

    int x = 100;
    while (x > 1) {
        x = x / 2;
        if (x < 10)
            break;
    }

    int factorial = 1;
    int n = 5;
    do {
        factorial *= n;
        n--;
    } while (n > 0);

    int result = ((sum + x) * 2 - factorial % 7) << 1;
    result += x;
    result >>= 1;

    if (result > 20) {
        int bias = 5;
        result = result - bias;
    } else {
        int bias = 10;
        result = result + bias;
    }

    {
        int x = result * 2;
        result = x > 50 ? 1 : 0;
    }

    if (result == 0)
        goto fixup;
    return result;

fixup:
    result = 42;
    return result;
}
```

```bash
$ crucible example.c
$ ./example
$ echo $?
1
```

Crucible handles:
- Arithmetic: `+` `-` `*` `/` `%`
- Bitwise: `&` `|` `^` `~` `<<` `>>`
- Logical: `&&` `||` `!` with short-circuit evaluation
- Comparison: `==` `!=` `<` `<=` `>` `>=`
- Increment/decrement: `++` `--` (prefix and postfix)
- Compound assignment: `+=` `-=` `*=` `/=` `%=` `&=` `|=` `^=` `<<=` `>>=`
- Control flow: `if`/`else`, ternary (`? :`), compound statements (`{ ... }`)
- Loops: `while`, `do`/`while`, `for` with `break` and `continue`
- Labeled statements and `goto`
- Block scoping: nested scopes with variable shadowing (including `for` loop headers)
- Local variables with declarations, assignments, and chained assignment (`a = b = 5`)
- Operator precedence and associativity (17 levels, parsed via precedence climbing)

## Usage

```bash
# Full compilation
crucible program.c

# Stop after specific stages
crucible --lex program.c         # tokens
crucible --parse program.c       # AST
crucible --validate program.c    # AST after semantic analysis
crucible --ir program.c          # three-address code IR
crucible --codegen program.c     # x86-64 instruction selection
crucible -S program.c            # final assembly output
```

## Compilation Pipeline

The compiler is split into a target-independent *Frontend* and a target-dependent *Backend*. Each stage is a self-contained transformation with well-defined input/output boundaries. No stage knows about the internals of another. Data flows forward through the pipeline as distinct intermediate representations.

| Stage | Module | Input / Output |
|-------|--------|---------------|
| Lexing | `frontend/lexer.rs` | Source -> `Vec<Token>` |
| Parsing | `frontend/parser.rs` | Tokens -> AST |
| Semantic Analysis | `frontend/semantic/` | AST -> AST (validated, variables renamed, labels resolved, loops labeled) |
| IR Generation | `frontend/irgen.rs` | AST -> Three-Address Code |
| Code Generation | `backend/codegen.rs` | TAC -> x86-64 instructions |
| Emission | `backend/emit.rs` | Instructions -> Assembly text |

## Architecture

```
src/
├── frontend/             # Target-independent analysis & lowering
│   ├── token.rs          # Token definitions
│   ├── lexer.rs          # Regex-based tokenizer
│   ├── ast.rs            # AST node types
│   ├── parser.rs         # Recursive descent + precedence climbing
│   ├── semantic.rs       # Orchestrates the semantic analysis passes
│   ├── semantic/         # Semantic analysis passes
│   │   ├── variable.rs   # Variable resolution and lvalue checks
│   │   ├── gotos.rs      # Label collection and goto resolution
│   │   └── loops.rs      # Loop labeling for break/continue
│   ├── ir.rs             # Three-address code definitions
│   └── irgen.rs          # AST -> TAC lowering
├── backend/              # Target-dependent x86-64 generation
│   ├── asm.rs            # x86-64 instruction types
│   ├── codegen.rs        # Instruction selection + register fixups
│   └── emit.rs           # Assembly text emission (Intel syntax)
└── main.rs               # Driver and pipeline coordinator
```

## Design

### Parsing Strategy

The parser uses recursive descent for statements and declarations, and switches to **precedence climbing** for expressions. A single `parse_exp(min_prec)` function handles all 17 precedence levels and both associativity directions through a tight loop. No grammar duplication, no per-level functions, and trivially extensible when new operators are added. Assignment and conditional expressions (`? :`) are treated as right-associative special cases within the precedence climber: assignment produces an `Assignment` node, and the ternary produces a `Conditional` node, while all other operators flow through the standard binary path. Compound assignments (`+=`, `-=`, etc.) are **desugared in the parser** into plain assignments (`a += b` becomes `a = a + b`), keeping the AST, IR, and codegen unchanged.

Labeled statements (`label: <statement>`) are disambiguated from expression statements by peeking one token ahead when an identifier is encountered. If the next token is `:`, the parser commits to a labeled statement; otherwise it falls through to expression parsing. `goto` is parsed as a simple keyword-identifier-semicolon production.

### Semantic Analysis

Semantic analysis is structured as **three independent passes** over the AST, each owning a single concern. The passes run in a fixed order from `semantic::analyze()`, and each one walks the AST independently, with no shared mutable state and no interleaved responsibilities. Adding a future pass (e.g. a typechecker, or `switch`/`case` validation) is a new file plus one line in the orchestrator.

**Pass 1: Variable Resolution (`variable.rs`).** Every variable is renamed with a unique identifier (`x` -> `x.0`, `y` -> `y.1`), eliminating name collisions between user-defined variables and compiler-generated temporaries in all subsequent stages.

The pass enforces three invariants:
1. **No duplicate declarations**: a variable name may only be declared once within a single scope
2. **No undeclared references**: every variable use must have a corresponding prior declaration
3. **Valid lvalues**: the left side of an assignment, compound assignment, or `++`/`--` must be an addressable location

Block scoping is implemented with a stack of hash maps. Each entry on the stack is one active scope, with the innermost scope on top. Entering a compound statement or `for` loop header pushes a fresh empty scope; exiting pops it. Declarations insert into the top scope only, so the duplicate check looks only at the top. Variable lookups walk the stack from top to bottom, so inner scopes naturally shadow outer ones. The `for` loop header gets its own scope so a declaration like `for (int i = 0; ...)` is visible throughout the loop header and body but not after it. Lvalue checking is centralized in a single helper so that adding new lvalue forms (pointer dereference, struct field access) in the future is a one-line change.

**Pass 2: Label Resolution (`gotos.rs`).** Labels in C have **function scope**, meaning a `goto` can jump forward to a label that hasn't been seen yet in source order. This forces a two-phase structure:

- **Phase A (collect)**: read-only walk over the function body. Every `Labeled` statement is recorded in a function-wide map with a unique name (e.g., `start` -> `label.start.0`). Duplicate labels are rejected here.
- **Phase B (rewrite)**: mutating walk. Every `Labeled` statement and every `goto` has its name rewritten using the map. Forward gotos resolve correctly because the map is already complete. A `goto` to a name not in the map is reported as an undefined label.

The read-only / read-write split is enforced by Rust's borrow checker: phase A takes `&Block`, phase B takes `&mut Block`, making the two phases visibly distinct at the call site.

**Pass 3: Loop Labeling (`loops.rs`).** Every loop statement (`while`, `do`/`while`, `for`) is assigned a unique ID (e.g., `loop.0`, `loop.1`), and every `break` and `continue` inside the loop is annotated with the ID of its enclosing loop. The current loop label is threaded through the traversal as an `Option<&str>` parameter rather than stored in a stack, so the call stack itself serves as the loop-nesting stack. When `break` or `continue` is encountered with no current loop in scope, the compiler reports an error. This decouples loop validation from both parsing and IR generation: the parser doesn't need to track loop nesting, and the IR generator can unconditionally emit jumps to deterministic label names derived from these IDs.

### IR Lowering

The AST is flattened into **three-address code**, a linear sequence of instructions where each operation has at most one operator and up to two source operands, writing to a single destination. This representation is chosen because it maps naturally to x86-64 instruction semantics while remaining target-independent.

Compiler-generated temporaries (`tmp.0`, `tmp.1`, ...) are introduced to decompose complex expressions into discrete steps. The namespace separation between resolver-generated names (`x.0`) and IR temporaries (`tmp.0`) is maintained by convention, ensuring no collisions without requiring a global symbol table at this stage.

Short-circuit evaluation for `&&` and `||` is lowered here through **control flow linearization**. Logical operators become sequences of conditional jumps, labels, and copy instructions rather than value-producing binary operations. This correctly models C's evaluation semantics where the right operand may never execute. The same mechanism handles `if`/`else` statements (conditional jumps around statement blocks) and ternary expressions (conditional jumps with both branches writing to a shared result variable), keeping the IR uniformly flat. Compound statements are transparent at this level: their block items are simply flattened inline, since scoping has already been resolved by the semantic analysis pass.

**Loop lowering** follows the same linearization pattern. Each loop construct is translated into a canonical sequence of labels and jumps:
- **`while`**: condition check at the top, conditional exit, body, unconditional jump back
- **`do`/`while`**: body first, then condition check with conditional jump back to the top
- **`for`**: init clause, then the `while` pattern with a post-expression inserted between the body and the back-jump

Each loop emits two well-known labels derived from its unique ID: a **continue label** (jump target for `continue`, placed where execution should resume) and a **break label** (jump target for `break`, placed after the loop). `break` and `continue` statements compile to a single `Jump` instruction targeting the appropriate label. When a `for` loop's condition is absent, the conditional exit jump is omitted entirely rather than emitting a trivially-true check, producing tighter IR.

**`goto` and labeled statements** lower trivially: a `goto` becomes a `Jump` to the label's unique name, and a labeled statement becomes a `Label` instruction followed by the inner statement's IR. Since label names were already made unique during semantic analysis, no further work is needed at the IR level.

### Code Generation

Code generation is structured as a **multi-pass pipeline** rather than a single monolithic translation. Each pass has a single responsibility, making the system easier to debug, test, and extend.

**Pass 1: Instruction Selection.** IR instructions are translated to x86-64 assembly using pseudo-registers (virtual operands that haven't been assigned physical locations yet). This pass focuses purely on choosing the right x86-64 instruction forms without worrying about operand constraints.

**Pass 2: Stack Allocation.** Pseudo-registers are lowered to concrete stack slots. Each unique variable gets a 4-byte slot at a fixed offset from `rbp`. The total frame size is **rounded up to 16 bytes** to satisfy the System V AMD64 ABI alignment requirement. This is critical on macOS where the runtime and Rosetta 2 rely on SSE instructions that fault on misaligned stacks.

**Pass 3: Instruction Fixups.** x86-64 has encoding constraints that the instruction selector intentionally ignores for simplicity. Dedicated fix-up passes rewrite illegal instruction forms after the fact:
- **Memory-to-memory moves**: split into move-to-register, move-from-register
- **Binary ops with two stack operands**: source operand routed through a scratch register
- **Multiply targeting a stack location**: detoured through `r11d`
- **Immediate operand in `idiv`**: moved to `r10d` first
- **Immediate first operand in `cmp`**: moved to `r11d` first
- **Shift with non-immediate count**: count moved to `ecx` so the instruction can use `cl`, the only register x86-64 permits as a shift count

This separation means the instruction selector never needs to reason about register constraints, and new fixups can be added independently as the compiler grows.

## Roadmap

- [x] Bitwise operators: `&` `|` `^` `~` `<<` `>>`
- [x] Control flow: `if`/`else`, ternary
- [x] Compound statements and block scoping
- [x] Loops: `while`, `for`, `do`/`while`, `break`, `continue`
- [x] Compound assignment: `+=`, `-=`, `*=`, etc.
- [x] Increment/decrement: `++`, `--`
- [x] Labeled statements and `goto`
- [ ] `switch`, `case`, `default`
- [ ] Functions: declarations, calls, parameters
- [ ] Pointers and arrays
- [ ] Multiple translation units
- [ ] ARM64 backend (native Apple Silicon)

## Requirements

- Rust (stable)
- Clang (preprocessing, assembling, linking)
- macOS (targets x86-64 Darwin, runs via Rosetta 2 on Apple Silicon)
