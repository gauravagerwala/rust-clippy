# Design of Workflow #7: Deprecate Lint

## Overview

The \"deprecate-lint\" workflow is a development tool in Clippy's \`clippy_dev\` crate that automates the deprecation of an existing lint. It is invoked via \`cargo dev deprecate &lt;lint_name&gt; --reason \"&lt;deprecation reason&gt;\"\`. The process involves parsing current lint declarations, validating the lint, adding it to the deprecated list, removing its active implementation references, cleaning up tests, and regenerating all derived files such as registries, documentation, and test harnesses. This ensures consistency across the codebase and documentation. Manual intervention is required for removing the actual lint logic code and updating any remaining references, followed by running tests.

## Components

- **CLI Entry Point**: \`clippy_dev/src/main.rs\` parses arguments and dispatches to \`deprecate_lint::deprecate\`.
- **ParseCx**: A lightweight parser (\`clippy_dev/src/parse.rs\`) using a token cursor to extract lint metadata from \`declare_clippy_lint!\` macros in \`clippy_lints/src/\` files and from \`deprecated_lints.rs\`.
- **Deprecate Function**: \`clippy_dev/src/deprecate_lint.rs\` orchestrates validation, list updates, file removals/edits, and regeneration.
- **File Removal Logic**: Handles deletion or editing of lint declaration files, removal from lint pass registrations, and deletion of UI test files.
- **Update Lints**: \`clippy_dev/src/update_lints.rs\` regenerates generated content like lint counts, links, module declarations, lint arrays, and test files.
- **Supporting Utilities**: \`FileUpdater\` for safe file modifications, token cursor for parsing without full AST.

## Sequence Diagram

```mermaid
sequenceDiagram
    participant U as User
    participant M as Cargo Dev CLI
    participant P as ParseCx
    participant S as Lint Source Files
    participant D as Deprecated Lints File
    participant G as Generated Files
    participant T as UI Test Files
    U->>+M: Run &quot;cargo dev deprecate &lt;lint_name&gt; --reason &lt;reason&gt;&quot;
    M->>+P: Initialize ParseCx and call deprecate function
    P->>+S: Parse all lint declarations (find_lint_decls)
    S->>-P: Return list of active Lints
    P->>+D: Parse deprecated and renamed lints
    D->>-P: Return lists of DeprecatedLint and RenamedLint
    alt Lint does not exist or already deprecated
        P->>U: Print error and exit
    else Proceed with deprecation
        P->>D: Add new entry to deprecated list (clippy::&lt;name&gt;, reason, version)
        Note right of P: Remove lint from active list
        P->>+S: Invoke remove_lint_declaration on module file
        Note right of S: - Remove/edit declaration in mod file&lt;br/&gt;- Remove lint pass registration&lt;br/&gt;- Delete per-lint impl file if applicable&lt;br/&gt;- Print warning for manual cleanup
        S->>+T: Remove test assets (tests/ui/&lt;lint_name&gt;*)
        T->>-S: Tests removed
        S->>-P: Removal successful
        P->>+G: Call generate_lint_files to update derived files
        Note right of G: - Lint counts in READMEs&lt;br/&gt;- Links in CHANGELOG&lt;br/&gt;- Mod declarations in lib.rs&lt;br/&gt;- LINTS array in declared_lints.rs&lt;br/&gt;- Macro in deprecated_lints.rs&lt;br/&gt;- UI test files for deprecated/renamed
        G->>-P: Files regenerated without the lint, with it deprecated
    end
    P->>-M: Deprecation complete
    M->>-U: Print success message and note to run &quot;cargo uitest&quot;
```

## Detailed Flow

### Parsing Phase
- Scans all \`.rs\` files in \`clippy_lints/src/\` and subdirectories for \`declare_clippy_lint!\` invocations.
- Extracts lint name (lowercased), group, module path, file path, and declaration range.
- Parses \`clippy_lints/src/deprecated_lints.rs\` for existing deprecated and renamed lints using macro patterns.

### Validation
- Checks if the provided lint name exists in active lints and is not already deprecated.
- Prefixes name with &quot;clippy::&quot; for internal representation.

### Deprecation Update
- Inserts new \`DeprecatedLint\` entry into the sorted list.

### Source Cleanup
- Determines the module file path based on lint.module.
- Removes lint from active list.
- Edits or deletes the module file:
  - If dedicated file (module == name), deletes entire file.
  - Otherwise:
    - Deletes separate impl file if in mod.rs.
    - Removes the \`declare_clippy_lint!\` macro invocation using pre-computed range.
    - Removes \`mod &lt;name&gt;;\` declaration.
    - Removes the lint name from \`impl_lint_pass!\` or \`declare_lint_pass!\` macro.
- Warns developer to manually remove any remaining lint implementation code.
- Deletes UI test files: \`tests/ui/&lt;lint_name&gt;.{rs,stderr,fixed}\` or entire directory if exists.

### Regeneration
- Calls \`generate_lint_files\` to update:
  - Lint counts in root and book README.md (rounded to nearest 50).
  - Markdown links in CHANGELOG.md for all lints, deprecated, and renamed.
  - \`deprecated_lints.rs\` macro arrays for DEPRECATED and RENAMED.
  - Per-crate \`src/lib.rs\`: \`pub mod\` declarations between special comments.
  - Per-crate \`src/declared_lints.rs\`: \`LINTS\` static array of lint infos.
  - \`tests/ui/deprecated.rs\`: \`#![warn(clippy::&lt;deprecated_lint&gt;)]\` for each.
  - \`tests/ui/rename.rs\`: \`#![allow(&lt;new_name&gt;) ]\` and \`#![warn(&lt;old_name&gt;)]\` deduped.
- Supports multiple lint crates if present (e.g., internal).

## Manual Steps Post-Deprecation
- Manually edit the module file to remove any lint-specific code (visitor impls, checks, etc.).
- Run \`cargo uitest\` or \`cargo test\` to update any affected test outputs (e.g., bless new stderr).
- Verify no regressions with \`cargo lintcheck\`.
- Update documentation or migration guides if the lint was widely used.
- Commit changes and ensure CI passes.

## Edge Cases
- Lints in shared modules: Partial file edits preserve other lints.
- Already deprecated: Early exit to avoid duplicates.
- Non-existent lint: Error message.
- Test directories: Full recursive deletion for lint-specific UI tests.
- Version tracking: Uses current Clippy version for deprecation metadata.

This design promotes safety by using precise token-based parsing for edits and regeneration to maintain consistency, while leaving complex code removal to humans.