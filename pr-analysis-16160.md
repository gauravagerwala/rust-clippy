# PR #16160: Workflow Design Impact Analysis

[PR #16160](https://github.com/rust-lang/rust-clippy/pull/16160)

## Metadata
- **Title:** Fix `empty_enum_variants_with_brackets` misses removing brackets in patterns
- **Description:** Closes #16157. Changelog entry for fixing the lint to handle brackets in patterns.

## Affected Workflows
- **lint-development**: The PR directly modifies the lint implementation in `clippy_lints/src/empty_with_brackets.rs` (adding `check_pat` support, refactoring usage tracking, extending detection for patterns and struct variants) and aligns with the workflow for modifying existing lints, including testing integration. Evidence: Changes to lint pass methods and UI tests in relevant files listed in workflow doc.
- **testing**: Updates UI test files (`tests/ui/empty_enum_variants_with_brackets.rs`, `.fixed`, `.stderr`) to include new cases for patterns and braces, validating the lint fix. Evidence: Test directory updates as part of UI test maintenance.

Other workflows like `cargo-clippy` and `clippy-driver` indirectly benefit from improved lint coverage but their designs (driver invocation, lint registration) remain unchanged.

## lint-development Analysis
### Summary of design changes
The PR fixes and extends the `EmptyWithBrackets` lint:
- Introduces `check_pat` to detect redundant brackets in HIR patterns (e.g., `let Enum::V() = ...`), addressing the reported issue.
- Refactors inline logic into `add_enum_variant` and `update_enum_variant_usage` for cleaner handling of variant definitions and use sites in `FxIndexMap`.
- Updates `check_expr_for_enum_as_function` to handle struct exprs and return spans; adds `check_pat_for_enum_as_function` analogously.
- Removes FIXME for braces support (from prior commit).

These are internal improvements to the lint pass implementation using standard `rustc_lint::LateLintPass` hooks and compiler queries (HIR nodes, typeck results). No new steps added to scaffolding or registration; `update_lints` not invoked as existing lint modified manually. Benefits: Comprehensive coverage for enum variants in both expr and pat contexts, better auto-fixes via spans. Implications: Potential new diagnostics in code using patterns with empty tuple/struct variants.

The documented design (e.g., using visitor methods like `check_expr`) is consistent; `check_pat` is a natural extension not requiring doc updates.

No mermaid diagrams need updates:
- Scaffolding sequence unaffected (for new lints).
- Integration sequence: Lint pass extension now includes pattern checking, but high-level flow (register -> execute passes) unchanged. No additions/changes/removals to show.

## testing Analysis
### Summary of design changes
Updates specific UI test artifacts:
- `.rs`: Adds `issue16157()` test for patterns (`let E::V() = ...`, qualified paths) and `variant_with_braces()` for `{}` syntax.
- `.stderr` and `.fixed`: Adjusted to match new lint messages and fix applications (removing brackets in pats).

This exemplifies the UI test validation process: after lint logic changes, update expected outputs (potentially via `cargo bless`). No modifications to `compile-test.rs`, dogfooding, or other components. Benefits: Ensures the fix works as intended across scenarios, including feature-gated paths. Implications: Tests now cover patterns, improving regression detection.

No mermaid diagrams need updates:
- UI Tests sequence: Spawning driver and comparing outputs proceeds identically.
- Dogfooding sequence: Unaffected, as this is UI-specific.

## Design Document Updates
No updates required to `.exp/design-workflow-*.md` files or mermaid diagrams, as the PR does not alter high-level workflow structures, components, or sequences—only refines a specific lint and its tests within established patterns.

## General PR Summary
The PR enhances the `empty_enum_variants_with_brackets` lint to handle redundant brackets in pattern positions and refines support for braces/tuples, closing a bug report. This improves code quality suggestions in Clippy without impacting core infrastructure.