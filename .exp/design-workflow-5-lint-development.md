# High-Level Design of Workflow #5: Lint Development

## Overview

The lint-development workflow enables developers to create, modify, and integrate new lints into Clippy. A lint is a static analysis rule that detects specific patterns or issues in Rust code, providing diagnostics and often suggestions or fixes. This workflow automates much of the boilerplate through tools in the \`clippy_dev\` crate, ensuring consistency in declaration, registration, and testing. Key steps include scaffolding new lint files and tests, implementing the analysis logic using compiler APIs, testing via UI tests and dogfooding, and updating generated registration files via \`cargo dev update_lints\`. This integrates the lint into Clippy's compilation pipeline, making it available via \`cargo clippy\`.

From analysis of source code (e.g., \`clippy_dev/src/new_lint.rs\`, \`clippy_dev/src/update_lints.rs\`, \`clippy_lints/src/lib.rs\`, \`src/driver.rs\`), the workflow leverages Rust's compiler internals for deep analysis, with lints running as passes at early (pre-type-check) or late (post-type-check) stages. The \`declare_clippy_lint!\` macro handles metadata, and generated files ensure automatic inclusion.

## Components

- **clippy_dev**: CLI tools (\`cargo dev\`) including \`new_lint\` for scaffolding and \`update_lints\` for generating registration code from parsed lint declarations.
- **clippy_lints**: Core library housing all lint implementations as structs implementing \`rustc_lint::EarlyLintPass\` or \`LateLintPass\`. Each lint file (\`src/<lint>.rs\` or in submodules like \`methods/\`) contains a \`declare_clippy_lint!\` invocation defining metadata (name, level, category, description, version, location).
- **declare_clippy_lint**: Crate providing the macro for lint declaration (generates \`Lint\` static and \`LintInfo\`) and \`LintListBuilder\` for bulk registration of individual lints and groups (e.g., \`clippy::all\`, \`clippy::correctness\`).
- **clippy_utils**: Shared utilities for common lint tasks (e.g., type queries, def path matching, expression analysis, symbol interning via `sym.rs` for method and identifier matching, MSRV tracking via `msrvs.rs` for version-gated lints).
- **tests/ui/**: UI test infrastructure where each lint has a subdirectory with input \`.rs\` files and expected \`.stderr\` outputs (and \`.fixed\` for fixes) from Clippy runs to verify diagnostics.
- **clippy-driver** (in root \`src/driver.rs\`): Custom compiler driver that loads \`clippy_lints\`, uses generated \`declared_lints::LINTS\` to register lints via \`LintListBuilder\`, calls \`register_lint_passes\` to add passes, and hooks into rustc's pipeline.
- **lintcheck**: Separate tool (\`lintcheck/src/main.rs\`) for running lints on external crates listed in \`.toml\` files to detect regressions or false positives/negatives.
- **clippy_config**: Handles lint configuration from \`clippy.toml\` and attributes, used in passes.

Other aspects: Lints support configuration options, MSRV restrictions via attributes, categories for grouping, and integration with rustfix for auto-fixes.

## Scaffolding Sequence Diagram

```mermaid
sequenceDiagram
    participant Developer
    participant "cargo dev new_lint" as NewLintTool
    participant LintImpl as "clippy_lints/src/lint.rs (or submod)"
    participant "clippy_utils/src/" as Utils
    participant UITests as "tests/ui/lint/"
    participant "update_lints" as UpdateTool
    participant "declared_lints.rs" as DeclaredLints
    participant ModRs as "submod/mod.rs"
    Developer->>NewLintTool: cargo dev new_lint name category [options]
    NewLintTool->>LintImpl: Generate file with declare_clippy_lint! and pass skeleton
    NewLintTool->>UITests: Create initial test dir
    alt submodule case
        NewLintTool->>LintImpl: Place in submod/lint.rs
        NewLintTool->>ModRs: Add pub mod lint; in submod/mod.rs
    end
    NewLintTool->>UpdateTool: Invoke initial update_lints
    UpdateTool->>LintImpl: Parse invocations
    UpdateTool->>DeclaredLints: Update LINTS incl. new
    UpdateTool->> "lib.rs or mod.rs": Update mod lists
    Developer->>LintImpl: Implement logic
    Developer->>Utils: Extend sym, msrvs etc. if needed
    Developer->>UITests: Expand tests
    Developer->>UpdateTool: Re-run update_lints
    UpdateTool->>DeclaredLints: Final update
    Note over UpdateTool: Deprec/rename handling
    Note over Utils: Addition highlighted (green in analysis)
```
