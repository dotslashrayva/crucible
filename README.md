# Crucible

A C compiler written in Rust targeting x86-64 assembly (Intel syntax) on macOS (Darwin).

> ⚠️ **Note:** This project is currently under construction and supports a limited subset of C.

## Overview
1. **Lexical Analysis** - Tokenizing source code
2. **Parsing** - Building an Abstract Syntax Tree (AST)
3. **IR Generation** - Converting to Three-Address Code intermediate representation
4. **Code Generation** - Translating IR to x86-64 assembly
5. **Code Emission** - Outputting Intel syntax assembly
6. **Assembly & Linking** - Using Clang to produce executables

## Supported Language Features

- Integer constants and arithmetic: `+`, `-`, `*`, `/`, `%`
- Bitwise operators: `&`, `|`, `^`, `<<`, `>>`
- Comparison operators: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical operators: `&&`, `||`, `!` (with short-circuit evaluation)
- Unary operators: `-` (negate), `~` (complement), `!` (logical not)
- Parenthesized expressions
- Single function with `return` statement


### Module Breakdown

- **`token.rs`** - Token definitions for the lexer
- **`lexer.rs`** - Tokenizes source code using regex patterns
- **`parser.rs`** - Recursive descent parser with precedence climbing for expressions
- **`ast.rs`** - Abstract Syntax Tree data structures
- **`intrep.rs`** - Flattens AST into three-address code IR
- **`ir.rs`** - Intermediate representation definitions (three-address code)
- **`codegen.rs`** - Generates assembly from IR with register allocation
- **`asm.rs`** - Assembly-level data structures
- **`emit.rs`** - Outputs Intel syntax x86-64 assembly
- **`main.rs`** - Compiler driver and CLI

## Prerequisites

- **Rust** (latest stable version)
- **clang** (for preprocessing, assembling, and linking)
- **macOS** (currently targets x86-64 Darwin, runs under Rosetta 2)

## Installation

```bash
# Clone the repository
git clone https://github.com/dotslashrayva/crucible
cd crucible

# Build the compiler
cargo build --release
```

## Usage

```bash
# Full compilation
crucible program.c

# Stop after specific stages (useful for debugging)
crucible --lex program.c        # Tokenize only
crucible --parse program.c      # Parse and dump AST
crucible --ir program.c         # Generate and dump IR
crucible --codegen program.c    # Generate and dump assembly IR
crucible -S program.c           # Emit assembly text
```

## Example

Given `example.c`:

```c
int main(void) {
    return (5 + 3) * 2 - 10 / 2;
}
```

```bash
$ crucible example.c
$ ./example
$ echo $? # Should print: 11
```

## Implementation Details

### Short-Circuit Evaluation

Crucible implements proper short-circuit evaluation for logical operators:

- **`&&` (Logical AND)**: If the left operand is false, the right operand is not evaluated
- **`||` (Logical OR)**: If the left operand is true, the right operand is not evaluated

This is achieved through control flow in the IR generation phase using conditional jumps.

### Register Allocation

The code generator uses a simple register allocation scheme:

- **`eax`** - Return values and accumulator
- **`edx`** - Division remainder
- **`r10d`** - Temporary for fixing invalid instruction forms
- **`r11d`** - Temporary for multiply operations

### Stack Management

Variables are allocated on the stack with 4-byte slots. Pseudo-registers from the IR are mapped to stack offsets during code generation.

### Instruction Fixing

The code generator includes several "fix" passes to handle x86-64 constraints:

- **`fix_moves`** - Breaks invalid memory-to-memory moves
- **`fix_binary`** - Fixes binary operations with two memory operands
- **`fix_multiply`** - Handles multiply with stack operands
- **`fix_div_imm`** - Moves immediate values to registers for division

## Limitations

### Current Limitations

- Only supports `int main(void)` functions
- No variable declarations or assignments
- No control flow statements (if, while, for)
- No function calls (besides implicit return)
- No pointers or arrays
- Limited to integer types
- macOS/Darwin target only

### Known Issues

- Error messages could be more descriptive
- No optimization passes
- Limited type system

## **Project Status:** 🚧 Under Active Development

