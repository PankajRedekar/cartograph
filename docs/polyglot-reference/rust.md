# 🦀 Rust Language Reference

- **Tree-Sitter Grammar**: `tree-sitter-rust`
- **Supported Editions**: Rust 2015, 2018, 2021, and 2024.

## AST Symbol Extraction

- **Functions & Methods**: Extracts `fn`, `async fn`, `const fn`, and `extern "C" fn`.
- **Traits & Implementations**: Extracts trait definitions and `impl Trait for Struct` blocks, creating direct `Implements` edges.
- **Enums & Structs**: Captures fields, visibility (`pub`, `pub(crate)`), and derive attributes.
- **Macros**: Extracts `macro_rules!` definitions and invocations.
