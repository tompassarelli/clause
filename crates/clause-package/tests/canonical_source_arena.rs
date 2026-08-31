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

    let plan = plan_independent_canonical_source_allocations_v1(&cst)
        .expect("the declaration slice has an explicit independent allocation plan");
    assert_eq!(plan.artifact(), cst.artifact());
    assert!(plan.allocations().iter().all(|allocation| matches!(
        allocation.judgment,
        CanonicalAllocationJudgmentV1::Fresh { .. }
    )));
    assert_eq!(
        plan.allocations()
            .iter()
            .map(|allocation| allocation.identity)
            .collect::<BTreeSet<_>>()
            .len(),
        plan.allocations().len(),
        "every nominal product receives one distinct typed local identity"
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

    let constitution = compiled.checked_package.constitution().preimage();
    assert_eq!(constitution.formations.len(), 21);
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
    assert_eq!(compiled.emissions.len(), 62);
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
    assert_eq!(unsupported_counts, [3, 3, 14, 5]);
    let include_emissions = compiled
        .unsupported
        .iter()
        .flat_map(|unsupported| &unsupported.emissions)
        .collect::<Vec<_>>();
    assert_eq!(include_emissions.len(), 10);
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
            .map(|emission| &emission.slot)
            .collect::<BTreeSet<_>>()
            .len(),
        include_emissions.len(),
        "independent include emissions retain distinct stable semantic slots"
    );

    let carrier = ProcessCarrier::replay(&compiled.checked_package, &AuthorityStore::new())
        .expect("the existing package carrier consumes the checked declaration package");
    assert_eq!(carrier.application_count(), 0);
}
