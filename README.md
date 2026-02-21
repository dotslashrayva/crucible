# Crucible

A C compiler, handwritten in Rust.

Crucible compiles a subset of C down to x86-64 assembly (Intel syntax), performing lexical analysis, parsing, semantic analysis, IR generation, and code generation. No LLVM, no parser generators, no shortcuts.

```
source.c -> Lexer -> Parser -> Resolver -> IR Gen -> CodeGen -> Emitter -> x86-64 assembly
```

## Quick Start

```bash
git clone https://github.com/dotslashrayva/crucible
cd crucible
cargo build --release
```

## What It Compiles

```c
int main(void) {
    int x = 10;
    int y = 3;
    int result = ((x + y) * 2 - x % y) << 1;
    result += x;
    result >>= 1;
    return result != 0 && !(x == y);
}
```

```bash
$ crucible example.c
$ ./example
$ echo $?  # 1
```

Crucible handles:
- Arithmetic: `+` `-` `*` `/` `%`
- Bitwise: `&` `|` `^` `~` `<<` `>>`
- Logical: `&&` `||` `!` with short-circuit evaluation
- Comparison: `==` `!=` `<` `<=` `>` `>=`
- Compound assignment: `+=` `-=` `*=` `/=` `%=` `&=` `|=` `^=` `<<=` `>>=`
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

Each stage is a self-contained transformation with well-defined input/output boundaries. No stage knows about the internals of another. Data flows forward through the pipeline as distinct intermediate representations.

| Stage | Module | Input / Output |
|-------|--------|---------------|
| Lexing | `lexer.rs` | Source -> `Vec<Token>` |
| Parsing | `parser.rs` | Tokens -> AST |
| Semantic Analysis | `resolve.rs` | AST -> AST (validated, variables renamed) |
| IR Generation | `irgen.rs` | AST -> Three-Address Code |
| Code Generation | `codegen.rs` | TAC -> x86-64 instructions |
| Emission | `emit.rs` | Instructions -> Assembly text |

## Architecture

```
src/
├── token.rs      # Token definitions
├── lexer.rs      # Regex-based tokenizer
├── ast.rs        # AST node types
├── parser.rs     # Recursive descent + precedence climbing
├── resolve.rs    # Variable resolution (semantic analysis)
├── ir.rs         # Three-address code definitions
├── irgen.rs      # AST -> TAC lowering
├── asm.rs        # x86-64 instruction types
├── codegen.rs    # Instruction selection + register fixups
├── emit.rs       # Assembly text emission (Intel syntax)
└── main.rs       # Driver
```

## Design

### Parsing Strategy

The parser uses recursive descent for statements and declarations, and switches to **precedence climbing** for expressions. A single `parse_exp(min_prec)` function handles all 17 precedence levels and both associativity directions through a tight loop. No grammar duplication, no per-level functions, and trivially extensible when new operators are added. Assignment is treated as the lowest-precedence right-associative binary operator, which allows `a = b = 5` to parse correctly without special-casing. Compound assignments (`+=`, `-=`, etc.) are **desugared in the parser** into plain assignments (`a += b` becomes `a = a + b`), keeping the AST, IR, and codegen unchanged.

### Semantic Analysis

Variable resolution is implemented as a **dedicated AST-to-AST transformation pass**, decoupled from both the parser and the IR generator. This separation is a deliberate design choice: the parser stays grammar-focused with no symbol table concerns, and downstream passes receive a pre-validated AST where every variable reference is guaranteed to be valid and globally unique.

The resolver enforces three invariants:
1. **No duplicate declarations**: a variable name may only be declared once within a scope
2. **No undeclared references**: every variable use must have a corresponding prior declaration
3. **Valid lvalues**: the left side of an assignment must be an addressable location

Every variable is renamed with a unique identifier during this pass (`x` -> `x.0`, `y` -> `y.1`), which eliminates the possibility of name collisions between user-defined variables and compiler-generated temporaries in all subsequent stages.

### IR Lowering

The AST is flattened into **three-address code**, a linear sequence of instructions where each operation has at most one operator and up to two source operands, writing to a single destination. This representation is chosen because it maps naturally to x86-64 instruction semantics while remaining target-independent.

Compiler-generated temporaries (`tmp.0`, `tmp.1`, ...) are introduced to decompose complex expressions into discrete steps. The namespace separation between resolver-generated names (`x.0`) and IR temporaries (`tmp.0`) is maintained by convention, ensuring no collisions without requiring a global symbol table at this stage.

Short-circuit evaluation for `&&` and `||` is lowered here through **control flow linearization**. Logical operators become sequences of conditional jumps, labels, and copy instructions rather than value-producing binary operations. This correctly models C's evaluation semantics where the right operand may never execute.

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

- [ ] Control flow: `if`/`else`, ternary, `while`, `for`, `do-while`
- [x] Compound assignment: `+=`, `-=`, `*=`, etc.
- [ ] Increment/decrement: `++`, `--`
- [ ] Functions: declarations, calls, parameters
- [ ] Pointers and arrays
- [ ] Multiple translation units
- [ ] ARM64 backend (native Apple Silicon)

## Requirements

- Rust (stable)
- Clang (preprocessing, assembling, linking)
- macOS (targets x86-64 Darwin, runs via Rosetta 2 on Apple Silicon)
