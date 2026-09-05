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
