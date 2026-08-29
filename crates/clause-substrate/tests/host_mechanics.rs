use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use clause_substrate::artifacts::{ArtifactStore, CompilerPackageArtifact};
use clause_substrate::compiler_package_v2::{
    CompilerEvidence, CompilerInterface, CompilerLineage, CompilerPackage, CompilerSubject,
    CoreManifest, Definition, Id32, KExpr, KSort, Term, encode,
};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    BinOp, Block, Expr, ExprBinary, ExprCall, ExprForLoop, ExprIf, ExprLoop, ExprMatch,
    ExprMethodCall, ExprTry, ExprWhile, FnArg, ImplItem, Item, ItemFn, Pat, ReturnType, Signature,
    Stmt, Type,
};

const ROOTS: &[&str] = &[
    "compiler_package_v2::codec::encode",
    "compiler_package_v2::codec::decode",
    "evaluator::Evaluator::new",
    "evaluator::Evaluator::check_definitions",
    "evaluator::Evaluator::infer_sort",
    "evaluator::Evaluator::evaluate",
    "evaluator::Evaluator::build_certificate",
    "artifacts::CompilerPackageArtifact::decode_and_intern",
];

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
fn typed_reachability_and_information_flow_match_generated_evidence() {
    let analysis = Analysis::build(trusted_sources(), ROOTS).expect("trusted closure is closed");
    assert_eq!(analysis.roots.len(), ROOTS.len());
    assert!(
        analysis.reachable.len() > ROOTS.len(),
        "roots must reach their implementation closure"
    );
    assert!(
        analysis.rows.iter().all(|row| !row.targets.is_empty()),
        "every reachable call and branch has a fixed target or outcome set"
    );
    assert!(
        analysis
            .rows
            .iter()
            .filter(|row| row.kind == "indirect-call")
            .all(|row| row.targets.iter().all(|target| {
                target.starts_with("closure@")
                    || target.starts_with("compiler_package_v2::")
                    || target.starts_with("evaluator::")
                    || target.starts_with("physical::")
                    || target.starts_with("artifacts::")
            })),
        "indirect calls must resolve to finite source functions or literal closures"
    );
    for class in [
        Mechanic::WireCodec,
        Mechanic::CoreAbi,
        Mechanic::ByteMachine,
        Mechanic::DefinitionTable,
        Mechanic::KernelStep,
        Mechanic::CertificateStep,
        Mechanic::PhysicalDispatch,
    ] {
        assert!(
            analysis.rows.iter().any(|row| row.class == class),
            "trusted closure omitted {class:?}"
        );
    }

    assert_or_update_fixture(
        "tests/fixtures/compiler_runtime/host-mechanics.tsv",
        &analysis.summary_evidence(),
    );
    assert_or_update_fixture(
        "tests/fixtures/compiler_runtime/source-ast-mechanics.tsv",
        &analysis.site_evidence(),
    );
}

#[test]
fn analyzer_rejects_dynamic_and_unresolved_callable_authority() {
    let dynamic = [Source::new(
        "probe",
        "probe.rs",
        "fn root(callback: fn(u8)) { callback(0); }",
    )];
    let error = Analysis::build(&dynamic, &["probe::root"]).expect_err("fn pointer must reject");
    assert!(matches!(error, AuditError::DynamicCallableType { .. }));

    let trait_object = [Source::new(
        "probe",
        "probe.rs",
        "trait Select { fn run(&self); } fn root(value: &dyn Select) { value.run(); }",
    )];
    let error =
        Analysis::build(&trait_object, &["probe::root"]).expect_err("trait dispatch must reject");
    assert!(matches!(error, AuditError::DynamicCallableType { .. }));

    let unresolved = [Source::new(
        "probe",
        "probe.rs",
        "fn apply(callback: impl Fn()) { callback(); } fn root() { apply(root); }",
    )];
    let error = Analysis::build(&unresolved, &["probe::apply"])
        .expect_err("a callback root without a closed callsite must reject");
    assert!(matches!(error, AuditError::UnresolvedIndirectCall { .. }));
}

#[test]
fn analyzer_resolves_local_types_at_their_lexical_call_sites() {
    let sources = [Source::new(
        "artifacts",
        "probe.rs",
        r#"
struct Outer {}
impl Outer { fn outer(&self) {} }
struct Inner {}
impl Inner { fn inner(&self) {} }
fn root() {
    let value = Outer {};
    value.outer();
    {
        let value = Inner {};
        value.inner();
    }
    value.outer();
}
"#,
    )];
    let analysis =
        Analysis::build(&sources, &["artifacts::root"]).expect("lexical receiver types resolve");
    let targets: Vec<_> = analysis
        .rows
        .iter()
        .flat_map(|row| row.targets.iter().map(String::as_str))
        .collect();
    assert_eq!(
        targets
            .iter()
            .filter(|target| **target == "artifacts::Outer::outer")
            .count(),
        2
    );
    assert_eq!(
        targets
            .iter()
            .filter(|target| **target == "artifacts::Inner::inner")
            .count(),
        1
    );
}

#[test]
fn changing_a_golden_cannot_classify_a_new_mechanic() {
    let unknown = [Source::new(
        "foreign_semantics",
        "foreign.rs",
        "fn root(input: &[u8]) -> bool { if input.is_empty() { true } else { false } }",
    )];
    let error = Analysis::build(&unknown, &["foreign_semantics::root"])
        .expect_err("unknown domain must not acquire authority from a TSV row");
    assert!(matches!(error, AuditError::UnclassifiedSite { .. }));
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

#[derive(Clone, Copy)]
struct Source<'a> {
    module: &'a str,
    path: &'a str,
    text: &'a str,
}

impl<'a> Source<'a> {
    const fn new(module: &'a str, path: &'a str, text: &'a str) -> Self {
        Self { module, path, text }
    }
}

const TRUSTED_SOURCES: &[Source<'static>] = &[
    Source::new(
        "compiler_package_v2::codec",
        "src/compiler_package_v2/codec.rs",
        include_str!("../src/compiler_package_v2/codec.rs"),
    ),
    Source::new(
        "compiler_package_v2::types",
        "src/compiler_package_v2/types.rs",
        include_str!("../src/compiler_package_v2/types.rs"),
    ),
    Source::new(
        "compiler_package_v2::manifest",
        "src/compiler_package_v2/manifest.rs",
        include_str!("../src/compiler_package_v2/manifest.rs"),
    ),
    Source::new(
        "evaluator",
        "src/evaluator/mod.rs",
        include_str!("../src/evaluator/mod.rs"),
    ),
    Source::new(
        "physical",
        "src/physical/mod.rs",
        include_str!("../src/physical/mod.rs"),
    ),
    Source::new(
        "artifacts",
        "src/artifacts/mod.rs",
        include_str!("../src/artifacts/mod.rs"),
    ),
];

fn trusted_sources() -> &'static [Source<'static>] {
    TRUSTED_SOURCES
}

#[derive(Clone)]
struct Parameter {
    name: String,
    ty: String,
    callable: bool,
}

#[derive(Clone)]
struct Function {
    id: String,
    module: String,
    path: String,
    owner: Option<String>,
    parent: Option<String>,
    name: String,
    signature: Signature,
    parameters: Vec<Parameter>,
    locals: Vec<LocalBinding>,
    body: Block,
}

#[derive(Clone)]
struct LocalBinding {
    name: String,
    ty: String,
    available_line: usize,
    available_column: usize,
    scope_end_line: usize,
    scope_end_column: usize,
    scope_depth: usize,
}

impl Function {
    fn parameter_type(&self, name: &str) -> Option<&str> {
        self.parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .map(|parameter| parameter.ty.as_str())
    }

    fn local_type_at(&self, name: &str, line: usize, column: usize) -> Option<&str> {
        let call = (line, column);
        self.locals
            .iter()
            .filter(|binding| {
                binding.name == name
                    && (binding.available_line, binding.available_column) <= call
                    && call <= (binding.scope_end_line, binding.scope_end_column)
            })
            .max_by_key(|binding| {
                (
                    binding.scope_depth,
                    binding.available_line,
                    binding.available_column,
                )
            })
            .map(|binding| binding.ty.as_str())
    }
}

#[derive(Clone)]
struct RawCall {
    line: usize,
    column: usize,
    callee: CallKind,
    arguments: Vec<Expr>,
    controls: Expr,
}

#[derive(Clone)]
enum CallKind {
    Direct(Expr),
    Method { receiver: Expr, method: String },
    Macro(String),
}

#[derive(Clone)]
struct RawBranch {
    line: usize,
    column: usize,
    kind: &'static str,
    control: Expr,
}

#[derive(Default)]
struct FunctionSyntax {
    calls: Vec<RawCall>,
    branches: Vec<RawBranch>,
}

struct SyntaxVisitor {
    syntax: FunctionSyntax,
}

impl SyntaxVisitor {
    fn call(
        &mut self,
        span: proc_macro2::Span,
        callee: CallKind,
        arguments: Vec<Expr>,
        control: Expr,
    ) {
        let start = span.start();
        self.syntax.calls.push(RawCall {
            line: start.line,
            column: start.column + 1,
            callee,
            arguments,
            controls: control,
        });
    }

    fn branch(&mut self, span: proc_macro2::Span, kind: &'static str, control: Expr) {
        let start = span.start();
        self.syntax.branches.push(RawBranch {
            line: start.line,
            column: start.column + 1,
            kind,
            control,
        });
    }
}

impl<'ast> Visit<'ast> for SyntaxVisitor {
    fn visit_item_fn(&mut self, _function: &'ast ItemFn) {}

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.call(
            call.span(),
            CallKind::Direct((*call.func).clone()),
            call.args.iter().cloned().collect(),
            (*call.func).clone(),
        );
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.call(
            call.span(),
            CallKind::Method {
                receiver: (*call.receiver).clone(),
                method: call.method.to_string(),
            },
            call.args.iter().cloned().collect(),
            (*call.receiver).clone(),
        );
        visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let path = compact_path(&mac.path.to_token_stream().to_string());
        let control = syn::parse_quote! { () };
        self.call(mac.span(), CallKind::Macro(path), Vec::new(), control);
        visit::visit_macro(self, mac);
    }

    fn visit_expr_match(&mut self, expression: &'ast ExprMatch) {
        self.branch(expression.span(), "match", (*expression.expr).clone());
        visit::visit_expr_match(self, expression);
    }

    fn visit_expr_if(&mut self, expression: &'ast ExprIf) {
        self.branch(expression.span(), "if", (*expression.cond).clone());
        visit::visit_expr_if(self, expression);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast ExprForLoop) {
        self.branch(expression.span(), "for", (*expression.expr).clone());
        visit::visit_expr_for_loop(self, expression);
    }

    fn visit_expr_while(&mut self, expression: &'ast ExprWhile) {
        self.branch(expression.span(), "while", (*expression.cond).clone());
        visit::visit_expr_while(self, expression);
    }

    fn visit_expr_loop(&mut self, expression: &'ast ExprLoop) {
        let control = syn::parse_quote! { true };
        self.branch(expression.span(), "loop", control);
        visit::visit_expr_loop(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast ExprBinary) {
        let kind = match expression.op {
            BinOp::And(_) => Some("logical-and"),
            BinOp::Or(_) => Some("logical-or"),
            _ => None,
        };
        if let Some(kind) = kind {
            self.branch(expression.span(), kind, (*expression.left).clone());
        }
        visit::visit_expr_binary(self, expression);
    }

    fn visit_expr_try(&mut self, expression: &'ast ExprTry) {
        self.branch(expression.span(), "try", (*expression.expr).clone());
        visit::visit_expr_try(self, expression);
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Target {
    Local(String),
    External(String),
    Callback(String),
}

impl Target {
    fn render(&self) -> String {
        match self {
            Self::Local(value) | Self::External(value) => value.clone(),
            Self::Callback(value) => format!("callback:{value}"),
        }
    }
}

#[derive(Clone)]
struct ResolvedCall {
    raw: RawCall,
    targets: BTreeSet<Target>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Mechanic {
    WireCodec,
    CoreAbi,
    ByteMachine,
    DefinitionTable,
    KernelStep,
    CertificateStep,
    PhysicalDispatch,
}

impl Mechanic {
    const fn name(self) -> &'static str {
        match self {
            Self::WireCodec => "WireCodec",
            Self::CoreAbi => "CoreABI",
            Self::ByteMachine => "ByteMachine",
            Self::DefinitionTable => "DefinitionTable",
            Self::KernelStep => "KernelStep",
            Self::CertificateStep => "CertificateStep",
            Self::PhysicalDispatch => "PhysicalDispatch",
        }
    }

    const fn outcome(self) -> &'static str {
        match self {
            Self::WireCodec => "canonical-data|fixed-error",
            Self::CoreAbi => "canonical-data|fixed-error",
            Self::ByteMachine => "canonical-data|child-KExpr|fixed-error",
            Self::DefinitionTable => "selected-package-definition|fixed-error",
            Self::KernelStep => "canonical-data|child-KExpr|fixed-error",
            Self::CertificateStep => "canonical-data|fixed-error",
            Self::PhysicalDispatch => "fixed-Sha256-handler|fixed-error",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EvidenceRow {
    path: String,
    line: usize,
    column: usize,
    function: String,
    kind: String,
    class: Mechanic,
    tainted: bool,
    taint_sources: String,
    control_type: String,
    outcome: String,
    targets: Vec<String>,
}

impl EvidenceRow {
    fn render(&self) -> String {
        format!(
            "{}:{}:{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.path,
            self.line,
            self.column,
            self.function,
            self.kind,
            self.class.name(),
            if self.tainted { "package" } else { "fixed" },
            self.taint_sources,
            self.control_type,
            self.outcome,
            self.targets.join("|")
        )
    }
}

#[derive(Debug)]
enum AuditError {
    Parse { path: String, error: String },
    MissingRoot(String),
    DynamicCallableType { function: String, ty: String },
    DynamicCall { function: String, location: String },
    UnresolvedTarget { function: String, method: String },
    UnresolvedIndirectCall { function: String, parameter: String },
    UnclassifiedSite { function: String, location: String },
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { path, error } => write!(formatter, "{path} did not parse: {error}"),
            Self::MissingRoot(root) => write!(formatter, "audit root is absent: {root}"),
            Self::DynamicCallableType { function, ty } => {
                write!(formatter, "{function} exposes dynamic callable type {ty}")
            }
            Self::DynamicCall { function, location } => {
                write!(formatter, "{function} has dynamic call at {location}")
            }
            Self::UnresolvedTarget { function, method } => {
                write!(
                    formatter,
                    "{function} cannot resolve method target {method}"
                )
            }
            Self::UnresolvedIndirectCall {
                function,
                parameter,
            } => write!(
                formatter,
                "{function} callback {parameter} has no closed points-to set"
            ),
            Self::UnclassifiedSite { function, location } => {
                write!(formatter, "{function} has unclassified site at {location}")
            }
        }
    }
}

impl std::error::Error for AuditError {}

struct Program {
    functions: BTreeMap<String, Function>,
    syntax: BTreeMap<String, FunctionSyntax>,
    by_name: BTreeMap<String, BTreeSet<String>>,
    methods: BTreeMap<(String, String), String>,
    struct_fields: BTreeMap<(String, String), String>,
}

impl Program {
    fn parse(sources: &[Source<'_>]) -> Result<Self, AuditError> {
        let mut functions = BTreeMap::new();
        let mut struct_fields = BTreeMap::new();
        for source in sources {
            let file = syn::parse_file(source.text).map_err(|error| AuditError::Parse {
                path: source.path.to_owned(),
                error: error.to_string(),
            })?;
            for item in &file.items {
                match item {
                    Item::Fn(function) => {
                        collect_function(&mut functions, source, None, None, function)
                    }
                    Item::Impl(implementation) => {
                        let owner = nominal_type(&canonical_tokens(&implementation.self_ty));
                        for item in &implementation.items {
                            if let ImplItem::Fn(function) = item {
                                let item = ItemFn {
                                    attrs: function.attrs.clone(),
                                    vis: function.vis.clone(),
                                    sig: function.sig.clone(),
                                    block: Box::new(function.block.clone()),
                                };
                                collect_function(
                                    &mut functions,
                                    source,
                                    Some(owner.clone()),
                                    None,
                                    &item,
                                );
                            }
                        }
                    }
                    Item::Struct(item) => {
                        let owner = item.ident.to_string();
                        for field in &item.fields {
                            if type_is_callable(&field.ty) || type_has_dynamic_callable(&field.ty) {
                                return Err(AuditError::DynamicCallableType {
                                    function: format!("{}::{owner}", source.module),
                                    ty: canonical_tokens(&field.ty),
                                });
                            }
                            if let Some(name) = &field.ident {
                                struct_fields.insert(
                                    (owner.clone(), name.to_string()),
                                    canonical_tokens(&field.ty),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut syntax = BTreeMap::new();
        let mut by_name: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut methods = BTreeMap::new();
        for (id, function) in &functions {
            by_name
                .entry(function.name.clone())
                .or_default()
                .insert(id.clone());
            if let Some(owner) = &function.owner {
                methods.insert((owner.clone(), function.name.clone()), id.clone());
            }
            let mut visitor = SyntaxVisitor {
                syntax: FunctionSyntax::default(),
            };
            visitor.visit_block(&function.body);
            syntax.insert(id.clone(), visitor.syntax);
        }
        Ok(Self {
            functions,
            syntax,
            by_name,
            methods,
            struct_fields,
        })
    }

    fn resolve_call(
        &self,
        function: &Function,
        call: &RawCall,
    ) -> Result<BTreeSet<Target>, AuditError> {
        match &call.callee {
            CallKind::Direct(callee) => self.resolve_direct(function, callee),
            CallKind::Method { receiver, method } => {
                let mut targets = BTreeSet::new();
                if is_self(receiver)
                    && let Some(owner) = &function.owner
                    && let Some(target) = self.methods.get(&(owner.clone(), method.clone()))
                {
                    targets.insert(Target::Local(target.clone()));
                    return Ok(targets);
                }
                let receiver_nominal =
                    self.receiver_nominal(function, receiver, call.line, call.column);
                if let Some(owner) = &receiver_nominal {
                    if let Some(target) = self.methods.get(&(owner.clone(), method.clone())) {
                        targets.insert(Target::Local(target.clone()));
                        return Ok(targets);
                    }
                } else if let Some(candidates) = self.by_name.get(method) {
                    for candidate in candidates {
                        if self.functions[candidate].owner.is_some() {
                            targets.insert(Target::Local(candidate.clone()));
                        }
                    }
                }
                if let Some(external) = external_method_targets(method) {
                    targets.extend(external.into_iter().map(Target::External));
                }
                if targets.is_empty() {
                    return Err(AuditError::UnresolvedTarget {
                        function: function.id.clone(),
                        method: method.clone(),
                    });
                }
                Ok(targets)
            }
            CallKind::Macro(path) => {
                let mut targets = BTreeSet::new();
                targets.insert(Target::External(format!("macro::{path}")));
                Ok(targets)
            }
        }
    }

    fn resolve_direct(
        &self,
        function: &Function,
        callee: &Expr,
    ) -> Result<BTreeSet<Target>, AuditError> {
        let Expr::Path(path) = callee else {
            return Err(AuditError::DynamicCall {
                function: function.id.clone(),
                location: location(function, callee.span()),
            });
        };
        let Some(last) = path.path.segments.last() else {
            return Err(AuditError::DynamicCall {
                function: function.id.clone(),
                location: location(function, callee.span()),
            });
        };
        let name = last.ident.to_string();
        if function
            .parameters
            .iter()
            .any(|parameter| parameter.name == name && parameter.callable)
        {
            let mut targets = BTreeSet::new();
            targets.insert(Target::Callback(name));
            return Ok(targets);
        }

        let path_text = compact_path(&path.path.to_token_stream().to_string());
        let segments: Vec<_> = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        if segments.len() >= 2 {
            let owner = &segments[segments.len() - 2];
            if let Some(target) = self.methods.get(&(owner.clone(), name.clone())) {
                let mut targets = BTreeSet::new();
                targets.insert(Target::Local(target.clone()));
                return Ok(targets);
            }
        }
        if let Some(parent) = &function.parent {
            let candidate = format!("{parent}::{name}");
            if self.functions.contains_key(&candidate) {
                let mut targets = BTreeSet::new();
                targets.insert(Target::Local(candidate));
                return Ok(targets);
            }
        }
        let same_module = format!("{}::{name}", function.module);
        if self.functions.contains_key(&same_module) {
            let mut targets = BTreeSet::new();
            targets.insert(Target::Local(same_module));
            return Ok(targets);
        }
        if let Some(candidates) = self.by_name.get(&name) {
            let free: Vec<_> = candidates
                .iter()
                .filter(|candidate| self.functions[*candidate].owner.is_none())
                .cloned()
                .collect();
            if free.len() == 1 {
                let mut targets = BTreeSet::new();
                targets.insert(Target::Local(free[0].clone()));
                return Ok(targets);
            }
        }
        let mut targets = BTreeSet::new();
        targets.insert(Target::External(format!("static::{path_text}")));
        Ok(targets)
    }

    fn receiver_nominal(
        &self,
        function: &Function,
        expression: &Expr,
        line: usize,
        column: usize,
    ) -> Option<String> {
        match expression {
            Expr::Path(path) => {
                let name = path.path.get_ident()?.to_string();
                if name == "self" {
                    return function.owner.clone();
                }
                function
                    .local_type_at(&name, line, column)
                    .map(ToOwned::to_owned)
                    .or_else(|| function.parameter_type(&name).map(nominal_type))
            }
            Expr::Field(field) => {
                let owner = self.receiver_nominal(function, &field.base, line, column)?;
                let member = canonical_tokens(&field.member);
                self.struct_fields
                    .get(&(owner, member))
                    .map(|ty| nominal_type(ty))
            }
            Expr::Reference(reference) => {
                self.receiver_nominal(function, &reference.expr, line, column)
            }
            Expr::Paren(paren) => self.receiver_nominal(function, &paren.expr, line, column),
            Expr::Group(group) => self.receiver_nominal(function, &group.expr, line, column),
            _ => None,
        }
    }
}

fn external_method_targets(method: &str) -> Option<BTreeSet<String>> {
    let names: &[&str] = match method {
        "and_then" => &[
            "std::option::Option::and_then",
            "std::result::Result::and_then",
        ],
        "as_bytes" => &["str::as_bytes"],
        "as_slice" => &["std::vec::Vec::as_slice"],
        "binary_search_by_key" => &["slice::binary_search_by_key"],
        "checked_add" => &["usize::checked_add", "u64::checked_add"],
        "checked_mul" => &["usize::checked_mul", "u64::checked_mul"],
        "checked_sub" => &["usize::checked_sub", "u64::checked_sub"],
        "contains" => &["core::ops::RangeInclusive::contains", "slice::contains"],
        "copied" => &["std::option::Option::copied", "std::iter::Iterator::copied"],
        "enumerate" => &["std::iter::Iterator::enumerate"],
        "expect" => &["std::option::Option::expect", "std::result::Result::expect"],
        "expect_err" => &["std::result::Result::expect_err"],
        "extend" => &["std::iter::Extend::extend"],
        "extend_from_slice" => &["std::vec::Vec::extend_from_slice"],
        "finalize" => &["sha2::Digest::finalize"],
        "get" => &["slice::get"],
        "insert" => &["std::vec::Vec::insert"],
        "into" => &["std::convert::Into::into"],
        "into_iter" => &["std::iter::IntoIterator::into_iter"],
        "is_empty" => &["slice::is_empty", "std::vec::Vec::is_empty"],
        "iter" => &["slice::iter"],
        "len" => &["slice::len", "str::len", "std::vec::Vec::len"],
        "map" => &[
            "std::iter::Iterator::map",
            "std::option::Option::map",
            "std::result::Result::map",
        ],
        "map_err" => &["std::result::Result::map_err"],
        "ok" => &["std::result::Result::ok"],
        "ok_or" => &["std::option::Option::ok_or"],
        "ok_or_else" => &["std::option::Option::ok_or_else"],
        "pop" => &["std::vec::Vec::pop"],
        "push" => &["std::vec::Vec::push"],
        "remove" => &["std::vec::Vec::remove"],
        "rev" => &["std::iter::DoubleEndedIterator::rev"],
        "reverse" => &["slice::reverse"],
        "sort" => &["slice::sort"],
        "then_some" => &["bool::then_some"],
        "to_be_bytes" => &["u32::to_be_bytes", "u64::to_be_bytes"],
        "try_into" => &["std::convert::TryInto::try_into"],
        "try_reserve" => &["std::vec::Vec::try_reserve"],
        "try_reserve_exact" => &["std::vec::Vec::try_reserve_exact"],
        "update" => &["sha2::Digest::update"],
        "windows" => &["slice::windows"],
        "zip" => &["std::iter::Iterator::zip"],
        _ => return None,
    };
    Some(names.iter().map(|name| (*name).to_owned()).collect())
}

fn collect_function(
    functions: &mut BTreeMap<String, Function>,
    source: &Source<'_>,
    owner: Option<String>,
    parent: Option<String>,
    function: &ItemFn,
) {
    let name = function.sig.ident.to_string();
    let id = if let Some(parent) = &parent {
        format!("{parent}::{name}")
    } else if let Some(owner) = &owner {
        format!("{}::{owner}::{name}", source.module)
    } else {
        format!("{}::{name}", source.module)
    };
    let parameters = parameters(&function.sig);
    let locals = local_types(&function.block, owner.as_deref());
    functions.insert(
        id.clone(),
        Function {
            id: id.clone(),
            module: source.module.to_owned(),
            path: source.path.to_owned(),
            owner,
            parent: parent.clone(),
            name,
            signature: function.sig.clone(),
            parameters,
            locals,
            body: (*function.block).clone(),
        },
    );
    collect_nested_functions(functions, source, &id, &function.block);
}

fn collect_nested_functions(
    functions: &mut BTreeMap<String, Function>,
    source: &Source<'_>,
    parent: &str,
    block: &Block,
) {
    for statement in &block.stmts {
        if let Stmt::Item(Item::Fn(function)) = statement {
            collect_function(functions, source, None, Some(parent.to_owned()), function);
        }
    }
}

fn parameters(signature: &Signature) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for input in &signature.inputs {
        match input {
            FnArg::Receiver(_) => parameters.push(Parameter {
                name: "self".to_owned(),
                ty: "Self".to_owned(),
                callable: false,
            }),
            FnArg::Typed(argument) => {
                if let Pat::Ident(identifier) = argument.pat.as_ref() {
                    let ty = canonical_tokens(&argument.ty);
                    parameters.push(Parameter {
                        name: identifier.ident.to_string(),
                        callable: type_is_callable(&argument.ty),
                        ty,
                    });
                }
            }
        }
    }
    parameters
}

fn local_types(block: &Block, owner: Option<&str>) -> Vec<LocalBinding> {
    struct LocalTypes<'a> {
        owner: Option<&'a str>,
        values: Vec<LocalBinding>,
        scopes: Vec<(usize, usize)>,
    }
    impl Visit<'_> for LocalTypes<'_> {
        fn visit_item_fn(&mut self, _function: &ItemFn) {}

        fn visit_block(&mut self, block: &Block) {
            let end = block.span().end();
            self.scopes.push((end.line, end.column + 1));
            visit::visit_block(self, block);
            self.scopes.pop();
        }

        fn visit_local(&mut self, local: &syn::Local) {
            let (identifier, annotated) = match &local.pat {
                Pat::Ident(identifier) => (identifier, None),
                Pat::Type(typed) => {
                    let Pat::Ident(identifier) = typed.pat.as_ref() else {
                        visit::visit_local(self, local);
                        return;
                    };
                    (identifier, Some(typed.ty.as_ref()))
                }
                _ => {
                    visit::visit_local(self, local);
                    return;
                }
            };
            let ty = annotated.map(canonical_tokens).or_else(|| {
                local
                    .init
                    .as_ref()
                    .and_then(|initializer| initializer_nominal(&initializer.expr, self.owner))
            });
            if let Some(ty) = ty {
                let available = local.span().end();
                let (scope_end_line, scope_end_column) = self
                    .scopes
                    .last()
                    .copied()
                    .expect("local belongs to a block");
                self.values.push(LocalBinding {
                    name: identifier.ident.to_string(),
                    ty,
                    available_line: available.line,
                    available_column: available.column + 1,
                    scope_end_line,
                    scope_end_column,
                    scope_depth: self.scopes.len(),
                });
            }
            visit::visit_local(self, local);
        }
    }

    fn initializer_nominal(expression: &Expr, owner: Option<&str>) -> Option<String> {
        match expression {
            Expr::Try(value) => initializer_nominal(&value.expr, owner),
            Expr::Await(value) => initializer_nominal(&value.base, owner),
            Expr::Paren(value) => initializer_nominal(&value.expr, owner),
            Expr::Group(value) => initializer_nominal(&value.expr, owner),
            Expr::Reference(value) => initializer_nominal(&value.expr, owner),
            Expr::Call(value) => {
                let Expr::Path(path) = value.func.as_ref() else {
                    return None;
                };
                let segments: Vec<_> = path
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect();
                if segments.len() >= 2 {
                    let candidate = &segments[segments.len() - 2];
                    if candidate == "Self" {
                        owner.map(ToOwned::to_owned)
                    } else {
                        Some(candidate.clone())
                    }
                } else {
                    None
                }
            }
            Expr::Struct(value) => {
                let candidate = value.path.segments.last()?.ident.to_string();
                if candidate == "Self" {
                    owner.map(ToOwned::to_owned)
                } else {
                    Some(candidate)
                }
            }
            Expr::Macro(value) if value.mac.path.is_ident("vec") => Some("Vec".to_owned()),
            Expr::MethodCall(_) => None,
            _ => None,
        }
    }

    let mut visitor = LocalTypes {
        owner,
        values: Vec::new(),
        scopes: Vec::new(),
    };
    visitor.visit_block(block);
    visitor.values
}

fn reject_dynamic_signature(function: &Function) -> Result<(), AuditError> {
    for parameter in &function.parameters {
        let typed = function
            .signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(argument) => Some(argument),
                FnArg::Receiver(_) => None,
            })
            .find(|argument| matches!(argument.pat.as_ref(), Pat::Ident(identifier) if identifier.ident == parameter.name));
        if let Some(argument) = typed
            && type_has_dynamic_callable(&argument.ty)
        {
            return Err(AuditError::DynamicCallableType {
                function: function.id.clone(),
                ty: parameter.ty.clone(),
            });
        }
    }
    if let ReturnType::Type(_, ty) = &function.signature.output
        && (type_is_callable(ty) || type_has_dynamic_callable(ty))
    {
        return Err(AuditError::DynamicCallableType {
            function: function.id.clone(),
            ty: canonical_tokens(ty),
        });
    }
    struct StoredCallable {
        found: Option<String>,
    }
    impl Visit<'_> for StoredCallable {
        fn visit_item_fn(&mut self, _function: &ItemFn) {}

        fn visit_local(&mut self, local: &syn::Local) {
            if let Pat::Type(pattern) = &local.pat
                && (type_is_callable(&pattern.ty) || type_has_dynamic_callable(&pattern.ty))
            {
                self.found = Some(canonical_tokens(&pattern.ty));
                return;
            }
            if let Some(initializer) = &local.init
                && matches!(initializer.expr.as_ref(), Expr::Closure(_))
            {
                self.found = Some("stored closure".to_owned());
                return;
            }
            visit::visit_local(self, local);
        }
    }
    let mut stored = StoredCallable { found: None };
    stored.visit_block(&function.body);
    if let Some(ty) = stored.found {
        return Err(AuditError::DynamicCallableType {
            function: function.id.clone(),
            ty,
        });
    }
    Ok(())
}

fn type_is_callable(ty: &Type) -> bool {
    match ty {
        Type::ImplTrait(value) => value
            .bounds
            .iter()
            .any(|bound| canonical_tokens(bound).contains("Fn")),
        Type::TraitObject(value) => value
            .bounds
            .iter()
            .any(|bound| canonical_tokens(bound).contains("Fn")),
        Type::BareFn(_) => true,
        _ => false,
    }
}

fn type_has_dynamic_callable(ty: &Type) -> bool {
    struct DynamicType {
        found: bool,
    }
    impl Visit<'_> for DynamicType {
        fn visit_type_bare_fn(&mut self, _ty: &syn::TypeBareFn) {
            self.found = true;
        }

        fn visit_type_trait_object(&mut self, _ty: &syn::TypeTraitObject) {
            self.found = true;
        }
    }
    let mut dynamic = DynamicType { found: false };
    dynamic.visit_type(ty);
    dynamic.found
}

#[derive(Debug)]
struct Analysis {
    roots: BTreeSet<String>,
    reachable: BTreeSet<String>,
    rows: Vec<EvidenceRow>,
}

impl Analysis {
    fn build(sources: &[Source<'_>], roots: &[&str]) -> Result<Self, AuditError> {
        let program = Program::parse(sources)?;
        let mut root_set = BTreeSet::new();
        let mut reachable = BTreeSet::new();
        let mut queue = VecDeque::new();
        for root in roots {
            if !program.functions.contains_key(*root) {
                return Err(AuditError::MissingRoot((*root).to_owned()));
            }
            root_set.insert((*root).to_owned());
            reachable.insert((*root).to_owned());
            queue.push_back((*root).to_owned());
        }

        let mut calls: BTreeMap<String, Vec<ResolvedCall>> = BTreeMap::new();
        while let Some(function_id) = queue.pop_front() {
            let function = &program.functions[&function_id];
            reject_dynamic_signature(function)?;
            let mut resolved = Vec::new();
            for raw in &program.syntax[&function_id].calls {
                let targets = program.resolve_call(function, raw)?;
                for target in &targets {
                    if let Target::Local(target) = target
                        && reachable.insert(target.clone())
                    {
                        queue.push_back(target.clone());
                    }
                }
                resolved.push(ResolvedCall {
                    raw: raw.clone(),
                    targets,
                });
            }
            calls.insert(function_id, resolved);
        }

        let mut callback_points: BTreeMap<(String, String), BTreeSet<Target>> = BTreeMap::new();
        loop {
            let mut changed = false;
            let current: Vec<_> = reachable.iter().cloned().collect();
            for function_id in current {
                let function = &program.functions[&function_id];
                for call in calls.get(&function_id).into_iter().flatten() {
                    for target in &call.targets {
                        let Target::Local(target_id) = target else {
                            continue;
                        };
                        let target_function = &program.functions[target_id];
                        for (index, parameter) in target_function
                            .parameters
                            .iter()
                            .filter(|parameter| parameter.name != "self")
                            .enumerate()
                        {
                            if !parameter.callable {
                                continue;
                            }
                            let Some(argument) = call.raw.arguments.get(index) else {
                                continue;
                            };
                            let points = callable_argument_targets(
                                &program,
                                function,
                                argument,
                                call.raw.line,
                                call.raw.column,
                                &callback_points,
                            )?;
                            let entry = callback_points
                                .entry((target_id.clone(), parameter.name.clone()))
                                .or_default();
                            let old_len = entry.len();
                            entry.extend(points);
                            changed |= entry.len() != old_len;
                        }
                    }
                }
            }
            for points in callback_points.values() {
                for target in points {
                    if let Target::Local(target) = target
                        && reachable.insert(target.clone())
                    {
                        queue.push_back(target.clone());
                        changed = true;
                    }
                }
            }
            while let Some(function_id) = queue.pop_front() {
                let function = &program.functions[&function_id];
                reject_dynamic_signature(function)?;
                let mut resolved = Vec::new();
                for raw in &program.syntax[&function_id].calls {
                    let targets = program.resolve_call(function, raw)?;
                    for target in &targets {
                        if let Target::Local(target) = target
                            && reachable.insert(target.clone())
                        {
                            queue.push_back(target.clone());
                        }
                    }
                    resolved.push(ResolvedCall {
                        raw: raw.clone(),
                        targets,
                    });
                }
                calls.insert(function_id, resolved);
            }
            if !changed {
                break;
            }
        }

        for function_id in &reachable {
            let function = &program.functions[function_id];
            reject_dynamic_signature(function)?;
            for call in calls.get(function_id).into_iter().flatten() {
                for target in &call.targets {
                    if let Target::Callback(parameter) = target {
                        let points = callback_points.get(&(function_id.clone(), parameter.clone()));
                        if points.is_none_or(BTreeSet::is_empty) {
                            return Err(AuditError::UnresolvedIndirectCall {
                                function: function_id.clone(),
                                parameter: parameter.clone(),
                            });
                        }
                    }
                }
            }
        }

        let parameter_taint =
            propagate_parameter_taint(&program, &reachable, &calls, &root_set, &callback_points);
        let mut rows = Vec::new();
        for function_id in &reachable {
            let function = &program.functions[function_id];
            let tainted = parameter_taint
                .get(function_id)
                .cloned()
                .unwrap_or_default();
            for call in calls.get(function_id).into_iter().flatten() {
                let mut targets = BTreeSet::new();
                let mut indirect = false;
                for target in &call.targets {
                    match target {
                        Target::Callback(parameter) => {
                            indirect = true;
                            if let Some(points) =
                                callback_points.get(&(function_id.clone(), parameter.clone()))
                            {
                                targets.extend(points.iter().map(Target::render));
                            }
                        }
                        other => {
                            targets.insert(other.render());
                        }
                    }
                }
                let sources = information_sources(&call.raw.controls, &tainted);
                let class = classify(function, Some(call), &call.raw.controls, &sources)
                    .ok_or_else(|| AuditError::UnclassifiedSite {
                        function: function_id.clone(),
                        location: format!(
                            "{}:{}:{}",
                            function.path, call.raw.line, call.raw.column
                        ),
                    })?;
                rows.push(EvidenceRow {
                    path: function.path.clone(),
                    line: call.raw.line,
                    column: call.raw.column,
                    function: function_id.clone(),
                    kind: if indirect {
                        "indirect-call".to_owned()
                    } else {
                        "call".to_owned()
                    },
                    class,
                    tainted: !sources.is_empty(),
                    taint_sources: render_sources(&sources),
                    control_type: control_type(function, &call.raw.controls),
                    outcome: class.outcome().to_owned(),
                    targets: targets.into_iter().collect(),
                });
            }
            for branch in &program.syntax[function_id].branches {
                let sources = information_sources(&branch.control, &tainted);
                let class =
                    classify(function, None, &branch.control, &sources).ok_or_else(|| {
                        AuditError::UnclassifiedSite {
                            function: function_id.clone(),
                            location: format!(
                                "{}:{}:{}",
                                function.path, branch.line, branch.column
                            ),
                        }
                    })?;
                rows.push(EvidenceRow {
                    path: function.path.clone(),
                    line: branch.line,
                    column: branch.column,
                    function: function_id.clone(),
                    kind: branch.kind.to_owned(),
                    class,
                    tainted: !sources.is_empty(),
                    taint_sources: render_sources(&sources),
                    control_type: control_type(function, &branch.control),
                    outcome: class.outcome().to_owned(),
                    targets: vec![class.outcome().to_owned()],
                });
            }
        }
        rows.sort();
        Ok(Self {
            roots: root_set,
            reachable,
            rows,
        })
    }

    fn summary_evidence(&self) -> String {
        let mut counts: BTreeMap<Mechanic, (usize, usize, BTreeSet<String>)> = BTreeMap::new();
        for row in &self.rows {
            let entry = counts.entry(row.class).or_default();
            entry.0 += 1;
            entry.1 += usize::from(row.tainted);
            entry.2.extend(row.targets.iter().cloned());
        }
        let mut output =
            String::from("class\treachable_sites\tpackage_tainted_sites\tcode_target_count\n");
        for (class, (sites, tainted, targets)) in counts {
            output.push_str(class.name());
            output.push('\t');
            output.push_str(&sites.to_string());
            output.push('\t');
            output.push_str(&tainted.to_string());
            output.push('\t');
            output.push_str(&targets.len().to_string());
            output.push('\n');
        }
        output
    }

    fn site_evidence(&self) -> String {
        let mut output = String::from(
            "location\tfunction\tkind\tclass\tinfluence\ttaint_sources\tcontrol_type\tallowed_outcome\tcode_targets\n",
        );
        for row in &self.rows {
            output.push_str(&row.render());
            output.push('\n');
        }
        output
    }
}

fn callable_argument_targets(
    program: &Program,
    function: &Function,
    argument: &Expr,
    line: usize,
    column: usize,
    callback_points: &BTreeMap<(String, String), BTreeSet<Target>>,
) -> Result<BTreeSet<Target>, AuditError> {
    match argument {
        Expr::Closure(_) => {
            let mut targets = BTreeSet::new();
            targets.insert(Target::External(format!(
                "closure@{}:{line}:{column}",
                function.path
            )));
            Ok(targets)
        }
        Expr::Path(path) => {
            if let Some(name) = path.path.get_ident().map(ToString::to_string)
                && function
                    .parameters
                    .iter()
                    .any(|parameter| parameter.name == name && parameter.callable)
            {
                return Ok(callback_points
                    .get(&(function.id.clone(), name))
                    .cloned()
                    .unwrap_or_default());
            }
            program.resolve_direct(function, argument)
        }
        _ => Err(AuditError::DynamicCall {
            function: function.id.clone(),
            location: location(function, argument.span()),
        }),
    }
}

fn propagate_parameter_taint(
    program: &Program,
    reachable: &BTreeSet<String>,
    calls: &BTreeMap<String, Vec<ResolvedCall>>,
    roots: &BTreeSet<String>,
    callback_points: &BTreeMap<(String, String), BTreeSet<Target>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut taint: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for root in roots {
        let function = &program.functions[root];
        let entry = taint.entry(root.clone()).or_default();
        for parameter in &function.parameters {
            entry.insert(parameter.name.clone());
        }
    }
    loop {
        let mut changed = false;
        for function_id in reachable {
            let caller_taint = taint.get(function_id).cloned().unwrap_or_default();
            for call in calls.get(function_id).into_iter().flatten() {
                let argument_taint: Vec<bool> = call
                    .raw
                    .arguments
                    .iter()
                    .map(|argument| !information_sources(argument, &caller_taint).is_empty())
                    .collect();
                let receiver_tainted = match &call.raw.callee {
                    CallKind::Method { receiver, .. } => {
                        !information_sources(receiver, &caller_taint).is_empty()
                    }
                    CallKind::Direct(_) | CallKind::Macro(_) => false,
                };
                for target in &call.targets {
                    match target {
                        Target::Local(target) => {
                            let function = &program.functions[target];
                            let entry = taint.entry(target.clone()).or_default();
                            if receiver_tainted
                                && function
                                    .parameters
                                    .iter()
                                    .any(|parameter| parameter.name == "self")
                            {
                                changed |= entry.insert("self".to_owned());
                            }
                            for (index, parameter) in function
                                .parameters
                                .iter()
                                .filter(|parameter| parameter.name != "self")
                                .enumerate()
                            {
                                if argument_taint.get(index).copied().unwrap_or(false) {
                                    changed |= entry.insert(parameter.name.clone());
                                }
                            }
                        }
                        Target::Callback(parameter) => {
                            if let Some(points) =
                                callback_points.get(&(function_id.clone(), parameter.clone()))
                            {
                                for point in points {
                                    if let Target::Local(target) = point {
                                        let function = &program.functions[target];
                                        let entry = taint.entry(target.clone()).or_default();
                                        for (index, parameter) in function
                                            .parameters
                                            .iter()
                                            .filter(|parameter| parameter.name != "self")
                                            .enumerate()
                                        {
                                            if argument_taint.get(index).copied().unwrap_or(false) {
                                                changed |= entry.insert(parameter.name.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Target::External(_) => {}
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    taint
}

fn classify(
    function: &Function,
    call: Option<&ResolvedCall>,
    control: &Expr,
    _sources: &BTreeSet<String>,
) -> Option<Mechanic> {
    let signature = canonical_tokens(&function.signature);
    let target_text = call
        .map(|call| {
            call.targets
                .iter()
                .map(Target::render)
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_default();
    if function.module == "physical" {
        if function.owner.as_deref() == Some("SealedPhysical") || target_text.contains("Sha256") {
            return Some(Mechanic::PhysicalDispatch);
        }
        return Some(Mechanic::CoreAbi);
    }
    if function.module == "evaluator" {
        if function.owner.as_deref() == Some("DefinitionTable")
            || target_text.contains("DefinitionTable")
            || target_text.contains("binary_search")
        {
            return Some(Mechanic::DefinitionTable);
        }
        if signature.contains("EvalNode")
            || signature.contains("NodeContext")
            || signature.contains("Certificate")
            || target_text.contains("premise")
            || target_text.contains("nodes")
        {
            return Some(Mechanic::CertificateStep);
        }
        if target_text.contains("std::vec::Vec::extend_from_slice")
            || target_text.contains("std::vec::Vec::remove")
            || target_text.contains("slice::is_empty")
            || (signature.contains("KValue")
                && signature.contains("Result < Vec < u8 >")
                && matches!(control, Expr::Match(_)))
            || matches!(control, Expr::Binary(binary) if matches!(binary.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_)))
        {
            return Some(Mechanic::ByteMachine);
        }
        return Some(Mechanic::KernelStep);
    }
    if function.module.starts_with("compiler_package_v2::") || function.module == "artifacts" {
        return Some(Mechanic::WireCodec);
    }
    None
}

fn control_type(function: &Function, expression: &Expr) -> String {
    if let Expr::Path(path) = expression
        && let Some(name) = path.path.get_ident()
        && let Some(ty) = function.parameter_type(&name.to_string())
    {
        return ty.to_owned();
    }
    match expression {
        Expr::Lit(_) => "literal".to_owned(),
        Expr::Path(_) => "local-value".to_owned(),
        Expr::Field(_) => "typed-field".to_owned(),
        Expr::MethodCall(_) => "static-method-result".to_owned(),
        Expr::Call(_) => "static-call-result".to_owned(),
        Expr::Binary(_) => "bool".to_owned(),
        Expr::Unary(_) => "unary-result".to_owned(),
        Expr::Index(_) => "indexed-value".to_owned(),
        Expr::Reference(_) => "reference".to_owned(),
        Expr::Tuple(_) => "tuple".to_owned(),
        _ => format!("ast::{}", expression_kind(expression)),
    }
}

fn expression_kind(expression: &Expr) -> &'static str {
    match expression {
        Expr::Array(_) => "array",
        Expr::Assign(_) => "assign",
        Expr::Async(_) => "async",
        Expr::Await(_) => "await",
        Expr::Binary(_) => "binary",
        Expr::Block(_) => "block",
        Expr::Break(_) => "break",
        Expr::Call(_) => "call",
        Expr::Cast(_) => "cast",
        Expr::Closure(_) => "closure",
        Expr::Const(_) => "const",
        Expr::Continue(_) => "continue",
        Expr::Field(_) => "field",
        Expr::ForLoop(_) => "for",
        Expr::Group(_) => "group",
        Expr::If(_) => "if",
        Expr::Index(_) => "index",
        Expr::Infer(_) => "infer",
        Expr::Let(_) => "let",
        Expr::Lit(_) => "literal",
        Expr::Loop(_) => "loop",
        Expr::Macro(_) => "macro",
        Expr::Match(_) => "match",
        Expr::MethodCall(_) => "method-call",
        Expr::Paren(_) => "paren",
        Expr::Path(_) => "path",
        Expr::Range(_) => "range",
        Expr::RawAddr(_) => "raw-address",
        Expr::Reference(_) => "reference",
        Expr::Repeat(_) => "repeat",
        Expr::Return(_) => "return",
        Expr::Struct(_) => "struct",
        Expr::Try(_) => "try",
        Expr::TryBlock(_) => "try-block",
        Expr::Tuple(_) => "tuple",
        Expr::Unary(_) => "unary",
        Expr::Unsafe(_) => "unsafe",
        Expr::Verbatim(_) => "verbatim",
        Expr::While(_) => "while",
        Expr::Yield(_) => "yield",
        _ => "unknown",
    }
}

fn taint_sources(expression: &Expr, tainted: &BTreeSet<String>) -> BTreeSet<String> {
    struct Identifiers<'a> {
        tainted: &'a BTreeSet<String>,
        found: BTreeSet<String>,
    }
    impl<'ast> Visit<'ast> for Identifiers<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if let Some(identifier) = path.path.get_ident() {
                let name = identifier.to_string();
                if self.tainted.contains(&name) {
                    self.found.insert(name);
                }
            }
            visit::visit_expr_path(self, path);
        }
    }
    let mut identifiers = Identifiers {
        tainted,
        found: BTreeSet::new(),
    };
    identifiers.visit_expr(expression);
    identifiers.found
}

fn information_sources(expression: &Expr, tainted: &BTreeSet<String>) -> BTreeSet<String> {
    let mut sources = taint_sources(expression, tainted);
    if sources.is_empty() && !tainted.is_empty() && potentially_data_dependent(expression) {
        sources.insert("transitive-data".to_owned());
    }
    sources
}

fn potentially_data_dependent(expression: &Expr) -> bool {
    match expression {
        Expr::Lit(_) | Expr::Closure(_) => false,
        Expr::Path(path) => path.path.get_ident().is_some_and(|identifier| {
            let name = identifier.to_string();
            name == "self" || name.chars().next().is_some_and(char::is_lowercase)
        }),
        _ => true,
    }
}

fn render_sources(sources: &BTreeSet<String>) -> String {
    if sources.is_empty() {
        "-".to_owned()
    } else {
        sources.iter().cloned().collect::<Vec<_>>().join("|")
    }
}

fn is_self(expression: &Expr) -> bool {
    matches!(expression, Expr::Path(path) if path.path.is_ident("self"))
}

fn nominal_type(ty: &str) -> String {
    let before_generics = ty.split('<').next().unwrap_or(ty);
    let last_word = before_generics
        .split_whitespace()
        .last()
        .unwrap_or(before_generics);
    last_word
        .split("::")
        .last()
        .unwrap_or(last_word)
        .trim_matches(|character: char| !character.is_alphanumeric() && character != '_')
        .to_owned()
}

fn location(function: &Function, span: proc_macro2::Span) -> String {
    let start = span.start();
    format!("{}:{}:{}", function.path, start.line, start.column + 1)
}

fn canonical_tokens(value: &impl ToTokens) -> String {
    one_line(&value.to_token_stream().to_string())
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

fn assert_or_update_fixture(relative: &str, actual: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    if std::env::var_os("CLAUSE_UPDATE_HOST_AUDIT").is_some() {
        std::fs::write(&path, actual).expect("audit fixture updates");
        return;
    }
    let expected = std::fs::read_to_string(&path).expect("audit fixture is readable");
    assert_eq!(actual, expected, "generated host audit drifted: {relative}");
}
