use sha2::{Digest, Sha256};

use crate::identity::{
    ActivationId, AdmissionOccurrenceId, ApplicationShapeId, ClauseSemanticsId, IDENTITY_BYTES,
    ProcessPackageId, ProgramChangeOccurrenceId, ProgramId, ProgramRevisionId, ProgramSnapshotId,
    RunId, RuntimePolicyId, RuntimeSessionId, SessionStartOccurrenceId, StateRevisionId, StepId,
};

const PROGRAM_SNAPSHOT_DOMAIN: &str = "clause/program-snapshot/v1";
const APPLICATION_SHAPE_DOMAIN: &str = "clause/application-shape/v1";
const PROGRAM_REVISION_DOMAIN: &str = "clause/program-revision/v1";
const STATE_REVISION_DOMAIN: &str = "clause/state-revision/v1";
const PROCESS_PACKAGE_DOMAIN: &str = "clause/process-package/v1";
const ROOT_LINEAGE_TAG: [u8; 1] = [0];
const PREDECESSOR_LINEAGE_TAG: u8 = 1;
const SESSION_START_CAUSE_TAG: u8 = 0;
const ADMISSION_CAUSE_TAG: u8 = 1;

/// Constitutional State-revision cause fields before identity derivation.
///
/// This is a hash input, not evidence that the referenced occurrence or Step
/// exists or is authoritative. The authority store must establish that first.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StateRevisionCausePreimage {
    SessionStart(SessionStartOccurrenceId),
    Admission {
        occurrence: AdmissionOccurrenceId,
        run: RunId,
        activation: ActivationId,
        step: StepId,
    },
}

fn encode_predecessor_or_root(predecessor: Option<&[u8; IDENTITY_BYTES]>) -> Vec<u8> {
    match predecessor {
        None => ROOT_LINEAGE_TAG.to_vec(),
        Some(predecessor) => {
            let mut encoded = Vec::with_capacity(1 + IDENTITY_BYTES);
            encoded.push(PREDECESSOR_LINEAGE_TAG);
            encoded.extend_from_slice(predecessor);
            encoded
        }
    }
}

fn encode_state_revision_cause(cause: StateRevisionCausePreimage) -> Vec<u8> {
    match cause {
        StateRevisionCausePreimage::SessionStart(start) => {
            let mut encoded = Vec::with_capacity(1 + IDENTITY_BYTES);
            encoded.push(SESSION_START_CAUSE_TAG);
            encoded.extend_from_slice(start.as_bytes());
            encoded
        }
        StateRevisionCausePreimage::Admission {
            occurrence,
            run,
            activation,
            step,
        } => {
            let mut encoded = Vec::with_capacity(1 + (4 * IDENTITY_BYTES));
            encoded.push(ADMISSION_CAUSE_TAG);
            encoded.extend_from_slice(occurrence.as_bytes());
            encoded.extend_from_slice(run.as_bytes());
            encoded.extend_from_slice(activation.as_bytes());
            encoded.extend_from_slice(step.as_bytes());
            encoded
        }
    }
}

/// Domain-separated SHA-256 used by Clause identity derivations.
///
/// The exact preimage is
/// `U32(len(domain)) || ASCII(domain) || each(U64(len(component)) || component)`.
/// Hash agreement establishes identity only; it grants no authority.
#[must_use]
pub(crate) fn domain_hash(domain: &str, components: &[&[u8]]) -> [u8; IDENTITY_BYTES] {
    assert!(domain.is_ascii(), "Clause hash domains are ASCII");

    let mut hasher = Sha256::new();
    let domain_bytes = domain.as_bytes();
    let domain_length =
        u32::try_from(domain_bytes.len()).expect("Clause hash domains have bounded length");
    hasher.update(domain_length.to_be_bytes());
    hasher.update(domain_bytes);
    for component in components {
        let component_length =
            u64::try_from(component.len()).expect("a Rust slice length fits U64");
        hasher.update(component_length.to_be_bytes());
        hasher.update(component);
    }
    hasher.finalize().into()
}

#[must_use]
pub(crate) fn derive_program_snapshot_id(
    semantics: ClauseSemanticsId,
    canonical_snapshot_preimage: &[u8],
) -> ProgramSnapshotId {
    ProgramSnapshotId::from_bytes(domain_hash(
        PROGRAM_SNAPSHOT_DOMAIN,
        &[semantics.as_bytes(), canonical_snapshot_preimage],
    ))
}

/// Derives an Application-shape identity in domain
/// `clause/application-shape/v1`.
///
/// The supplied canonical form must contain the exact resolved schema,
/// operator, eligible modes, named-role bindings, context requirements, and
/// dependency closure, and must omit the shape identity being derived.
#[must_use]
pub(crate) fn derive_application_shape_id(
    semantics: ClauseSemanticsId,
    snapshot: ProgramSnapshotId,
    canonical_resolved_form_without_shape_id: &[u8],
) -> ApplicationShapeId {
    ApplicationShapeId::from_bytes(domain_hash(
        APPLICATION_SHAPE_DOMAIN,
        &[
            semantics.as_bytes(),
            snapshot.as_bytes(),
            canonical_resolved_form_without_shape_id,
        ],
    ))
}

/// Derives a Program-revision identity in domain
/// `clause/program-revision/v1`.
///
/// `None` is the root lineage marker; `Some` binds one exact predecessor.
/// The lineage tag and optional predecessor are one framed hash component.
/// Admission evidence is deliberately excluded because identity does not grant
/// or prove admission authority.
#[must_use]
pub(crate) fn derive_program_revision_id(
    semantics: ClauseSemanticsId,
    program: ProgramId,
    predecessor: Option<ProgramRevisionId>,
    snapshot: ProgramSnapshotId,
    change: ProgramChangeOccurrenceId,
) -> ProgramRevisionId {
    let predecessor = encode_predecessor_or_root(predecessor.as_ref().map(|id| id.as_bytes()));
    ProgramRevisionId::from_bytes(domain_hash(
        PROGRAM_REVISION_DOMAIN,
        &[
            semantics.as_bytes(),
            program.as_bytes(),
            &predecessor,
            snapshot.as_bytes(),
            change.as_bytes(),
        ],
    ))
}

/// Derives a State-revision identity in domain `clause/state-revision/v1`.
///
/// The preimage binds the exact semantics, session, root/predecessor lineage,
/// typed causal occurrence, canonical StateSnapshot payload, and runtime
/// policy. Lineage and cause are each one framed tagged-sum component.
/// Materialization identities and admission evidence are excluded.
#[must_use]
pub(crate) fn derive_state_revision_id(
    semantics: ClauseSemanticsId,
    session: RuntimeSessionId,
    predecessor: Option<StateRevisionId>,
    cause: StateRevisionCausePreimage,
    canonical_state_snapshot: &[u8],
    policy: RuntimePolicyId,
) -> StateRevisionId {
    let predecessor = encode_predecessor_or_root(predecessor.as_ref().map(|id| id.as_bytes()));
    let cause = encode_state_revision_cause(cause);
    StateRevisionId::from_bytes(domain_hash(
        STATE_REVISION_DOMAIN,
        &[
            semantics.as_bytes(),
            session.as_bytes(),
            &predecessor,
            &cause,
            canonical_state_snapshot,
            policy.as_bytes(),
        ],
    ))
}

/// Derives the exact checked package-byte binding in domain
/// `clause/process-package/v1`. This identity authenticates no authority.
#[must_use]
pub(crate) fn derive_process_package_id(
    semantics: ClauseSemanticsId,
    exact_canonical_package_bytes: &[u8],
) -> ProcessPackageId {
    ProcessPackageId::from_bytes(domain_hash(
        PROCESS_PACKAGE_DOMAIN,
        &[semantics.as_bytes(), exact_canonical_package_bytes],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_hash_matches_independently_computed_framing_vector() {
        assert_eq!(
            domain_hash("clause/hash-test/v1", &[b"abc", b"", b"\x00\xff"]),
            [
                0x09, 0x0c, 0x92, 0x5f, 0x08, 0x6b, 0x7c, 0x97, 0x23, 0xd8, 0x55, 0x29, 0x9e, 0x6c,
                0x91, 0x19, 0xd3, 0x98, 0xec, 0x1f, 0x5c, 0xae, 0x8c, 0xf0, 0x3c, 0x82, 0x49, 0xef,
                0x4f, 0xbe, 0x86, 0x67,
            ]
        );
    }

    #[test]
    fn domain_and_component_frames_are_injective_for_nearby_inputs() {
        let baseline = domain_hash("clause/hash-test/v1", &[b"ab", b"c"]);
        assert_ne!(baseline, domain_hash("clause/hash-test/v2", &[b"ab", b"c"]));
        assert_ne!(baseline, domain_hash("clause/hash-test/v1", &[b"a", b"bc"]));
        assert_ne!(
            domain_hash("clause/hash-test/v1", &[b"", b"x"]),
            domain_hash("clause/hash-test/v1", &[b"x", b""])
        );
    }

    #[test]
    fn revision_hashes_frame_tagged_sums_as_single_components() {
        let semantics = ClauseSemanticsId::from_bytes([0x01; IDENTITY_BYTES]);
        let program = ProgramId::from_bytes([0x03; IDENTITY_BYTES]);
        let snapshot = ProgramSnapshotId::from_bytes([0x04; IDENTITY_BYTES]);
        let change = ProgramChangeOccurrenceId::from_bytes([0x05; IDENTITY_BYTES]);

        assert_eq!(
            derive_program_revision_id(semantics, program, None, snapshot, change).as_bytes(),
            &[
                0x72, 0x6f, 0x27, 0xf0, 0xfd, 0x69, 0x6a, 0x44, 0xb2, 0xa1, 0xce, 0x69, 0x8b, 0x56,
                0xd0, 0x0d, 0x95, 0x1d, 0x9b, 0x79, 0x46, 0xc5, 0x6e, 0xc0, 0x36, 0x15, 0x45, 0xaf,
                0xa4, 0xd2, 0xc0, 0x0a,
            ]
        );
        assert_eq!(
            derive_program_revision_id(
                semantics,
                program,
                Some(ProgramRevisionId::from_bytes([0x06; IDENTITY_BYTES])),
                snapshot,
                change,
            )
            .as_bytes(),
            &[
                0xde, 0xe2, 0x79, 0x60, 0x07, 0x7f, 0x74, 0x90, 0x52, 0x5e, 0x4f, 0xf9, 0x1d, 0xb1,
                0xf6, 0x96, 0x78, 0xb2, 0xac, 0xc6, 0x5f, 0xc5, 0x8a, 0x7f, 0x32, 0x8c, 0x32, 0x3b,
                0x4d, 0x24, 0xa2, 0x07,
            ]
        );

        let session = RuntimeSessionId::from_bytes([0x02; IDENTITY_BYTES]);
        let policy = RuntimePolicyId::from_bytes([0x04; IDENTITY_BYTES]);
        assert_eq!(
            derive_state_revision_id(
                semantics,
                session,
                None,
                StateRevisionCausePreimage::SessionStart(SessionStartOccurrenceId::from_bytes(
                    [0x03; IDENTITY_BYTES]
                ),),
                b"state",
                policy,
            )
            .as_bytes(),
            &[
                0xf9, 0xd8, 0x49, 0xcb, 0x31, 0x78, 0x99, 0x04, 0xe9, 0xbb, 0xe4, 0x22, 0x38, 0xec,
                0xe3, 0x7d, 0x0b, 0xdd, 0x5d, 0x90, 0xaf, 0x8e, 0xd9, 0x5f, 0xff, 0xa6, 0x86, 0xc5,
                0x07, 0xc0, 0xa2, 0xf0,
            ]
        );
        assert_eq!(
            derive_state_revision_id(
                semantics,
                session,
                Some(StateRevisionId::from_bytes([0x05; IDENTITY_BYTES])),
                StateRevisionCausePreimage::Admission {
                    occurrence: AdmissionOccurrenceId::from_bytes([0x06; IDENTITY_BYTES]),
                    run: RunId::from_bytes([0x07; IDENTITY_BYTES]),
                    activation: ActivationId::from_bytes([0x08; IDENTITY_BYTES]),
                    step: StepId::from_bytes([0x09; IDENTITY_BYTES]),
                },
                b"state",
                policy,
            )
            .as_bytes(),
            &[
                0xfa, 0x27, 0x69, 0x1e, 0x4d, 0x27, 0xf5, 0x12, 0x95, 0xd0, 0x7d, 0xa1, 0x68, 0xef,
                0xc1, 0xa7, 0x5e, 0x87, 0xce, 0x60, 0x32, 0x6a, 0xbc, 0xc7, 0x79, 0x7c, 0x0f, 0x90,
                0x00, 0xc7, 0xf2, 0xc7,
            ]
        );
    }

    #[test]
    fn typed_derivations_separate_domains_and_bind_every_field() {
        let semantics = ClauseSemanticsId::from_bytes([0x01; IDENTITY_BYTES]);
        let other_semantics = ClauseSemanticsId::from_bytes([0x02; IDENTITY_BYTES]);
        let snapshot = derive_program_snapshot_id(semantics, b"snapshot");
        let other_snapshot = derive_program_snapshot_id(semantics, b"other snapshot");

        assert_ne!(
            snapshot,
            derive_program_snapshot_id(other_semantics, b"snapshot")
        );
        assert_ne!(snapshot, other_snapshot);
        let shape = derive_application_shape_id(semantics, snapshot, b"form");
        assert_ne!(
            shape,
            derive_application_shape_id(other_semantics, snapshot, b"form")
        );
        assert_ne!(
            shape,
            derive_application_shape_id(semantics, other_snapshot, b"form")
        );
        assert_ne!(
            shape,
            derive_application_shape_id(semantics, snapshot, b"other form")
        );

        let program = ProgramId::from_bytes([0x03; IDENTITY_BYTES]);
        let other_program = ProgramId::from_bytes([0x13; IDENTITY_BYTES]);
        let change = ProgramChangeOccurrenceId::from_bytes([0x04; IDENTITY_BYTES]);
        let other_change = ProgramChangeOccurrenceId::from_bytes([0x14; IDENTITY_BYTES]);
        let revision = derive_program_revision_id(semantics, program, None, snapshot, change);
        assert_ne!(
            revision,
            derive_program_revision_id(other_semantics, program, None, snapshot, change)
        );
        assert_ne!(
            revision,
            derive_program_revision_id(semantics, other_program, None, snapshot, change)
        );
        assert_ne!(
            revision,
            derive_program_revision_id(
                semantics,
                program,
                Some(ProgramRevisionId::from_bytes([0x23; IDENTITY_BYTES])),
                snapshot,
                change,
            )
        );
        assert_ne!(
            revision,
            derive_program_revision_id(semantics, program, None, other_snapshot, change)
        );
        assert_ne!(
            revision,
            derive_program_revision_id(semantics, program, None, snapshot, other_change)
        );

        let session = RuntimeSessionId::from_bytes([0x05; IDENTITY_BYTES]);
        let other_session = RuntimeSessionId::from_bytes([0x15; IDENTITY_BYTES]);
        let policy = RuntimePolicyId::from_bytes([0x06; IDENTITY_BYTES]);
        let other_policy = RuntimePolicyId::from_bytes([0x16; IDENTITY_BYTES]);
        let start = SessionStartOccurrenceId::from_bytes([0x07; IDENTITY_BYTES]);
        let other_start = SessionStartOccurrenceId::from_bytes([0x17; IDENTITY_BYTES]);
        let state = derive_state_revision_id(
            semantics,
            session,
            None,
            StateRevisionCausePreimage::SessionStart(start),
            b"state",
            policy,
        );
        assert_ne!(
            state,
            derive_state_revision_id(
                other_semantics,
                session,
                None,
                StateRevisionCausePreimage::SessionStart(start),
                b"state",
                policy,
            )
        );
        assert_ne!(
            state,
            derive_state_revision_id(
                semantics,
                other_session,
                None,
                StateRevisionCausePreimage::SessionStart(start),
                b"state",
                policy,
            )
        );
        assert_ne!(
            state,
            derive_state_revision_id(
                semantics,
                session,
                Some(StateRevisionId::from_bytes([0x24; IDENTITY_BYTES])),
                StateRevisionCausePreimage::SessionStart(start),
                b"state",
                policy,
            )
        );
        assert_ne!(
            state,
            derive_state_revision_id(
                semantics,
                session,
                None,
                StateRevisionCausePreimage::SessionStart(other_start),
                b"state",
                policy,
            )
        );
        assert_ne!(
            state,
            derive_state_revision_id(
                semantics,
                session,
                None,
                StateRevisionCausePreimage::SessionStart(start),
                b"other state",
                policy,
            )
        );
        assert_ne!(
            state,
            derive_state_revision_id(
                semantics,
                session,
                None,
                StateRevisionCausePreimage::SessionStart(start),
                b"state",
                other_policy,
            )
        );

        let occurrence = AdmissionOccurrenceId::from_bytes([0x08; IDENTITY_BYTES]);
        let run = RunId::from_bytes([0x09; IDENTITY_BYTES]);
        let activation = ActivationId::from_bytes([0x0a; IDENTITY_BYTES]);
        let step = StepId::from_bytes([0x0b; IDENTITY_BYTES]);
        let admission_cause = StateRevisionCausePreimage::Admission {
            occurrence,
            run,
            activation,
            step,
        };
        assert_ne!(
            state,
            derive_state_revision_id(semantics, session, None, admission_cause, b"state", policy,)
        );
        for changed_cause in [
            StateRevisionCausePreimage::Admission {
                occurrence: AdmissionOccurrenceId::from_bytes([0x18; IDENTITY_BYTES]),
                run,
                activation,
                step,
            },
            StateRevisionCausePreimage::Admission {
                occurrence,
                run: RunId::from_bytes([0x19; IDENTITY_BYTES]),
                activation,
                step,
            },
            StateRevisionCausePreimage::Admission {
                occurrence,
                run,
                activation: ActivationId::from_bytes([0x1a; IDENTITY_BYTES]),
                step,
            },
            StateRevisionCausePreimage::Admission {
                occurrence,
                run,
                activation,
                step: StepId::from_bytes([0x1b; IDENTITY_BYTES]),
            },
        ] {
            assert_ne!(
                derive_state_revision_id(
                    semantics,
                    session,
                    None,
                    admission_cause,
                    b"state",
                    policy,
                ),
                derive_state_revision_id(semantics, session, None, changed_cause, b"state", policy,)
            );
        }

        assert_ne!(
            derive_process_package_id(semantics, b"package"),
            derive_process_package_id(other_semantics, b"package")
        );
        assert_ne!(
            derive_process_package_id(semantics, b"package"),
            derive_process_package_id(semantics, b"other package")
        );
        assert_ne!(
            derive_program_snapshot_id(semantics, b"same bytes").as_bytes(),
            derive_process_package_id(semantics, b"same bytes").as_bytes()
        );
    }
}
