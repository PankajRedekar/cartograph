# 🟨 TypeScript & JavaScript Reference

- **Tree-Sitter Grammars**: `tree-sitter-typescript` (TS & TSX), `tree-sitter-javascript`.

## AST Symbol Extraction

- **Modules & Exports**: Handles ESM (`import`/`export`), CommonJS (`require`/`module.exports`), and namespace re-exports.
- **Classes & Interfaces**: Resolves `implements` and `extends` hierarchies.
- **React / JSX / TSX**: Recognizes functional React components, hooks (`useState`, `useEffect`), and JSX tag usages.
- **Type Annotations**: Extracts parameter types and return type annotations.
