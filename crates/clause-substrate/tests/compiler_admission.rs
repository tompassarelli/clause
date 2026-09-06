use clause_package as process;
use clause_substrate::compiler_admission::{
    self, CompilerAdmissionError, CompilerProgramCandidate,
};
use clause_substrate::compiler_package_v3::*;

const COMPILER0: &[u8] = include_bytes!("fixtures/compiler-host-freeze/compiler0.clcp");

fn process_package(exact_compiler_bytes: &[u8]) -> process::CheckedProcessPackage {
    let scope = process::TermScope {
        universe: process::UniverseId::from_bytes([1; 32]),
        semantics: process::ClauseSemanticsId::from_bytes([2; 32]),
    };
    let atom = |kind: &[u8], payload: &[u8]| {
        process::Term::atom(
            scope,
            kind.to_vec(),
            payload.to_vec(),
            process::EqualityContract::ExactOctetsV1,
        )
        .unwrap()
    };
    let snapshot = process::ProgramSnapshotPreimageV2 {
        constitution: process::ProgramConstitutionPreimageV2 {
            universe: scope.universe,
            semantics: scope.semantics,
            formations: vec![process::FormationJudgmentPreimageV2 {
                id: process::FormationLocalId::new(1),
                context: vec![],
                term: atom(b"test/compiler-bytes", exact_compiler_bytes),
                target: process::FormationTargetV2 {
                    type_term: atom(b"test/type", b"compiler-package"),
                    interpretation: atom(b"test/interpretation", b"exact bytes"),
                },
                direct_dependencies: vec![],
            }],
            schemas: vec![],
            capabilities: vec![],
            operators: vec![],
            applications: vec![],
        },
        successor_grants: vec![],
        static_execution_grants: vec![],
        state_admission_grants: vec![],
        judgment_authority_grants: vec![],
    };
    let package = process::ProcessPackageV2 {
        claimed_snapshot: process::derive_program_snapshot_id(&snapshot).unwrap(),
        snapshot,
        initial_state_views: vec![],
        records: vec![],
    };
    process::check_process_package(
        process::decode_process_package(&process::encode_process_package(&package).unwrap())
            .unwrap(),
    )
    .unwrap()
}

fn request<'a>(
    package: &'a CompilerPackage,
    anchor: OwnerAnchorInput<'static>,
) -> GenesisAuthorizationRequest<'a> {
    GenesisAuthorizationRequest {
        owner_anchor: anchor,
        build_request: &package.subject.build_request,
        evidence: &package.evidence,
        compile_fuel_limit: 1_000_000_000,
        admission_fuel_limit: 1_000_000_000,
        final_identity: FinalPackageIdentityInput {
            package_hash: compiler_package_hash(COMPILER0),
            exact_package_bytes: COMPILER0,
        },
    }
}

#[test]
fn checking_cannot_replace_exact_outer_genesis_admission() {
    let decoded = decode(COMPILER0).unwrap();
    let package = decoded.package();
    let checked = process_package(COMPILER0);
    let revision = process::ProgramRevisionPreimage {
        semantics: checked.constitution().semantics(),
        program: process::ProgramId::from_bytes([3; 32]),
        predecessor: None,
        snapshot: checked.constitution().snapshot(),
        change: process::ProgramChangeOccurrenceId::from_bytes([4; 32]),
    }
    .derived_claim();
    let candidate = || CompilerProgramCandidate {
        exact_compiler_bytes: COMPILER0,
        checked_process: &checked,
        compiler_formation: process::FormationLocalId::new(1),
        revision,
    };
    let policy = process::RootPolicyId::from_bytes([5; 32]);
    let authorization = process::RootAdmissionAuthorizationRef {
        policy,
        local: process::AdmissionAuthorizationLocalId::new(1),
    };
    // This is the test's explicit external selection, not Tom's genesis act.
    let anchor = OwnerAnchorInput::Supplied(OwnerAnchorWitness::from_external_selection(
        OwnerAnchorObservation {
            exact_selected_bytes: COMPILER0,
            selected_byte_length: COMPILER0.len() as u64,
            selected_package_hash: compiler_package_hash(COMPILER0),
        },
    ));
    let mut authority = process::AuthorityStore::new();
    assert!(matches!(
        compiler_admission::admit_genesis(
            &mut authority,
            candidate(),
            request(package, anchor),
            policy,
            authorization
        ),
        Err(CompilerAdmissionError::Authority(
            process::AuthorityError::UnknownRootPolicy(_)
        ))
    ));
    assert!(authority.revision(revision.id).is_none());
    authority
        .establish_root_policy(
            process::RootPolicyAnchor::establish(
                policy,
                vec![process::RootGenesisGrant {
                    authorization,
                    scope: process::RootGenesisScope {
                        semantics: revision.preimage.semantics,
                        program: revision.preimage.program,
                        snapshot: revision.preimage.snapshot,
                        change: revision.preimage.change,
                    },
                }],
                vec![],
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(
        compiler_admission::admit_genesis(
            &mut authority,
            candidate(),
            request(package, OwnerAnchorInput::Missing),
            policy,
            authorization
        ),
        Err(CompilerAdmissionError::Rejected(AuthorizationFailure {
            code: AuthorizationCode::MissingAnchor,
            ..
        }))
    ));
    assert!(authority.revision(revision.id).is_none());
    let wrong = process_package(b"different compiler");
    assert!(matches!(
        compiler_admission::admit_genesis(
            &mut authority,
            CompilerProgramCandidate {
                exact_compiler_bytes: COMPILER0,
                checked_process: &wrong,
                compiler_formation: process::FormationLocalId::new(1),
                revision: process::ProgramRevisionPreimage {
                    snapshot: wrong.constitution().snapshot(),
                    ..revision.preimage
                }
                .derived_claim(),
            },
            request(package, anchor),
            policy,
            authorization
        ),
        Err(CompilerAdmissionError::FormationBytesMismatch)
    ));
    let admitted = compiler_admission::admit_genesis(
        &mut authority,
        candidate(),
        request(package, anchor),
        policy,
        authorization,
    )
    .unwrap();
    assert_eq!(admitted.exact_bytes(), COMPILER0);
    assert_eq!(admitted.revision(), revision.id);
    assert!(authority.revision(revision.id).is_some());
}
