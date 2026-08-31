use std::collections::BTreeSet;

use clause_package::*;

const WORLD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-vectors/jump-arena/world.clause"
));

fn raw_id(tag: u8) -> [u8; IDENTITY_BYTES] {
    let mut bytes = [0; IDENTITY_BYTES];
    bytes[0] = tag;
    bytes[IDENTITY_BYTES - 1] = tag;
    bytes
}

#[test]
fn canonical_world_declarations_reach_the_checked_package_with_exact_remainder() {
    let cst = read_canonical_source_v1(WORLD).expect("canonical world source reads losslessly");
    assert_eq!(cst.exact_source(), WORLD);

    let root = ProgramChangeOccurrenceId::from_bytes(raw_id(3));
    let plan = plan_independent_canonical_source_allocations_v1(&cst, root)
        .expect("the declaration slice has an explicit independent allocation plan");
    assert_eq!(plan.artifact(), cst.artifact());
    assert_eq!(plan.root(), root);
    assert!(plan.allocations().iter().all(|allocation| {
        let nonzero = match allocation.identity {
            CanonicalAllocatedIdentityV1::Formation(id) => id.get() != 0,
            CanonicalAllocatedIdentityV1::RelationSchema(id) => id.get() != 0,
            CanonicalAllocatedIdentityV1::Role(id) => id.role.get() != 0,
            CanonicalAllocatedIdentityV1::Operator(id) => id.get() != 0,
            CanonicalAllocatedIdentityV1::Mode(id) => id.mode.get() != 0,
        };
        nonzero
            && matches!(
                &allocation.judgment,
                CanonicalAllocationJudgmentV1::Fresh {
                    basis: CanonicalFreshBasisV1::ConstitutedProgramChange(actual_root),
                    producer,
                    slot: CanonicalAllocationSlotV1::Emission(slot),
                    collision:
                        CanonicalAllocationCollisionDispositionV1::RejectTypedCollision,
                    cycle: CanonicalAllocationCycleDispositionV1::RejectDependencyCycle,
                } if *actual_root == root
                    && !producer.semantic_key.is_empty()
                    && !slot.local.is_empty()
            )
    }));
    assert_eq!(
        plan.allocations()
            .iter()
            .map(|allocation| allocation.identity)
            .collect::<BTreeSet<_>>()
            .len(),
        plan.allocations().len(),
        "every nominal product receives one distinct typed local identity"
    );
    let rematerialized = rematerialize_canonical_source_allocation_plan_v1(&cst, &plan)
        .expect("the recorded plan rematerializes without allocating again");
    assert_eq!(rematerialized, plan);

    let other_root = ProgramChangeOccurrenceId::from_bytes(raw_id(4));
    let other_plan = plan_independent_canonical_source_allocations_v1(&cst, other_root)
        .expect("a distinct constituted change root has an independent plan");
    assert_ne!(other_plan, plan);
    assert_ne!(
        other_plan
            .allocations()
            .iter()
            .map(|allocation| allocation.identity)
            .collect::<Vec<_>>(),
        plan.allocations()
            .iter()
            .map(|allocation| allocation.identity)
            .collect::<Vec<_>>(),
        "equal source under a distinct constituted root has distinct nominal coordinates"
    );
    assert!(
        other_plan
            .allocations()
            .iter()
            .zip(plan.allocations())
            .all(|(other, original)| other.identity != original.identity),
        "every corresponding fixture allocation changes under a distinct root"
    );

    let reordered_source = std::str::from_utf8(WORLD)
        .expect("fixture is UTF-8")
        .replacen(
            "referent F64\nreferent Bool",
            "referent Bool\nreferent F64",
            1,
        );
    let reordered = read_canonical_source_v1(reordered_source.as_bytes())
        .expect("reordered unrelated declarations remain canonical source");
    assert_eq!(
        rematerialize_canonical_source_allocation_plan_v1(&reordered, &plan),
        Err(CanonicalSourceErrorV1::AllocationArtifactMismatch),
        "source reorder cannot masquerade as rematerialization or retention"
    );
    let reordered_plan = plan_independent_canonical_source_allocations_v1(
        &reordered,
        ProgramChangeOccurrenceId::from_bytes(raw_id(5)),
    )
    .expect("reordered source requires a fresh constituted plan");
    assert!(
        reordered_plan
            .allocations()
            .iter()
            .all(|allocation| matches!(
                allocation.judgment,
                CanonicalAllocationJudgmentV1::Fresh { .. }
            ))
    );

    let compiled = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &plan,
    )
    .expect("the supported declaration slice encodes, decodes, and checks");
    let compiled_again = elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes(raw_id(1)),
            semantics: ClauseSemanticsId::from_bytes(raw_id(2)),
        },
        &rematerialized,
    )
    .expect("the exact recorded plan rematerializes the same checked package");
    assert_eq!(
        compiled_again.checked_package.exact_bytes(),
        compiled.checked_package.exact_bytes()
    );
    assert_eq!(compiled_again.emissions, compiled.emissions);

    let constitution = compiled.checked_package.constitution().preimage();
    assert_eq!(constitution.formations.len(), 24);
    assert_eq!(constitution.schemas.len(), 13);
    assert_eq!(constitution.operators.len(), 13);
    assert!(constitution.applications.is_empty());
    assert_eq!(
        constitution
            .schemas
            .iter()
            .map(|schema| schema.roles.len())
            .sum::<usize>(),
        28
    );
    assert_eq!(
        constitution
            .schemas
            .iter()
            .filter(|schema| schema.roles.len() == 4)
            .count(),
        1,
        "the four-role clamped-between declaration remains structurally distinct"
    );
    assert_eq!(compiled.emissions.len(), 65);
    assert!(compiled.emissions.iter().all(|emission| {
        cst.source_slice(emission.origin)
            .is_some_and(|source| !source.is_empty())
    }));

    let unsupported_counts =
        compiled
            .unsupported
            .iter()
            .fold([0_usize; 4], |mut counts, unsupported| {
                let index = match unsupported.production {
                    CanonicalSourceProductionV1::Law => 0,
                    CanonicalSourceProductionV1::Derive => 1,
                    CanonicalSourceProductionV1::Assertion => 2,
                    CanonicalSourceProductionV1::Handler => 3,
                    other => panic!("unexpected unsupported production: {other:?}"),
                };
                counts[index] += 1;
                counts
            });
    assert_eq!(unsupported_counts, [3, 3, 13, 4]);
    let include_emissions = compiled
        .unsupported
        .iter()
        .flat_map(|unsupported| &unsupported.emissions)
        .collect::<Vec<_>>();
    assert_eq!(include_emissions.len(), 9);
    assert!(include_emissions.iter().all(|emission| {
        emission.slot.production == CanonicalSourceProductionV1::HandlerInclude
            && emission.allocations.is_empty()
            && cst
                .source_slice(emission.origin)
                .is_some_and(|source| source.starts_with(b"    "))
    }));
    assert_eq!(
        include_emissions
            .iter()
            .map(|emission| (&emission.producer, &emission.slot))
            .collect::<BTreeSet<_>>()
            .len(),
        include_emissions.len(),
        "independent include emissions retain distinct stable semantic slots"
    );

    let input = compiled
        .input_handler
        .expect("the bounded source profile lowers the actual on-input handler");
    assert_eq!(input.artifact, cst.artifact());
    assert_eq!(input.initial_x, 0.0_f64.to_bits());
    assert_eq!(input.initial_z, 0.0_f64.to_bits());
    assert_eq!(input.result_x, CanonicalInputScalarV1::Parameter(0));
    assert_eq!(input.result_z, CanonicalInputScalarV1::Parameter(1));
    assert!(cst.source_slice(input.handler_origin).is_some());
    assert!(cst.source_slice(input.initial_assertion_origin).is_some());

    let carrier = ProcessCarrier::replay(&compiled.checked_package, &AuthorityStore::new())
        .expect("the existing package carrier consumes the checked declaration package");
    assert_eq!(carrier.application_count(), 0);
}
