# 🐹 Go Language Reference

- **Tree-Sitter Grammar**: `tree-sitter-go`
- **Supported Versions**: Go 1.18 through 1.23+.

## AST Symbol Extraction

- **Functions & Methods**: Top-level `func` and receiver methods `func (r *Receiver) Method()`.
- **Structs & Interfaces**: Type declarations and embedded structs.
- **Implicit Interface Satisfaction**: Evaluates method sets to connect structs to the interfaces they implement.
- **Packages & Imports**: Resolves package-level symbols and internal package imports.
