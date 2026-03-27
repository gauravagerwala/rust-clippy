# PR #16123: Workflow Design Impact Analysis

## Affected Workflows
- **cargo-clippy**: The PR adds a new lint to `clippy_lints/src/`, which is a relevant file for this workflow. The new lint will be executed during `cargo clippy` runs, potentially emitting new diagnostics for projects using type aliases with explicit default generic arguments. Evidence: Changed files include `clippy_lints/src/*`.

- **clippy-driver**: Similar to `cargo-clippy`, the new LateLintPass is registered and will run during the compiler pipeline in `clippy-driver` invocations. Evidence: `clippy_lints/src/` is relevant, and registration updates in `lib.rs`.

- **testing**: New UI test files added in `tests/ui/explicit_default_arguments*`, which will be processed by the `compile-test` binary in the UI tests loop. Evidence: Changed files in `tests/ui/`.

- **lint-development**: Primary workflow impacted; the PR implements and integrates a new lint `explicit_default_arguments`, including module creation, declaration, registration in `declared_lints.rs` and `lib.rs`, and UI tests. Evidence: Matches workflow description and relevant files `clippy_lints/src/`, `tests/ui/`, `declared_lints.rs` updated.

- **release-process**: CHANGELOG.md updated with a new entry and link for the lint. Evidence: Changed `CHANGELOG.md`, relevant file for changelog updates during releases.

## cargo-clippy Analysis
### Summary of design changes
No structural changes to components, sequences, or flows. The PR extends the lint set by adding metadata to `declared_lints::LINTS` and a registration call in `register_lint_passes`, integrating the new style lint into existing registration and execution paths. The lint checks for redundant explicit default generic arguments in type alias usages by recursively walking types in HIR items (fns, impls, structs, etc.) using paired Ty and hir::Ty analysis.

How implemented: New `explicit_default_arguments.rs` with `LateLintPass` impl, custom `walk_ty_recursive` function handling various type kinds (Adt, FnPtr, Dynamic, Projection, Opaque, Tuple, etc.), matching generic args to alias defaults.

Benefits: Encourages concise code by linting e.g., `Result<()>` when `type Result<T=()>` has default, reducing clutter. Implications: Minor perf impact from type walking, new lint available in style category.

Mermaid diagrams needing update: The sequence diagram highlights the registration steps affected by additions (no removal/change).

```mermaid
sequenceDiagram
    participant U as User
    participant CC as "cargo-clippy"
    participant C as cargo
    participant CD as "clippy-driver"
    participant RI as rustc_interface
    participant LS as "Lint Store"
    participant CL as "Clippy Lints (clippy_lints)"
    participant Config as "clippy_config"

    U->>CC: cargo clippy [options]
    note right of CC: Parse args, set mode<br/>Set env vars
    CC->>C: cargo check/fix [user args]
    C->>CD: Invoke as rustc wrapper
    CD->>RI: run_compiler with ClippyCallbacks
    RI->>CD: config()
    CD->>Config: load conf
    Config-->>CD: Conf
    note right of CD: Adjust opts
    CD->>LS: register_lints
    LS->>CL: Register LINTS
    note over LS,CL: green addition: + EXPLICIT_DEFAULT_ARGUMENTS_INFO
    CL-->>LS: metadata registered
    LS->>CL: register_lint_passes(conf)
    note over LS,CL: green addition: + ExplicitDefaultArguments pass
    CL-->>LS: passes registered
    note over RI: Pipeline executes lints
    note over RI,CL: green: new lint executes in late phase on types
    alt --fix
        note over CL,RI: yellow: potential future fixes
    else check
        RI->>U: diagnostics (green: + new lint diags)
    end
```

## clippy-driver Analysis
### Summary of design changes
Similar to cargo-clippy; no design alterations. The new lint is added as a LateLintPass in the registration flow, extending the late passes executed during type checking and MIR phases. The custom type walker enables deep analysis of generic args in complex types (trait objects, projections, GATs in impl traits per recent commit).

Benefits: Enables standalone or IDE use of the new lint for direct file linting. Implications: Same as above, integrated into driver for non-Cargo setups.

Mermaid diagrams needing update: Main execution and lint execution flows extended with new pass.

```mermaid
sequenceDiagram
    participant U as User
    participant D as clippy-driver
    participant RD as rustc_driver
    participant C as Config & Callbacks
    participant LS as Lint Store
    participant CP as Compilation Pipeline
    participant L as Lint Passes

    U->>D: clippy-driver file.rs [options]
    D->>D: Parse args
    alt Clippy enabled
        D->>RD: run_compiler with ClippyCallbacks
        RD->>C: config - load conf, opts
        C->>LS: register_lint_passes
        note over C,LS: green: + new late lint pass registration
        RD->>C: psess_created - track deps
        RD->>CP: Run phases
        loop Phases
            CP->>L: Execute passes
            note over CP,L: green: + execution of new ExplicitDefaultArguments
            L->>CP: diagnostics
        end
    else Rustc mode
        D->>RD: standard rustc
    end
    CP->>D: output
    D->>U: diagnostics
```

## testing Analysis
### Summary of design changes
No changes to design; PR adds new test cases ( .rs, .stderr, .fixed, .stdout ) in `tests/ui/explicit_default_arguments` to verify lint triggers, messages, spans, and fixes across various item kinds (fns, structs, impls, etc.). These are processed in the UI tests category loop by `compile-test.rs` using ui_test framework, comparing outputs and potentially blessing.

How implemented: Test files simulate code with explicit default args in type aliases, expected outputs from lint emissions.

Benefits: Ensures the new lint's correctness, covers edge cases like recursive types, GATs, trait objects (with noted limitations). Implications: Expanded test suite, must pass `cargo test` before merge.

Mermaid diagrams needing update: UI tests sequence diagram's loop includes new tests.

```mermaid
sequenceDiagram
    participant U as User/Developer
    participant C as Cargo
    participant CT as Compile-Test
    participant CD as Clippy-Driver
    participant FS as File System
    participant R as Rustc Pipeline

    U->>C: cargo test
    C->>CT: Run compile-test main()
    loop For each UI test category (ui, etc.)
        note over CT: green addition: includes new explicit_default_arguments tests
        CT->>CT: Configure ui_test for dir
        CT->>CD: Spawn clippy-driver on test.rs
        CD->>R: Compile with lints
        R->>CD: Lints emit to stderr
        CD->>CT: Output files
        CT->>FS: Compare .stderr etc.
        alt Mismatch & bless
            CT->>FS: Update expected files
        end
        CT->>CT: Validate
    end
    CT-->>C: pass/fail
```

## lint-development Analysis
### Summary of design changes
No fundamental changes to the workflow design or automation tools (e.g., `cargo dev new_lint`, `update_lints`). The PR manually adds the lint module, declaration via `declare_clippy_lint!`, UI test dir, updates to `lib.rs` (mod and register call), and `declared_lints.rs` (LINTS entry), following the integration pattern. The lint's recursive type walker is a self-contained impl in the pass, using existing `clippy_utils` and rustc APIs, without new utils or macros.

How implemented: `check_item` collects Ty/hir::Ty pairs from items/generics/fn sigs/impl items/trait items, walks recursively matching kinds, checks alias generics for explicit defaults matching resolved.

Benefits: Streamlines lint development for type-related analyses; new lint promotes idiomatic use of type alias defaults. Implications: As draft, needs `cargo dev update_lints` run, full tests, fmt; limitation on bounds in trait objects/impl types noted.

Mermaid diagrams needing update: Integration sequence extended with new lint; scaffolding used manually but diagram shows tool flow (no change needed as optional).

```mermaid
sequenceDiagram
    participant User
    participant "cargo clippy" as CargoClippy
    participant "clippy-driver" as Driver
    participant "clippy_lints" as LintsCrate
    participant "LintStore" as Store
    participant Compiler as Rustc
    User->>CargoClippy: run cargo clippy on project
    CargoClippy->>Driver: invoke driver
    Driver->>LintsCrate: load & call register_lint_passes on store
    note over Driver,LintsCrate: green: registers new ExplicitDefaultArguments
    LintsCrate->>Store: extend early and late passes with lint impls
    note over LintsCrate,Store: green addition: + new ExplicitDefaultArguments late pass
    Driver->>Store: create LintListBuilder, insert declared_lints LINTS, register lints and groups
    note over Driver,Store: green addition: + EXPLICIT_DEFAULT_ARGUMENTS_INFO in LINTS
    Driver->>Compiler: run with registered lints
    Store->>Compiler: execute lint passes during compilation phases
    note over Store,Compiler: green addition: execution includes new lint pass
    Compiler->>User: output diagnostics from lints
    note over Compiler,User: green addition: + diagnostics from explicit_default_arguments if triggered
```

## release-process Analysis
### Summary of design changes
No changes to the release steps or tools. The PR proactively adds a changelog entry `changelog: [`explicit_default_arguments`]: TODO` and link in the lints list, which will be curated/expanded during the "Update CHANGELOG.md" step using `fetch_prs_between.sh` and manual editing for categories like New Lints.

How implemented: Minimal placeholder in CHANGELOG.md for future release notes.

Benefits: Prepares documentation for the new lint's introduction. Implications: Will be included in next release changelog after completion.

Mermaid diagrams needing update: The sequence diagram's changelog update step now incorporates this PR's contribution.

```mermaid
sequenceDiagram
    participant M as Maintainer
    participant CR as ClippyRepo
    participant GH as GitHub
    rect rgb(204,229,255)
    note over M,GH: 5. Update CHANGELOG.md
    M->>+GH: execute fetch_prs_between.sh for PRs
    GH->>-M: provide formatted PR details and links
    note over GH,M: green addition: includes details from #16123 (new lint PR)
    M->>CR: curate content, edit CHANGELOG.md sections
    note over M,CR: green addition: add/expand entry for explicit_default_arguments lint
    note over M,CR: green: new lint mentioned in New Lints category
    M->>CR: commit/push updates
    end
```
