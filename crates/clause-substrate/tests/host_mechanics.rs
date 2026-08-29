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
use syn::{
    BinOp, Expr, ExprBinary, ExprCall, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprMethodCall,
    ExprTry, ExprWhile, ImplItemFn, ItemFn, Local,
};

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

    let source_rows = source_ast_rows();
    for site in HOST_MECHANIC_SITES {
        let (path, function) = registry_source(site.site);
        let class = registry_class_name(site.class);
        assert!(
            source_rows.iter().any(|row| {
                row.path == path
                    && row.function == function
                    && row.class == class
                    && row.code_targets == site.code_target
            }),
            "registry site does not resolve to an extracted source target: {site:?}"
        );
    }
}

#[test]
fn source_ast_audit_enumerates_every_trusted_control_and_call_site() {
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
    for target in [
        "BTreeMap::get",
        "Sha256::digest",
        "encode_named_signature",
        "decode_subject_value",
    ] {
        assert!(
            actual.lines().any(|line| line.ends_with(target)),
            "trusted source target is absent from the audit: {target}"
        );
    }
}

#[test]
fn source_ast_audit_recognizes_each_control_and_call_form() {
    let rows = audit_source(
        "src/compiler_package_v2/codec.rs",
        r#"
fn sample(
    items: &[u8],
    left: bool,
    right: bool,
    callback: impl Fn(u8) -> Result<(), ()>,
) -> Result<(), ()> {
    for item in items {
        if left && right {
            callback(*item)?;
        }
    }
    while left || right {
        break;
    }
    loop {
        break;
    }
    Ok(())
}
"#,
    );
    for kind in [
        "for",
        "while",
        "loop",
        "logical-and",
        "logical-or",
        "try",
        "direct-call",
        "indirect-call",
    ] {
        assert!(
            rows.iter().any(|row| row.kind == kind),
            "AST audit omitted {kind}"
        );
    }
    assert!(
        rows.iter().any(|row| {
            row.kind == "indirect-call"
                && row.controls == "* item"
                && row.code_targets == "callback"
        }),
        "callback target and controlling argument must both remain visible"
    );
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
    let rows = source_ast_rows();
    let mut output =
        String::from("location\tfunction\tkind\tclass\tcontrols\tfixed_tags\tcode_targets\n");
    for row in rows {
        output.push_str(&row.render());
        output.push('\n');
    }
    output
}

fn source_ast_rows() -> Vec<SourceAstRow> {
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
        rows.extend(audit_source(path, source));
    }
    rows.sort();
    rows
}

fn audit_source(path: &'static str, source: &str) -> Vec<SourceAstRow> {
    let file = syn::parse_file(source).expect("trusted Rust source parses as a syntax tree");
    let mut audit = SourceAstAudit {
        path,
        function: String::new(),
        fixed_callback_parameters: Vec::new(),
        rows: Vec::new(),
    };
    audit.visit_file(&file);
    audit.rows
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SourceAstRow {
    path: &'static str,
    line: usize,
    column: usize,
    function: String,
    kind: &'static str,
    class: &'static str,
    controls: String,
    fixed_tags: &'static str,
    code_targets: String,
}

impl SourceAstRow {
    fn render(&self) -> String {
        format!(
            "{}:{}:{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.path,
            self.line,
            self.column,
            self.function,
            self.kind,
            self.class,
            self.controls,
            self.fixed_tags,
            self.code_targets
        )
    }
}

struct SourceAstAudit {
    path: &'static str,
    function: String,
    fixed_callback_parameters: Vec<String>,
    rows: Vec<SourceAstRow>,
}

impl SourceAstAudit {
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

    fn class(&self, controls: &str, targets: &str) -> (&'static str, &'static str) {
        match self.path {
            "src/compiler_package_v2/codec.rs" => ("WireCodec", "closed-wire-tags"),
            "src/physical/mod.rs" if self.function == "request" => {
                ("PhysicalDispatch", "Sha256OpId")
            }
            "src/physical/mod.rs" => ("CoreABI", "fixed-ABI-shapes"),
            "src/evaluator/mod.rs" if matches!(self.function.as_str(), "new" | "resolve") => {
                ("DefinitionTable", "opaque-Id32-order-hit-miss")
            }
            "src/evaluator/mod.rs" if self.function == "child" => ("CertificateStep", "30..3e"),
            "src/evaluator/mod.rs"
                if self.function == "step"
                    && (controls.contains("bytes")
                        || controls.contains("left == right")
                        || targets == "slice::split_first") =>
            {
                ("ByteMachine", "empty-head-tail-concat-equality")
            }
            "src/evaluator/mod.rs" => ("KernelStep", "KSort-KExpr-value-fuel"),
            _ => unreachable!("all audited sources have one fixed class"),
        }
    }

    fn row(
        &mut self,
        span: proc_macro2::Span,
        kind: &'static str,
        controls: String,
        targets: String,
    ) {
        if !self.trusted() {
            return;
        }
        let controls = one_line(&controls);
        let targets = one_line(&targets);
        let (class, fixed_tags) = self.class(&controls, &targets);
        let start = span.start();
        self.rows.push(SourceAstRow {
            path: self.path,
            line: start.line,
            column: start.column + 1,
            function: self.function.clone(),
            kind,
            class,
            controls,
            fixed_tags,
            code_targets: targets,
        });
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

    fn method_target(&self, call: &ExprMethodCall) -> String {
        let receiver = one_line(&call.receiver.to_token_stream().to_string());
        let method = call.method.to_string();
        match (
            self.path,
            self.function.as_str(),
            receiver.as_str(),
            method.as_str(),
        ) {
            ("src/compiler_package_v2/codec.rs", "decode", "cursor", "frame") => {
                "Cursor::frame".to_owned()
            }
            ("src/evaluator/mod.rs", "resolve", "self . by_id", "get") => {
                "BTreeMap::get".to_owned()
            }
            ("src/evaluator/mod.rs", "step", "self", "child") => "Evaluator::child".to_owned(),
            ("src/evaluator/mod.rs", "child", "self", "step") => "Evaluator::step".to_owned(),
            ("src/evaluator/mod.rs", "step", "bytes", "split_first") => {
                "slice::split_first".to_owned()
            }
            _ => format!("{receiver}.{method}"),
        }
    }

    fn fixed_callback_target(&self, call: &ExprMethodCall) -> Option<String> {
        if self.path != "src/compiler_package_v2/codec.rs" {
            return None;
        }
        let callback = match call.method.to_string().as_str() {
            "sequence" => call.args.last(),
            "frame" if call.args.len() == 2 => call.args.last(),
            _ => None,
        }?;
        Some(compact_path(&callback.to_token_stream().to_string()))
    }
}

impl<'ast> Visit<'ast> for SourceAstAudit {
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

    fn visit_expr_for_loop(&mut self, expression: &'ast ExprForLoop) {
        self.row(
            expression.span(),
            "for",
            expression.expr.to_token_stream().to_string(),
            expression.pat.to_token_stream().to_string(),
        );
        visit::visit_expr_for_loop(self, expression);
    }

    fn visit_expr_while(&mut self, expression: &'ast ExprWhile) {
        self.row(
            expression.span(),
            "while",
            expression.cond.to_token_stream().to_string(),
            "loop-body | exit".to_owned(),
        );
        visit::visit_expr_while(self, expression);
    }

    fn visit_expr_loop(&mut self, expression: &'ast ExprLoop) {
        self.row(
            expression.span(),
            "loop",
            "unconditional".to_owned(),
            "loop-body | break".to_owned(),
        );
        visit::visit_expr_loop(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast ExprBinary) {
        let (kind, target) = match &expression.op {
            BinOp::And(_) => ("logical-and", "right-if-left-true"),
            BinOp::Or(_) => ("logical-or", "right-if-left-false"),
            _ => {
                visit::visit_expr_binary(self, expression);
                return;
            }
        };
        self.row(
            expression.span(),
            kind,
            expression.left.to_token_stream().to_string(),
            target.to_owned(),
        );
        visit::visit_expr_binary(self, expression);
    }

    fn visit_expr_try(&mut self, expression: &'ast ExprTry) {
        self.row(
            expression.span(),
            "try",
            expression.expr.to_token_stream().to_string(),
            "continue | return-error".to_owned(),
        );
        visit::visit_expr_try(self, expression);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        let callback = if let Expr::Path(path) = call.func.as_ref()
            && let Some(identifier) = path.path.get_ident()
            && self
                .fixed_callback_parameters
                .iter()
                .any(|candidate| candidate == &identifier.to_string())
        {
            Some(identifier.to_string())
        } else {
            None
        };
        let target = callback
            .clone()
            .unwrap_or_else(|| compact_path(&call.func.to_token_stream().to_string()));
        let controls = call
            .args
            .iter()
            .map(ToTokens::to_token_stream)
            .map(|tokens| tokens.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        self.row(
            call.span(),
            if callback.is_some() {
                "indirect-call"
            } else {
                "direct-call"
            },
            controls,
            target,
        );
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let controls = std::iter::once(call.receiver.to_token_stream().to_string())
            .chain(
                call.args
                    .iter()
                    .map(ToTokens::to_token_stream)
                    .map(|tokens| tokens.to_string()),
            )
            .collect::<Vec<_>>()
            .join(" | ");
        if let Some(target) = self.fixed_callback_target(call) {
            self.row(
                call.span(),
                "indirect-target",
                self.method_target(call),
                target,
            );
        }
        self.row(
            call.span(),
            "method-call",
            controls,
            self.method_target(call),
        );
        visit::visit_expr_method_call(self, call);
    }
}

fn registry_source(site: &str) -> (&'static str, &str) {
    let path = if site.starts_with("compiler_package_v2::codec::") {
        "src/compiler_package_v2/codec.rs"
    } else if site.starts_with("evaluator::") {
        "src/evaluator/mod.rs"
    } else if site.starts_with("physical::") {
        "src/physical/mod.rs"
    } else {
        panic!("registry site has no audited source: {site}")
    };
    let function = site
        .rsplit_once("::")
        .expect("registry site names a function")
        .1
        .split('/')
        .next()
        .expect("registry function is nonempty");
    (path, function)
}

fn registry_class_name(class: HostMechanicClass) -> &'static str {
    match class {
        HostMechanicClass::WireCodec => "WireCodec",
        HostMechanicClass::CoreAbi => "CoreABI",
        HostMechanicClass::ByteMachine => "ByteMachine",
        HostMechanicClass::DefinitionTable => "DefinitionTable",
        HostMechanicClass::KernelStep => "KernelStep",
        HostMechanicClass::CertificateStep => "CertificateStep",
        HostMechanicClass::PhysicalDispatch => "PhysicalDispatch",
    }
}

fn compact_path(value: &str) -> String {
    one_line(value).replace(" :: ", "::")
}

fn one_line(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\t', " ")
}
