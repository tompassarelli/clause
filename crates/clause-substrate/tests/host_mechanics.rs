use std::sync::Arc;

use clause_substrate::artifacts::{ArtifactStore, CompilerPackageArtifact};
use clause_substrate::compiler_package_v2::{
    CompilerEvidence, CompilerInterface, CompilerLineage, CompilerPackage, CompilerSubject,
    CoreManifest, Definition, Id32, KExpr, KSort, Term, encode,
};
use clause_substrate::physical::{HOST_MECHANIC_SITES, HostMechanicClass, host_mechanics_evidence};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprIf, ExprMatch, ImplItemFn, ItemFn, Local};

fn package() -> CompilerPackage {
    let first = Id32([1; 32]);
    let second = Id32([2; 32]);
    CompilerPackage {
        core_manifest: CoreManifest::canonical_v1(),
        subject: CompilerSubject {
            lineage: CompilerLineage::Genesis,
            nominal_declarations: Vec::new(),
            interface: CompilerInterface {
                compile: first,
                admit_propose: second,
            },
            program: vec![
                Definition {
                    id: first,
                    arguments: vec![KSort::Term],
                    result: KSort::Term,
                    body: KExpr::Var(0),
                },
                Definition {
                    id: second,
                    arguments: vec![KSort::Term],
                    result: KSort::Term,
                    body: KExpr::Var(0),
                },
            ],
            build_request: Term::Atom {
                kind: b"opaque".to_vec(),
                canonical_payload: b"opaque".to_vec(),
                equality_contract: b"opaque".to_vec(),
            },
        },
        evidence: CompilerEvidence::Genesis,
    }
}

#[test]
fn typed_host_mechanics_manifest_matches_the_checked_fixture() {
    let expected = include_str!("fixtures/compiler_runtime/host-mechanics.tsv");
    assert_eq!(host_mechanics_evidence(), expected);
    assert_eq!(HOST_MECHANIC_SITES.len(), 8);
    assert_eq!(
        HOST_MECHANIC_SITES
            .iter()
            .filter(|site| site.class == HostMechanicClass::PhysicalDispatch)
            .count(),
        1
    );
    assert!(HOST_MECHANIC_SITES.iter().all(|site| {
        !site.controls.contains("SemanticId")
            && !site.controls.contains("Atom")
            && !site.controls.contains("token")
            && !site.code_target.contains("callback")
            && !site.code_target.contains("plugin")
    }));
}

#[test]
fn source_ast_audit_enumerates_and_classifies_every_trusted_branch_and_indirect_target() {
    let expected = include_str!("fixtures/compiler_runtime/source-ast-mechanics.tsv");
    let actual = source_ast_evidence();
    assert_eq!(actual, expected);
    assert!(
        actual.lines().count() > 30,
        "audit unexpectedly found too few sites"
    );
    for prohibited in [
        "SemanticId",
        "semantic_id",
        "canonical_payload ==",
        "kind ==",
        "token ==",
        "grammar",
        "binding",
        "effect",
        "macro",
        "diagnostic",
        "compiler_revision",
        "plugin",
        "callback-target-from-package",
    ] {
        assert!(
            !actual.contains(prohibited),
            "prohibited semantic control reached the host audit: {prohibited}"
        );
    }
}

#[test]
fn artifacts_are_exact_deduplicated_and_candidate_only() {
    let bytes: Arc<[u8]> = encode(&package()).expect("package encodes").into();
    let mut store = ArtifactStore::new();
    let first = store
        .intern_source(Arc::clone(&bytes))
        .expect("artifact interns");
    let second = store
        .intern_source(Arc::clone(&bytes))
        .expect("artifact deduplicates");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.exact_bytes(), &*bytes);
    assert!(Arc::ptr_eq(&store.get(first.id()).unwrap(), &first));

    let candidate = CompilerPackageArtifact::decode_and_intern(&mut store, bytes)
        .expect("strict candidate decode");
    assert_ne!(candidate.artifact().id(), first.id());
    assert_eq!(
        candidate.candidate().exact_input(),
        candidate.artifact().exact_bytes()
    );
    assert!(matches!(
        candidate.candidate().package().evidence,
        CompilerEvidence::Genesis
    ));
}

fn source_ast_evidence() -> String {
    let sources = [
        (
            "src/compiler_package_v2/codec.rs",
            include_str!("../src/compiler_package_v2/codec.rs"),
        ),
        (
            "src/evaluator/mod.rs",
            include_str!("../src/evaluator/mod.rs"),
        ),
        (
            "src/physical/mod.rs",
            include_str!("../src/physical/mod.rs"),
        ),
    ];
    let mut rows = Vec::new();
    for (path, source) in sources {
        let file = syn::parse_file(source).expect("trusted Rust source parses as a syntax tree");
        let mut audit = SourceAstAudit {
            path,
            function: String::new(),
            fixed_callback_parameters: Vec::new(),
            rows: Vec::new(),
        };
        audit.visit_file(&file);
        rows.extend(audit.rows);
    }
    rows.sort();
    let mut output =
        String::from("location\tfunction\tkind\tclass\tcontrols\tfixed_tags\tcode_targets\n");
    for row in rows {
        output.push_str(&row);
        output.push('\n');
    }
    output
}

struct SourceAstAudit<'a> {
    path: &'a str,
    function: String,
    fixed_callback_parameters: Vec<String>,
    rows: Vec<String>,
}

impl SourceAstAudit<'_> {
    fn with_function(&mut self, name: String, run: impl FnOnce(&mut Self)) {
        let prior_function = std::mem::replace(&mut self.function, name);
        let prior_callbacks = std::mem::take(&mut self.fixed_callback_parameters);
        run(self);
        self.fixed_callback_parameters = prior_callbacks;
        self.function = prior_function;
    }

    fn trusted(&self) -> bool {
        match self.path {
            "src/compiler_package_v2/codec.rs" => true,
            "src/evaluator/mod.rs" => !matches!(self.function.as_str(), "fmt" | "source"),
            "src/physical/mod.rs" => matches!(
                self.function.as_str(),
                "request"
                    | "atom"
                    | "tag"
                    | "bytes"
                    | "id"
                    | "nat64"
                    | "list"
                    | "record"
                    | "value_term"
                    | "observations_term"
            ),
            _ => false,
        }
    }

    fn class(&self, controls: &str) -> (&'static str, &'static str) {
        match self.path {
            "src/compiler_package_v2/codec.rs" => ("WireCodec", "closed-wire-tags"),
            "src/physical/mod.rs" if self.function == "request" => {
                ("PhysicalDispatch", "Sha256OpId")
            }
            "src/physical/mod.rs" => ("CoreABI", "fixed-ABI-shapes"),
            "src/evaluator/mod.rs" if matches!(self.function.as_str(), "new" | "resolve") => {
                ("DefinitionTable", "opaque-Id32-order-hit-miss")
            }
            "src/evaluator/mod.rs"
                if self.function == "step"
                    && (controls.contains("bytes")
                        || controls.contains("left == right")
                        || controls.contains("split_first")) =>
            {
                ("ByteMachine", "empty-head-tail-concat-equality")
            }
            "src/evaluator/mod.rs" => ("KernelStep", "KSort-KExpr-value-fuel"),
            _ => unreachable!("all audited sources have one fixed class"),
        }
    }

    fn row(&mut self, span: proc_macro2::Span, kind: &str, controls: String, targets: String) {
        if !self.trusted() {
            return;
        }
        let controls = one_line(&controls);
        let targets = one_line(&targets);
        let (class, fixed_tags) = self.class(&controls);
        let start = span.start();
        self.rows.push(format!(
            "{}:{}:{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.path,
            start.line,
            start.column + 1,
            self.function,
            kind,
            class,
            controls,
            fixed_tags,
            targets
        ));
    }

    fn remember_fixed_callbacks(&mut self, signature: &syn::Signature) {
        for input in &signature.inputs {
            let syn::FnArg::Typed(argument) = input else {
                continue;
            };
            if !argument.ty.to_token_stream().to_string().contains("Fn") {
                continue;
            }
            if let syn::Pat::Ident(identifier) = argument.pat.as_ref() {
                self.fixed_callback_parameters
                    .push(identifier.ident.to_string());
            }
        }
    }
}

impl<'ast> Visit<'ast> for SourceAstAudit<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.with_function(function.sig.ident.to_string(), |audit| {
            audit.remember_fixed_callbacks(&function.sig);
            visit::visit_item_fn(audit, function);
        });
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.with_function(function.sig.ident.to_string(), |audit| {
            audit.remember_fixed_callbacks(&function.sig);
            visit::visit_impl_item_fn(audit, function);
        });
    }

    fn visit_expr_match(&mut self, expression: &'ast ExprMatch) {
        let controls = expression.expr.to_token_stream().to_string();
        let targets = expression
            .arms
            .iter()
            .map(|arm| arm.pat.to_token_stream().to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        self.row(expression.span(), "match", controls, targets);
        visit::visit_expr_match(self, expression);
    }

    fn visit_expr_if(&mut self, expression: &'ast ExprIf) {
        self.row(
            expression.span(),
            "if",
            expression.cond.to_token_stream().to_string(),
            if expression.else_branch.is_some() {
                "then | else"
            } else {
                "then | continue"
            }
            .to_owned(),
        );
        visit::visit_expr_if(self, expression);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if let Some(initializer) = &local.init
            && initializer.diverge.is_some()
        {
            self.row(
                local.span(),
                "let-else",
                initializer.expr.to_token_stream().to_string(),
                format!("{} | diverge", local.pat.to_token_stream()),
            );
        }
        visit::visit_local(self, local);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref()
            && let Some(identifier) = path.path.get_ident()
            && self
                .fixed_callback_parameters
                .iter()
                .any(|candidate| candidate == &identifier.to_string())
        {
            self.row(
                call.span(),
                "indirect-call",
                identifier.to_string(),
                "monomorphized-source-supplied-wire-function".to_owned(),
            );
        }
        visit::visit_expr_call(self, call);
    }
}

fn one_line(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\t', " ")
}
