use std::fmt::Write as _;

/// One curated, executable Clause authoring example.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthoringExampleV1 {
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub source: &'static str,
}

/// A deliberately non-exhaustive authoring vocabulary owned by the compiler.
///
/// Each source is opened through [`crate::ResidentSourceWorkbenchV1`] by the
/// focused authoring-card test. Add a form here only when that complete path
/// accepts it.
pub const AUTHORING_EXAMPLES_V1: &[AuthoringExampleV1] = &[
    AuthoringExampleV1 {
        slug: "transition-reading",
        title: "Declared transition sentences",
        summary: "A reading names a sentence whose body supplies the ordinary checked preconditions and atomic changes. Invoke it as the sole body of an on handler. Each pattern binding occurs once and is used in the body; other bindings are local. Role contracts infer capture types and reject wrong types or unbound effects. Change the body once to change every use. This first slice does not nest readings or combine several sentences in one handler.",
        source: include_str!("../../../test-vectors/authoring/transition-reading.clause"),
    },
    AuthoringExampleV1 {
        slug: "pure-callable",
        title: "Typed pure callables and text interpolation",
        summary: "A named callable gives each argument and result its type, and its body returns one value without mutable output state. `export` exposes its checked signature to generated JavaScript and declarations. Text interpolation checks the same named bindings and pure expressions; effects and type mismatches reject. `compile-js SOURCE.clause OUTPUT.js` opens and checks the source before emitting the module and adjacent declarations.",
        source: include_str!("../../../test-vectors/authoring/pure-callable.clause"),
    },
    AuthoringExampleV1 {
        slug: "pure-composition",
        title: "Typed callable composition",
        summary: "A callable can invoke another checked pure callable in its source scope, including a later definition. Arguments evaluate once before the body; an unused argument may still fail. Conditional branches remain lazy, and private definitions remain private in generated JavaScript.",
        source: include_str!("../../../test-vectors/authoring/pure-composition.clause"),
    },
    AuthoringExampleV1 {
        slug: "callable-outcomes",
        title: "Checked alternative outcomes",
        summary: "An explicit alternative contract such as Execution | Diagnostic accepts either exact value shape. match checks each case binding against its declared alternative, requires every alternative exactly once, and evaluates only the selected body. Missing cases, overlapping alternatives, and invalid payload fields reject. Values retain their ordinary record representation in native invocation and JavaScript; no tag or empty filler fields are needed.",
        source: include_str!("../../../test-vectors/authoring/callable-outcomes.clause"),
    },
    AuthoringExampleV1 {
        slug: "inferred-construction",
        title: "Inferred records and exact foreign construction",
        summary: "A callable may omit its result annotation when its checked body determines one exact type. Nested records infer every field, including delayed foreign values. A private foreign declaration may bind a Record type parameter; each use retains the complete actual record contract, and repeated uses of a parameter must agree. Foreign results, target, member and failure remain explicit. This complete example includes all foreign declarations; it does not assume a shared library or import mechanism.",
        source: include_str!("../../../test-vectors/authoring/foreign-construction-inferred.clause"),
    },
    AuthoringExampleV1 {
        slug: "foreign-cli",
        title: "Ordered arguments and typed foreign procedures",
        summary: "Sequences preserve order and repeated values; named record contracts check each field. Pure dispatch composes a typed message and status. Explicit foreign declarations identify the actual module/member, input/output types and throwing failure contract; procedures permit those accesses. Native invocation without a foreign binding rejects. This executable comparison covers only the existing firn module-add missing-name branch.",
        source: include_str!("../../../test-vectors/authoring/foreign-cli.clause"),
    },
    AuthoringExampleV1 {
        slug: "command-text",
        title: "Typed command arguments and selected Text output",
        summary: "`run-text SOURCE.clause HANDLER SUBJECT ROLE [TEXT ...]` passes each argument as one exact Text value to a checked handler and prints the selected source-owned Text field after admission. Arity, type, execution and projection failures produce no output. Arguments are not split or evaluated as source.",
        source: include_str!("../../../test-vectors/authoring/command-text.clause"),
    },
    AuthoringExampleV1 {
        slug: "ordered-measurements",
        title: "Positioned measurement occurrences",
        summary: "Calibrate each active batch's readings once while preserving its occurrence identities and explicit positions. Equal readings remain independent, including newly appended measurements. An invalid numeric result rejects the whole change.",
        source: include_str!("../../../test-vectors/authoring/ordered-measurements.clause"),
    },
    AuthoringExampleV1 {
        slug: "coherent-declarations",
        title: "Bindings, focus, and structured values",
        summary: "Ordinary conformance premises constrain the same bindings in declarations, laws, and handlers. Shared-edge focus states a common constraint once and keeps the Reading clean; executable direction remains a separate Mode. A role's declared range selects ordinary field edges for values and patterns, with no repeated constructor or field Referents. A four-binding numeric law and typed field replacement share one atomic transition; checked scalar-effect edits retain the live state identities.",
        source: include_str!("../../../test-vectors/authoring/coherent-declarations.clause"),
    },
    AuthoringExampleV1 {
        slug: "derived-capacity",
        title: "Derived structured totals",
        summary: "A law can bind finite sums and derive one optional structured value shared by its consumers. Aggregate queries read completed prerequisite relations, including positive recursive closure. Source changes recompute totals atomically; cycles through aggregate dependencies reject without publishing partial values.",
        source: include_str!("../../../test-vectors/authoring/derived-capacity.clause"),
    },
    AuthoringExampleV1 {
        slug: "optional-derived-formation",
        title: "Reusable optional structured relations",
        summary: "Positive laws may derive a cardinality-maybe value, including a structured value. Queries consume the same current selection, liveness and health definition. Equal proofs share one value; conflicting conclusions reject, and withdrawn premises remove their consequences.",
        source: include_str!("../../../test-vectors/authoring/optional-derived-formation.clause"),
    },
    AuthoringExampleV1 {
        slug: "nested-readiness",
        title: "Checked laws inside checked laws",
        summary: "An acyclic scalar law may call another typed scalar law in its premises. Both execution and feedback consume that definition. Nested laws retain their source origins, and bounded expansion rejects recursion or exhaustion rather than guessing a result.",
        source: include_str!("../../../test-vectors/authoring/nested-readiness.clause"),
    },
    AuthoringExampleV1 {
        slug: "text-search",
        title: "Composable text search",
        summary: "contains-text(text, query) tests exact substring membership, including an empty query. lowercase(text) applies Unicode lowercase mapping, not locale-specific collation or Unicode normalization. Compose them explicitly for case-insensitive search; both inputs remain typed Text.",
        source: include_str!("../../../test-vectors/authoring/text-search.clause"),
    },
    AuthoringExampleV1 {
        slug: "text-selectors",
        title: "Exact text-valued conditions",
        summary: "A focused relation condition matches a Text literal using the same typed row equality as numbers and Booleans. Quoting, Unicode, and escaping have their ordinary Text meaning, including for runtime-created subjects.",
        source: include_str!("../../../test-vectors/authoring/text-selectors.clause"),
    },
    AuthoringExampleV1 {
        slug: "role-contracts",
        title: "Ordinary role contracts",
        summary: "Domain, range, and cardinality facts constrain a binary role. Actual properties establish structural participation without a membership registry. Required properties remain checked across atomic changes.",
        source: include_str!("../../../test-vectors/authoring/role-contracts.clause"),
    },
    AuthoringExampleV1 {
        slug: "recursive-dependencies",
        title: "Recursive dependencies with withdrawal",
        summary: "Positive laws derive blockers through prerequisites. Independent support preserves a conclusion; removing the last root removes its consequences, even through cycles. Bounded closure either completes or returns an error without admitting a prefix.",
        source: include_str!("../../../test-vectors/authoring/recursive-dependencies.clause"),
    },
    AuthoringExampleV1 {
        slug: "text-operations",
        title: "Typed Unicode text operations",
        summary: "trim(text), first-word(text), remaining-words(text), and starts-with(text, prefix) parse bounded UTF-8 text inside checked source. Word boundaries use Unicode whitespace; the remainder preserves internal and trailing whitespace. Empty text yields empty words. Prefix comparison is exact and case-sensitive.",
        source: include_str!("../../../test-vectors/authoring/text-operations.clause"),
    },
    AuthoringExampleV1 {
        slug: "query-laws",
        title: "Reusable checked laws inside finite queries",
        summary: "Query-local scalar laws compose with typed rows, explicit inputs and predicates. Each matching row contributes once even when equal-result law cases overlap. A missing law result excludes that row; an invalid expression or exhausted search remains an error.",
        source: include_str!("../../../test-vectors/authoring/query-laws.clause"),
    },
    AuthoringExampleV1 {
        slug: "scalar-conditional",
        title: "Typed lazy value choice",
        summary: "if(condition, yes, no) requires Bool and two values of the expected type. Only the selected branch executes; both branches are checked. It composes with source laws, query contributions, structured fields and atomic updates.",
        source: include_str!("../../../test-vectors/authoring/scalar-conditional.clause"),
    },
    AuthoringExampleV1 {
        slug: "query-inputs",
        title: "Explicit finite-query inputs",
        summary: "A query's given list passes exact typed values from the enclosing rule. All other query variables remain local. Count matching optional rows to distinguish presence from absence, including runtime-created referents and withdrawal; exhaustion still fails explicitly.",
        source: include_str!("../../../test-vectors/authoring/query-inputs.clause"),
    },
    AuthoringExampleV1 {
        slug: "scalar-equality",
        title: "Checked scalar equality",
        summary: "Equality expressions compare matching Boolean, numeric or Text values and produce Bool. Toggle uses the current admitted pre-state, including runtime-created rows; mixed scalar types are rejected.",
        source: include_str!("../../../test-vectors/authoring/scalar-equality.clause"),
    },
    AuthoringExampleV1 {
        slug: "scalar-comparison",
        title: "Boolean results from scalar comparisons",
        summary: "Ordered F64 comparisons >, >=, < and <= produce Bool values. Numeric arithmetic binds more tightly; all assigned values read the same pre-transition state, so a numeric update and its completion flag can be one atomic rule.",
        source: include_str!("../../../test-vectors/authoring/scalar-comparison.clause"),
    },
    AuthoringExampleV1 {
        slug: "structured-value-copy",
        title: "Atomic structured value copies",
        summary: "Copy a whole typed record between relations while changing other state in the same rule. All fields read one pre-transition state; runtime-created rows use the same rule and incompatible record types are rejected.",
        source: include_str!("../../../test-vectors/authoring/structured-value-copy.clause"),
    },
    AuthoringExampleV1 {
        slug: "scalar-square-root",
        title: "Checked scalar square root",
        summary: "sqrt(expression) computes the finite F64 square root, including zero. It composes with arithmetic and source-law bindings; nonnumeric values are rejected, and negative inputs fail without admitting a changed world.",
        source: include_str!("../../../test-vectors/authoring/scalar-square-root.clause"),
    },
    AuthoringExampleV1 {
        slug: "finite-sums",
        title: "Closed finite query sums",
        summary: "Sums F64 contributions over exact finite row matches in the same pre-state. Query-local variables do not capture the enclosing handler; an empty query yields zero, distinct equal-valued referents contribute independently, and exhausted search is an error.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/finite-sums.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "explicit-semantic-applications",
        title: "Explicit semantic applications",
        summary: "Applies one Shape and two scalar roles to a subject without confusing those applications with denotation or representation.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/explicit-applications.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "scalar-state-transition",
        title: "Scalar state transition",
        summary: "Declares referents and a cardinality-one relation, then replaces one numeric state value atomically.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/ledger/ledger.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "structured-keyboard-transition",
        title: "Structured keyboard transition",
        summary: "Declares structured and Boolean state, binds a physical key, and updates a Vec3 with scalar arithmetic.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/structured-keyboard-transition.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "scalar-input-transition",
        title: "Scalar input transition",
        summary: "Binds one named physical scalar channel to a typed one-argument handler and records its finite observed value.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/scalar-input-transition.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "many-valued-relation",
        title: "Many-valued relation",
        summary: "Retains idempotent values in a cardinality-many relation and requires membership before selecting one.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/supported-many-insertion.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "referent-input-transition",
        title: "Typed occurrence input",
        summary: "Transports an exact projected Item referent to one reusable selection rule; two items of the same class remain distinct and only selected items advance on tick. Retain the projection's generation with the input.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/referent-input-transition.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "selected-account-contributions",
        title: "Independent target selection and explicit contributions",
        summary: "Stores a typed Account input on an independent controller, then sums explicitly declared numeric contributions from eligible occurrences against the same pre-step state. Ordinary overlapping replacements reject atomically; accumulate does not imply source-order execution.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/selected-account-contributions.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "text-state-transition",
        title: "Text state transition",
        summary: "Accepts bounded UTF-8 text as handler input, stores it in optional state, and replaces it atomically.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/text-state-transition.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "multiline-text-output",
        title: "Multiline Text output",
        summary: "Projects an indented multiline Text value while preserving the document's own quotes, layout, and final newline.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/multiline-text-output.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "dynamic-relational-rows",
        title: "Runtime-created Referent and keyed rows",
        summary: "Creates one typed Referent inside a handler, uses it as the key for several relational rows, and retains immutable Text history on redirect.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/dynamic-text-goals.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "created-timed-contributions",
        title: "Finite created relations and per-occurrence contributions",
        summary: "Joins actual runtime-created Goal rows, updates each matching timer, and accumulates each distinct occurrence against one pre-step account balance. Equal-valued creations remain distinct; exact withdrawal removes only its own row. Finite resource exhaustion is an error, never absence. See docs/created-collections.md for bounds and remaining limits.",
        source: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-vectors/authoring/created-timed-contributions.clause")),
    },
    AuthoringExampleV1 {
        slug: "derived-combat-transition",
        title: "Derived combat transition",
        summary: "Authorizes scalar laws, binds their result in a handler, and publishes one atomic multi-state combat change.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/derived-combat-transition.clause"
        )),
    },
    AuthoringExampleV1 {
        slug: "relational-nix-flake",
        title: "Relational Nix flake",
        summary: "Selects the compiler-owned Nix vocabulary and describes a development shell entirely through typed focused relations.",
        source: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../flake.clause")),
    },
    AuthoringExampleV1 {
        slug: "composed-scalar-laws",
        title: "Symbolic relations compose",
        summary: "Defines absolute value with ordinary guarded laws and a symbolic Reading, then composes two uses in one transition. No formula name selects compiler behavior.",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-vectors/authoring/composed-scalar-laws.clause"
        )),
    },
];

/// Render the checked examples as the concise authoring card shipped with this
/// compiler revision.
#[must_use]
pub fn render_authoring_card_v1() -> String {
    let mut card = String::from(
        "# Clause authoring card\n\n\
This card is generated from compiler-owned examples. It is a curated current vocabulary, not an exhaustive language specification. The checked examples and diagnostics from the consuming project's immutable Clause compiler pin are authoritative.\n\n\
Use that pin's workbench directly:\n\n\
- `clause-workbench authoring-card` prints this card.\n\
- `clause-workbench check-source FILE.clause` reads, elaborates, lowers, and opens the source in the resident execution workbench.\n\
- `clause-workbench project-nix FILE.clause [OUTPUT]` checks `using Nix` relations and renders their typed flake projection.\n\n\
Live source tooling offers an explicit checked scalar-effect replacement, not arbitrary text-reload continuity. Use `scalar_effects()` and `edit_scalar_effect()` with the captured generation and exact offered node; settle any pending candidate first. Native and Wasm carry the actual live world internally through the checked operation. Retained explanations describe accepted Steps; finite interventions query an isolated recorded pre-state without applying input or admitting a world. See `docs/live-source-semantics.md` for the compiler/runtime and passive browser contract, bounds, and remaining limits.\n\n",
    );

    for (index, example) in AUTHORING_EXAMPLES_V1.iter().enumerate() {
        writeln!(card, "## {}\n", example.title).expect("writing to a String cannot fail");
        writeln!(card, "{}\n", example.summary).expect("writing to a String cannot fail");
        writeln!(card, "Catalog ID: `{}`\n", example.slug)
            .expect("writing to a String cannot fail");
        card.push_str("```clause\n");
        card.push_str(example.source.trim_end());
        card.push_str("\n```\n");
        if index + 1 < AUTHORING_EXAMPLES_V1.len() {
            card.push('\n');
        }
    }

    card.push_str("\n## Shared foreign declarations and callable definitions\n\nAn explicit `import \"nixpkgs.clause\"` brings checked foreign types, foreign functions, and exported callable definitions into the consumer's scope. File commands resolve the path relative to that consumer. Imports are direct, and duplicate names reject. The resident API accepts the same finite import context as exact source bytes. Ordinary generic helpers specialize and check their bodies with each exact argument record type, retaining strict argument evaluation.\n\n");
    for (path, source) in [
        ("clause:test-vectors/authoring/shared-foreign/nixpkgs.clause", include_str!("../../../test-vectors/authoring/shared-foreign/nixpkgs.clause")),
        ("clause:test-vectors/authoring/shared-foreign/btop.clause", include_str!("../../../test-vectors/authoring/shared-foreign/btop.clause")),
        ("clause:test-vectors/authoring/shared-foreign/jq.clause", include_str!("../../../test-vectors/authoring/shared-foreign/jq.clause")),
    ] {
        writeln!(card, "`{path}`\n\n```clause\n{source}```\n").expect("writing to a String cannot fail");
    }
    card.push_str("\n## Static field paths\n\n`path(details.title)` names fixed field segments. A `?path: FieldPath` parameter is specialized at each call; it never accepts runtime Text or becomes a runtime value. `record-at(?path, value)` constructs the exact nested record and `field-at(record, ?path)` checks every selected field. Delayed foreign declarations may use `get: ?path` for a declared static path while keeping their target, external root, result and failure contracts explicit. These complete module sources state the option path once and select the package independently; the shared module meaning remains Clause source.\n\n");
    for (path, source) in [
        ("clause:test-vectors/authoring/static-modules/nixpkgs.clause", include_str!("../../../test-vectors/authoring/static-modules/nixpkgs.clause")),
        ("clause:test-vectors/authoring/static-modules/btop.clause", include_str!("../../../test-vectors/authoring/static-modules/btop.clause")),
        ("clause:test-vectors/authoring/static-modules/jq.clause", include_str!("../../../test-vectors/authoring/static-modules/jq.clause")),
    ] {
        writeln!(card, "`{path}`\n\n```clause\n{source}```\n").expect("writing to a String cannot fail");
    }
    card.truncate(card.trim_end().len());
    card.push('\n');
    card
}
