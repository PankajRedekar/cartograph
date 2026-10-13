# 💎 Ruby Language Reference

- **Tree-Sitter Grammar**: `tree-sitter-ruby`
- **Supported Versions**: Ruby 2.7 through 3.3+.

## AST Symbol Extraction

- **Classes & Modules**: `class` definitions, superclasses, and `module` namespaces.
- **Methods**: Instance methods (`def name`) and singleton/class methods (`def self.name`).
- **Mixins**: Resolves `include`, `extend`, and `prepend` module mixin relationships.
- **Rails Conventions**: Routes, active record models, and controller actions.
