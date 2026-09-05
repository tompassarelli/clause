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

    card
}
