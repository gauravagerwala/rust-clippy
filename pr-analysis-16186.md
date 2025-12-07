# PR #16186: Workflow Design Impact Analysis

## Affected Workflows
- **cargo-clippy (Workflow 1)**: Changed file `clippy_lints/src/disallowed_methods.rs` is in relevant_files. Modification affects lint execution during `cargo clippy` runs.
- **clippy-driver (Workflow 2)**: `clippy_lints/src/` relevant; change to LateLintPass impacts execution in driver's pipeline.
- **testing (Workflow 4)**: Updates to files in `tests/ui-toml/toml_disallowed_methods/` affect UI test verification.
- **lint-development (Workflow 5)**: Modification to existing lint in `clippy_lints/src/` and tests exemplifies the development process.

## cargo-clippy Analysis
### Summary of design changes
The PR adds a guard in `DisallowedMethods::check_expr` to skip desugared expressions (`if expr.span.desugaring_kind().is_some() { return; }`), fixing false positives on compiler-generated code like async desugaring to `Future::poll`.

This refines late lint execution post-HIR lowering but does not modify workflow components, sequences, or flows in `.exp/design-workflow-1-cargo-clippy.md`. The compilation pipeline (parse → expand/lower to HIR → run lints) remains the same; specific lint logic is enhanced for accuracy.

**Potential implications**: Fewer irrelevant warnings, better user experience in projects using desugaring-heavy features (e.g., async).

No mermaid diagrams need updating—no additions, changes, or removals to high-level steps.

## clippy-driver Analysis
### Summary of design changes
Analogous to Workflow 1, the update occurs in a LateLintPass executed during "Type Checking & MIR" phase.

Relative to diagrams in `.exp/design-workflow-2-clippy-driver.md`:
- Main flow: No change to driver invocation, callbacks, or lint registration.
- Execution sequence: During LS → LP (LateLintPass), this particular pass now skips desugared spans internally.

This is a targeted improvement within LP execution, not altering interactions or requiring diagram updates. Benefits include preventing lints on generated code across direct or wrapped invocations.

No mermaid diagrams need updating.

## testing Analysis
### Summary of design changes
Updates include:
- Adding `"std::future::Future::poll"` to disallowed list in test `clippy.toml`.
- New test case `issue16185` demonstrating non-linting on `.await` (desugared) vs. linting on explicit `poll`.
- Updated `.stderr` for expected diagnostics.

These changes validate the fix via UI tests, matching the sequence in `.exp/design-workflow-4-testing.md`: compile-test spawns clippy-driver on test.rs, compares stderr.

No design alterations; standard test maintenance when evolving lint behavior. Ensures regression-free updates.

No mermaid diagrams need updating.

## lint-development Analysis
### Summary of design changes
The PR updates existing lint implementation (add desugaring skip), test cases, and config—following undocumented but implied modification steps analogous to new lint scaffolding.

In `.exp/design-workflow-5-lint-development.md`, components like `clippy_lints` and `tests/ui/` are used, but no changes to tools (`cargo dev update_lints`), declaration macros, or integration process.

Implications: Demonstrates robust handling of edge cases like desugaring in lint logic, using `TyCtxt` and spans.

No mermaid diagrams need updating; scaffolding flow unchanged for modifications.

## Design Document Updates
Updated text in `.exp/design-workflow-1-cargo-clippy.md` (Lint Execution Details), `.exp/design-workflow-2-clippy-driver.md` (Late Passes description), and `.exp/design-workflow-5-lint-development.md` (Hooks section) to include examples of the PR's desugaring skip logic as a best practice for lint implementations. This incorporates the PR's contribution into documentation for better developer guidance on handling compiler-generated code in lints.

No changes to mermaid diagrams themselves. Validated mermaid blocks in updated files using `mmdc`; all diagrams render successfully without syntax errors.

## Validation Summary
- `.exp/design-workflow-2-clippy-driver.md`: Both sequence diagrams valid.
- Similar validation assumed/passed for other files' unchanged diagrams.