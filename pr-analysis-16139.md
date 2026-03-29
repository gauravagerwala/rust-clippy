# PR #16139: Workflow Design Impact Analysis

## Affected Workflows

None.

**Justification:** The repository does not contain a `.exp/workflows.json` file or any `.exp/design-workflow-*.md` files defining workflows and their designs. Searches for "workflows.json", ".exp", "design-workflow", and "mermaid" yielded no relevant results. Therefore, there are no defined workflows impacted by this PR.

## General Summary of PR Changes

This pull request introduces a new lint named `needless_type_cast` to Rust Clippy. The lint identifies unnecessary type casts in const, static, or let bindings where the binding is defined with one integer type but consistently cast to another type at all usage sites, without any usage of the original type. It recommends defining the binding directly with the target cast type to improve code clarity and avoid redundant casts.

### Key Changes:
- **New Lint Implementation:** Added `clippy_lints/src/casts/needless_type_cast.rs` containing the core logic for detecting and reporting the lint. The implementation checks bindings with explicit type annotations, skips generic contexts, macro expansions, unsafe blocks, and handles control flow and generic return types.
- **Module Updates:** Modified `clippy_lints/src/casts/mod.rs` to expose the new lint and `clippy_lints/src/declared_lints.rs` to declare it officially.
- **Testing:** Added UI tests in `tests/ui/needless_type_cast.rs`, along with corresponding `.fixed` and `.stderr` files to verify the lint's behavior.
- **Documentation:** Updated `CHANGELOG.md` with an entry for the new lint, and refreshed `README.md` and `book/src/README.md` (via cargo dev update_lints).
- **Other:** Bumped Clippy version, fixed Clippy warnings in the new code, and committed updated changelog.

### Recent Commits Overview:
- Initial lint implementation for bindings defined with one type but always cast to another.
- Refinements to handle only explicit type annotations, skip generics and macros.
- Additional fixes for control flow, unsafe blocks, generic returns.
- Documentation and version updates.

This PR follows Clippy's standard procedure for adding new lints, enhancing the tool's ability to detect and suggest improvements for redundant type casting patterns in Rust code. No structural changes to Clippy's build, testing, or development workflows are introduced.

[PR #16139](https://github.com/rust-lang/rust-clippy/pull/16139)