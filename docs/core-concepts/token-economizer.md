# 🪙 Token Economizer & AST Degradation

In enterprise repositories with high-degree symbols, returning full function bodies for every caller and callee causes catastrophic context window starvation for AI coding agents.

The **Token Economizer** (`crates/cartograph-engine/src/skills/budget.rs`) enforces strict token ceilings (`--max-tokens <N>`) through a dynamic 4-tier AST degradation algorithm.

## 4-Tier Degradation Algorithm

```
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 1: Target Root Symbol (Guaranteed 100% Fidelity)                  │
│ • Full source code body, inlined parameter types, and docstrings       │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Budget > 75%
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 2: Inlined Type Contracts (100% Retained)                         │
│ • Complete signatures of parameter structs, interfaces, and enums      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Budget < 50%
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 3: Neighbor Signatures (Body Collapsing)                          │
│ • Function bodies collapsed to single-line typed signatures:          │
│   fn verify(token: &str) -> bool /* [42 lines collapsed] */           │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Budget < 20%
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 4: Edge Metadata Compression                                      │
│ • Secondary callers grouped into comma-separated coordinate IDs        │
└────────────────────────────────────────────────────────────────────────┘
```

This guarantees **zero hallucination** and preserves 100% graph topology while slashing token consumption by **80% to 90%**.
