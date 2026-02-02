# PR #16144: Workflow Design Impact Analysis

## Affected Workflows
- Workflow 5 (lint-development): Justification - The PR introduces a new pattern for implementing documentation lints by refactoring `doc_link_code` and `doc_paragraphs_missing_punctuation` into state machines that process events from a centralized markdown parser in `check_doc` (in `clippy_lints/src/doc/mod.rs`). This optimizes parsing (once per doc comment) and modularizes lint logic, impacting the documented lint implementation design in workflow 5. Evidence from PR changes to lint files and tests, matching relevant_files like \`clippy_lints/src/\` and \`tests/ui/\`.

No other workflows are affected, as changes are internal to specific lint implementations and do not alter scaffolding, registration, testing execution, or other high-level flows.

## Workflow 5 Analysis
### Summary of design changes
The PR affects the \`clippy_lints\` component, specifically the implementation pattern for documentation lints in \`src/doc/\`. Previously, \`doc_link_code\` logic was inline in \`mod.rs\` with its own event processing via \`check_for_code_clusters\`, and \`doc_paragraphs_missing_punctuation\` performed its own markdown parsing. The PR extracts \`doc_link_code\` to a new file with a \`LinkCode\` state machine struct and refactors the other to a \`MissingPunctuation\` state machine. These now process events from a single parsing in \`check_doc\`, called from the \`Documentation\` late lint pass via \`check_attrs\`.

This changes the design by introducing:
- Centralized event generation to avoid redundant \`pulldown_cmark\` parsing.
- Stateful, event-driven processing for modularity and efficiency.
- Modular files for sub-lint logic within the doc module.

The PR implements this by updating \`mod.rs\` to instantiate the state machines and invoke their \`check\` methods in the event loop of \`check_doc\`, removing old redundant code. Benefits include performance improvement (single parse), easier maintenance, and a reusable pattern for future doc lints. Implications: Developers implementing new doc lints should follow this event-processing state machine pattern; the design doc for workflow 5 has been updated accordingly.

The affected diagram is the \"Integration and Execution Sequence Diagram\", where during \"execute lint passes during compilation phases\", for doc lints, the flow now includes event dispatching to state machines. Below is a diagram highlighting the differences in the doc lint processing sub-flow.

### Diagram showing changes
```mermaid
flowchart TD
    subgraph before[\"Before PR - Redundant Processing (to be removed)\"]
        Parse1[pulldown_cmark Parser in check_for_code_clusters]
        Parse2[pulldown_cmark Parser in doc_paragraphs_missing_punctuation::check]
        Inline[Inline state in mod.rs for doc_link_code]
        style Parse1 fill:#ffcccc,stroke:#ff0000
        style Parse2 fill:#ffcccc,stroke:#ff0000
        style Inline fill:#ffcccc,stroke:#ff0000
    end

    subgraph changed[\"Modified - Central Dispatch (yellow)\"]
        CheckDoc[check_doc function in mod.rs]
        EventLoop[event loop over events]
        Dispatch[dispatches to state machines via .check(event)]
        style CheckDoc fill:#ffff99,stroke:#ffff00
        style EventLoop fill:#ffff99,stroke:#ffff00
        style Dispatch fill:#ffff99,stroke:#ffff00
    end

    subgraph after[\"After PR - Additions (green)\"]
        SingleParse[single pulldown_cmark Parser in check_doc]
        SM1[LinkCode state machine - new file doc_link_code.rs]
        SM2[MissingPunctuation state machine - refactored file]
        Modul[Modular state accumulation across events]
        style SingleParse fill:#ccffcc,stroke:#00ff00
        style SM1 fill:#ccffcc,stroke:#00ff00
        style SM2 fill:#ccffcc,stroke:#00ff00
        style Modul fill:#ccffcc,stroke:#00ff00
    end

    before -.->|replaced by| changed
    changed -.->|enhanced with| after

    classDef removal fill:#ffcccc,stroke:#ff0000,stroke-width:2px
    classDef change fill:#ffff99,stroke:#ffff00,stroke-width:2px
    classDef addition fill:#ccffcc,stroke:#00ff00,stroke-width:2px
```
