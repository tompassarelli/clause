use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use clause_substrate::artifacts::{ArtifactStore, CompilerArtifactError, CompilerPackageArtifact};
use clause_substrate::compiler_package_v3::{
    CompilerEvidence, CompilerInterface, CompilerLineage, CompilerPackage, CompilerSubject,
    CoreManifest, Definition, Id32, KExpr, KSort, Term, compiler_package_hash, encode,
};
use quote::ToTokens;
use sha2::{Digest, Sha256};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ImplItem, Item, ItemFn};

const AUDIT_SCHEMA: &str = "clause-host-mechanics-mir-v2";

const ROOTS: &[RootContract] = &[
    RootContract::new(
        "compiler_package_v3::codec::encode",
        Mechanic::WireCodec,
        &[
            "compiler_package_v3::types::CompilerPackage",
            "compiler_package_v3::types::EncodeError",
        ],
    ),
    RootContract::new(
        "compiler_package_v3::codec::decode",
        Mechanic::WireCodec,
        &[
            "compiler_package_v3::types::DecodedCompilerPackage",
            "compiler_package_v3::types::DecodeFailure",
        ],
    ),
    RootContract::new(
        "evaluator::Evaluator::new",
        Mechanic::KernelStep,
        &["evaluator::Evaluator", "evaluator::StaticError"],
    ),
    RootContract::new(
        "evaluator::Evaluator::check_definitions",
        Mechanic::KernelStep,
        &["evaluator::Evaluator", "evaluator::StaticError"],
    ),
    RootContract::new(
        "evaluator::Evaluator::infer_sort",
        Mechanic::KernelStep,
        &[
            "evaluator::Evaluator",
            "compiler_package_v3::types::KExpr",
            "compiler_package_v3::types::KSort",
            "evaluator::StaticError",
        ],
    ),
    RootContract::new(
        "evaluator::Evaluator::evaluate",
        Mechanic::KernelStep,
        &[
            "evaluator::Evaluator",
            "compiler_package_v3::types::KExpr",
            "compiler_package_v3::types::KValue",
            "evaluator::Evaluation",
            "evaluator::EvalError",
        ],
    ),
    RootContract::new(
        "evaluator::Evaluator::build_receipt",
        Mechanic::ReceiptStep,
        &[
            "evaluator::Evaluator",
            "compiler_package_v3::types::Id32",
            "compiler_package_v3::types::KValue",
            "compiler_package_v3::types::EvalReceipt",
            "evaluator::EvalError",
        ],
    ),
    RootContract::new(
        "artifacts::CompilerPackageArtifact::decode_and_intern",
        Mechanic::WireCodec,
        &[
            "artifacts::ArtifactStore",
            "artifacts::CompilerPackageArtifact",
            "artifacts::CompilerArtifactError",
        ],
    ),
    RootContract::new(
        "compiler_package_v3::checker::authorize_genesis",
        Mechanic::AuthorizationStep,
        &[
            "compiler_package_v3::types::DecodedCompilerPackage",
            "compiler_package_v3::checker::GenesisAuthorizationRequest",
            "compiler_package_v3::checker::AuthorizationVerdict",
            "compiler_package_v3::checker::AuthorizationCheckError",
        ],
    ),
    RootContract::new(
        "compiler_package_v3::checker::authorize_successor",
        Mechanic::AuthorizationStep,
        &[
            "compiler_package_v3::types::DecodedCompilerPackage",
            "compiler_package_v3::checker::SuccessorAuthorizationRequest",
            "compiler_package_v3::checker::AuthorizationVerdict",
            "compiler_package_v3::checker::AuthorizationCheckError",
        ],
    ),
];

const CLOSED_WIRE_CODEC_FUNCTIONS: &[&str] = &[
    "artifacts::ArtifactStore::intern_compiler_package",
    "artifacts::ArtifactStore::intern_with_id",
    "artifacts::ImmutableArtifact::exact_bytes",
    "artifacts::ImmutableArtifact::new",
    "compiler_package_v3::codec::Cursor::blob",
    "compiler_package_v3::codec::Cursor::frame",
    "compiler_package_v3::codec::Cursor::hash32",
    "compiler_package_v3::codec::Cursor::id32",
    "compiler_package_v3::codec::Cursor::read",
    "compiler_package_v3::codec::Cursor::rejection",
    "compiler_package_v3::codec::Cursor::sequence",
    "compiler_package_v3::codec::Cursor::sequence_count",
    "compiler_package_v3::codec::Cursor::top",
    "compiler_package_v3::codec::Cursor::u32",
    "compiler_package_v3::codec::Cursor::u64",
    "compiler_package_v3::codec::Cursor::u8",
    "compiler_package_v3::codec::DecodeBudget::expression_node",
    "compiler_package_v3::codec::DecodeBudget::item",
    "compiler_package_v3::codec::DecodeBudget::new",
    "compiler_package_v3::codec::DecodeBudget::term_node",
    "compiler_package_v3::codec::EncodeBudget::new",
    "compiler_package_v3::codec::Encoder::blob",
    "compiler_package_v3::codec::Encoder::expression_node",
    "compiler_package_v3::codec::Encoder::fixed",
    "compiler_package_v3::codec::Encoder::frame",
    "compiler_package_v3::codec::Encoder::items",
    "compiler_package_v3::codec::Encoder::length",
    "compiler_package_v3::codec::Encoder::new",
    "compiler_package_v3::codec::Encoder::reserve",
    "compiler_package_v3::codec::Encoder::sequence",
    "compiler_package_v3::codec::Encoder::term_node",
    "compiler_package_v3::codec::Encoder::u32",
    "compiler_package_v3::codec::Encoder::u64",
    "compiler_package_v3::codec::Encoder::u8",
    "compiler_package_v3::codec::canonical_term_bytes",
    "compiler_package_v3::codec::canonical_value_bytes",
    "compiler_package_v3::codec::canonical_evidence_bytes",
    "compiler_package_v3::codec::canonical_subject_bytes",
    "compiler_package_v3::codec::decode_box",
    "compiler_package_v3::codec::decode_core_manifest_value",
    "compiler_package_v3::codec::decode_definition",
    "compiler_package_v3::codec::decode_evidence_value",
    "compiler_package_v3::codec::decode_expr",
    "compiler_package_v3::codec::decode_named_signature",
    "compiler_package_v3::codec::decode_nominal_declaration",
    "compiler_package_v3::codec::decode_nominal_ref",
    "compiler_package_v3::codec::decode_physical_profile",
    "compiler_package_v3::codec::decode_receipt",
    "compiler_package_v3::codec::decode_rule_signature",
    "compiler_package_v3::codec::decode_sort",
    "compiler_package_v3::codec::decode_subject_value",
    "compiler_package_v3::codec::decode_term",
    "compiler_package_v3::codec::encode_core_manifest_value_with_budget",
    "compiler_package_v3::codec::encode_core_manifest_value",
    "compiler_package_v3::codec::encode_definition",
    "compiler_package_v3::codec::encode_evidence_value",
    "compiler_package_v3::codec::encode_expr",
    "compiler_package_v3::codec::encode_expression_depth",
    "compiler_package_v3::codec::encode_named_signature",
    "compiler_package_v3::codec::encode_nominal_declaration",
    "compiler_package_v3::codec::encode_nominal_ref",
    "compiler_package_v3::codec::encode_physical_profile",
    "compiler_package_v3::codec::encode_physical_profile_value",
    "compiler_package_v3::codec::encode_receipt",
    "compiler_package_v3::codec::encode_rule_signature",
    "compiler_package_v3::codec::encode_sort",
    "compiler_package_v3::codec::encode_subject_value",
    "compiler_package_v3::codec::encode_term",
    "compiler_package_v3::codec::encode_term_depth",
    "compiler_package_v3::codec::encode_u8_sequence",
    "compiler_package_v3::codec::encode_value",
    "compiler_package_v3::codec::expression_depth",
    "compiler_package_v3::codec::pop_decoded_expression",
    "compiler_package_v3::codec::push_decode_expression",
    "compiler_package_v3::codec::push_decoded_expression",
    "compiler_package_v3::codec::push_encode_expression",
    "compiler_package_v3::codec::reserve_decode_expressions",
    "compiler_package_v3::codec::reserve_encode_expressions",
    "compiler_package_v3::codec::schedule_fixed_expression",
    "compiler_package_v3::codec::term_depth",
    "compiler_package_v3::codec::unknown",
];

const CLOSED_CORE_ABI_FUNCTIONS: &[&str] = &[
    "compiler_package_v3::manifest::compiler_package_hash",
    "compiler_package_v3::manifest::CoreManifest::canonical_v1",
    "compiler_package_v3::manifest::CoreManifest::try_canonical_v1",
    "compiler_package_v3::manifest::PhysicalProfile::try_sealed_sha256",
    "compiler_package_v3::manifest::clauses",
    "compiler_package_v3::manifest::copy_bytes",
    "compiler_package_v3::manifest::core_contract_id",
    "compiler_package_v3::manifest::domain_hash",
    "compiler_package_v3::manifest::eval_receipt_observations_hash",
    "compiler_package_v3::manifest::eval_receipt_value_hash",
    "compiler_package_v3::manifest::exact_core_manifest_bytes",
    "compiler_package_v3::manifest::exact_physical_profile_bytes",
    "compiler_package_v3::manifest::physical_profile_id",
    "compiler_package_v3::manifest::PhysicalProfile::sealed_sha256",
    "compiler_package_v3::manifest::named",
    "compiler_package_v3::manifest::rule",
    "compiler_package_v3::manifest::reserve",
    "compiler_package_v3::manifest::sha256_operation_id",
    "compiler_package_v3::manifest::source_artifact_id",
    "compiler_package_v3::manifest::singleton",
    "compiler_package_v3::manifest::tag_range",
    "compiler_package_v3::types::DecodedCompilerPackage::new",
    "compiler_package_v3::types::DecodedCompilerPackage::exact_input",
    "compiler_package_v3::types::DecodedCompilerPackage::exact_core_manifest",
    "compiler_package_v3::types::DecodedCompilerPackage::exact_evidence",
    "compiler_package_v3::types::DecodedCompilerPackage::exact_subject",
    "compiler_package_v3::types::DecodedCompilerPackage::package",
    "compiler_package_v3::types::FallibleBox::into_inner",
    "compiler_package_v3::types::FallibleBox::try_new",
    "compiler_package_v3::types::Hash32::as_bytes",
    "compiler_package_v3::types::Id32::as_bytes",
    "compiler_package_v3::types::KExpr::validate_resource_bounds",
    "compiler_package_v3::types::KValue::sort",
    "compiler_package_v3::types::KValue::try_clone_resource",
    "compiler_package_v3::types::KValue::validate_resource_bounds",
    "compiler_package_v3::types::Term::try_clone_resource",
    "compiler_package_v3::types::Term::try_triple",
    "compiler_package_v3::types::Term::validate_resource_bounds",
    "compiler_package_v3::types::clone_term",
    "compiler_package_v3::types::push_term",
    "compiler_package_v3::types::try_box",
    "compiler_package_v3::types::try_copy_bytes",
    "physical::ObservationLog::try_to_term",
    "physical::atom",
    "physical::bytes",
    "physical::id",
    "physical::list",
    "physical::nat64",
    "physical::observations_term",
    "physical::one_field",
    "physical::record",
    "physical::tag",
    "physical::value_term",
];

const CLOSED_DEFINITION_TABLE_FUNCTIONS: &[&str] = &[
    "evaluator::DefinitionTable::new",
    "evaluator::DefinitionTable::resolve",
];

const CLOSED_KERNEL_STEP_FUNCTIONS: &[&str] = &[
    "evaluator::EvalTask::environment",
    "evaluator::EvaluationMachine::continue_call",
    "evaluator::EvaluationMachine::continue_case_bytes",
    "evaluator::EvaluationMachine::continue_case_bytes_equal",
    "evaluator::EvaluationMachine::continue_case_term",
    "evaluator::EvaluationMachine::continue_let",
    "evaluator::EvaluationMachine::enter",
    "evaluator::EvaluationMachine::finish_concat",
    "evaluator::EvaluationMachine::finish_make_atom",
    "evaluator::EvaluationMachine::finish_make_triple",
    "evaluator::EvaluationMachine::finish_request",
    "evaluator::EvaluationMachine::new",
    "evaluator::EvaluationMachine::pop_result",
    "evaluator::EvaluationMachine::prepare_owned_allocation",
    "evaluator::EvaluationMachine::push_result",
    "evaluator::EvaluationMachine::push_task",
    "evaluator::EvaluationMachine::push_reserved_task",
    "evaluator::EvaluationMachine::reserve_tasks",
    "evaluator::EvaluationMachine::run",
    "evaluator::EvaluationMachine::take_results",
    "evaluator::LivenessWorkBudget::charge",
    "evaluator::LivenessWorkBudget::new",
    "evaluator::Evaluator::check_physical_profile",
    "evaluator::Evaluator::infer",
    "evaluator::Evaluator::new_unprofiled",
    "evaluator::Evaluator::replay_entrypoint",
    "evaluator::RuntimeByteStorage::as_slice",
    "evaluator::RuntimeByteStorage::owned_len",
    "evaluator::RuntimeByteStore::borrowed",
    "evaluator::RuntimeByteStore::get",
    "evaluator::RuntimeByteStore::insert",
    "evaluator::RuntimeByteStore::materialize",
    "evaluator::RuntimeByteStore::new",
    "evaluator::RuntimeByteStore::owned",
    "evaluator::RuntimeByteStore::release",
    "evaluator::RuntimeByteStore::replaced_owned_bytes",
    "evaluator::RuntimeByteStore::retain",
    "evaluator::RuntimeByteStore::split_first",
    "evaluator::RuntimeByteStore::try_copy",
    "evaluator::RuntimeTerm::into_term",
    "evaluator::RuntimeTerm::validate_resource_bounds",
    "evaluator::RuntimeValue::borrowed",
    "evaluator::RuntimeValue::into_kvalue",
    "evaluator::RuntimeValue::owned",
    "evaluator::RuntimeValue::release",
    "evaluator::RuntimeValue::sort",
    "evaluator::RuntimeValue::try_clone_resource",
    "evaluator::RuntimeValue::validate_resource_bounds",
    "evaluator::RuntimeEnvironments::entry",
    "evaluator::RuntimeEnvironments::entry_mut",
    "evaluator::RuntimeEnvironments::begin_live_epoch",
    "evaluator::RuntimeEnvironments::discard_unmarked",
    "evaluator::RuntimeEnvironments::extend",
    "evaluator::RuntimeEnvironments::get",
    "evaluator::RuntimeEnvironments::locate",
    "evaluator::RuntimeEnvironments::mark_live",
    "evaluator::RuntimeEnvironments::new",
    "evaluator::RuntimeEnvironments::reserve_environment_pass",
    "evaluator::RuntimeEnvironments::release",
    "evaluator::RuntimeEnvironments::retain",
    "evaluator::RuntimeEnvironments::try_clone_value",
    "evaluator::RuntimeValues::len",
    "evaluator::SortEnvironments::extend",
    "evaluator::SortEnvironments::extend_one",
    "evaluator::SortEnvironments::extend_three",
    "evaluator::SortEnvironments::extend_two",
    "evaluator::SortEnvironments::get",
    "evaluator::SortEnvironments::new",
    "evaluator::SortValues::get",
    "evaluator::SortValues::len",
    "evaluator::common_sort",
    "evaluator::checked_concat_length",
    "evaluator::expect_runtime_bytes",
    "evaluator::expect_runtime_term",
    "evaluator::mark_live_expression",
    "evaluator::mark_live_expression_inner",
    "evaluator::mark_task_live_slots",
    "evaluator::next_live_child",
    "evaluator::push_infer_task",
    "evaluator::push_live_scan_frame",
    "evaluator::push_sort",
    "evaluator::require_sort",
    "evaluator::require_runtime_value_sort",
    "evaluator::reserve_infer_tasks",
    "evaluator::value_sorts",
];

const CLOSED_RECEIPT_STEP_FUNCTIONS: &[&str] = &["evaluator::value_literal"];

const CLOSED_AUTHORIZATION_STEP_FUNCTIONS: &[&str] = &[
    "compiler_package_v3::checker::OwnerAnchorWitness::observation",
    "compiler_package_v3::checker::admission_replay",
    "compiler_package_v3::checker::admission_request_term",
    "compiler_package_v3::checker::allocations_are_acyclic",
    "compiler_package_v3::checker::as_bytes",
    "compiler_package_v3::checker::as_hash",
    "compiler_package_v3::checker::as_id",
    "compiler_package_v3::checker::as_tag",
    "compiler_package_v3::checker::as_u64",
    "compiler_package_v3::checker::atom",
    "compiler_package_v3::checker::build_failure",
    "compiler_package_v3::checker::bytes_term",
    "compiler_package_v3::checker::change_occurrence_domain",
    "compiler_package_v3::checker::common_failure",
    "compiler_package_v3::checker::compile_replay",
    "compiler_package_v3::checker::compiler_revision_id",
    "compiler_package_v3::checker::copy_authorized",
    "compiler_package_v3::checker::core_failure",
    "compiler_package_v3::checker::decode_base",
    "compiler_package_v3::checker::decode_build_request",
    "compiler_package_v3::checker::decode_identity_plan",
    "compiler_package_v3::checker::decode_nominal_ref",
    "compiler_package_v3::checker::decode_ref_list",
    "compiler_package_v3::checker::decode_ref_wrapper",
    "compiler_package_v3::checker::decode_source_unit",
    "compiler_package_v3::checker::definition_domain",
    "compiler_package_v3::checker::deny",
    "compiler_package_v3::checker::exactly",
    "compiler_package_v3::checker::expression_nominal_references_valid",
    "compiler_package_v3::checker::failure",
    "compiler_package_v3::checker::final_failure",
    "compiler_package_v3::checker::find_nominal",
    "compiler_package_v3::checker::find_nominal_index",
    "compiler_package_v3::checker::list_items",
    "compiler_package_v3::checker::list_term",
    "compiler_package_v3::checker::map_encode",
    "compiler_package_v3::checker::new_nominal_id",
    "compiler_package_v3::checker::nominal_domain",
    "compiler_package_v3::checker::nominal_exists",
    "compiler_package_v3::checker::nominal_parts",
    "compiler_package_v3::checker::nominal_reference",
    "compiler_package_v3::checker::nominal_table_valid",
    "compiler_package_v3::checker::package_is_exactly_canonical",
    "compiler_package_v3::checker::plan_valid",
    "compiler_package_v3::checker::predecessor_evaluator",
    "compiler_package_v3::checker::program_strictly_sorted",
    "compiler_package_v3::checker::receipt_shape_valid",
    "compiler_package_v3::checker::record_fields",
    "compiler_package_v3::checker::record_term",
    "compiler_package_v3::checker::references_strictly_sorted",
    "compiler_package_v3::checker::replay_entrypoint",
    "compiler_package_v3::checker::replay_error_is_resource_exhausted",
    "compiler_package_v3::checker::resolve_predecessor",
    "compiler_package_v3::checker::resource",
    "compiler_package_v3::checker::result_bytes",
    "compiler_package_v3::checker::source_unit_domain",
    "compiler_package_v3::checker::tag",
    "compiler_package_v3::checker::term_nominal_references_valid",
    "compiler_package_v3::checker::wire_reference",
];

const CLOSED_PHYSICAL_DISPATCH_FUNCTIONS: &[&str] = &[
    "physical::SealedPhysical::new",
    "physical::SealedPhysical::request",
];

const CLOSED_ENUMS: &[(&str, &[&str])] = &[
    ("compiler_package_v3::types::Term", &["Atom", "Triple"]),
    ("compiler_package_v3::types::KSort", &["Bytes", "Term"]),
    (
        "compiler_package_v3::types::KExpr",
        &[
            "BytesLiteral",
            "TermLiteral",
            "Var",
            "MakeAtom",
            "MakeTriple",
            "Let",
            "CaseTerm",
            "CaseBytes",
            "ConcatBytes",
            "CaseBytesEqual",
            "Call",
            "Request",
        ],
    ),
    (
        "compiler_package_v3::types::CompilerLineage",
        &["Genesis", "Successor"],
    ),
    (
        "compiler_package_v3::types::NominalDeclaration",
        &["Seed", "RetainedSeed", "Allocated"],
    ),
    ("compiler_package_v3::types::KValue", &["Bytes", "Term"]),
    (
        "compiler_package_v3::types::CompilerEvidence",
        &["Genesis", "Successor"],
    ),
    (
        "compiler_package_v3::types::DecodeCode",
        &[
            "WrongMagic",
            "UnknownVersion",
            "FrameTagOrderOrCount",
            "Truncated",
            "LengthOrCountOverflow",
            "InvalidFixedWidth",
            "UnknownSumTag",
            "BoundedValueUnderConsumed",
            "BoundedValueOverConsumed",
            "TrailingBytes",
        ],
    ),
    (
        "compiler_package_v3::types::DecodeFailure",
        &["Rejected", "ResourceExhausted"],
    ),
    (
        "compiler_package_v3::types::EncodeError",
        &["LengthExceedsU32", "InvalidClosedTag", "ResourceExhausted"],
    ),
    (
        "artifacts::ArtifactError",
        &["HashCollision", "ResourceExhausted"],
    ),
    ("artifacts::CompilerArtifactError", &["Artifact", "Decode"]),
    (
        "evaluator::StaticError",
        &[
            "DefinitionsNotStrictlySorted",
            "DuplicateDefinition",
            "VariableOutOfBounds",
            "SortMismatch",
            "BranchSortMismatch",
            "UnknownDefinition",
            "ArgumentCount",
            "OperationOutsideSealedProfile",
            "RecursionLimit",
            "ResourceExhausted",
        ],
    ),
    (
        "evaluator::EvalError",
        &[
            "Static",
            "OutOfFuel",
            "VariableOutOfBounds",
            "ValueSort",
            "UnknownDefinition",
            "ArgumentCount",
            "Physical",
            "ByteLengthOverflow",
            "RecursionLimit",
            "ResourceExhausted",
        ],
    ),
    (
        "physical::PhysicalError",
        &[
            "UnknownOperation",
            "SignatureMismatch",
            "ObservationIndexOverflow",
            "ResourceExhausted",
        ],
    ),
    (
        "compiler_package_v3::codec::FixedExpression",
        &[
            "MakeAtom",
            "MakeTriple",
            "Let",
            "CaseTerm",
            "CaseBytes",
            "CaseBytesEqual",
        ],
    ),
    (
        "compiler_package_v3::codec::SequenceExpression",
        &["ConcatBytes", "Call", "Request"],
    ),
    (
        "compiler_package_v3::codec::DecodeExpressionTask",
        &["Read", "BuildFixed", "Sequence", "SequenceItem"],
    ),
    (
        "compiler_package_v3::checker::AuthorizationStage",
        &[
            "CoreManifest",
            "CoreWellFormedness",
            "GenesisAnchor",
            "ExactPredecessor",
            "BuildRequest",
            "CompileEvaluation",
            "AdmissionEvaluation",
            "EvidenceAttachment",
            "FinalAuthorization",
        ],
    ),
    (
        "compiler_package_v3::checker::AuthorizationCode",
        &[
            "ManifestMismatch",
            "SubjectStructure",
            "NominalTable",
            "DefinitionOrderOrDuplicate",
            "EntrypointResolution",
            "EntrypointAliased",
            "EntrypointSignature",
            "StaticRule",
            "PhysicalRequestSignature",
            "GenesisWrongLineage",
            "GenesisEvidenceNotEmpty",
            "MissingAnchor",
            "AnchorBytesMismatch",
            "SuccessorWrongLineage",
            "PredecessorNotAccepted",
            "CandidateOrSelfPredecessor",
            "LocatorMismatch",
            "PredecessorBytesMismatch",
            "BuildRequestShape",
            "DetachedBuildRequest",
            "BaseMismatch",
            "CoreContractMismatch",
            "PhysicalProfileMismatch",
            "SourceOrderOrDuplicate",
            "SourceArtifactMismatch",
            "IdentityPlanMismatch",
            "ChangeOccurrenceMismatch",
            "PhysicalInputsNonempty",
            "FuelInvalid",
            "EvidenceShapeMismatch",
            "ReceiptValueMismatch",
            "ReceiptFuelMismatch",
            "EvaluationFault",
            "UnexpectedResult",
            "SubjectMismatch",
            "ObservationMismatch",
            "EvidenceDetached",
            "SubjectChangedAfterCompile",
            "PackageChangedAfterEvidence",
            "FinalIdentityMismatch",
        ],
    ),
    (
        "compiler_package_v3::checker::AuthorizationVerdict",
        &["Authorized", "Unauthorized"],
    ),
    (
        "compiler_package_v3::checker::AuthorizationCheckError",
        &["Decode", "ResourceExhausted"],
    ),
    (
        "compiler_package_v3::checker::OwnerAnchorInput",
        &["Missing", "Supplied"],
    ),
    (
        "compiler_package_v3::checker::PredecessorInput",
        &["Absent", "Accepted"],
    ),
    (
        "compiler_package_v3::checker::BaseView",
        &["Genesis", "Accepted"],
    ),
    (
        "evaluator::SortValues",
        &["Borrowed", "One", "Two", "Three"],
    ),
    (
        "evaluator::InferTask",
        &[
            "Expression",
            "Require",
            "Return",
            "DiscardAndReturn",
            "Common",
            "LetBody",
            "CaseTermBodies",
            "CaseBytesBodies",
        ],
    ),
    ("evaluator::RuntimeByteStorage", &["Borrowed", "Owned"]),
    ("evaluator::RuntimeByteSlot", &["Occupied", "Vacant"]),
    ("evaluator::RuntimeTerm", &["Borrowed", "Owned"]),
    ("evaluator::RuntimeValue", &["Bytes", "Term"]),
    ("evaluator::RuntimeValues", &["Borrowed", "Owned"]),
    ("evaluator::RuntimeEnvironmentSlot", &["Occupied", "Vacant"]),
    ("evaluator::RuntimeValueReference", &["Borrowed", "Owned"]),
    (
        "evaluator::EvalTask",
        &[
            "Expression",
            "MakeAtom",
            "MakeTriple",
            "Let",
            "CaseTerm",
            "CaseBytes",
            "ConcatBytes",
            "CaseBytesEqual",
            "Call",
            "Request",
        ],
    ),
    (
        "compiler_package_v3::types::clone_term::Task",
        &["Read", "Triple"],
    ),
];

const CONSTITUTIONAL_NOMINALS: &[&str] = &[
    "artifacts::ImmutableArtifact",
    "artifacts::ArtifactStore",
    "artifacts::CompilerPackageArtifact",
    "artifacts::ArtifactError",
    "artifacts::CompilerArtifactError",
    "compiler_package_v3::codec::EncodeBudget",
    "compiler_package_v3::codec::Encoder",
    "compiler_package_v3::codec::EncodeExpressionTask",
    "compiler_package_v3::codec::DecodeBudget",
    "compiler_package_v3::codec::Cursor",
    "compiler_package_v3::codec::FixedExpression",
    "compiler_package_v3::codec::SequenceExpression",
    "compiler_package_v3::codec::DecodeExpressionTask",
    "compiler_package_v3::types::FallibleBox",
    "compiler_package_v3::types::Id32",
    "compiler_package_v3::types::Hash32",
    "compiler_package_v3::types::Span",
    "compiler_package_v3::types::Term",
    "compiler_package_v3::types::KSort",
    "compiler_package_v3::types::KExpr",
    "compiler_package_v3::types::NamedSignature",
    "compiler_package_v3::types::RuleSignature",
    "compiler_package_v3::types::PhysicalOperation",
    "compiler_package_v3::types::PhysicalProfile",
    "compiler_package_v3::types::CoreManifest",
    "compiler_package_v3::types::CompilerLineage",
    "compiler_package_v3::types::NominalWireRef",
    "compiler_package_v3::types::NominalDeclaration",
    "compiler_package_v3::types::CompilerInterface",
    "compiler_package_v3::types::Definition",
    "compiler_package_v3::types::CompilerSubject",
    "compiler_package_v3::types::KValue",
    "compiler_package_v3::types::ResourceLimit",
    "compiler_package_v3::types::EvalReceipt",
    "compiler_package_v3::types::CompilerEvidence",
    "compiler_package_v3::types::CompilerPackage",
    "compiler_package_v3::types::DecodedCompilerPackage",
    "compiler_package_v3::types::DecodeCode",
    "compiler_package_v3::types::DecodeRejection",
    "compiler_package_v3::types::DecodeFailure",
    "compiler_package_v3::types::EncodeError",
    "compiler_package_v3::checker::AuthorizationStage",
    "compiler_package_v3::checker::AuthorizationCode",
    "compiler_package_v3::checker::AuthorizationFailure",
    "compiler_package_v3::checker::AuthorizationVerdict",
    "compiler_package_v3::checker::AuthorizationCheckError",
    "compiler_package_v3::checker::FinalPackageIdentityInput",
    "compiler_package_v3::checker::OwnerAnchorObservation",
    "compiler_package_v3::checker::OwnerAnchorWitness",
    "compiler_package_v3::checker::OwnerAnchorInput",
    "compiler_package_v3::checker::GenesisAuthorizationRequest",
    "compiler_package_v3::checker::AcceptedExact",
    "compiler_package_v3::checker::PredecessorInput",
    "compiler_package_v3::checker::SuccessorAuthorizationRequest",
    "compiler_package_v3::checker::NominalRefView",
    "compiler_package_v3::checker::BaseView",
    "compiler_package_v3::checker::SourceUnitView",
    "compiler_package_v3::checker::IdentityPlanView",
    "compiler_package_v3::checker::BuildRequestView",
    "compiler_package_v3::checker::AcceptedPredecessor",
    "evaluator::StaticError",
    "evaluator::EvalError",
    "evaluator::Evaluation",
    "evaluator::DefinitionTable",
    "evaluator::Evaluator",
    "evaluator::SortValues",
    "evaluator::SortEnvironment",
    "evaluator::SortEnvironments",
    "evaluator::InferTask",
    "evaluator::RuntimeBytes",
    "evaluator::RuntimeByteStorage",
    "evaluator::RuntimeByteEntry",
    "evaluator::RuntimeByteSlot",
    "evaluator::RuntimeByteStore",
    "evaluator::RuntimeTerm",
    "evaluator::RuntimeValue",
    "evaluator::RuntimeValues",
    "evaluator::RuntimeOwnedValue",
    "evaluator::RuntimeEnvironment",
    "evaluator::RuntimeEnvironmentSlot",
    "evaluator::RuntimeValueReference",
    "evaluator::RuntimeEnvironments",
    "evaluator::EvalTask",
    "evaluator::LiveExpression",
    "evaluator::LiveScanFrame",
    "evaluator::LivenessWorkBudget",
    "evaluator::RuntimeResult",
    "evaluator::MachineResult",
    "evaluator::EvaluationMachine",
    "physical::PhysicalObservation",
    "physical::ObservationLog",
    "physical::SealedPhysical",
    "physical::Private",
    "physical::PhysicalError",
    "compiler_package_v3::types::clone_term::Task",
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
fn compiler_mir_proves_reachability_information_flow_and_fixed_dispatch() {
    let analysis = Analysis::production().expect("compiler-derived host proof closes");
    assert_eq!(analysis.roots.len(), ROOTS.len());
    assert!(
        analysis.contexts.len() > ROOTS.len(),
        "constitutional roots must reach their compiler-derived body closure"
    );
    assert!(
        analysis
            .rows
            .iter()
            .all(|row| !row.targets.is_empty() || row.kind != SiteKind::Call),
        "every reachable call must have an exact compiler-resolved target"
    );
    for class in [
        Mechanic::WireCodec,
        Mechanic::CoreAbi,
        Mechanic::ByteMachine,
        Mechanic::DefinitionTable,
        Mechanic::KernelStep,
        Mechanic::ReceiptStep,
        Mechanic::AuthorizationStep,
        Mechanic::PhysicalDispatch,
    ] {
        assert!(
            analysis.rows.iter().any(|row| row.class == class),
            "compiler closure omitted {class:?}"
        );
    }
    assert_eq!(
        analysis.fixed_handlers.len(),
        1,
        "the sealed physical profile has exactly one fixed host handler"
    );

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
fn artifacts_are_exact_deduplicated_and_candidate_only() {
    let bytes = encode(&package()).expect("package encodes");
    let mut store = ArtifactStore::new();
    let (source_id, source_address) = {
        let first = store.intern_source(&bytes).expect("artifact interns");
        assert_eq!(first.exact_bytes(), bytes);
        (first.id(), std::ptr::from_ref(first))
    };
    {
        let second = store.intern_source(&bytes).expect("artifact deduplicates");
        assert_eq!(second.id(), source_id);
        assert_eq!(std::ptr::from_ref(second), source_address);
    }
    assert_eq!(
        std::ptr::from_ref(store.get(source_id).expect("artifact remains indexed")),
        source_address
    );

    let candidate = CompilerPackageArtifact::decode_and_intern(&mut store, &bytes)
        .expect("strict candidate decode");
    assert_ne!(candidate.artifact().id(), source_id);
    assert_eq!(
        candidate.candidate().exact_input(),
        candidate.artifact().exact_bytes()
    );
    assert!(matches!(
        candidate.candidate().package().evidence,
        CompilerEvidence::Genesis
    ));
}

#[test]
fn malformed_compiler_package_ingestion_does_not_mutate_the_store() {
    let malformed = b"not a CLCP-v3 package";
    let malformed_id = compiler_package_hash(malformed);
    let mut store = ArtifactStore::new();

    assert!(matches!(
        CompilerPackageArtifact::decode_and_intern(&mut store, malformed),
        Err(CompilerArtifactError::Decode(_))
    ));
    assert!(store.get(malformed_id).is_none());
}

#[derive(Clone, Debug)]
struct SourceFile {
    module: String,
    path: String,
    digest: String,
}

#[derive(Clone, Debug)]
struct SourceFunction {
    id: String,
    module: String,
    owner: Option<String>,
    name: String,
    path: String,
    line: usize,
    column: usize,
    impl_line: Option<usize>,
}

#[derive(Clone, Debug)]
struct SourceInventory {
    manifest_dir: PathBuf,
    workspace_root: PathBuf,
    files: Vec<SourceFile>,
    functions: Vec<SourceFunction>,
    nominals: BTreeSet<String>,
    enum_variants: BTreeMap<String, BTreeSet<String>>,
    source_digest: String,
    cargo_lock_digest: String,
    cargo_manifest_digest: String,
}

impl SourceInventory {
    fn load() -> Result<Self, AuditError> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| AuditError::Io("crate is not inside its workspace".to_owned()))?
            .to_path_buf();
        let mut paths = Vec::new();
        rust_files(&manifest_dir.join("src"), &mut paths)?;
        paths.sort();
        if paths.is_empty() {
            return Err(AuditError::Io("crate source inventory is empty".to_owned()));
        }

        let mut files = Vec::new();
        let mut functions = Vec::new();
        let mut nominals = BTreeSet::new();
        let mut enum_variants = BTreeMap::new();
        let mut source_hasher = Sha256::new();
        for absolute in paths {
            let relative = absolute
                .strip_prefix(&manifest_dir)
                .map_err(|error| AuditError::Io(error.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&absolute)
                .map_err(|error| AuditError::Io(format!("{relative}: {error}")))?;
            let module = module_for_source(&relative)?;
            let file = syn::parse_file(&text).map_err(|error| AuditError::SourceParse {
                path: relative.clone(),
                error: error.to_string(),
            })?;
            collect_source_items(
                &file.items,
                &module,
                &relative,
                None,
                &mut functions,
                &mut nominals,
                &mut enum_variants,
            )?;
            source_hasher.update(relative.as_bytes());
            source_hasher.update([0]);
            source_hasher.update(text.as_bytes());
            source_hasher.update([0xff]);
            files.push(SourceFile {
                module,
                path: relative,
                digest: sha256_hex(text.as_bytes()),
            });
        }

        validate_closed_enums(&enum_variants)?;
        let cargo_lock_digest = hash_file(&workspace_root.join("Cargo.lock"))?;
        let mut manifest_hasher = Sha256::new();
        let crate_manifest = manifest_dir
            .strip_prefix(&workspace_root)
            .map_err(|error| AuditError::Io(error.to_string()))?
            .join("Cargo.toml")
            .to_string_lossy()
            .replace('\\', "/");
        for (label, path) in [
            ("Cargo.toml".to_owned(), workspace_root.join("Cargo.toml")),
            (crate_manifest, manifest_dir.join("Cargo.toml")),
        ] {
            let bytes = fs::read(&path)
                .map_err(|error| AuditError::Io(format!("{}: {error}", path.display())))?;
            manifest_hasher.update(label.as_bytes());
            manifest_hasher.update([0]);
            manifest_hasher.update(bytes);
        }
        Ok(Self {
            manifest_dir,
            workspace_root,
            files,
            functions,
            nominals,
            enum_variants,
            source_digest: hex_digest(source_hasher.finalize()),
            cargo_lock_digest,
            cargo_manifest_digest: hex_digest(manifest_hasher.finalize()),
        })
    }

    fn source(&self, path: &str) -> Option<&SourceFile> {
        self.files.iter().find(|source| source.path == path)
    }

    fn resolve_nominal(&self, spelling: &str) -> Result<Option<String>, AuditError> {
        let suffix = format!("::{spelling}");
        let candidates: Vec<_> = self
            .nominals
            .iter()
            .filter(|candidate| candidate.as_str() == spelling || candidate.ends_with(&suffix))
            .cloned()
            .collect();
        match candidates.as_slice() {
            [] => Ok(None),
            [candidate] => Ok(Some(candidate.clone())),
            _ => Err(AuditError::AmbiguousSource(format!(
                "nominal type {spelling} resolves to {candidates:?}"
            ))),
        }
    }

    fn resolve_source_owner(
        &self,
        module: &str,
        spelling: &str,
    ) -> Result<Option<String>, AuditError> {
        self.resolve_nominal_in_module(module, spelling)
    }

    fn resolve_nominal_in_module(
        &self,
        module: &str,
        spelling: &str,
    ) -> Result<Option<String>, AuditError> {
        if self.nominals.contains(spelling) {
            return Ok(Some(spelling.to_owned()));
        }
        let local = qualified_source_name(module, spelling);
        if self.nominals.contains(&local) {
            return Ok(Some(local));
        }
        let suffix = format!("::{spelling}");
        let visible: Vec<_> = self
            .nominals
            .iter()
            .filter(|candidate| {
                (candidate.as_str() == spelling || candidate.ends_with(&suffix))
                    && nominal_visible_from(module, candidate)
            })
            .cloned()
            .collect();
        match visible.as_slice() {
            [candidate] => Ok(Some(candidate.clone())),
            [] => self.resolve_nominal(spelling),
            _ => Err(AuditError::AmbiguousSource(format!(
                "nominal type {spelling} in module {module} resolves to {visible:?}"
            ))),
        }
    }

    fn bind_function(&self, display: &str, declaration: &str) -> Result<SourceBinding, AuditError> {
        let name = terminal_name(display);
        let source_ref = extract_source_ref(declaration);
        if display.contains("{closure") {
            let (path, line, column) =
                source_ref.ok_or_else(|| AuditError::UnboundMirFunction {
                    function: display.to_owned(),
                })?;
            let source = self
                .source(&path)
                .ok_or_else(|| AuditError::MissingSourceMirror(path.clone()))?;
            return Ok(SourceBinding {
                id: format!("{path}:{line}:{column}::{name}"),
                module: source.module.clone(),
                owner: None,
                path,
                line,
                column,
            });
        }

        let mut candidates: Vec<&SourceFunction> = self
            .functions
            .iter()
            .filter(|function| function.name == name)
            .collect();
        if source_ref.is_none() && top_level_path_segments(display).len() == 1 {
            candidates.retain(|function| function.owner.is_none());
        }
        if let Some((path, line, _)) = &source_ref {
            candidates.retain(|function| {
                function.path == *path
                    && (function.impl_line == Some(*line) || function.line == *line)
            });
        }
        if candidates.len() > 1 {
            let mut display_segments = top_level_path_segments(display);
            display_segments.pop();
            let display_module = display_segments.join("::");
            candidates.retain(|function| {
                display_module.ends_with(&function.module)
                    || function.module.ends_with(&display_module)
            });
        }
        if candidates.len() == 1 {
            let function = candidates[0];
            return Ok(SourceBinding {
                id: function.id.clone(),
                module: function.module.clone(),
                owner: function.owner.clone(),
                path: function.path.clone(),
                line: function.line,
                column: function.column,
            });
        }
        if let Some((path, line, column)) = source_ref {
            let source = self
                .source(&path)
                .ok_or_else(|| AuditError::MissingSourceMirror(path.clone()))?;
            return Ok(SourceBinding {
                id: format!("{path}:{line}:{column}::{name}"),
                module: source.module.clone(),
                owner: None,
                path,
                line,
                column,
            });
        }
        Err(AuditError::UnboundMirFunction {
            function: display.to_owned(),
        })
    }
}

#[derive(Clone, Debug)]
struct SourceBinding {
    id: String,
    module: String,
    owner: Option<String>,
    path: String,
    line: usize,
    column: usize,
}

fn rust_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), AuditError> {
    let entries = fs::read_dir(directory)
        .map_err(|error| AuditError::Io(format!("{}: {error}", directory.display())))?;
    for entry in entries {
        let entry = entry.map_err(|error| AuditError::Io(error.to_string()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| AuditError::Io(error.to_string()))?;
        if file_type.is_dir() {
            rust_files(&path, output)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == "rs") {
            output.push(path);
        }
    }
    Ok(())
}

fn module_for_source(path: &str) -> Result<String, AuditError> {
    let path = path
        .strip_prefix("src/")
        .ok_or_else(|| AuditError::Io(format!("source is outside src: {path}")))?;
    if path == "lib.rs" {
        return Ok(String::new());
    }
    let mut pieces: Vec<&str> = path.split('/').collect();
    let file = pieces
        .pop()
        .ok_or_else(|| AuditError::Io(format!("invalid source path: {path}")))?;
    let stem = file
        .strip_suffix(".rs")
        .ok_or_else(|| AuditError::Io(format!("non-Rust source: {path}")))?;
    if stem != "mod" {
        pieces.push(stem);
    }
    Ok(pieces.join("::"))
}

fn nominal_visible_from(module: &str, nominal: &str) -> bool {
    let namespace = nominal.rsplit_once("::").map_or("", |(prefix, _)| prefix);
    match module {
        "artifacts" => matches!(namespace, "artifacts" | "compiler_package_v3::types"),
        "compiler_package_v3::codec" => matches!(
            namespace,
            "compiler_package_v3::codec" | "compiler_package_v3::types"
        ),
        "compiler_package_v3::manifest" => matches!(
            namespace,
            "compiler_package_v3::codec" | "compiler_package_v3::types"
        ),
        "compiler_package_v3::types" => namespace == "compiler_package_v3::types",
        "evaluator" => matches!(
            namespace,
            "evaluator" | "compiler_package_v3::types" | "physical"
        ),
        "physical" => matches!(namespace, "physical" | "compiler_package_v3::types"),
        _ => namespace == module,
    }
}

fn collect_source_items(
    items: &[Item],
    module: &str,
    path: &str,
    inline_owner: Option<&str>,
    functions: &mut Vec<SourceFunction>,
    nominals: &mut BTreeSet<String>,
    enum_variants: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<(), AuditError> {
    for item in items {
        match item {
            Item::Fn(function) => {
                collect_source_function(module, path, inline_owner, None, function, functions);
                collect_nested_source_functions(module, path, &function.block, functions);
                collect_local_source_nominals(
                    module,
                    &function.sig.ident.to_string(),
                    &function.block,
                    nominals,
                    enum_variants,
                )?;
            }
            Item::Impl(implementation) => {
                let owner = nominal_type(&canonical_tokens(&implementation.self_ty));
                let impl_line = implementation.span().start().line;
                for item in &implementation.items {
                    if let ImplItem::Fn(function) = item {
                        let item = ItemFn {
                            attrs: function.attrs.clone(),
                            vis: function.vis.clone(),
                            sig: function.sig.clone(),
                            block: Box::new(function.block.clone()),
                        };
                        collect_source_function(
                            module,
                            path,
                            Some(&owner),
                            Some(impl_line),
                            &item,
                            functions,
                        );
                        collect_nested_source_functions(module, path, &function.block, functions);
                        collect_local_source_nominals(
                            module,
                            &function.sig.ident.to_string(),
                            &function.block,
                            nominals,
                            enum_variants,
                        )?;
                    }
                }
            }
            Item::Struct(item) => {
                let qualified = qualified_source_name(module, &item.ident.to_string());
                if !nominals.insert(qualified.clone()) {
                    return Err(AuditError::AmbiguousSource(format!(
                        "duplicate nominal type {qualified}"
                    )));
                }
            }
            Item::Enum(item) => {
                let name = item.ident.to_string();
                let qualified = qualified_source_name(module, &name);
                if !nominals.insert(qualified.clone()) {
                    return Err(AuditError::AmbiguousSource(format!(
                        "duplicate nominal type {qualified}"
                    )));
                }
                let variants: BTreeSet<_> = item
                    .variants
                    .iter()
                    .map(|variant| variant.ident.to_string())
                    .collect();
                if enum_variants.insert(qualified.clone(), variants).is_some() {
                    return Err(AuditError::AmbiguousSource(format!(
                        "duplicate semantic enum {qualified}"
                    )));
                }
            }
            Item::Mod(item) => {
                if let Some((_, nested)) = &item.content {
                    let nested_module = if module.is_empty() {
                        item.ident.to_string()
                    } else {
                        format!("{module}::{}", item.ident)
                    };
                    collect_source_items(
                        nested,
                        &nested_module,
                        path,
                        inline_owner,
                        functions,
                        nominals,
                        enum_variants,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_nested_source_functions(
    module: &str,
    path: &str,
    block: &syn::Block,
    functions: &mut Vec<SourceFunction>,
) {
    struct Collector<'a> {
        module: &'a str,
        path: &'a str,
        functions: &'a mut Vec<SourceFunction>,
    }

    impl<'ast> Visit<'ast> for Collector<'_> {
        fn visit_item_fn(&mut self, function: &'ast ItemFn) {
            collect_source_function(self.module, self.path, None, None, function, self.functions);
            visit::visit_item_fn(self, function);
        }
    }

    Collector {
        module,
        path,
        functions,
    }
    .visit_block(block);
}

fn collect_local_source_nominals(
    module: &str,
    function: &str,
    block: &syn::Block,
    nominals: &mut BTreeSet<String>,
    enum_variants: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<(), AuditError> {
    #[derive(Default)]
    struct Collector {
        enums: Vec<(String, BTreeSet<String>)>,
    }

    impl<'ast> Visit<'ast> for Collector {
        fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
            self.enums.push((
                item.ident.to_string(),
                item.variants
                    .iter()
                    .map(|variant| variant.ident.to_string())
                    .collect(),
            ));
        }

        fn visit_item_fn(&mut self, _function: &'ast ItemFn) {}
    }

    let mut collector = Collector::default();
    collector.visit_block(block);
    for (name, variants) in collector.enums {
        let qualified = qualified_source_name(module, &format!("{function}::{name}"));
        if !nominals.insert(qualified.clone())
            || enum_variants.insert(qualified.clone(), variants).is_some()
        {
            return Err(AuditError::AmbiguousSource(format!(
                "duplicate local nominal type {qualified}"
            )));
        }
    }
    Ok(())
}

fn qualified_source_name(module: &str, name: &str) -> String {
    if module.is_empty() {
        name.to_owned()
    } else {
        format!("{module}::{name}")
    }
}

fn collect_source_function(
    module: &str,
    path: &str,
    owner: Option<&str>,
    impl_line: Option<usize>,
    function: &ItemFn,
    output: &mut Vec<SourceFunction>,
) {
    let name = function.sig.ident.to_string();
    let id = match (module.is_empty(), owner) {
        (true, Some(owner)) => format!("{owner}::{name}"),
        (true, None) => name.clone(),
        (false, Some(owner)) => format!("{module}::{owner}::{name}"),
        (false, None) => format!("{module}::{name}"),
    };
    let start = function.span().start();
    output.push(SourceFunction {
        id,
        module: module.to_owned(),
        owner: owner.map(ToOwned::to_owned),
        name,
        path: path.to_owned(),
        line: start.line,
        column: start.column + 1,
        impl_line,
    });
}

fn validate_closed_enums(actual: &BTreeMap<String, BTreeSet<String>>) -> Result<(), AuditError> {
    for (name, expected) in CLOSED_ENUMS {
        let expected: BTreeSet<_> = expected.iter().map(|value| (*value).to_owned()).collect();
        let observed = actual
            .get(*name)
            .ok_or_else(|| AuditError::ClosedEnumChanged {
                name: (*name).to_owned(),
                expected: expected.clone(),
                actual: BTreeSet::new(),
            })?;
        if observed != &expected {
            return Err(AuditError::ClosedEnumChanged {
                name: (*name).to_owned(),
                expected,
                actual: observed.clone(),
            });
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct CompilerEvidenceArtifact {
    mir_text: String,
    mir_digest: String,
    rustc_digest: String,
    cfg_digest: String,
    features: String,
    binding_digest: String,
}

impl CompilerEvidenceArtifact {
    fn generate(inventory: &SourceInventory) -> Result<Self, AuditError> {
        let target = std::env::temp_dir().join(format!(
            "clause-host-mechanics-{}-{}",
            &inventory.source_digest[..16],
            std::process::id()
        ));
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let output = Command::new(cargo)
            .current_dir(&inventory.workspace_root)
            .env("CARGO_TARGET_DIR", &target)
            .env("CARGO_TERM_COLOR", "never")
            .args([
                "rustc",
                "--locked",
                "--manifest-path",
                inventory
                    .manifest_dir
                    .join("Cargo.toml")
                    .to_str()
                    .ok_or_else(|| AuditError::Io("non-UTF8 manifest path".to_owned()))?,
                "--lib",
                "--",
                "--emit=mir",
            ])
            .output()
            .map_err(|error| AuditError::Compiler(error.to_string()))?;
        if !output.status.success() {
            return Err(AuditError::Compiler(format!(
                "cargo rustc failed with {}\nstdout:\n{}\nstderr:\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let mut mir_paths = Vec::new();
        files_with_extension(&target, "mir", &mut mir_paths)?;
        mir_paths.retain(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("clause_substrate-"))
        });
        if mir_paths.len() != 1 {
            return Err(AuditError::Compiler(format!(
                "expected one clause-substrate MIR artifact, found {} under {}",
                mir_paths.len(),
                target.display()
            )));
        }
        let mir_text = fs::read_to_string(&mir_paths[0])
            .map_err(|error| AuditError::Io(format!("{}: {error}", mir_paths[0].display())))?;
        let mir_digest = sha256_hex(mir_text.as_bytes());

        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let rustc_output = Command::new(&rustc)
            .arg("-Vv")
            .output()
            .map_err(|error| AuditError::Compiler(error.to_string()))?;
        if !rustc_output.status.success() {
            return Err(AuditError::Compiler(format!(
                "rustc -Vv failed with {}",
                rustc_output.status
            )));
        }
        let rustc_verbose = String::from_utf8(rustc_output.stdout)
            .map_err(|error| AuditError::Compiler(error.to_string()))?;
        let rustc_digest = sha256_hex(rustc_verbose.as_bytes());
        let cfg_output = Command::new(&rustc)
            .args(["--print", "cfg"])
            .output()
            .map_err(|error| AuditError::Compiler(error.to_string()))?;
        if !cfg_output.status.success() {
            return Err(AuditError::Compiler(format!(
                "rustc --print cfg failed with {}",
                cfg_output.status
            )));
        }
        let mut cfg_lines: Vec<_> = String::from_utf8(cfg_output.stdout)
            .map_err(|error| AuditError::Compiler(error.to_string()))?
            .lines()
            .map(ToOwned::to_owned)
            .collect();
        cfg_lines.sort();
        let cfg = cfg_lines.join("|");
        let cfg_digest = sha256_hex(cfg.as_bytes());
        let mut features: Vec<_> = std::env::vars()
            .filter(|(name, _)| name.starts_with("CARGO_FEATURE_"))
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        features.sort();
        let features = if features.is_empty() {
            "-".to_owned()
        } else {
            features.join("|")
        };
        let binding_digest = sha256_hex(
            format!(
                "{AUDIT_SCHEMA}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
                inventory.source_digest,
                inventory.cargo_lock_digest,
                inventory.cargo_manifest_digest,
                mir_digest,
                rustc_digest,
                cfg_digest,
                features
            )
            .as_bytes(),
        );
        Ok(Self {
            mir_text,
            mir_digest,
            rustc_digest,
            cfg_digest,
            features,
            binding_digest,
        })
    }
}

fn files_with_extension(
    directory: &Path,
    extension: &str,
    output: &mut Vec<PathBuf>,
) -> Result<(), AuditError> {
    let entries = fs::read_dir(directory)
        .map_err(|error| AuditError::Io(format!("{}: {error}", directory.display())))?;
    for entry in entries {
        let entry = entry.map_err(|error| AuditError::Io(error.to_string()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| AuditError::Io(error.to_string()))?;
        if file_type.is_dir() {
            files_with_extension(&path, extension, output)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == extension) {
            output.push(path);
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct MirAssignment {
    ordinal: usize,
    destination: String,
    value: String,
    dependencies: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct MirCall {
    block: String,
    ordinal: usize,
    result: String,
    target: String,
    arguments: Vec<String>,
    closure_refs: BTreeSet<String>,
    successors: Vec<String>,
    raw: String,
}

#[derive(Clone, Debug)]
struct MirSwitch {
    block: String,
    ordinal: usize,
    operand: String,
    successors: Vec<String>,
    arms: String,
}

#[derive(Clone, Debug)]
struct MirTerminator {
    block: String,
    ordinal: usize,
    kind: SiteKind,
    control: String,
    successors: Vec<String>,
    raw: String,
}

#[derive(Clone, Debug)]
struct MirBlock {
    id: String,
    assignments: Vec<MirAssignment>,
    calls: Vec<MirCall>,
    switches: Vec<MirSwitch>,
    terminator: Option<MirTerminator>,
}

#[derive(Clone, Debug)]
struct MirFunction {
    display: String,
    return_type: String,
    parameter_types: Vec<String>,
    locals: BTreeMap<String, String>,
    debug_names: BTreeMap<String, String>,
    blocks: BTreeMap<String, MirBlock>,
    source: Option<SourceBinding>,
}

impl MirFunction {
    fn local_type(&self, operand: &str) -> String {
        let mut types = BTreeSet::new();
        for local in local_references(operand) {
            if let Some(ty) = self.locals.get(&local) {
                types.insert(one_line(ty));
            }
        }
        if types.is_empty() {
            "constant".to_owned()
        } else {
            types.into_iter().collect::<Vec<_>>().join("|")
        }
    }

    fn all_calls(&self) -> impl Iterator<Item = &MirCall> {
        self.blocks.values().flat_map(|block| block.calls.iter())
    }

    fn all_assignments(&self) -> impl Iterator<Item = &MirAssignment> {
        self.blocks
            .values()
            .flat_map(|block| block.assignments.iter())
    }
}

#[derive(Clone, Debug)]
struct MirProgram {
    functions: BTreeMap<String, MirFunction>,
}

impl MirProgram {
    fn parse(text: &str, inventory: &SourceInventory) -> Result<Self, AuditError> {
        let lines: Vec<&str> = text.lines().collect();
        let mut functions = BTreeMap::new();
        let mut index = 0;
        while index < lines.len() {
            if !lines[index].starts_with("fn ") {
                index += 1;
                continue;
            }
            let ctfe = index > 0 && lines[index - 1].trim() == "// MIR FOR CTFE";
            let start = index;
            index += 1;
            while index < lines.len() && lines[index] != "}" {
                index += 1;
            }
            if index == lines.len() {
                return Err(AuditError::MirParse(format!(
                    "unterminated function at MIR line {}",
                    start + 1
                )));
            }
            if !ctfe {
                let function = parse_mir_function(&lines[start..=index], inventory)?;
                if functions
                    .insert(function.display.clone(), function)
                    .is_some()
                {
                    return Err(AuditError::MirParse(format!(
                        "duplicate non-CTFE MIR function at line {}",
                        start + 1
                    )));
                }
            }
            index += 1;
        }
        if functions.is_empty() {
            return Err(AuditError::MirParse(
                "compiler emitted no MIR functions".to_owned(),
            ));
        }
        Ok(Self { functions })
    }

    fn resolve_root(&self, root: &str) -> Result<String, AuditError> {
        let candidates: Vec<_> = self
            .functions
            .iter()
            .filter(|(_, function)| {
                function
                    .source
                    .as_ref()
                    .is_some_and(|source| source.id == root)
            })
            .map(|(id, _)| id.clone())
            .collect();
        if candidates.len() == 1 {
            Ok(candidates[0].clone())
        } else {
            let root_name = terminal_name(root);
            let related = self
                .functions
                .iter()
                .filter_map(|(id, function)| {
                    function.source.as_ref().and_then(|source| {
                        (terminal_name(&source.id) == root_name)
                            .then(|| format!("{id}=>{}", source.id))
                    })
                })
                .collect();
            Err(AuditError::MissingRoot {
                root: root.to_owned(),
                candidates: related,
            })
        }
    }
}

#[derive(Clone, Debug)]
struct ProductionInput {
    inventory: SourceInventory,
    compiler: CompilerEvidenceArtifact,
    program: MirProgram,
}

static PRODUCTION_INPUT: OnceLock<Result<ProductionInput, AuditError>> = OnceLock::new();

fn production_input() -> Result<&'static ProductionInput, AuditError> {
    match PRODUCTION_INPUT.get_or_init(|| {
        let inventory = SourceInventory::load()?;
        let compiler = CompilerEvidenceArtifact::generate(&inventory)?;
        let program = MirProgram::parse(&compiler.mir_text, &inventory)?;
        Ok(ProductionInput {
            inventory,
            compiler,
            program,
        })
    }) {
        Ok(input) => Ok(input),
        Err(error) => Err(error.clone()),
    }
}

fn parse_mir_function(
    lines: &[&str],
    inventory: &SourceInventory,
) -> Result<MirFunction, AuditError> {
    let heading = lines[0].to_owned();
    let display = function_display(&heading)?;
    let (parameter_types, return_type) = function_signature(&heading)?;
    let source = match inventory.bind_function(&display, &heading) {
        Ok(source) => Some(source),
        Err(AuditError::UnboundMirFunction { .. }) => None,
        Err(error) => return Err(error),
    };
    let mut locals = BTreeMap::new();
    locals.insert("_0".to_owned(), return_type.clone());
    for (index, ty) in parameter_types.iter().enumerate() {
        locals.insert(format!("_{}", index + 1), ty.clone());
    }
    let mut debug_names = BTreeMap::new();
    let mut blocks = BTreeMap::new();
    let mut current: Option<MirBlock> = None;
    let mut ordinal = 0;

    for line in &lines[1..lines.len() - 1] {
        let trimmed = line.trim();
        if current.is_none() {
            if let Some((local, ty)) = parse_local_declaration(trimmed) {
                locals.insert(local, ty);
                continue;
            }
            if let Some((name, local)) = parse_debug_binding(trimmed) {
                debug_names.insert(local, name);
                continue;
            }
        }
        if is_basic_block_header(trimmed) {
            if let Some(block) = current.take() {
                finish_block(block, &mut blocks, &display)?;
            }
            let id = trimmed
                .split([' ', ':'])
                .next()
                .expect("basic block header has an identifier")
                .to_owned();
            current = Some(MirBlock {
                id,
                assignments: Vec::new(),
                calls: Vec::new(),
                switches: Vec::new(),
                terminator: None,
            });
            ordinal = 0;
            continue;
        }
        let Some(block) = current.as_mut() else {
            continue;
        };
        if trimmed == "}" || trimmed.is_empty() {
            continue;
        }
        ordinal += 1;
        if let Some((destination, value)) = parse_assignment(trimmed) {
            let destination = base_local(&destination).unwrap_or(destination);
            block.assignments.push(MirAssignment {
                ordinal,
                dependencies: local_references(&value),
                destination: destination.clone(),
                value: value.clone(),
            });
            let diverging_call = value
                .rsplit_once(" -> ")
                .is_some_and(|(_, successor)| is_basic_block_id(successor));
            if value.contains(" -> [") || diverging_call {
                let (target, arguments) = call_parts(&value).ok_or_else(|| {
                    AuditError::MirParse(format!(
                        "{display} {}:{ordinal}: malformed call {trimmed}",
                        block.id
                    ))
                })?;
                let call = MirCall {
                    block: block.id.clone(),
                    ordinal,
                    result: destination,
                    closure_refs: closure_references(trimmed),
                    successors: basic_block_references(trimmed),
                    target,
                    arguments,
                    raw: trimmed.to_owned(),
                };
                block.calls.push(call);
                block.terminator = Some(MirTerminator {
                    block: block.id.clone(),
                    ordinal,
                    kind: SiteKind::Call,
                    control: trimmed.to_owned(),
                    successors: basic_block_references(trimmed),
                    raw: trimmed.to_owned(),
                });
                continue;
            }
        }
        if let Some((operand, arms)) = parse_switch(trimmed) {
            let switch = MirSwitch {
                block: block.id.clone(),
                ordinal,
                operand: operand.clone(),
                successors: basic_block_references(trimmed),
                arms,
            };
            block.switches.push(switch);
            block.terminator = Some(MirTerminator {
                block: block.id.clone(),
                ordinal,
                kind: SiteKind::Switch,
                control: operand,
                successors: basic_block_references(trimmed),
                raw: trimmed.to_owned(),
            });
        } else if is_terminator(trimmed) {
            block.terminator = Some(MirTerminator {
                block: block.id.clone(),
                ordinal,
                kind: terminator_kind(trimmed),
                control: trimmed.to_owned(),
                successors: basic_block_references(trimmed),
                raw: trimmed.to_owned(),
            });
        }
    }
    if let Some(block) = current {
        finish_block(block, &mut blocks, &display)?;
    }
    Ok(MirFunction {
        display,
        return_type,
        parameter_types,
        locals,
        debug_names,
        blocks,
        source,
    })
}

fn finish_block(
    block: MirBlock,
    blocks: &mut BTreeMap<String, MirBlock>,
    function: &str,
) -> Result<(), AuditError> {
    if block.terminator.is_none() {
        return Err(AuditError::MirParse(format!(
            "{function} {} has no recognized terminator",
            block.id
        )));
    }
    if blocks.insert(block.id.clone(), block).is_some() {
        return Err(AuditError::MirParse(format!(
            "{function} has a duplicate basic block"
        )));
    }
    Ok(())
}

fn function_display(heading: &str) -> Result<String, AuditError> {
    let body = heading
        .strip_prefix("fn ")
        .ok_or_else(|| AuditError::MirParse(format!("invalid function heading: {heading}")))?;
    let open = body
        .find('(')
        .ok_or_else(|| AuditError::MirParse(format!("function has no parameters: {heading}")))?;
    Ok(body[..open].trim().to_owned())
}

fn function_signature(heading: &str) -> Result<(Vec<String>, String), AuditError> {
    let body = heading
        .strip_prefix("fn ")
        .ok_or_else(|| AuditError::MirParse(format!("invalid function heading: {heading}")))?;
    let (_, return_type) = body
        .rsplit_once(") -> ")
        .ok_or_else(|| AuditError::MirParse(format!("function has no return type: {heading}")))?;
    let open = body
        .find('(')
        .ok_or_else(|| AuditError::MirParse(format!("function has no parameters: {heading}")))?;
    let close = body
        .len()
        .checked_sub(return_type.len() + 5)
        .ok_or_else(|| AuditError::MirParse(format!("invalid signature: {heading}")))?;
    let parameters = split_top_level(&body[open + 1..close], ',')
        .into_iter()
        .filter(|parameter| !parameter.trim().is_empty())
        .map(|parameter| {
            parameter
                .split_once(':')
                .map(|(_, ty)| one_line(ty))
                .ok_or_else(|| {
                    AuditError::MirParse(format!("invalid parameter in {heading}: {parameter}"))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        parameters,
        return_type
            .strip_suffix(" {")
            .unwrap_or(return_type)
            .trim()
            .to_owned(),
    ))
}

fn parse_local_declaration(line: &str) -> Option<(String, String)> {
    let line = line.strip_prefix("let ")?;
    let line = line.strip_prefix("mut ").unwrap_or(line);
    let (local, ty) = line.split_once(':')?;
    let local = local.trim();
    if !is_local(local) {
        return None;
    }
    Some((
        local.to_owned(),
        ty.trim().strip_suffix(';').unwrap_or(ty.trim()).to_owned(),
    ))
}

fn parse_debug_binding(line: &str) -> Option<(String, String)> {
    let line = line.strip_prefix("debug ")?;
    let (name, local) = line.split_once(" => ")?;
    let local = base_local(local)?;
    Some((name.trim().to_owned(), local))
}

fn is_basic_block_header(line: &str) -> bool {
    line.starts_with("bb")
        && line.as_bytes().get(2).is_some_and(u8::is_ascii_digit)
        && line.ends_with('{')
}

fn is_basic_block_id(value: &str) -> bool {
    value.strip_prefix("bb").is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn parse_assignment(line: &str) -> Option<(String, String)> {
    let (left, right) = line.split_once(" = ")?;
    let destination = base_local(left)?;
    Some((
        destination,
        right
            .trim()
            .strip_suffix(';')
            .unwrap_or(right.trim())
            .to_owned(),
    ))
}

fn call_parts(value: &str) -> Option<(String, Vec<String>)> {
    let (call, continuation) = value.rsplit_once(" -> ")?;
    if !((continuation.starts_with('[') && continuation.ends_with(']'))
        || is_basic_block_id(continuation))
    {
        return None;
    }
    let open = top_level_open_paren(call)?;
    let close = matching_delimiter(call, open, '(', ')')?;
    if !call[close + 1..].trim().is_empty() {
        return None;
    }
    let target = call[..open].trim().to_owned();
    let arguments = split_top_level(&call[open + 1..close], ',')
        .into_iter()
        .filter(|argument| !argument.trim().is_empty())
        .map(|argument| argument.trim().to_owned())
        .collect();
    Some((target, arguments))
}

fn parse_switch(line: &str) -> Option<(String, String)> {
    let body = line.strip_prefix("switchInt(")?;
    let marker = ") -> [";
    let split = body.rfind(marker)?;
    Some((
        body[..split].trim().to_owned(),
        body[split + marker.len()..]
            .trim()
            .strip_suffix("];")
            .unwrap_or(&body[split + marker.len()..])
            .to_owned(),
    ))
}

fn is_terminator(line: &str) -> bool {
    line.starts_with("goto ->")
        || line == "return;"
        || line == "unreachable;"
        || line == "resume;"
        || line.starts_with("drop(")
        || line.starts_with("assert(")
        || line.starts_with("terminate(")
}

fn terminator_kind(line: &str) -> SiteKind {
    if line.starts_with("drop(") {
        SiteKind::Drop
    } else if line.starts_with("assert(") {
        SiteKind::Assert
    } else {
        SiteKind::Control
    }
}

fn top_level_open_paren(value: &str) -> Option<usize> {
    let mut angle = 0_usize;
    let mut brace = 0_usize;
    let mut bracket = 0_usize;
    for (index, character) in value.char_indices() {
        match character {
            '<' => angle += 1,
            '>' => angle = angle.saturating_sub(1),
            '{' => brace += 1,
            '}' => brace = brace.saturating_sub(1),
            '[' => bracket += 1,
            ']' => bracket = bracket.saturating_sub(1),
            '(' if angle == 0 && brace == 0 && bracket == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_delimiter(value: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, character) in value[start..].char_indices() {
        if character == open {
            depth += 1;
        } else if character == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

fn split_top_level(value: &str, separator: char) -> Vec<&str> {
    let mut output = Vec::new();
    let mut start = 0;
    let mut angle = 0_usize;
    let mut paren = 0_usize;
    let mut bracket = 0_usize;
    let mut brace = 0_usize;
    for (index, character) in value.char_indices() {
        match character {
            '<' => angle += 1,
            '>' => angle = angle.saturating_sub(1),
            '(' => paren += 1,
            ')' => paren = paren.saturating_sub(1),
            '[' => bracket += 1,
            ']' => bracket = bracket.saturating_sub(1),
            '{' => brace += 1,
            '}' => brace = brace.saturating_sub(1),
            _ if character == separator
                && angle == 0
                && paren == 0
                && bracket == 0
                && brace == 0 =>
            {
                output.push(&value[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    output.push(&value[start..]);
    output
}

fn local_references(value: &str) -> BTreeSet<String> {
    let bytes = value.as_bytes();
    let mut output = BTreeSet::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'_' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit) {
            let start = index;
            index += 2;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            output.insert(value[start..index].to_owned());
        } else {
            index += 1;
        }
    }
    output
}

fn base_local(value: &str) -> Option<String> {
    local_references(value).into_iter().next()
}

fn is_local(value: &str) -> bool {
    value.strip_prefix('_').is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn closure_references(value: &str) -> BTreeSet<String> {
    let mut output = BTreeSet::new();
    let mut rest = value;
    while let Some(start) = rest.find("{closure@") {
        let candidate = &rest[start..];
        let Some(end) = candidate.find('}') else {
            break;
        };
        output.insert(candidate[..=end].to_owned());
        rest = &candidate[end + 1..];
    }
    output
}

fn basic_block_references(value: &str) -> Vec<String> {
    let bytes = value.as_bytes();
    let mut output = BTreeSet::new();
    let mut index = 0;
    while index + 2 < bytes.len() {
        if bytes[index] == b'b' && bytes[index + 1] == b'b' && bytes[index + 2].is_ascii_digit() {
            let start = index;
            index += 3;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            output.insert(value[start..index].to_owned());
        } else {
            index += 1;
        }
    }
    output.into_iter().collect()
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SiteKind {
    Call,
    Switch,
    Assert,
    Drop,
    Control,
}

impl SiteKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Switch => "switch",
            Self::Assert => "assert",
            Self::Drop => "drop",
            Self::Control => "control",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Mechanic {
    WireCodec,
    CoreAbi,
    ByteMachine,
    DefinitionTable,
    KernelStep,
    ReceiptStep,
    AuthorizationStep,
    PhysicalDispatch,
}

impl Mechanic {
    const fn name(self) -> &'static str {
        match self {
            Self::WireCodec => "wire-codec",
            Self::CoreAbi => "core-abi",
            Self::ByteMachine => "byte-machine",
            Self::DefinitionTable => "definition-table",
            Self::KernelStep => "kernel-step",
            Self::ReceiptStep => "receipt-step",
            Self::AuthorizationStep => "authorization-step",
            Self::PhysicalDispatch => "physical-dispatch",
        }
    }

    const fn allowed_outcomes(self) -> &'static [&'static str] {
        match self {
            Self::WireCodec | Self::CoreAbi | Self::ReceiptStep | Self::AuthorizationStep => {
                &["canonical-data", "fixed-error"]
            }
            Self::ByteMachine | Self::KernelStep => {
                &["canonical-data", "child-KExpr", "fixed-error"]
            }
            Self::DefinitionTable => &["selected-package-definition", "fixed-error"],
            Self::PhysicalDispatch => &["fixed-Sha256-handler", "fixed-error"],
        }
    }

    fn outcome_set(self) -> BTreeSet<String> {
        self.allowed_outcomes()
            .iter()
            .map(|outcome| (*outcome).to_owned())
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct RootContract {
    source: &'static str,
    class: Mechanic,
    required_nominals: &'static [&'static str],
}

impl RootContract {
    const fn new(
        source: &'static str,
        class: Mechanic,
        required_nominals: &'static [&'static str],
    ) -> Self {
        Self {
            source,
            class,
            required_nominals,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ContextKey {
    function: String,
    class: Mechanic,
    witness: String,
    callbacks: Vec<(usize, String)>,
}

impl ContextKey {
    fn callback(&self, index: usize) -> Option<&str> {
        self.callbacks
            .iter()
            .find(|(candidate, _)| *candidate == index)
            .map(|(_, target)| target.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ContractKind {
    Platform,
    DigestState,
    DigestOneShot,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ExternalContract {
    kind: ContractKind,
    target: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ResolvedTarget {
    Local(ContextKey),
    External(ExternalContract),
}

impl ResolvedTarget {
    fn render(&self) -> String {
        match self {
            Self::Local(context) => format!("local::{}", context.function),
            Self::External(contract) => format!("external::{}", contract.target),
        }
    }
}

type CallKey = (ContextKey, String, usize);
type ContextClosure = (
    BTreeSet<String>,
    BTreeSet<ContextKey>,
    BTreeMap<CallKey, Vec<ResolvedTarget>>,
);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EvidenceRow {
    ir_location: String,
    source_location: String,
    function: String,
    kind: SiteKind,
    class: Mechanic,
    witness: String,
    influenced: bool,
    provenance: String,
    control_type: String,
    tag_signature: String,
    arm_map: String,
    allowed_outcomes: String,
    actual_outcomes: String,
    targets: Vec<String>,
    edges: Vec<String>,
}

impl EvidenceRow {
    fn render(&self) -> String {
        [
            self.ir_location.clone(),
            self.source_location.clone(),
            self.function.clone(),
            self.kind.name().to_owned(),
            self.class.name().to_owned(),
            self.witness.clone(),
            if self.influenced { "package" } else { "fixed" }.to_owned(),
            self.provenance.clone(),
            self.control_type.clone(),
            self.tag_signature.clone(),
            self.arm_map.clone(),
            self.allowed_outcomes.clone(),
            self.actual_outcomes.clone(),
            render_set(self.targets.iter().cloned()),
            render_set(self.edges.iter().cloned()),
        ]
        .into_iter()
        .map(|value| tsv_field(&value))
        .collect::<Vec<_>>()
        .join("\t")
    }
}

#[derive(Debug)]
struct Analysis {
    inventory: SourceInventory,
    compiler: CompilerEvidenceArtifact,
    roots: BTreeSet<String>,
    contexts: BTreeSet<ContextKey>,
    rows: Vec<EvidenceRow>,
    fixed_handlers: BTreeSet<String>,
}

impl Analysis {
    fn production() -> Result<Self, AuditError> {
        let input = production_input()?;
        let inventory = input.inventory.clone();
        let compiler = input.compiler.clone();
        let program = input.program.clone();
        let (roots, contexts, resolved_calls) = build_context_closure(&inventory, &program)?;
        audit_fallible_allocations(&program, &contexts, &resolved_calls)?;
        let provenance = derive_provenance(&program, &roots, &contexts, &resolved_calls)?;
        let handler_sets = derive_handler_sets(&program, &contexts, &resolved_calls);
        let mut fixed_handlers = BTreeSet::new();
        for context in &contexts {
            if context.class == Mechanic::PhysicalDispatch {
                fixed_handlers.extend(handler_sets.get(context).into_iter().flatten().cloned());
            }
        }
        if fixed_handlers.is_empty() {
            return Err(AuditError::FixedDispatch(
                "physical closure contains no fixed handler".to_owned(),
            ));
        }
        let rows = build_rows(
            &inventory,
            &program,
            &contexts,
            &resolved_calls,
            &provenance,
            &handler_sets,
        )?;
        Ok(Self {
            inventory,
            compiler,
            roots,
            contexts,
            rows,
            fixed_handlers,
        })
    }

    fn summary_evidence(&self) -> String {
        let mut counts: BTreeMap<Mechanic, (usize, usize, BTreeSet<String>)> = BTreeMap::new();
        for row in &self.rows {
            let entry = counts.entry(row.class).or_default();
            entry.0 += 1;
            entry.1 += usize::from(row.influenced);
            entry.2.extend(row.targets.iter().cloned());
        }
        let mut output = String::from(
            "schema\tsource_sha256\tmir_sha256\tcargo_lock_sha256\tcargo_manifest_sha256\trustc_sha256\tcfg_sha256\tfeatures\tbinding_sha256\troot_count\tcontext_count\tsite_count\tfixed_handlers\n",
        );
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            AUDIT_SCHEMA,
            self.inventory.source_digest,
            self.compiler.mir_digest,
            self.inventory.cargo_lock_digest,
            self.inventory.cargo_manifest_digest,
            self.compiler.rustc_digest,
            self.compiler.cfg_digest,
            tsv_field(&self.compiler.features),
            self.compiler.binding_digest,
            self.roots.len(),
            self.contexts.len(),
            self.rows.len(),
            render_set(self.fixed_handlers.iter().cloned())
        )
        .expect("writing to String cannot fail");
        output
            .push_str("class\treachable_sites\tpackage_influenced_sites\tresolved_target_count\n");
        for (class, (sites, influenced, targets)) in counts {
            writeln!(
                output,
                "{}\t{}\t{}\t{}",
                class.name(),
                sites,
                influenced,
                targets.len()
            )
            .expect("writing to String cannot fail");
        }
        output
    }

    fn site_evidence(&self) -> String {
        let mut output = String::from("record\tkey\tvalue\nmetadata\tschema\t");
        output.push_str(AUDIT_SCHEMA);
        output.push('\n');
        for (key, value) in [
            ("source_sha256", self.inventory.source_digest.as_str()),
            ("mir_sha256", self.compiler.mir_digest.as_str()),
            (
                "cargo_lock_sha256",
                self.inventory.cargo_lock_digest.as_str(),
            ),
            (
                "cargo_manifest_sha256",
                self.inventory.cargo_manifest_digest.as_str(),
            ),
            ("rustc_sha256", self.compiler.rustc_digest.as_str()),
            ("cfg_sha256", self.compiler.cfg_digest.as_str()),
            ("features", self.compiler.features.as_str()),
            ("binding_sha256", self.compiler.binding_digest.as_str()),
        ] {
            writeln!(output, "metadata\t{}\t{}", key, tsv_field(value))
                .expect("writing to String cannot fail");
        }
        for source in &self.inventory.files {
            writeln!(
                output,
                "source\t{}\t{}|{}",
                source.path, source.module, source.digest
            )
            .expect("writing to String cannot fail");
        }
        output.push_str(
            "ir_location\tsource_location\tfunction\tkind\tclass\tclass_witness\tinfluence\tprovenance\tcontrol_type\ttag_signature\tarm_map\tallowed_outcomes\tactual_outcomes\tcode_targets\tedges\n",
        );
        for row in &self.rows {
            output.push_str(&row.render());
            output.push('\n');
        }
        output
    }
}

fn build_context_closure(
    inventory: &SourceInventory,
    program: &MirProgram,
) -> Result<ContextClosure, AuditError> {
    let mut roots = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut queue = VecDeque::new();
    for root in ROOTS {
        let function = program.resolve_root(root.source)?;
        let body = &program.functions[&function];
        let nominals = validate_function_nominals(inventory, body)?;
        let missing: BTreeSet<_> = root
            .required_nominals
            .iter()
            .filter(|required| !nominals.contains(**required))
            .map(|required| (*required).to_owned())
            .collect();
        if !missing.is_empty() {
            return Err(AuditError::Unclassified {
                function: function.clone(),
                detail: format!("constitutional root is missing exact carriers {missing:?}"),
            });
        }
        let context = ContextKey {
            function: function.clone(),
            class: root.class,
            witness: format!(
                "root-contract:{};carriers:{}",
                root.source,
                root.required_nominals.join("|")
            ),
            callbacks: Vec::new(),
        };
        roots.insert(function);
        if contexts.insert(context.clone()) {
            queue.push_back(context);
        }
    }

    let mut resolved_calls = BTreeMap::new();
    while let Some(context) = queue.pop_front() {
        let function = &program.functions[&context.function];
        if function.source.is_none() {
            return Err(AuditError::UnboundMirFunction {
                function: context.function.clone(),
            });
        }
        for call in function.all_calls() {
            let targets = resolve_call(inventory, program, &context, call)?;
            if targets.is_empty() {
                return Err(AuditError::UnresolvedTarget {
                    function: context.function.clone(),
                    target: call.target.clone(),
                });
            }
            for target in &targets {
                if let ResolvedTarget::Local(child) = target
                    && contexts.insert(child.clone())
                {
                    queue.push_back(child.clone());
                }
            }
            let key = (context.clone(), call.block.clone(), call.ordinal);
            if resolved_calls.insert(key, targets).is_some() {
                return Err(AuditError::MirParse(format!(
                    "duplicate call site in {} {}:{}",
                    context.function, call.block, call.ordinal
                )));
            }
        }
    }
    Ok((roots, contexts, resolved_calls))
}

fn resolve_call(
    inventory: &SourceInventory,
    program: &MirProgram,
    context: &ContextKey,
    call: &MirCall,
) -> Result<Vec<ResolvedTarget>, AuditError> {
    reject_dynamic_target(&call.target)?;
    if is_generic_callable_target(&call.target) {
        let argument = call
            .arguments
            .first()
            .ok_or_else(|| AuditError::DynamicCall {
                function: context.function.clone(),
                target: call.target.clone(),
            })?;
        let function = &program.functions[&context.function];
        let callable_parameters: Vec<_> = dependency_locals(function, argument)
            .into_iter()
            .filter_map(|local| {
                local
                    .strip_prefix('_')
                    .and_then(|digits| digits.parse::<usize>().ok())
                    .and_then(|value| value.checked_sub(1))
            })
            .filter(|index| {
                function
                    .parameter_types
                    .get(*index)
                    .is_some_and(|ty| type_is_callable_text(ty))
            })
            .collect();
        let index = match callable_parameters.as_slice() {
            [index] => *index,
            _ => {
                return Err(AuditError::DynamicCall {
                    function: context.function.clone(),
                    target: format!(
                        "{} via {} => callable parameters {callable_parameters:?}",
                        call.target, argument
                    ),
                });
            }
        };
        let target = context
            .callback(index)
            .ok_or_else(|| AuditError::DynamicCall {
                function: context.function.clone(),
                target: format!("{} via unbound callback parameter {index}", call.target),
            })?;
        let child = child_context(inventory, program, target, context, Vec::new())?;
        return Ok(vec![ResolvedTarget::Local(child)]);
    }

    if let Some(target) = resolve_closure_target(program, &call.target)? {
        return Ok(vec![ResolvedTarget::Local(child_context(
            inventory,
            program,
            &target,
            context,
            Vec::new(),
        )?)]);
    }
    if let Some(target) = resolve_local_target(program, &context.function, &call.target)? {
        let callbacks = callable_bindings(program, &target, call)?;
        return Ok(vec![ResolvedTarget::Local(child_context(
            inventory, program, &target, context, callbacks,
        )?)]);
    }
    let contract = external_contract(&call.target)?;
    let mut targets = vec![ResolvedTarget::External(contract)];
    for closure in &call.closure_refs {
        let target = resolve_closure_reference(program, closure)?;
        targets.push(ResolvedTarget::Local(child_context(
            inventory,
            program,
            &target,
            context,
            Vec::new(),
        )?));
    }
    let mut seen = BTreeSet::new();
    targets.retain(|target| seen.insert(target.clone()));
    Ok(targets)
}

fn child_context(
    inventory: &SourceInventory,
    program: &MirProgram,
    function: &str,
    parent: &ContextKey,
    mut callbacks: Vec<(usize, String)>,
) -> Result<ContextKey, AuditError> {
    let body = program
        .functions
        .get(function)
        .ok_or_else(|| AuditError::UnresolvedTarget {
            function: "<context>".to_owned(),
            target: function.to_owned(),
        })?;
    callbacks.sort();
    callbacks.dedup();
    let (class, witness) = classify_child(inventory, body, parent)?;
    Ok(ContextKey {
        function: function.to_owned(),
        class,
        witness,
        callbacks,
    })
}

fn callable_bindings(
    program: &MirProgram,
    target: &str,
    call: &MirCall,
) -> Result<Vec<(usize, String)>, AuditError> {
    let function = &program.functions[target];
    let mut bindings = Vec::new();
    for (index, ty) in function.parameter_types.iter().enumerate() {
        if !type_is_callable_text(ty) {
            continue;
        }
        let argument = call
            .arguments
            .get(index)
            .ok_or_else(|| AuditError::UnresolvedCallback {
                function: target.to_owned(),
                parameter: index,
            })?;
        let resolved = resolve_callable_argument(program, argument)?;
        bindings.push((index, resolved));
    }
    Ok(bindings)
}

fn resolve_callable_argument(program: &MirProgram, argument: &str) -> Result<String, AuditError> {
    if let Some(reference) = closure_references(argument).into_iter().next() {
        return resolve_closure_reference(program, &reference);
    }
    if base_local(argument).is_some() {
        return Err(AuditError::DynamicCall {
            function: "<callable-argument>".to_owned(),
            target: argument.to_owned(),
        });
    }
    let candidate = argument
        .trim()
        .strip_prefix("const ")
        .unwrap_or(argument.trim())
        .trim();
    resolve_local_target(program, "", candidate)?.ok_or_else(|| AuditError::UnresolvedTarget {
        function: "<callable-argument>".to_owned(),
        target: argument.to_owned(),
    })
}

fn resolve_closure_target(
    program: &MirProgram,
    target: &str,
) -> Result<Option<String>, AuditError> {
    let Some(reference) = closure_references(target).into_iter().next() else {
        return Ok(None);
    };
    resolve_closure_reference(program, &reference).map(Some)
}

fn resolve_closure_reference(program: &MirProgram, reference: &str) -> Result<String, AuditError> {
    let candidates: Vec<_> = program
        .functions
        .iter()
        .filter(|(_, function)| {
            function.display.contains("{closure")
                && function
                    .parameter_types
                    .first()
                    .is_some_and(|parameter| parameter.contains(reference))
        })
        .map(|(id, _)| id.clone())
        .collect();
    if candidates.len() == 1 {
        Ok(candidates[0].clone())
    } else {
        Err(AuditError::UnresolvedTarget {
            function: "<closure>".to_owned(),
            target: format!("{reference} => {candidates:?}"),
        })
    }
}

fn resolve_local_target(
    program: &MirProgram,
    caller: &str,
    target: &str,
) -> Result<Option<String>, AuditError> {
    if program.functions.contains_key(target) {
        return Ok(Some(target.to_owned()));
    }
    if target.starts_with('<') && !contains_local_nominal(program, target) {
        return Ok(None);
    }
    let name = terminal_name(target);
    let receiver = target_receiver(target);
    let source_suffix = target_source_suffix(target);
    let caller_source = program
        .functions
        .get(caller)
        .and_then(|function| function.source.as_ref());
    let mut candidates: Vec<_> = program
        .functions
        .iter()
        .filter(|(_, function)| terminal_name(&function.display) == name)
        .filter(|(_, function)| {
            if let Some(receiver) = &receiver {
                function.source.as_ref().is_some_and(|source| {
                    penultimate_path_segment(&source.id) == Some(receiver.as_str())
                        || function.display.contains(&format!("::{receiver}::"))
                })
            } else {
                true
            }
        })
        .filter(|(_, function)| {
            source_suffix.as_ref().is_none_or(|suffix| {
                function
                    .source
                    .as_ref()
                    .is_some_and(|source| source.id.ends_with(suffix))
            })
        })
        .filter(|(_, function)| {
            if source_suffix.is_some() || caller_source.is_none() {
                true
            } else {
                let source = function.source.as_ref();
                source.is_some_and(|source| {
                    caller_source.is_some_and(|caller| source.module == caller.module)
                })
            }
        })
        .map(|(id, _)| id.clone())
        .collect();
    candidates.sort();
    candidates.dedup();
    if candidates.len() == 1 {
        Ok(Some(candidates[0].clone()))
    } else if candidates.is_empty() {
        Ok(None)
    } else {
        Err(AuditError::UnresolvedTarget {
            function: caller.to_owned(),
            target: format!("{target} => {candidates:?}"),
        })
    }
}

fn contains_local_nominal(program: &MirProgram, target: &str) -> bool {
    program.functions.values().any(|function| {
        function.source.as_ref().is_some_and(|source| {
            penultimate_path_segment(&source.id).is_some_and(|owner| target.contains(owner))
        })
    })
}

fn validate_function_nominals(
    inventory: &SourceInventory,
    function: &MirFunction,
) -> Result<BTreeSet<String>, AuditError> {
    let source = function
        .source
        .as_ref()
        .ok_or_else(|| AuditError::UnboundMirFunction {
            function: function.display.clone(),
        })?;
    let mut nominals = BTreeSet::new();
    for ty in function.locals.values() {
        for spelling in type_path_spellings(ty) {
            if let Some(nominal) = inventory.resolve_nominal_in_module(&source.module, &spelling)? {
                if !CONSTITUTIONAL_NOMINALS.contains(&nominal.as_str()) {
                    return Err(AuditError::Unclassified {
                        function: function.display.clone(),
                        detail: format!("non-constitutional carrier {nominal}"),
                    });
                }
                nominals.insert(nominal);
            }
        }
    }
    Ok(nominals)
}

fn classify_child(
    inventory: &SourceInventory,
    function: &MirFunction,
    parent: &ContextKey,
) -> Result<(Mechanic, String), AuditError> {
    let nominals = validate_function_nominals(inventory, function)?;
    let source = function
        .source
        .as_ref()
        .ok_or_else(|| AuditError::UnboundMirFunction {
            function: function.display.clone(),
        })?;
    let owner = match source.owner.as_deref() {
        Some(spelling) => Some(
            inventory
                .resolve_source_owner(&source.module, spelling)?
                .ok_or_else(|| AuditError::Unclassified {
                    function: function.display.clone(),
                    detail: format!("owner {spelling} has no exact source declaration"),
                })?,
        ),
        None => None,
    };
    if let Some(owner) = owner.as_deref()
        && !CONSTITUTIONAL_NOMINALS.contains(&owner)
    {
        return Err(AuditError::Unclassified {
            function: function.display.clone(),
            detail: format!("non-constitutional owner {owner}"),
        });
    }

    if function.display.contains("{closure") {
        let expected_id = format!(
            "{}:{}:{}::{}",
            source.path,
            source.line,
            source.column,
            terminal_name(&function.display)
        );
        let exact_file = inventory
            .source(&source.path)
            .is_some_and(|file| file.module == source.module);
        if source.owner.is_some() || source.id != expected_id || !exact_file {
            return Err(AuditError::Unclassified {
                function: function.display.clone(),
                detail: format!(
                    "closure has no exact compiler source binding: {}",
                    source.id
                ),
            });
        }
        return Ok((
            parent.class,
            format!(
                "exact-closure-edge:{};inherited:{}",
                source.id,
                parent.class.name()
            ),
        ));
    }

    let exact_declaration = inventory.functions.iter().any(|declaration| {
        declaration.id == source.id
            && declaration.module == source.module
            && declaration.owner == source.owner
            && declaration.path == source.path
            && declaration.line == source.line
            && declaration.column == source.column
    });
    if !exact_declaration {
        return Err(AuditError::Unclassified {
            function: function.display.clone(),
            detail: format!("{} has no exact source declaration", source.id),
        });
    }
    let class = closed_source_mechanic(&source.id).ok_or_else(|| AuditError::Unclassified {
        function: function.display.clone(),
        detail: format!("{} has no closed mechanic contract", source.id),
    })?;
    Ok((
        class,
        format!(
            "closed-source-contract:{};class:{};owner:{};carriers:{}",
            source.id,
            class.name(),
            owner.as_deref().unwrap_or("-"),
            render_set(nominals)
        ),
    ))
}

fn closed_source_mechanic(source: &str) -> Option<Mechanic> {
    if let Some(root) = ROOTS.iter().find(|root| root.source == source) {
        Some(root.class)
    } else if CLOSED_WIRE_CODEC_FUNCTIONS.contains(&source) {
        Some(Mechanic::WireCodec)
    } else if CLOSED_CORE_ABI_FUNCTIONS.contains(&source) {
        Some(Mechanic::CoreAbi)
    } else if CLOSED_DEFINITION_TABLE_FUNCTIONS.contains(&source) {
        Some(Mechanic::DefinitionTable)
    } else if CLOSED_KERNEL_STEP_FUNCTIONS.contains(&source) {
        Some(Mechanic::KernelStep)
    } else if CLOSED_RECEIPT_STEP_FUNCTIONS.contains(&source) {
        Some(Mechanic::ReceiptStep)
    } else if CLOSED_AUTHORIZATION_STEP_FUNCTIONS.contains(&source) {
        Some(Mechanic::AuthorizationStep)
    } else if CLOSED_PHYSICAL_DISPATCH_FUNCTIONS.contains(&source) {
        Some(Mechanic::PhysicalDispatch)
    } else {
        None
    }
}

fn type_path_spellings(value: &str) -> BTreeSet<String> {
    let bytes = value.as_bytes();
    let mut output = BTreeSet::new();
    let mut index = 0;
    while index < bytes.len() {
        if !(bytes[index].is_ascii_alphabetic() || bytes[index] == b'_') {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while bytes
            .get(index)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            index += 1;
        }
        loop {
            if bytes.get(index..index + 2) != Some(b"::") {
                break;
            }
            let next = index + 2;
            if !bytes
                .get(next)
                .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
            {
                break;
            }
            index = next + 1;
            while bytes
                .get(index)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                index += 1;
            }
        }
        output.insert(value[start..index].to_owned());
    }
    output
}

fn contains_any(value: &str, candidates: &[&str]) -> bool {
    candidates.iter().any(|candidate| value.contains(candidate))
}

fn external_contract(target: &str) -> Result<ExternalContract, AuditError> {
    reject_dynamic_target(target)?;
    let target = one_line(target);
    if let Some(kind) = digest_contract_kind(&target) {
        return Ok(ExternalContract { kind, target });
    }
    if is_platform_contract(&target) {
        return Ok(ExternalContract {
            kind: ContractKind::Platform,
            target,
        });
    }
    Err(AuditError::UnsupportedExternalContract(target))
}

fn digest_contract_kind(target: &str) -> Option<ContractKind> {
    let contract = parse_ufcs_contract(target)?;
    if contract.trait_contract.path != "Digest"
        || !contract.trait_contract.arguments.is_empty()
        || !is_sha256_digest_receiver(contract.receiver)
    {
        return None;
    }
    match contract.method.name {
        "digest" if contract.method.generic_arguments.len() == 1 => {
            Some(ContractKind::DigestOneShot)
        }
        "new" | "finalize" if contract.method.generic_arguments.is_empty() => {
            Some(ContractKind::DigestState)
        }
        "update" if contract.method.generic_arguments.len() == 1 => Some(ContractKind::DigestState),
        _ => None,
    }
}

fn is_sha256_digest_receiver(value: &str) -> bool {
    parse_nominal_contract(value).is_some_and(|wrapper| {
        wrapper.path == "CoreWrapper"
            && wrapper.arguments.len() == 1
            && parse_nominal_contract(wrapper.arguments[0]).is_some_and(|core| {
                core.path == "CtVariableCoreWrapper"
                    && core.arguments.len() == 3
                    && core.arguments[0] == "Sha256VarCore"
                    && typenum_unsigned(core.arguments[1]) == Some(32)
                    && core.arguments[2] == "sha2::OidSha256"
            })
    })
}

fn typenum_unsigned(value: &str) -> Option<usize> {
    if value == "UTerm" {
        return Some(0);
    }
    let contract = parse_nominal_contract(value)?;
    if contract.path != "UInt" || contract.arguments.len() != 2 {
        return None;
    }
    let prefix = typenum_unsigned(contract.arguments[0])?;
    let bit = match contract.arguments[1] {
        "B0" => 0,
        "B1" => 1,
        _ => return None,
    };
    prefix.checked_mul(2)?.checked_add(bit)
}

fn is_platform_contract(target: &str) -> bool {
    if contains_any(
        target,
        &[
            "std::fs",
            "std::net",
            "std::process",
            "std::thread",
            "std::env",
            "std::time",
            "std::io",
            "std::sync::",
            "alloc::alloc",
            "exchange_malloc",
        ],
    ) {
        return false;
    }
    if is_primitive_contract(target) {
        return true;
    }
    is_ufcs_platform_contract(target) || is_inherent_platform_contract(target)
}

fn is_primitive_contract(target: &str) -> bool {
    matches!(
        target,
        "core::bool::<impl bool>::then_some"
            | "core::num::<impl u8>::is_ascii_whitespace"
            | "core::num::<impl u8>::checked_sub"
            | "core::num::<impl u32>::from_be_bytes"
            | "core::num::<impl u32>::to_be_bytes"
            | "core::num::<impl u64>::checked_sub"
            | "core::num::<impl u64>::from_be_bytes"
            | "core::num::<impl u64>::to_be_bytes"
            | "core::num::<impl u64>::wrapping_add"
            | "core::num::<impl usize>::checked_add"
            | "core::num::<impl usize>::checked_mul"
            | "core::num::<impl usize>::checked_sub"
            | "core::num::<impl usize>::is_multiple_of"
    )
}

#[derive(Debug)]
struct NominalContract<'a> {
    path: &'a str,
    arguments: Vec<&'a str>,
}

#[derive(Debug)]
struct MethodContract<'a> {
    name: &'a str,
    generic_arguments: Vec<&'a str>,
}

#[derive(Debug)]
struct UfcsContract<'a> {
    receiver: &'a str,
    trait_contract: NominalContract<'a>,
    method: MethodContract<'a>,
}

fn is_ufcs_platform_contract(target: &str) -> bool {
    let Some(contract) = parse_ufcs_contract(target) else {
        return false;
    };
    let trait_contract = &contract.trait_contract;
    let method = &contract.method;

    if trait_contract.path == "IntoIterator"
        && trait_contract.arguments.is_empty()
        && method.name == "into_iter"
        && method.generic_arguments.is_empty()
    {
        return is_into_iterator_receiver(contract.receiver);
    }
    if trait_contract.path == "Iterator" && trait_contract.arguments.is_empty() {
        return match method.name {
            "next" if method.generic_arguments.is_empty() => {
                is_iterator_receiver(contract.receiver)
            }
            "enumerate" if method.generic_arguments.is_empty() => {
                nominal_has_arity(contract.receiver, "Windows", 2)
            }
            "rev" if method.generic_arguments.is_empty() => {
                nominal_has_arity(contract.receiver, "Zip", 2)
                    || nominal_has_arity(contract.receiver, "std::slice::Iter", 2)
                    || nominal_has_arity(contract.receiver, "std::vec::IntoIter", 1)
            }
            "zip" if method.generic_arguments.len() == 1 => {
                nominal_has_arity(contract.receiver, "std::slice::Iter", 2)
                    || nominal_has_arity(contract.receiver, "std::vec::IntoIter", 1)
            }
            _ => false,
        };
    }
    if trait_contract.path == "Try"
        && trait_contract.arguments.is_empty()
        && method.name == "branch"
        && method.generic_arguments.is_empty()
    {
        return nominal_has_arity(contract.receiver, "Option", 1)
            || nominal_has_arity(contract.receiver, "Result", 2);
    }
    if trait_contract.path == "FromResidual"
        && trait_contract.arguments.len() == 1
        && method.name == "from_residual"
        && method.generic_arguments.is_empty()
    {
        return (nominal_has_arity(contract.receiver, "Option", 1)
            && is_option_infallible_residual(trait_contract.arguments[0]))
            || (nominal_has_arity(contract.receiver, "Result", 2)
                && is_result_infallible_residual(trait_contract.arguments[0]));
    }
    if trait_contract.path == "Deref"
        && trait_contract.arguments.is_empty()
        && method.name == "deref"
        && method.generic_arguments.is_empty()
    {
        return is_generic_array_u8(contract.receiver)
            || nominal_has_arity(contract.receiver, "Vec", 1)
            || nominal_has_arity(contract.receiver, "types::FallibleBox", 1);
    }
    if trait_contract.path == "DerefMut"
        && trait_contract.arguments.is_empty()
        && method.name == "deref_mut"
        && method.generic_arguments.is_empty()
    {
        return nominal_has_arity(contract.receiver, "Vec", 1);
    }
    if trait_contract.path == "Extend"
        && trait_contract.arguments.as_slice() == ["&types::KExpr"]
        && nominal_has_exact_argument(contract.receiver, "Vec", "&types::KExpr")
        && method.name == "extend"
        && method.generic_arguments.as_slice() == ["Rev<std::slice::Iter<'_, types::KExpr>>"]
    {
        return true;
    }
    if trait_contract.path == "PartialEq" && method.generic_arguments.is_empty() {
        let homogeneous = trait_contract.arguments.is_empty();
        let byte_vec_slice = nominal_has_exact_argument(contract.receiver, "Vec", "u8")
            && trait_contract.arguments.as_slice() == ["&[u8]"];
        let byte_slice_vec = is_immutable_u8_slice_reference(contract.receiver)
            && trait_contract.arguments.as_slice() == ["Vec<u8>"];
        let ksort_slice_array = contract.receiver == "&[types::KSort]"
            && trait_contract.arguments.as_slice() == ["[types::KSort; 1]"];
        return matches!(method.name, "eq" | "ne")
            && ((homogeneous
                && (is_immutable_u8_slice_reference(contract.receiver)
                    || nominal_has_exact_argument(contract.receiver, "Vec", "u8")))
                || byte_vec_slice
                || byte_slice_vec
                || ksort_slice_array
                || (homogeneous
                    && receiver_is_exact_nominal(
                        contract.receiver,
                        &[
                            "types::CompilerEvidence",
                            "types::CoreManifest",
                            "types::Hash32",
                            "types::Id32",
                            "types::KSort",
                            "types::NominalDeclaration",
                            "types::Term",
                            "Option<u8>",
                            "(types::Id32, types::Id32)",
                        ],
                    ))
                || (homogeneous
                    && contract.receiver == "types::DecodeCode"
                    && method.name == "eq"));
    }
    if trait_contract.path == "PartialOrd"
        && trait_contract.arguments.is_empty()
        && matches!(
            contract.receiver,
            "types::Id32" | "(types::Id32, types::Id32)"
        )
        && matches!(method.name, "gt" | "ge" | "lt" | "le")
        && method.generic_arguments.is_empty()
    {
        return true;
    }
    if trait_contract.path == "std::cmp::Ord"
        && trait_contract.arguments.is_empty()
        && contract.receiver == "usize"
        && method.name == "max"
        && method.generic_arguments.is_empty()
    {
        return true;
    }
    if trait_contract.path == "Default"
        && trait_contract.arguments.is_empty()
        && contract.receiver == "ObservationLog"
        && method.name == "default"
        && method.generic_arguments.is_empty()
    {
        return true;
    }
    if trait_contract.path == "Clone"
        && trait_contract.arguments.is_empty()
        && nominal_has_exact_argument(contract.receiver, "std::ops::Range", "usize")
        && method.name == "clone"
        && method.generic_arguments.is_empty()
    {
        return true;
    }
    if trait_contract.path == "Index"
        && matches!(
            trait_contract.arguments.as_slice(),
            ["std::ops::Range<usize>"] | ["usize"]
        )
        && nominal_has_exact_argument(contract.receiver, "Vec", "u8")
        && method.name == "index"
        && method.generic_arguments.is_empty()
    {
        return true;
    }
    if trait_contract.path == "IndexMut"
        && method.name == "index_mut"
        && method.generic_arguments.is_empty()
    {
        return (trait_contract.arguments.as_slice() == ["usize"]
            && nominal_has_exact_argument(contract.receiver, "Vec", "u8"))
            || (matches!(
                trait_contract.arguments.as_slice(),
                ["RangeTo<usize>"] | ["std::ops::RangeFrom<usize>"]
            ) && contract.receiver == "[u8; 64]");
    }
    if trait_contract.path == "TryInto"
        && trait_contract.arguments.len() == 1
        && method.name == "try_into"
        && method.generic_arguments.is_empty()
    {
        return (is_immutable_u8_slice_reference(contract.receiver)
            && is_u8_array(trait_contract.arguments[0], &[4, 8, 32]))
            || (nominal_has_exact_argument(contract.receiver, "Vec", "&types::Term")
                && trait_contract.arguments[0].starts_with("[&types::Term; ")
                && trait_contract.arguments[0].ends_with(']'));
    }
    if trait_contract.path == "Into"
        && trait_contract.arguments.len() == 1
        && is_generic_array_u8(contract.receiver)
        && method.name == "into"
        && method.generic_arguments.is_empty()
    {
        return is_u8_array(trait_contract.arguments[0], &[32]);
    }
    if trait_contract.path == "TryFrom"
        && trait_contract.arguments.len() == 1
        && method.name == "try_from"
        && method.generic_arguments.is_empty()
    {
        return matches!(
            (contract.receiver, trait_contract.arguments[0]),
            ("u32", "usize")
                | ("u64", "usize")
                | ("usize", "u32")
                | (
                    "&[types::NominalDeclaration; 2]",
                    "&[types::NominalDeclaration]"
                )
        );
    }
    if trait_contract.path == "From"
        && trait_contract.arguments.as_slice() == ["u8"]
        && contract.receiver == "usize"
        && method.name == "from"
        && method.generic_arguments.is_empty()
    {
        return true;
    }
    false
}

fn is_inherent_platform_contract(target: &str) -> bool {
    let segments = top_level_path_segments(target);
    if segments.len() == 4
        && segments[0] == "Option"
        && segments[2] == "map"
        && generic_segment_arguments(segments[1]).is_some_and(|receiver| {
            generic_segment_arguments(segments[3]).is_some_and(|mapping| {
                (receiver == ["&types::KValue"]
                    && mapping
                        == [
                            "RuntimeValueReference<'_, '_>",
                            "fn(&types::KValue) -> RuntimeValueReference<'_, '_> {RuntimeValueReference::<'_, '_>::Borrowed}",
                        ])
                    || (receiver == ["&RuntimeValue<'_>"]
                        && mapping
                            == [
                                "RuntimeValueReference<'_, '_>",
                                "fn(&RuntimeValue<'_>) -> RuntimeValueReference<'_, '_> {RuntimeValueReference::<'_, '_>::Owned}",
                            ])
            })
        })
    {
        return true;
    }
    if segments.len() == 3
        && segments[0] == "Option"
        && generic_segment_arity(segments[1]) == Some(1)
        && matches!(
            segments[2],
            "as_ref" | "copied" | "expect" | "is_none" | "is_some" | "take"
        )
    {
        return true;
    }
    if segments.len() == 4
        && segments[0] == "Option"
        && generic_segment_arity(segments[1]) == Some(1)
        && segments[2] == "ok_or"
        && generic_segment_arity(segments[3]) == Some(1)
    {
        return true;
    }
    if segments.len() == 3 && segments[0] == "Result" {
        let arguments = generic_segment_arguments(segments[1]);
        if arguments.as_deref().is_some_and(|arguments| {
            matches!(segments[2], "expect" | "ok") && arguments.len() == 2
                || segments[2] == "unwrap_or" && arguments == ["u32", "TryFromIntError"]
        }) {
            return true;
        }
    }
    if segments.len() == 4
        && segments[0] == "Result"
        && generic_segment_arity(segments[1]) == Some(2)
        && segments[2] == "map_err"
        && generic_segment_arity(segments[3]) == Some(2)
    {
        return true;
    }
    if segments.len() == 3
        && segments[0] == "Vec"
        && generic_segment_arity(segments[1]) == Some(1)
        && matches!(
            segments[2],
            "as_slice"
                | "clear"
                | "extend_from_slice"
                | "insert"
                | "is_empty"
                | "len"
                | "new"
                | "pop"
                | "push"
                | "resize"
                | "truncate"
                | "try_reserve"
                | "try_reserve_exact"
        )
    {
        return true;
    }
    if segments.len() == 5
        && segments[..3] == ["core", "bool", "<impl bool>"]
        && segments[3] == "then_some"
        && generic_segment_arity(segments[4]) == Some(1)
    {
        return true;
    }
    if segments.len() == 4
        && segments[..3] == ["std", "mem", "replace"]
        && generic_segment_arguments(segments[3]).is_some_and(|arguments| {
            arguments.len() == 1
                && matches!(
                    arguments[0],
                    "RuntimeByteSlot<'_>" | "RuntimeEnvironmentSlot<'_>"
                )
        })
    {
        return true;
    }
    if segments.len() >= 4
        && segments[0] == "core"
        && segments[1] == "slice"
        && generic_segment_arguments(segments[2]).is_some_and(|arguments| {
            arguments.len() == 1
                && arguments[0]
                    .strip_prefix("impl ")
                    .is_some_and(is_slice_type)
        })
    {
        return (segments.len() == 4
            && matches!(
                segments[3],
                "contains"
                    | "copy_from_slice"
                    | "first"
                    | "iter"
                    | "last"
                    | "last_mut"
                    | "reverse"
                    | "windows"
            ))
            || (segments.len() == 5
                && matches!(segments[3], "get" | "get_mut")
                && generic_segment_arity(segments[4]) == Some(1));
    }
    if segments.as_slice() == ["core", "str", "<impl str>", "as_bytes"] {
        return true;
    }
    if segments.len() == 5
        && segments[..3] == ["std", "ops", "RangeInclusive"]
        && generic_segment_arguments(segments[3]).is_some_and(|arguments| arguments == ["u8"])
        && segments[4] == "new"
    {
        return true;
    }
    segments.len() == 6
        && segments[..3] == ["std", "ops", "RangeInclusive"]
        && generic_segment_arguments(segments[3]).is_some_and(|arguments| arguments == ["u8"])
        && segments[4] == "contains"
        && generic_segment_arguments(segments[5]).is_some_and(|arguments| arguments == ["u8"])
}

fn parse_ufcs_contract(target: &str) -> Option<UfcsContract<'_>> {
    let end = matching_angle_end(target, 0)?;
    let method = target.get(end + 1..)?.strip_prefix("::")?;
    let inside = target.get(1..end)?;
    let (receiver, trait_value) = split_top_level_as(inside)?;
    Some(UfcsContract {
        receiver: receiver.trim(),
        trait_contract: parse_nominal_contract(trait_value.trim())?,
        method: parse_method_contract(method)?,
    })
}

fn parse_method_contract(value: &str) -> Option<MethodContract<'_>> {
    let segments = top_level_path_segments(value);
    match segments.as_slice() {
        [name] if is_contract_identifier(name) => Some(MethodContract {
            name,
            generic_arguments: Vec::new(),
        }),
        [name, arguments] if is_contract_identifier(name) => Some(MethodContract {
            name,
            generic_arguments: generic_segment_arguments(arguments)?,
        }),
        _ => None,
    }
}

fn parse_nominal_contract(value: &str) -> Option<NominalContract<'_>> {
    let value = value.trim();
    let Some(open) = value.find('<') else {
        return is_contract_path(value).then_some(NominalContract {
            path: value,
            arguments: Vec::new(),
        });
    };
    let end = matching_angle_end(value, open)?;
    if end + 1 != value.len() {
        return None;
    }
    let path = value[..open].trim();
    is_contract_path(path).then_some(NominalContract {
        path,
        arguments: split_top_level_arguments(&value[open + 1..end])?,
    })
}

fn matching_angle_end(value: &str, start: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    if bytes.get(start) != Some(&b'<') {
        return None;
    }
    let mut depth = 0_usize;
    for index in start..bytes.len() {
        match bytes[index] {
            b'<' => depth = depth.checked_add(1)?,
            b'>' if bytes.get(index.wrapping_sub(1)) != Some(&b'-') => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level_as(value: &str) -> Option<(&str, &str)> {
    let bytes = value.as_bytes();
    let mut angle = 0_usize;
    let mut bracket = 0_usize;
    let mut brace = 0_usize;
    let mut paren = 0_usize;
    let mut index = 0;
    while index < bytes.len() {
        if angle == 0
            && bracket == 0
            && brace == 0
            && paren == 0
            && bytes.get(index..index + 4) == Some(b" as ")
        {
            return Some((&value[..index], &value[index + 4..]));
        }
        match bytes[index] {
            b'<' => angle = angle.checked_add(1)?,
            b'>' if bytes.get(index.wrapping_sub(1)) != Some(&b'-') => {
                angle = angle.checked_sub(1)?;
            }
            b'[' => bracket = bracket.checked_add(1)?,
            b']' => bracket = bracket.checked_sub(1)?,
            b'{' => brace = brace.checked_add(1)?,
            b'}' => brace = brace.checked_sub(1)?,
            b'(' => paren = paren.checked_add(1)?,
            b')' => paren = paren.checked_sub(1)?,
            _ => {}
        }
        index += 1;
    }
    None
}

fn split_top_level_arguments(value: &str) -> Option<Vec<&str>> {
    if value.trim().is_empty() {
        return None;
    }
    let bytes = value.as_bytes();
    let mut arguments = Vec::new();
    let mut start = 0;
    let mut angle = 0_usize;
    let mut bracket = 0_usize;
    let mut brace = 0_usize;
    let mut paren = 0_usize;
    for index in 0..bytes.len() {
        match bytes[index] {
            b'<' => angle = angle.checked_add(1)?,
            b'>' if bytes.get(index.wrapping_sub(1)) != Some(&b'-') => {
                angle = angle.checked_sub(1)?;
            }
            b'[' => bracket = bracket.checked_add(1)?,
            b']' => bracket = bracket.checked_sub(1)?,
            b'{' => brace = brace.checked_add(1)?,
            b'}' => brace = brace.checked_sub(1)?,
            b'(' => paren = paren.checked_add(1)?,
            b')' => paren = paren.checked_sub(1)?,
            b',' if angle == 0 && bracket == 0 && brace == 0 && paren == 0 => {
                let argument = value[start..index].trim();
                if argument.is_empty() {
                    return None;
                }
                arguments.push(argument);
                start = index + 1;
            }
            _ => {}
        }
    }
    if angle != 0 || bracket != 0 || brace != 0 || paren != 0 {
        return None;
    }
    let argument = value[start..].trim();
    if argument.is_empty() {
        return None;
    }
    arguments.push(argument);
    Some(arguments)
}

fn generic_segment_arguments(value: &str) -> Option<Vec<&str>> {
    let end = matching_angle_end(value, 0)?;
    (end + 1 == value.len()).then(|| split_top_level_arguments(&value[1..end]))?
}

fn generic_segment_arity(value: &str) -> Option<usize> {
    Some(generic_segment_arguments(value)?.len())
}

fn nominal_has_arity(value: &str, path: &str, arity: usize) -> bool {
    parse_nominal_contract(value)
        .is_some_and(|contract| contract.path == path && contract.arguments.len() == arity)
}

fn nominal_has_exact_argument(value: &str, path: &str, argument: &str) -> bool {
    parse_nominal_contract(value).is_some_and(|contract| {
        contract.path == path && contract.arguments.as_slice() == [argument]
    })
}

fn is_into_iterator_receiver(value: &str) -> bool {
    if let Some((mutable, referent)) = reference_contract(value) {
        if mutable {
            return nominal_has_exact_argument(referent, "Vec", "RuntimeEnvironmentSlot<'_>")
                || nominal_has_exact_argument(referent, "Vec", "RuntimeOwnedValue<'_>");
        }
        return nominal_has_arity(referent, "Vec", 1)
            || is_slice_type(referent)
            || is_array_type(referent);
    }
    nominal_has_arity(value, "Enumerate", 1)
        || nominal_has_arity(value, "Rev", 1)
        || nominal_has_arity(value, "Windows", 2)
        || nominal_has_arity(value, "Vec", 1)
        || nominal_has_arity(value, "Zip", 2)
        || nominal_has_arity(value, "std::ops::Range", 1)
        || nominal_has_exact_argument(value, "std::ops::RangeInclusive", "u8")
}

fn is_iterator_receiver(value: &str) -> bool {
    nominal_has_arity(value, "Enumerate", 1)
        || nominal_has_arity(value, "Rev", 1)
        || nominal_has_arity(value, "Windows", 2)
        || nominal_has_arity(value, "Zip", 2)
        || nominal_has_arity(value, "std::ops::Range", 1)
        || nominal_has_exact_argument(value, "std::ops::RangeInclusive", "u8")
        || nominal_has_arity(value, "std::slice::Iter", 2)
        || nominal_has_arity(value, "std::slice::IterMut", 2)
        || nominal_has_arity(value, "std::vec::IntoIter", 1)
}

fn reference_contract(value: &str) -> Option<(bool, &str)> {
    let mut value = value.strip_prefix('&')?.trim_start();
    if value.starts_with('\'') {
        value = value.split_once(' ')?.1.trim_start();
    }
    let mutable = value.starts_with("mut ");
    if mutable {
        value = value.strip_prefix("mut ")?.trim_start();
    }
    (!value.is_empty()).then_some((mutable, value))
}

fn receiver_is_exact_nominal(value: &str, candidates: &[&str]) -> bool {
    candidates.contains(&value)
        || reference_contract(value)
            .is_some_and(|(mutable, referent)| !mutable && candidates.contains(&referent))
}

fn is_slice_type(value: &str) -> bool {
    value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .is_some_and(|inside| !inside.is_empty() && top_level_semicolon(inside).is_none())
}

fn is_array_type(value: &str) -> bool {
    value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .and_then(top_level_semicolon)
        .is_some()
}

fn is_immutable_u8_slice_reference(value: &str) -> bool {
    reference_contract(value).is_some_and(|(mutable, referent)| {
        !mutable
            && referent
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
                == Some("u8")
    })
}

fn is_u8_array(value: &str, widths: &[usize]) -> bool {
    let Some(inside) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let Some((element, width)) = top_level_semicolon(inside) else {
        return false;
    };
    element.trim() == "u8"
        && width
            .trim()
            .parse::<usize>()
            .ok()
            .is_some_and(|width| widths.contains(&width))
}

fn top_level_semicolon(value: &str) -> Option<(&str, &str)> {
    let bytes = value.as_bytes();
    let mut angle = 0_usize;
    let mut bracket = 0_usize;
    let mut brace = 0_usize;
    let mut paren = 0_usize;
    for index in 0..bytes.len() {
        match bytes[index] {
            b'<' => angle = angle.checked_add(1)?,
            b'>' if bytes.get(index.wrapping_sub(1)) != Some(&b'-') => {
                angle = angle.checked_sub(1)?;
            }
            b'[' => bracket = bracket.checked_add(1)?,
            b']' => bracket = bracket.checked_sub(1)?,
            b'{' => brace = brace.checked_add(1)?,
            b'}' => brace = brace.checked_sub(1)?,
            b'(' => paren = paren.checked_add(1)?,
            b')' => paren = paren.checked_sub(1)?,
            b';' if angle == 0 && bracket == 0 && brace == 0 && paren == 0 => {
                return Some((&value[..index], &value[index + 1..]));
            }
            _ => {}
        }
    }
    None
}

fn is_generic_array_u8(value: &str) -> bool {
    parse_nominal_contract(value).is_some_and(|contract| {
        contract.path == "GenericArray"
            && contract.arguments.len() == 2
            && contract.arguments[0] == "u8"
    })
}

fn is_option_infallible_residual(value: &str) -> bool {
    nominal_has_exact_argument(value, "Option", "Infallible")
}

fn is_result_infallible_residual(value: &str) -> bool {
    parse_nominal_contract(value).is_some_and(|contract| {
        contract.path == "Result"
            && contract.arguments.len() == 2
            && contract.arguments[0] == "Infallible"
    })
}

fn is_contract_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().enumerate().all(|(index, byte)| match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => true,
            b'0'..=b'9' => index > 0,
            _ => false,
        })
}

fn is_contract_path(value: &str) -> bool {
    !value.is_empty() && value.split("::").all(is_contract_identifier)
}

fn reject_dynamic_target(target: &str) -> Result<(), AuditError> {
    let target = target.trim();
    let dynamic = target
        .strip_prefix("move ")
        .or_else(|| target.strip_prefix("copy "))
        .unwrap_or(target);
    if base_local(dynamic).is_some() && top_level_open_paren(dynamic).is_none() {
        return Err(AuditError::DynamicCall {
            function: "<target>".to_owned(),
            target: target.to_owned(),
        });
    }
    if dynamic.starts_with('_') {
        return Err(AuditError::DynamicCall {
            function: "<target>".to_owned(),
            target: target.to_owned(),
        });
    }
    Ok(())
}

fn is_generic_callable_target(target: &str) -> bool {
    (target.contains("impl Fn")
        || target.contains("impl for<")
        || target.contains(" as FnMut>::")
        || target.contains(" as FnOnce>::"))
        && !target.contains("{closure@")
}

fn type_is_callable_text(ty: &str) -> bool {
    ty.contains("impl Fn") || ty.contains("dyn Fn") || ty.starts_with("fn(") || ty.contains(" fn(")
}

fn audit_fallible_allocations(
    program: &MirProgram,
    contexts: &BTreeSet<ContextKey>,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
) -> Result<(), AuditError> {
    for context in contexts {
        let function = &program.functions[&context.function];
        for ty in function.locals.values() {
            if contains_any(
                ty,
                &["std::boxed::Box<", "alloc::boxed::Box<", "std::sync::Arc<"],
            ) {
                return Err(AuditError::InfallibleAllocation {
                    function: context.function.clone(),
                    target: ty.clone(),
                });
            }
        }
        for call in function.all_calls() {
            let key = (context.clone(), call.block.clone(), call.ordinal);
            let targets = &resolved_calls[&key];
            for target in targets {
                let rendered = target.render();
                if contains_any(
                    &rendered,
                    &[
                        "Box::new",
                        "Box::<",
                        "Arc::new",
                        "Arc::<",
                        "exchange_malloc",
                        "alloc::alloc",
                    ],
                ) {
                    return Err(AuditError::InfallibleAllocation {
                        function: context.function.clone(),
                        target: rendered,
                    });
                }
                if rendered.contains(" as Clone>::clone")
                    && contains_any(
                        &rendered,
                        &[
                            "KExpr",
                            "KValue",
                            "types::Term",
                            "CompilerPackage",
                            "ObservationLog",
                            "PhysicalObservation",
                        ],
                    )
                {
                    return Err(AuditError::InfallibleAllocation {
                        function: context.function.clone(),
                        target: rendered,
                    });
                }
            }
        }
    }
    Ok(())
}

fn derive_provenance(
    program: &MirProgram,
    roots: &BTreeSet<String>,
    contexts: &BTreeSet<ContextKey>,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
) -> Result<BTreeMap<String, BTreeMap<String, BTreeSet<String>>>, AuditError> {
    let mut provenance: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = contexts
        .iter()
        .map(|context| (context.function.clone(), BTreeMap::new()))
        .collect();
    for root in roots {
        let function = &program.functions[root];
        let values = provenance.entry(root.clone()).or_default();
        for (index, ty) in function.parameter_types.iter().enumerate() {
            if root_parameter_is_package(ty) {
                let local = format!("_{}", index + 1);
                let name = function
                    .debug_names
                    .get(&local)
                    .cloned()
                    .unwrap_or_else(|| format!("arg{}", index + 1));
                values.entry(local).or_default().insert(format!(
                    "package:{}:{}",
                    name,
                    compact_type(ty)
                ));
            }
        }
    }

    loop {
        let mut changed = false;
        for context in contexts {
            let function = &program.functions[&context.function];
            for assignment in function.all_assignments() {
                let origins =
                    origins_for_locals(provenance.get(&context.function), &assignment.dependencies);
                changed |= extend_origins(
                    provenance
                        .entry(context.function.clone())
                        .or_default()
                        .entry(assignment.destination.clone())
                        .or_default(),
                    origins,
                );
            }
            for call in function.all_calls() {
                let key = (context.clone(), call.block.clone(), call.ordinal);
                let targets = &resolved_calls[&key];
                let argument_origins: Vec<_> = call
                    .arguments
                    .iter()
                    .map(|argument| {
                        origins_for_locals(
                            provenance.get(&context.function),
                            &local_references(argument),
                        )
                    })
                    .collect();
                let all_arguments: BTreeSet<_> =
                    argument_origins.iter().flatten().cloned().collect();
                let mut result_origins = all_arguments.clone();
                for target in targets {
                    let ResolvedTarget::Local(child) = target else {
                        continue;
                    };
                    let child_function = &program.functions[&child.function];
                    let closure = child_function.display.contains("{closure");
                    for (index, _) in child_function.parameter_types.iter().enumerate() {
                        let supplied = if closure {
                            all_arguments.clone()
                        } else {
                            argument_origins.get(index).cloned().unwrap_or_default()
                        };
                        changed |= extend_origins(
                            provenance
                                .entry(child.function.clone())
                                .or_default()
                                .entry(format!("_{}", index + 1))
                                .or_default(),
                            supplied,
                        );
                    }
                    if let Some(returned) = provenance
                        .get(&child.function)
                        .and_then(|values| values.get("_0"))
                    {
                        result_origins.extend(returned.iter().cloned());
                    }
                }
                changed |= extend_origins(
                    provenance
                        .entry(context.function.clone())
                        .or_default()
                        .entry(call.result.clone())
                        .or_default(),
                    result_origins,
                );
            }
        }
        if !changed {
            break;
        }
    }
    Ok(provenance)
}

fn root_parameter_is_package(ty: &str) -> bool {
    let ty = one_line(ty);
    if ty == "u64" || ty.contains("&mut artifacts::ArtifactStore") {
        return false;
    }
    contains_any(
        &ty,
        &[
            "CompilerPackage",
            "[u8]",
            "Definition",
            "KExpr",
            "KValue",
            "KSort",
            "Term",
            "Id32",
            "Evaluator",
        ],
    )
}

fn origins_for_locals(
    values: Option<&BTreeMap<String, BTreeSet<String>>>,
    locals: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut output = BTreeSet::new();
    let Some(values) = values else {
        return output;
    };
    for local in locals {
        output.extend(values.get(local).into_iter().flatten().cloned());
    }
    output
}

fn extend_origins(target: &mut BTreeSet<String>, values: BTreeSet<String>) -> bool {
    let before = target.len();
    target.extend(values);
    target.len() != before
}

fn derive_handler_sets(
    program: &MirProgram,
    contexts: &BTreeSet<ContextKey>,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
) -> BTreeMap<ContextKey, BTreeSet<String>> {
    let mut handlers: BTreeMap<_, BTreeSet<String>> = contexts
        .iter()
        .cloned()
        .map(|context| (context, BTreeSet::new()))
        .collect();
    loop {
        let mut changed = false;
        for context in contexts {
            let function = &program.functions[&context.function];
            let mut discovered = BTreeSet::new();
            for call in function.all_calls() {
                let key = (context.clone(), call.block.clone(), call.ordinal);
                for target in &resolved_calls[&key] {
                    match target {
                        ResolvedTarget::External(contract)
                            if contract.kind == ContractKind::DigestOneShot =>
                        {
                            discovered.insert(contract.target.clone());
                        }
                        ResolvedTarget::Local(child) => {
                            discovered.extend(handlers.get(child).into_iter().flatten().cloned());
                        }
                        ResolvedTarget::External(_) => {}
                    }
                }
            }
            changed |= extend_origins(handlers.entry(context.clone()).or_default(), discovered);
        }
        if !changed {
            break;
        }
    }
    handlers
}

fn build_rows(
    inventory: &SourceInventory,
    program: &MirProgram,
    contexts: &BTreeSet<ContextKey>,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
    provenance: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    handler_sets: &BTreeMap<ContextKey, BTreeSet<String>>,
) -> Result<Vec<EvidenceRow>, AuditError> {
    let (context_outcomes, block_outcomes) =
        derive_outcome_closure(inventory, program, contexts, resolved_calls)?;
    let mut rows = Vec::new();
    for context in contexts {
        let function = &program.functions[&context.function];
        let source = function
            .source
            .as_ref()
            .ok_or_else(|| AuditError::UnboundMirFunction {
                function: context.function.clone(),
            })?;
        let actual_function_outcomes = context_outcomes
            .get(context)
            .cloned()
            .unwrap_or_else(|| BTreeSet::from(["unresolved-outcome".to_owned()]));
        let allowed_function_outcomes = context.class.outcome_set();
        validate_outcome_subset(
            &allowed_function_outcomes,
            &actual_function_outcomes,
            &context.function,
        )?;
        let reachable = reachable_blocks(inventory, function)?;
        let block_handlers =
            block_handler_sets(program, context, function, resolved_calls, handler_sets);

        for block in function
            .blocks
            .values()
            .filter(|block| reachable.contains(&block.id))
        {
            for call in &block.calls {
                let key = (context.clone(), call.block.clone(), call.ordinal);
                let targets = &resolved_calls[&key];
                let call_origins = origins_for_locals(
                    provenance.get(&context.function),
                    &call
                        .arguments
                        .iter()
                        .flat_map(|argument| local_references(argument))
                        .collect(),
                );
                let result_type = function
                    .locals
                    .get(&call.result)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_owned());
                let (class, witness) = call_class(context, targets, &result_type);
                let allowed = class.outcome_set();
                let actual = outcomes_for_targets(
                    inventory,
                    targets,
                    class,
                    &result_type,
                    &context_outcomes,
                )?;
                validate_outcome_subset(
                    &allowed,
                    &actual,
                    &format!("{} {}:{}", context.function, call.block, call.ordinal),
                )?;
                let handler = targets.iter().find_map(|target| match target {
                    ResolvedTarget::External(contract)
                        if contract.kind == ContractKind::DigestOneShot =>
                    {
                        Some(contract.target.clone())
                    }
                    _ => None,
                });
                rows.push(EvidenceRow {
                    ir_location: format!("{}::{}:{}", context.function, call.block, call.ordinal),
                    source_location: format!("{}:{}:{}", source.path, source.line, source.column),
                    function: source.id.clone(),
                    kind: SiteKind::Call,
                    class,
                    witness,
                    influenced: !call_origins.is_empty(),
                    provenance: render_origins(&call_origins),
                    control_type: call
                        .arguments
                        .iter()
                        .map(|argument| function.local_type(argument))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join("|"),
                    tag_signature: handler.map_or_else(
                        || "-".to_owned(),
                        |_| "fixed-tag:Id32;signature:[KValue::Bytes]->KValue::Bytes".to_owned(),
                    ),
                    arm_map: "-".to_owned(),
                    allowed_outcomes: render_set(allowed),
                    actual_outcomes: render_set(actual),
                    targets: targets.iter().map(ResolvedTarget::render).collect(),
                    edges: call
                        .successors
                        .iter()
                        .map(|edge| format!("edge::{edge}"))
                        .collect(),
                });
            }

            for switch in &block.switches {
                let terminator = block.terminator.as_ref().ok_or_else(|| {
                    AuditError::MirParse(format!(
                        "{} {} has a switch without a terminator",
                        context.function, block.id
                    ))
                })?;
                let successors = outcome_successors(inventory, function, block, terminator)?;
                let dependencies = dependency_locals(function, &switch.operand);
                let origins = origins_for_locals(provenance.get(&context.function), &dependencies);
                let control_type = dependency_types(function, &dependencies);
                let category = control_category(&control_type);
                let arm_handlers: Vec<_> = successors
                    .iter()
                    .map(|successor| block_handlers.get(successor).cloned().unwrap_or_default())
                    .collect();
                let handler_union: BTreeSet<_> = arm_handlers.iter().flatten().cloned().collect();
                if !origins.is_empty() && !handler_union.is_empty() {
                    validate_handler_relation(
                        category,
                        &arm_handlers,
                        &format!("{} {}:{}", context.function, switch.block, switch.ordinal),
                    )?;
                }
                let class = site_class(context.class, &control_type);
                let mut branch_outcomes: BTreeSet<_> = successors
                    .iter()
                    .flat_map(|successor| {
                        block_outcomes
                            .get(&(context.clone(), successor.clone()))
                            .into_iter()
                            .flatten()
                            .cloned()
                    })
                    .collect();
                if branch_outcomes.is_empty() {
                    branch_outcomes.insert("unresolved-outcome".to_owned());
                }
                let allowed = class.outcome_set();
                validate_outcome_subset(
                    &allowed,
                    &branch_outcomes,
                    &format!("{} {}:{}", context.function, switch.block, switch.ordinal),
                )?;
                rows.push(EvidenceRow {
                    ir_location: format!(
                        "{}::{}:{}",
                        context.function, switch.block, switch.ordinal
                    ),
                    source_location: format!("{}:{}:{}", source.path, source.line, source.column),
                    function: source.id.clone(),
                    kind: SiteKind::Switch,
                    class,
                    witness: format!("typed-control:{}", compact_type(&control_type)),
                    influenced: !origins.is_empty(),
                    provenance: render_origins(&origins),
                    control_type,
                    tag_signature: if handler_union.is_empty() {
                        "-".to_owned()
                    } else {
                        format!(
                            "{};signature:[KValue::Bytes]->KValue::Bytes",
                            category.name()
                        )
                    },
                    arm_map: render_arm_map(&successors, &arm_handlers),
                    allowed_outcomes: render_set(allowed),
                    actual_outcomes: render_set(branch_outcomes.iter().cloned()),
                    targets: successors
                        .iter()
                        .map(|target| format!("block::{target}"))
                        .collect(),
                    edges: successors
                        .iter()
                        .map(|edge| format!("edge::{edge}"))
                        .collect(),
                });
            }

            if let Some(terminator) = &block.terminator
                && matches!(terminator.kind, SiteKind::Assert | SiteKind::Drop)
            {
                let dependencies = dependency_locals(function, &terminator.control);
                let origins = origins_for_locals(provenance.get(&context.function), &dependencies);
                let control_type = dependency_types(function, &dependencies);
                let class = site_class(context.class, &control_type);
                let outcomes = block_outcomes
                    .get(&(context.clone(), block.id.clone()))
                    .cloned()
                    .unwrap_or_else(|| BTreeSet::from(["unresolved-outcome".to_owned()]));
                let allowed = class.outcome_set();
                validate_outcome_subset(
                    &allowed,
                    &outcomes,
                    &format!(
                        "{} {}:{}",
                        context.function, terminator.block, terminator.ordinal
                    ),
                )?;
                rows.push(EvidenceRow {
                    ir_location: format!(
                        "{}::{}:{}",
                        context.function, terminator.block, terminator.ordinal
                    ),
                    source_location: format!("{}:{}:{}", source.path, source.line, source.column),
                    function: source.id.clone(),
                    kind: terminator.kind,
                    class,
                    witness: format!(
                        "fixed-terminator-contract:{};control:{}",
                        terminator.kind.name(),
                        compact_type(&control_type)
                    ),
                    influenced: !origins.is_empty(),
                    provenance: render_origins(&origins),
                    control_type,
                    tag_signature: "-".to_owned(),
                    arm_map: "-".to_owned(),
                    allowed_outcomes: render_set(allowed),
                    actual_outcomes: render_set(outcomes.iter().cloned()),
                    targets: vec![terminator.kind.name().to_owned()],
                    edges: terminator
                        .successors
                        .iter()
                        .map(|edge| format!("edge::{edge}"))
                        .collect(),
                });
            }
        }
    }
    rows.sort();
    Ok(rows)
}

fn block_handler_sets(
    _program: &MirProgram,
    context: &ContextKey,
    function: &MirFunction,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
    handler_sets: &BTreeMap<ContextKey, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut blocks: BTreeMap<String, BTreeSet<String>> = function
        .blocks
        .keys()
        .cloned()
        .map(|block| (block, BTreeSet::new()))
        .collect();
    for block in function.blocks.values() {
        let mut local = BTreeSet::new();
        for call in &block.calls {
            let key = (context.clone(), call.block.clone(), call.ordinal);
            for target in &resolved_calls[&key] {
                match target {
                    ResolvedTarget::External(contract)
                        if contract.kind == ContractKind::DigestOneShot =>
                    {
                        local.insert(contract.target.clone());
                    }
                    ResolvedTarget::Local(child) => {
                        local.extend(handler_sets.get(child).into_iter().flatten().cloned());
                    }
                    ResolvedTarget::External(_) => {}
                }
            }
        }
        blocks.insert(block.id.clone(), local);
    }
    loop {
        let mut changed = false;
        for block in function.blocks.values() {
            let mut discovered = blocks.get(&block.id).cloned().unwrap_or_default();
            if let Some(terminator) = &block.terminator {
                for successor in &terminator.successors {
                    discovered.extend(blocks.get(successor).into_iter().flatten().cloned());
                }
            }
            changed |= extend_origins(blocks.entry(block.id.clone()).or_default(), discovered);
        }
        if !changed {
            break;
        }
    }
    blocks
}

fn call_class(
    context: &ContextKey,
    targets: &[ResolvedTarget],
    result_type: &str,
) -> (Mechanic, String) {
    if let Some(contract) = targets.iter().find_map(|target| match target {
        ResolvedTarget::External(contract) => Some(contract),
        ResolvedTarget::Local(_) => None,
    }) {
        return match contract.kind {
            ContractKind::DigestOneShot if context.class == Mechanic::PhysicalDispatch => (
                Mechanic::PhysicalDispatch,
                "external-contract:digest-one-shot".to_owned(),
            ),
            ContractKind::DigestOneShot | ContractKind::DigestState => (
                Mechanic::CoreAbi,
                "external-contract:fixed-sha256".to_owned(),
            ),
            ContractKind::Platform => {
                let class = if context.class == Mechanic::KernelStep
                    && has_exact_byte_carrier(result_type)
                {
                    Mechanic::ByteMachine
                } else {
                    context.class
                };
                (
                    class,
                    format!(
                        "external-contract:platform;typed-result:{}",
                        compact_type(result_type)
                    ),
                )
            }
        };
    }
    if let Some(ResolvedTarget::Local(child)) = targets.first() {
        (child.class, child.witness.clone())
    } else {
        (context.class, context.witness.clone())
    }
}

fn site_class(context: Mechanic, control_type: &str) -> Mechanic {
    if context == Mechanic::KernelStep && has_exact_byte_carrier(control_type) {
        Mechanic::ByteMachine
    } else {
        context
    }
}

fn has_exact_byte_carrier(ty: &str) -> bool {
    let compact = one_line(ty)
        .replace(' ', "")
        .replace("std::vec::Vec", "Vec");
    ["Vec<u8>", "[u8]", "GenericArray<u8,"]
        .iter()
        .any(|fragment| contains_type_fragment(&compact, fragment))
}

fn contains_type_fragment(value: &str, fragment: &str) -> bool {
    let mut offset = 0;
    while let Some(found) = value[offset..].find(fragment) {
        let start = offset + found;
        let before = value[..start].chars().next_back();
        if !before.is_some_and(|character| character.is_ascii_alphanumeric() || character == '_') {
            return true;
        }
        offset = start + fragment.len();
    }
    false
}

type ContextOutcomes = BTreeMap<ContextKey, BTreeSet<String>>;
type BlockOutcomes = BTreeMap<(ContextKey, String), BTreeSet<String>>;

fn derive_outcome_closure(
    inventory: &SourceInventory,
    program: &MirProgram,
    contexts: &BTreeSet<ContextKey>,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
) -> Result<(ContextOutcomes, BlockOutcomes), AuditError> {
    let mut context_outcomes: ContextOutcomes = contexts
        .iter()
        .cloned()
        .map(|context| (context, BTreeSet::new()))
        .collect();
    let mut block_outcomes = BlockOutcomes::new();
    for context in contexts {
        for block in program.functions[&context.function].blocks.keys() {
            block_outcomes.insert((context.clone(), block.clone()), BTreeSet::new());
        }
    }

    loop {
        let mut changed = false;
        for context in contexts {
            let function = &program.functions[&context.function];
            for block in function.blocks.values() {
                let mut discovered = BTreeSet::new();
                for assignment in &block.assignments {
                    let outcomes = assignment_outcomes(
                        inventory,
                        context.class,
                        &function.return_type,
                        assignment,
                    )?;
                    discovered.extend(outcomes.into_iter().filter(|outcome| {
                        assignment.destination == "_0" || forbidden_outcome(outcome)
                    }));
                }
                for call in &block.calls {
                    let key = (context.clone(), call.block.clone(), call.ordinal);
                    let targets = &resolved_calls[&key];
                    let result_type = function
                        .locals
                        .get(&call.result)
                        .map_or("unknown", String::as_str);
                    let (class, _) = call_class(context, targets, result_type);
                    let outcomes = outcomes_for_targets(
                        inventory,
                        targets,
                        class,
                        result_type,
                        &context_outcomes,
                    )?;
                    discovered.extend(
                        outcomes
                            .into_iter()
                            .filter(|outcome| forbidden_outcome(outcome)),
                    );
                }
                if let Some(terminator) = &block.terminator {
                    for successor in outcome_successors(inventory, function, block, terminator)? {
                        discovered.extend(
                            block_outcomes
                                .get(&(context.clone(), successor))
                                .into_iter()
                                .flatten()
                                .cloned(),
                        );
                    }
                    if terminator.raw == "return;" {
                        discovered.extend(outcomes_for_return_type(
                            inventory,
                            context.class,
                            &function.return_type,
                        )?);
                    } else if terminator.raw.starts_with("assert(") {
                        discovered.insert(format!("host-panic:{}::{}", context.function, block.id));
                    } else if terminator.raw == "unreachable;" {
                        discovered.insert(format!(
                            "host-unreachable:{}::{}",
                            context.function, block.id
                        ));
                    } else if terminator.raw == "resume;"
                        || terminator.raw.starts_with("terminate(")
                    {
                        discovered.insert(format!("host-abort:{}::{}", context.function, block.id));
                    }
                }
                changed |= extend_origins(
                    block_outcomes
                        .entry((context.clone(), block.id.clone()))
                        .or_default(),
                    discovered,
                );
            }
            let entry = block_outcomes
                .get(&(context.clone(), "bb0".to_owned()))
                .cloned()
                .unwrap_or_default();
            changed |= extend_origins(context_outcomes.entry(context.clone()).or_default(), entry);
        }
        if !changed {
            break;
        }
    }

    for context in contexts {
        let actual = context_outcomes.entry(context.clone()).or_default();
        if actual.is_empty() {
            actual.insert("unresolved-outcome".to_owned());
        }
        validate_outcome_subset(&context.class.outcome_set(), actual, &context.function)?;
    }
    Ok((context_outcomes, block_outcomes))
}

fn forbidden_outcome(outcome: &str) -> bool {
    outcome.starts_with("host-")
        || outcome.starts_with("unauthorized-")
        || outcome == "unresolved-outcome"
}

fn outcome_successors(
    inventory: &SourceInventory,
    function: &MirFunction,
    block: &MirBlock,
    terminator: &MirTerminator,
) -> Result<Vec<String>, AuditError> {
    if terminator.kind == SiteKind::Switch
        && let Some(switch) = block
            .switches
            .iter()
            .find(|switch| switch.ordinal == terminator.ordinal)
        && let Some(cardinality) = fixed_discriminant_cardinality(inventory, function, switch)?
    {
        let mut explicit = BTreeMap::new();
        for arm in switch.arms.split(',') {
            let Some((label, target)) = arm.trim().split_once(':') else {
                continue;
            };
            let Some(target) = target.trim().strip_prefix("bb") else {
                continue;
            };
            let target = format!(
                "bb{}",
                target
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
            );
            if let Ok(discriminant) = label.trim().parse::<usize>() {
                explicit.insert(discriminant, target);
            }
        }
        if explicit.keys().copied().eq(0..cardinality) {
            return Ok(explicit.into_values().collect());
        }
    }
    let ordinary_label = if terminator.raw.contains(" -> [return:") {
        Some("return")
    } else if terminator.raw.contains(" -> [success:") {
        Some("success")
    } else {
        None
    };
    Ok(ordinary_label
        .and_then(|label| labeled_basic_block(&terminator.raw, label))
        .map_or_else(
            || terminator.successors.clone(),
            |successor| vec![successor],
        ))
}

fn reachable_blocks(
    inventory: &SourceInventory,
    function: &MirFunction,
) -> Result<BTreeSet<String>, AuditError> {
    let mut reachable = BTreeSet::new();
    let mut pending = VecDeque::from(["bb0".to_owned()]);
    while let Some(block_id) = pending.pop_front() {
        if !reachable.insert(block_id.clone()) {
            continue;
        }
        let block = function.blocks.get(&block_id).ok_or_else(|| {
            AuditError::MirParse(format!(
                "{} references missing block {block_id}",
                function.display
            ))
        })?;
        if let Some(terminator) = &block.terminator {
            for successor in outcome_successors(inventory, function, block, terminator)? {
                if !reachable.contains(&successor) {
                    pending.push_back(successor);
                }
            }
        }
    }
    Ok(reachable)
}

fn fixed_discriminant_cardinality(
    inventory: &SourceInventory,
    function: &MirFunction,
    switch: &MirSwitch,
) -> Result<Option<usize>, AuditError> {
    let control = base_local(&switch.operand);
    let discriminant = control.as_deref().and_then(|control| {
        function.all_assignments().find(|assignment| {
            assignment.destination == control
                && assignment.value.trim_start().starts_with("discriminant(")
        })
    });
    let Some(discriminant) = discriminant else {
        return Ok(None);
    };
    let mut type_witnesses = vec![discriminant.value.as_str()];
    if let Some(ty) = base_local(&discriminant.value)
        .as_deref()
        .and_then(|local| function.locals.get(local))
    {
        type_witnesses.push(ty);
    }
    if type_witnesses.iter().any(|witness| {
        let compact = one_line(witness).replace(' ', "");
        [
            "Result<",
            "Option<",
            "ControlFlow<",
            "std::result::Result<",
            "std::option::Option<",
            "std::ops::ControlFlow<",
        ]
        .iter()
        .any(|fragment| contains_type_fragment(&compact, fragment))
    }) {
        return Ok(Some(2));
    }
    let module = function
        .source
        .as_ref()
        .ok_or_else(|| AuditError::UnboundMirFunction {
            function: function.display.clone(),
        })?
        .module
        .as_str();
    for witness in type_witnesses {
        for spelling in type_path_spellings(witness) {
            let Some(nominal) = inventory.resolve_nominal_in_module(module, &spelling)? else {
                continue;
            };
            if let Some((_, variants)) = CLOSED_ENUMS
                .iter()
                .find(|(expected, _)| *expected == nominal)
            {
                return Ok(Some(variants.len()));
            }
        }
    }
    Ok(None)
}

fn labeled_basic_block(value: &str, label: &str) -> Option<String> {
    let marker = format!("{label}: ");
    let rest = value.split_once(&marker)?.1;
    let start = rest.find("bb")?;
    let digits = rest[start + 2..]
        .chars()
        .take_while(char::is_ascii_digit)
        .count();
    (digits > 0).then(|| rest[start..start + 2 + digits].to_owned())
}

fn assignment_outcomes(
    inventory: &SourceInventory,
    class: Mechanic,
    return_type: &str,
    assignment: &MirAssignment,
) -> Result<BTreeSet<String>, AuditError> {
    let mut outcomes = BTreeSet::new();
    for path in type_path_spellings(&assignment.value) {
        let Some((spelling, variant)) = path.rsplit_once("::") else {
            continue;
        };
        let Some(nominal) = inventory.resolve_nominal(spelling)? else {
            continue;
        };
        let Some(variants) = inventory.enum_variants.get(&nominal) else {
            continue;
        };
        if !variants.contains(variant) {
            continue;
        }
        let expected = CLOSED_ENUMS
            .iter()
            .find(|(qualified, _)| *qualified == nominal)
            .map(|(_, variants)| *variants);
        if !expected.is_some_and(|expected| expected.contains(&variant)) {
            outcomes.insert(format!("unauthorized-enum:{nominal}::{variant}"));
        } else if is_fixed_error_enum(&nominal) {
            outcomes.insert("fixed-error".to_owned());
        } else {
            outcomes.insert(successful_outcome(class, std::iter::once(nominal.as_str())));
        }
    }

    if assignment.destination == "_0" {
        if contains_any(&assignment.value, &["::Err(", "from_residual", "::None"]) {
            outcomes.insert("fixed-error".to_owned());
        }
        if contains_any(&assignment.value, &["::Ok(", "::Some("]) {
            outcomes.extend(outcomes_for_return_type(inventory, class, return_type)?);
        }
    }
    Ok(outcomes)
}

fn outcomes_for_targets(
    inventory: &SourceInventory,
    targets: &[ResolvedTarget],
    class: Mechanic,
    result_type: &str,
    context_outcomes: &ContextOutcomes,
) -> Result<BTreeSet<String>, AuditError> {
    let mut outcomes = BTreeSet::new();
    for target in targets {
        match target {
            ResolvedTarget::Local(child) => {
                outcomes.extend(context_outcomes.get(child).into_iter().flatten().cloned());
            }
            ResolvedTarget::External(contract) => match contract.kind {
                ContractKind::DigestOneShot if class == Mechanic::PhysicalDispatch => {
                    outcomes.insert("fixed-Sha256-handler".to_owned());
                }
                ContractKind::DigestOneShot | ContractKind::DigestState => {
                    outcomes.insert("canonical-data".to_owned());
                }
                ContractKind::Platform => {
                    outcomes.extend(outcomes_for_return_type(inventory, class, result_type)?);
                }
            },
        }
    }
    if outcomes.is_empty()
        && targets
            .iter()
            .any(|target| matches!(target, ResolvedTarget::External(_)))
    {
        outcomes.extend(outcomes_for_return_type(inventory, class, result_type)?);
    }
    Ok(outcomes)
}

fn outcomes_for_return_type(
    inventory: &SourceInventory,
    class: Mechanic,
    ty: &str,
) -> Result<BTreeSet<String>, AuditError> {
    let mut nominals = BTreeSet::new();
    let paths = type_path_spellings(ty);
    for path in &paths {
        if let Some(nominal) = inventory.resolve_nominal(path)? {
            if !CONSTITUTIONAL_NOMINALS.contains(&nominal.as_str()) {
                return Ok(BTreeSet::from([format!(
                    "unauthorized-return-carrier:{nominal}"
                )]));
            }
            nominals.insert(nominal);
        }
    }
    let mut outcomes = BTreeSet::from([successful_outcome(
        class,
        nominals.iter().map(String::as_str),
    )]);
    if paths.iter().any(|path| {
        matches!(
            path.rsplit_once("::")
                .map_or(path.as_str(), |(_, name)| name),
            "Result" | "Option" | "ControlFlow"
        )
    }) {
        outcomes.insert("fixed-error".to_owned());
    }
    Ok(outcomes)
}

fn successful_outcome<'a>(class: Mechanic, nominals: impl IntoIterator<Item = &'a str>) -> String {
    let nominals: BTreeSet<_> = nominals.into_iter().collect();
    match class {
        Mechanic::DefinitionTable => "selected-package-definition",
        Mechanic::PhysicalDispatch => "fixed-Sha256-handler",
        Mechanic::ByteMachine | Mechanic::KernelStep
            if nominals.iter().any(|nominal| {
                matches!(
                    *nominal,
                    "compiler_package_v3::types::KExpr"
                        | "evaluator::InferTask"
                        | "evaluator::EvalTask"
                )
            }) =>
        {
            "child-KExpr"
        }
        _ => "canonical-data",
    }
    .to_owned()
}

fn is_fixed_error_enum(nominal: &str) -> bool {
    matches!(
        nominal,
        "artifacts::ArtifactError"
            | "artifacts::CompilerArtifactError"
            | "compiler_package_v3::types::DecodeCode"
            | "compiler_package_v3::types::DecodeFailure"
            | "compiler_package_v3::types::EncodeError"
            | "evaluator::StaticError"
            | "evaluator::EvalError"
            | "compiler_package_v3::checker::AuthorizationCheckError"
            | "physical::PhysicalError"
    )
}

fn validate_outcome_subset(
    allowed: &BTreeSet<String>,
    actual: &BTreeSet<String>,
    location: &str,
) -> Result<(), AuditError> {
    let forbidden: BTreeSet<_> = actual.difference(allowed).cloned().collect();
    if forbidden.is_empty() {
        Ok(())
    } else {
        Err(AuditError::OutcomeViolation {
            location: location.to_owned(),
            allowed: allowed.clone(),
            actual: actual.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControlCategory {
    Tag,
    Signature,
    Payload,
    Other,
}

impl ControlCategory {
    const fn name(self) -> &'static str {
        match self {
            Self::Tag => "tag:Id32",
            Self::Signature => "signature:KValue",
            Self::Payload => "payload:Bytes",
            Self::Other => "other-package-control",
        }
    }
}

fn control_category(types: &str) -> ControlCategory {
    let paths = type_path_spellings(types);
    if paths.iter().any(|path| {
        path.rsplit_once("::")
            .map_or(path.as_str(), |(_, name)| name)
            == "Id32"
    }) {
        ControlCategory::Tag
    } else if paths.iter().any(|path| {
        path.rsplit_once("::")
            .map_or(path.as_str(), |(_, name)| name)
            == "KValue"
    }) {
        ControlCategory::Signature
    } else if has_exact_byte_carrier(types) {
        ControlCategory::Payload
    } else {
        ControlCategory::Other
    }
}

fn validate_handler_relation(
    category: ControlCategory,
    arms: &[BTreeSet<String>],
    location: &str,
) -> Result<(), AuditError> {
    let union: BTreeSet<_> = arms.iter().flatten().cloned().collect();
    if union.len() > 1 {
        return Err(AuditError::PackageSelectedTarget {
            location: location.to_owned(),
            category: category.name().to_owned(),
            arms: arms.to_vec(),
        });
    }
    Ok(())
}

fn dependency_locals(function: &MirFunction, expression: &str) -> BTreeSet<String> {
    let mut dependencies = local_references(expression);
    loop {
        let before = dependencies.len();
        for assignment in function.all_assignments() {
            if dependencies.contains(&assignment.destination) {
                dependencies.extend(assignment.dependencies.iter().cloned());
            }
        }
        if dependencies.len() == before {
            break;
        }
    }
    dependencies
}

fn dependency_types(function: &MirFunction, dependencies: &BTreeSet<String>) -> String {
    let types: BTreeSet<_> = dependencies
        .iter()
        .filter_map(|local| function.locals.get(local))
        .map(|ty| one_line(ty))
        .collect();
    if types.is_empty() {
        "constant".to_owned()
    } else {
        types.into_iter().collect::<Vec<_>>().join("|")
    }
}

fn render_arm_map(successors: &[String], handlers: &[BTreeSet<String>]) -> String {
    if successors.is_empty() {
        return "-".to_owned();
    }
    successors
        .iter()
        .enumerate()
        .map(|(index, successor)| {
            format!(
                "{}=>{}",
                successor,
                handlers
                    .get(index)
                    .map(|values| render_set(values.iter().cloned()))
                    .unwrap_or_else(|| "-".to_owned())
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn terminal_name(value: &str) -> String {
    let segments = top_level_path_segments(value);
    let candidate = segments
        .iter()
        .rev()
        .copied()
        .map(str::trim)
        .find(|segment| !is_standalone_generic_segment(segment))
        .unwrap_or(value.trim());
    candidate
        .split("::<")
        .next()
        .unwrap_or(candidate)
        .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .to_owned()
}

fn target_receiver(value: &str) -> Option<String> {
    if let Some((_, receiver, _)) = inherent_impl_target(value) {
        return Some(receiver);
    }
    let segments: Vec<_> = top_level_path_segments(value)
        .into_iter()
        .map(str::trim)
        .filter(|segment| !is_standalone_generic_segment(segment))
        .collect();
    let receiver = segments.get(segments.len().checked_sub(2)?)?;
    Some(
        receiver
            .split("::<")
            .next()
            .unwrap_or(receiver)
            .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .to_owned(),
    )
    .filter(|value| !value.is_empty())
}

fn target_source_suffix(value: &str) -> Option<String> {
    if value.trim_start().starts_with('<') {
        return None;
    }
    if let Some((module, receiver, method)) = inherent_impl_target(value) {
        return Some(format!("{module}::{receiver}::{method}"));
    }
    let segments: Vec<_> = top_level_path_segments(value)
        .into_iter()
        .map(str::trim)
        .filter(|segment| !is_standalone_generic_segment(segment))
        .collect();
    (segments.len() > 1).then(|| segments.join("::"))
}

fn inherent_impl_target(value: &str) -> Option<(String, String, String)> {
    let (module, rest) = value.trim().split_once("::<impl ")?;
    let (owner, method) = rest.split_once(">::")?;
    let receiver = owner
        .rsplit("::")
        .next()?
        .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '_');
    let method = terminal_name(method);
    (!module.is_empty() && !receiver.is_empty() && !method.is_empty())
        .then(|| (module.to_owned(), receiver.to_owned(), method))
}

fn is_standalone_generic_segment(segment: &str) -> bool {
    segment.starts_with('<') && segment.ends_with('>') && !segment.contains(" as ")
}

fn penultimate_path_segment(value: &str) -> Option<&str> {
    let segments: Vec<_> = value.split("::").collect();
    segments
        .len()
        .checked_sub(2)
        .and_then(|index| segments.get(index).copied())
}

fn top_level_path_segments(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut output = Vec::new();
    let mut start = 0;
    let mut angle = 0_usize;
    let mut brace = 0_usize;
    let mut paren = 0_usize;
    let mut index = 0;
    while index + 1 < bytes.len() {
        match bytes[index] {
            b'<' => angle += 1,
            b'>' => angle = angle.saturating_sub(1),
            b'{' => brace += 1,
            b'}' => brace = brace.saturating_sub(1),
            b'(' => paren += 1,
            b')' => paren = paren.saturating_sub(1),
            b':' if bytes[index + 1] == b':' && angle == 0 && brace == 0 && paren == 0 => {
                output.push(&value[start..index]);
                start = index + 2;
                index += 1;
            }
            _ => {}
        }
        index += 1;
    }
    output.push(&value[start..]);
    output
}

fn extract_source_ref(value: &str) -> Option<(String, usize, usize)> {
    let start = value.find("src/")?;
    let candidate = &value[start..];
    let file_end = candidate.find(".rs")? + 3;
    let path = candidate[..file_end].to_owned();
    let rest = candidate[file_end..].strip_prefix(':')?;
    let mut pieces = rest.split(':');
    let line = pieces.next()?.parse().ok()?;
    let column = pieces.next()?.parse().ok()?;
    Some((path, line, column))
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
        .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .to_owned()
}

fn canonical_tokens(value: &impl ToTokens) -> String {
    one_line(&value.to_token_stream().to_string())
}

fn one_line(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\t', " ")
}

fn compact_type(value: &str) -> String {
    one_line(value)
        .replace("compiler_package_v3::types::", "types::")
        .replace("std::result::", "std::")
        .replace("std::vec::", "std::")
}

fn sha256_hex(value: &[u8]) -> String {
    hex_digest(Sha256::digest(value))
}

fn hex_digest(value: impl AsRef<[u8]>) -> String {
    let mut output = String::with_capacity(value.as_ref().len() * 2);
    for byte in value.as_ref() {
        write!(output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn hash_file(path: &Path) -> Result<String, AuditError> {
    let bytes =
        fs::read(path).map_err(|error| AuditError::Io(format!("{}: {error}", path.display())))?;
    Ok(sha256_hex(&bytes))
}

fn render_origins(origins: &BTreeSet<String>) -> String {
    if origins.is_empty() {
        "-".to_owned()
    } else {
        render_set(origins.iter().cloned())
    }
}

fn render_set(values: impl IntoIterator<Item = String>) -> String {
    let values: BTreeSet<_> = values.into_iter().collect();
    if values.is_empty() {
        "-"
    } else {
        return values.into_iter().collect::<Vec<_>>().join("|");
    }
    .to_owned()
}

fn tsv_field(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

fn assert_or_update_fixture(relative: &str, actual: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    if std::env::var_os("CLAUSE_UPDATE_HOST_AUDIT").is_some() {
        fs::write(&path, actual).expect("audit fixture updates");
        return;
    }
    let expected = fs::read_to_string(&path).expect("audit fixture is readable");
    assert_eq!(actual, expected, "generated host audit drifted: {relative}");
}

#[derive(Clone, Debug)]
enum AuditError {
    Io(String),
    Compiler(String),
    SourceParse {
        path: String,
        error: String,
    },
    MirParse(String),
    AmbiguousSource(String),
    ClosedEnumChanged {
        name: String,
        expected: BTreeSet<String>,
        actual: BTreeSet<String>,
    },
    MissingSourceMirror(String),
    UnboundMirFunction {
        function: String,
    },
    MissingRoot {
        root: String,
        candidates: Vec<String>,
    },
    Unclassified {
        function: String,
        detail: String,
    },
    DynamicCall {
        function: String,
        target: String,
    },
    UnresolvedTarget {
        function: String,
        target: String,
    },
    UnresolvedCallback {
        function: String,
        parameter: usize,
    },
    UnsupportedExternalContract(String),
    InfallibleAllocation {
        function: String,
        target: String,
    },
    OutcomeViolation {
        location: String,
        allowed: BTreeSet<String>,
        actual: BTreeSet<String>,
    },
    PackageSelectedTarget {
        location: String,
        category: String,
        arms: Vec<BTreeSet<String>>,
    },
    FixedDispatch(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(detail) => write!(formatter, "I/O error: {detail}"),
            Self::Compiler(detail) => write!(formatter, "compiler evidence error: {detail}"),
            Self::SourceParse { path, error } => {
                write!(formatter, "source parse failed for {path}: {error}")
            }
            Self::MirParse(detail) => write!(formatter, "MIR parse failed: {detail}"),
            Self::AmbiguousSource(detail) => {
                write!(formatter, "source identity is ambiguous: {detail}")
            }
            Self::ClosedEnumChanged {
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "closed enum {name} changed: expected {expected:?}, observed {actual:?}"
            ),
            Self::MissingSourceMirror(path) => {
                write!(formatter, "MIR source has no inventoried mirror: {path}")
            }
            Self::UnboundMirFunction { function } => {
                write!(
                    formatter,
                    "MIR function has no exact source binding: {function}"
                )
            }
            Self::MissingRoot { root, candidates } => {
                write!(
                    formatter,
                    "constitutional root {root} resolved to {candidates:?}"
                )
            }
            Self::Unclassified { function, detail } => {
                write!(formatter, "function {function} is unclassified: {detail}")
            }
            Self::DynamicCall { function, target } => {
                write!(
                    formatter,
                    "function {function} has dynamic call target {target}"
                )
            }
            Self::UnresolvedTarget { function, target } => {
                write!(
                    formatter,
                    "function {function} has unresolved target {target}"
                )
            }
            Self::UnresolvedCallback {
                function,
                parameter,
            } => write!(
                formatter,
                "function {function} has unresolved callback parameter {parameter}"
            ),
            Self::UnsupportedExternalContract(target) => {
                write!(formatter, "unsupported external contract: {target}")
            }
            Self::InfallibleAllocation { function, target } => {
                write!(
                    formatter,
                    "function {function} reaches infallible allocation {target}"
                )
            }
            Self::OutcomeViolation {
                location,
                allowed,
                actual,
            } => write!(
                formatter,
                "outcomes at {location} exceed {allowed:?}: observed {actual:?}"
            ),
            Self::PackageSelectedTarget {
                location,
                category,
                arms,
            } => write!(
                formatter,
                "package-controlled {category} at {location} selects host targets {arms:?}"
            ),
            Self::FixedDispatch(detail) => {
                write!(formatter, "fixed physical dispatch failed: {detail}")
            }
        }
    }
}

impl std::error::Error for AuditError {}

#[test]
fn trusted_namespace_does_not_authorize_semantics() {
    let (inventory, mut program) = cloned_production_program();
    let (_, contexts, _) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (context, call) = first_reachable_call(&program, &contexts, |_, _| true);
    replace_call_target(
        &mut program,
        &context,
        &call,
        "evaluator::TrustedNamespace::grammar_handler",
    );
    let error = build_context_closure(&inventory, &program)
        .expect_err("trusted spellings cannot authorize a mutated production call");
    assert!(matches!(error, AuditError::UnsupportedExternalContract(_)));
}

#[test]
fn same_tag_payload_cannot_select_handler() {
    let (inventory, mut program) = cloned_production_program();
    let (roots, contexts, mut resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let mut provenance = derive_provenance(&program, &roots, &contexts, &resolved_calls)
        .expect("production provenance closes");
    let (context, switch) = first_switch_for_handler_mutation(&program, &contexts);
    let dependencies = {
        let function = &program.functions[&context.function];
        dependency_locals(function, &switch.operand)
    };
    assert!(
        !dependencies.is_empty(),
        "production switch has typed control"
    );
    {
        let function = program
            .functions
            .get_mut(&context.function)
            .expect("switch owner remains present");
        for dependency in &dependencies {
            function
                .locals
                .insert(dependency.clone(), "std::vec::Vec<u8>".to_owned());
        }
    }
    for dependency in &dependencies {
        provenance
            .entry(context.function.clone())
            .or_default()
            .entry(dependency.clone())
            .or_default()
            .insert("package:mutated-payload:types::KValue".to_owned());
    }
    assert_eq!(
        control_category(&dependency_types(
            &program.functions[&context.function],
            &dependencies,
        )),
        ControlCategory::Payload,
        "the mutated production CFG is payload-controlled"
    );

    let successors: Vec<_> = switch.successors.iter().take(2).cloned().collect();
    let sibling_contexts: Vec<_> = contexts
        .iter()
        .filter(|candidate| candidate.function == context.function)
        .cloned()
        .collect();
    for (index, successor) in successors.iter().enumerate() {
        let ordinal = append_actual_call_to_block(
            &mut program,
            &context.function,
            successor,
            format!("fixed-handler-{}", index + 1),
        );
        let target = ResolvedTarget::External(ExternalContract {
            kind: ContractKind::DigestOneShot,
            target: format!("fixed-handler-{}", index + 1),
        });
        for sibling in &sibling_contexts {
            resolved_calls.insert(
                (sibling.clone(), successor.clone(), ordinal),
                vec![target.clone()],
            );
        }
    }

    let handler_sets = derive_handler_sets(&program, &contexts, &resolved_calls);
    let error = build_rows(
        &inventory,
        &program,
        &contexts,
        &resolved_calls,
        &provenance,
        &handler_sets,
    )
    .expect_err("payload-selected production CFG arms cannot select two handlers");
    assert!(
        matches!(error, AuditError::PackageSelectedTarget { .. }),
        "unexpected payload-selected handler verdict: {error:?}"
    );
}

#[test]
fn inferred_or_enum_callable_is_dynamic() {
    let (inventory, mut program) = cloned_production_program();
    let (_, contexts, _) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (context, call) = first_reachable_call(&program, &contexts, |_, _| true);
    replace_call_target(&mut program, &context, &call, "move _1");
    let error = build_context_closure(&inventory, &program)
        .expect_err("an inferred operand cannot replace an actual production target");
    assert!(matches!(error, AuditError::DynamicCall { .. }));
}

#[test]
fn reachable_new_module_cannot_escape_inventory() {
    let (inventory, mut program) = cloned_production_program();
    let (roots, _, resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (call_key, child) = resolved_calls
        .iter()
        .find_map(|(key, targets)| {
            (roots.contains(&key.0.function))
                .then(|| {
                    targets.iter().find_map(|target| match target {
                        ResolvedTarget::Local(child)
                            if !program.functions[&child.function]
                                .parameter_types
                                .iter()
                                .any(|ty| type_is_callable_text(ty)) =>
                        {
                            Some((key.clone(), child.clone()))
                        }
                        _ => None,
                    })
                })
                .flatten()
        })
        .expect("a real root reaches a non-callback production helper");
    let mut cloned = program.functions[&child.function].clone();
    let injected = "new_module::production_helper_clone".to_owned();
    cloned.display = injected.clone();
    let source = cloned
        .source
        .as_mut()
        .expect("reachable production helper has a source binding");
    source.id = injected.clone();
    source.module = "new_module".to_owned();
    source.owner = None;
    program.functions.insert(injected.clone(), cloned);
    let original_call = program.functions[&call_key.0.function].blocks[&call_key.1]
        .calls
        .iter()
        .find(|call| call.ordinal == call_key.2)
        .expect("root call remains present")
        .clone();
    replace_call_target(&mut program, &call_key.0, &original_call, &injected);
    let error = build_context_closure(&inventory, &program)
        .expect_err("a cloned production helper in a new module must reject");
    assert!(matches!(error, AuditError::Unclassified { .. }));
}

#[test]
fn same_module_helper_requires_a_closed_mechanic_contract() {
    let (mut inventory, mut program) = cloned_production_program();
    let (_, _, resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (call_key, child) = first_owned_local_call_edge(&program, &resolved_calls);
    let mut cloned = program.functions[&child.function].clone();
    let original_source = cloned
        .source
        .as_ref()
        .expect("reachable production helper has source identity")
        .clone();
    let injected = format!("{}::unauthorized_helper", original_source.id);
    cloned.display = injected.clone();
    cloned
        .source
        .as_mut()
        .expect("cloned helper retains source identity")
        .id = injected.clone();

    let mut declaration = inventory
        .functions
        .iter()
        .find(|declaration| declaration.id == original_source.id)
        .expect("production helper has an exact source declaration")
        .clone();
    declaration.id = injected.clone();
    declaration.name = "unauthorized_helper".to_owned();
    inventory.functions.push(declaration);
    program.functions.insert(injected.clone(), cloned);

    let original_call = program.functions[&call_key.0.function].blocks[&call_key.1]
        .calls
        .iter()
        .find(|call| call.ordinal == call_key.2)
        .expect("production call remains present")
        .clone();
    replace_call_target(&mut program, &call_key.0, &original_call, &injected);
    let error = build_context_closure(&inventory, &program)
        .expect_err("same-module source membership cannot mint a mechanic contract");
    assert!(matches!(error, AuditError::Unclassified { .. }));
}

#[test]
fn macro_expansion_is_audited() {
    let (inventory, mut program) = cloned_production_program();
    let (_, contexts, _) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (context, call) =
        first_reachable_call(&program, &contexts, |_, call| !call.closure_refs.is_empty());
    replace_call_target(
        &mut program,
        &context,
        &call,
        "expanded_foreign::binding_handler",
    );
    let error = build_context_closure(&inventory, &program)
        .expect_err("a foreign target substituted at a macro/closure call must reject");
    assert!(matches!(error, AuditError::UnsupportedExternalContract(_)));
}

#[test]
fn trusted_platform_substrings_do_not_authorize_target() {
    let (inventory, mut program) = cloned_production_program();
    let (_, _, resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (key, original) = resolved_calls
        .iter()
        .find_map(|(key, targets)| {
            targets
                .iter()
                .any(|target| {
                    matches!(
                        target,
                        ResolvedTarget::External(ExternalContract {
                            kind: ContractKind::Platform,
                            ..
                        })
                    )
                })
                .then(|| {
                    let call = program.functions[&key.0.function].blocks[&key.1]
                        .calls
                        .iter()
                        .find(|call| call.ordinal == key.2)?;
                    Some((key.clone(), call.clone()))
                })
                .flatten()
        })
        .expect("production closure contains an actual platform call");
    replace_call_target(
        &mut program,
        &key.0,
        &original,
        "<ForeignVec<types::KExpr> as ForeignIndex>::index",
    );
    let error = build_context_closure(&inventory, &program)
        .expect_err("trusted-looking Vec and Index tokens cannot mint a platform contract");
    assert!(matches!(error, AuditError::UnsupportedExternalContract(_)));
}

#[test]
fn production_digest_lookalike_does_not_authorize_target() {
    let (inventory, mut program) = cloned_production_program();
    let (_, _, resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (key, original, digest_target) = resolved_calls
        .iter()
        .find_map(|(key, targets)| {
            let digest_target = targets.iter().find_map(|target| match target {
                ResolvedTarget::External(ExternalContract {
                    kind: ContractKind::DigestOneShot,
                    target,
                }) => Some(target.clone()),
                _ => None,
            })?;
            let call = program.functions[&key.0.function].blocks[&key.1]
                .calls
                .iter()
                .find(|call| call.ordinal == key.2)?
                .clone();
            Some((key.clone(), call, digest_target))
        })
        .expect("production closure contains the one-shot SHA-256 call");
    let lookalike = format!("<ForeignDigestTarget<{digest_target}> as ForeignDigest>::invoke");
    assert!(
        lookalike.contains(" as Digest>::digest"),
        "the mutation preserves the formerly trusted substring"
    );
    replace_call_target(&mut program, &key.0, &original, &lookalike);

    let error = build_context_closure(&inventory, &program)
        .expect_err("a wrapped production digest spelling is not an exact digest contract");
    assert!(matches!(error, AuditError::UnsupportedExternalContract(_)));
}

#[test]
fn actual_outcome_must_be_subset() {
    let (mut inventory, mut program) = cloned_production_program();
    let (_, contexts, resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (context, block) = contexts
        .iter()
        .find_map(|context| {
            program.functions[&context.function]
                .blocks
                .get("bb0")
                .filter(|block| !block.assignments.is_empty())
                .map(|_| (context.clone(), "bb0".to_owned()))
        })
        .expect("a reachable production entry block has an assignment");
    inventory
        .enum_variants
        .get_mut("compiler_package_v3::types::KExpr")
        .expect("KExpr is in the exact closed inventory")
        .insert("HostSemanticHandler".to_owned());
    let assignment = program
        .functions
        .get_mut(&context.function)
        .expect("outcome owner remains present")
        .blocks
        .get_mut(&block)
        .expect("outcome block remains present")
        .assignments
        .first_mut()
        .expect("outcome assignment remains present");
    assignment.value = "compiler_package_v3::types::KExpr::HostSemanticHandler".to_owned();
    assignment.dependencies.clear();
    let error = derive_outcome_closure(&inventory, &program, &contexts, &resolved_calls)
        .expect_err("a new constructor in production MIR cannot widen fixed outcomes");
    assert!(matches!(error, AuditError::OutcomeViolation { .. }));
}

#[test]
fn classification_is_name_independent() {
    let (inventory, mut program) = cloned_production_program();
    let (_, _, resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (parent, child) = first_owned_local_edge(&program, &resolved_calls);
    let mut renamed = program.functions[&child.function].clone();
    let renamed_id = format!("{}__renamed", child.function);
    renamed.display = renamed_id.clone();
    let source = renamed
        .source
        .as_ref()
        .expect("production child has source identity");
    assert!(!source.id.is_empty(), "exact source identity remains fixed");
    program.functions.insert(renamed_id.clone(), renamed);
    let renamed_context = child_context(
        &inventory,
        &program,
        &renamed_id,
        &parent,
        child.callbacks.clone(),
    )
    .expect("renaming the MIR display does not alter exact source classification");
    assert_eq!(renamed_context.class, child.class);
    assert_eq!(renamed_context.witness, child.witness);
}

#[test]
fn lookalike_owner_and_carrier_are_not_constitutional_witnesses() {
    let (inventory, program) = cloned_production_program();
    let (_, _, resolved_calls) =
        build_context_closure(&inventory, &program).expect("production closure is valid");
    let (parent, child) = first_owned_local_edge(&program, &resolved_calls);

    let mut owner_inventory = inventory.clone();
    let mut owner_function = program.functions[&child.function].clone();
    let owner_source = owner_function
        .source
        .as_mut()
        .expect("production child has source identity");
    owner_source.owner = Some("EvaluatorLookalike".to_owned());
    owner_inventory
        .nominals
        .insert("evaluator::EvaluatorLookalike".to_owned());
    let owner_error = classify_child(&owner_inventory, &owner_function, &parent)
        .expect_err("a lookalike owner cannot inherit a mechanic class");
    assert!(matches!(owner_error, AuditError::Unclassified { .. }));

    let mut carrier_inventory = inventory;
    let mut carrier_function = program.functions[&child.function].clone();
    carrier_inventory
        .nominals
        .insert("evaluator::EvaluatorLookalike".to_owned());
    *carrier_function
        .locals
        .values_mut()
        .next()
        .expect("production MIR function has a return local") =
        "evaluator::EvaluatorLookalike".to_owned();
    let carrier_error = classify_child(&carrier_inventory, &carrier_function, &parent)
        .expect_err("a lookalike carrier cannot inherit a mechanic class");
    assert!(matches!(carrier_error, AuditError::Unclassified { .. }));
}

fn cloned_production_program() -> (SourceInventory, MirProgram) {
    let input = production_input().expect("compiler-derived production input is available");
    (input.inventory.clone(), input.program.clone())
}

fn first_reachable_call(
    program: &MirProgram,
    contexts: &BTreeSet<ContextKey>,
    predicate: impl Fn(&ContextKey, &MirCall) -> bool,
) -> (ContextKey, MirCall) {
    contexts
        .iter()
        .find_map(|context| {
            program.functions[&context.function]
                .all_calls()
                .find(|call| predicate(context, call))
                .cloned()
                .map(|call| (context.clone(), call))
        })
        .expect("production closure contains the requested call shape")
}

fn replace_call_target(
    program: &mut MirProgram,
    context: &ContextKey,
    original: &MirCall,
    replacement: &str,
) {
    let call = program
        .functions
        .get_mut(&context.function)
        .expect("production function remains present")
        .blocks
        .get_mut(&original.block)
        .expect("production block remains present")
        .calls
        .iter_mut()
        .find(|call| call.ordinal == original.ordinal)
        .expect("production call remains present");
    call.target = replacement.to_owned();
    call.closure_refs.clear();
}

fn first_owned_local_edge(
    program: &MirProgram,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
) -> (ContextKey, ContextKey) {
    let (key, child) = first_owned_local_call_edge(program, resolved_calls);
    (key.0, child)
}

fn first_owned_local_call_edge(
    program: &MirProgram,
    resolved_calls: &BTreeMap<CallKey, Vec<ResolvedTarget>>,
) -> (CallKey, ContextKey) {
    resolved_calls
        .iter()
        .find_map(|(key, targets)| {
            targets.iter().find_map(|target| match target {
                ResolvedTarget::Local(child)
                    if program.functions[&child.function]
                        .source
                        .as_ref()
                        .is_some_and(|source| source.owner.is_some()) =>
                {
                    Some((key.clone(), child.clone()))
                }
                _ => None,
            })
        })
        .expect("production closure contains an exact owned local edge")
}

fn first_switch_for_handler_mutation(
    program: &MirProgram,
    contexts: &BTreeSet<ContextKey>,
) -> (ContextKey, MirSwitch) {
    contexts
        .iter()
        .filter(|context| {
            matches!(
                context.class,
                Mechanic::CoreAbi | Mechanic::KernelStep | Mechanic::ByteMachine
            )
        })
        .find_map(|context| {
            let function = &program.functions[&context.function];
            if function.blocks.values().any(|block| {
                block
                    .terminator
                    .as_ref()
                    .is_some_and(|terminator| terminator.raw == "unreachable;")
            }) {
                return None;
            }
            function
                .blocks
                .values()
                .flat_map(|block| block.switches.iter())
                .find(|switch| {
                    switch.successors.len() >= 2
                        && switch.successors.iter().take(2).all(|successor| {
                            program.functions[&context.function]
                                .blocks
                                .contains_key(successor)
                        })
                        && !local_references(&switch.operand).is_empty()
                })
                .cloned()
                .map(|switch| (context.clone(), switch))
        })
        .expect("production closure contains a two-arm typed switch")
}

fn append_actual_call_to_block(
    program: &mut MirProgram,
    function: &str,
    block: &str,
    target: String,
) -> usize {
    let template = program
        .functions
        .values()
        .flat_map(MirFunction::all_calls)
        .next()
        .expect("production MIR contains a call")
        .clone();
    let block = program
        .functions
        .get_mut(function)
        .expect("handler mutation function remains present")
        .blocks
        .get_mut(block)
        .expect("handler mutation block remains present");
    let ordinal = block
        .assignments
        .iter()
        .map(|assignment| assignment.ordinal)
        .chain(block.calls.iter().map(|call| call.ordinal))
        .chain(block.switches.iter().map(|switch| switch.ordinal))
        .chain(block.terminator.iter().map(|terminator| terminator.ordinal))
        .max()
        .unwrap_or(0)
        + 100;
    let mut call = template;
    call.block = block.id.clone();
    call.ordinal = ordinal;
    call.result = "_0".to_owned();
    call.target = target;
    call.arguments.clear();
    call.closure_refs.clear();
    call.successors.clear();
    call.raw = "mutated production handler call".to_owned();
    block.calls.push(call);
    ordinal
}
