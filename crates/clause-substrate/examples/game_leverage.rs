use clause_substrate::game_leverage::{
    Change, IndexedMaterialization, IndexedPlan, ProgramIr, World, reference_materialize,
};
use std::time::Instant;

const SOURCE: &str = include_str!("spatial_visibility.clause");

fn world(program: &ProgramIr, noise: usize) -> World {
    let mut world = World::default();
    for index in 0..64 {
        let cell = format!("cell-{}", index % 16);
        world
            .admit(
                program
                    .fact(
                        "spatial/viewer-cell",
                        &[("observer", &format!("viewer-{index}")), ("cell", &cell)],
                    )
                    .expect("generated viewer fact has the declared roles"),
            )
            .expect("generated viewer fact is unique");
        world
            .admit(
                program
                    .fact(
                        "spatial/target-cell",
                        &[("target", &format!("target-{index}")), ("cell", &cell)],
                    )
                    .expect("generated target fact has the declared roles"),
            )
            .expect("generated target fact is unique");
    }
    for index in 0..noise {
        world
            .admit(
                program
                    .fact(
                        "diagnostic/noise",
                        &[
                            ("subject", &format!("noise-{index}")),
                            ("value", "unrelated"),
                        ],
                    )
                    .expect("generated noise fact has the declared roles"),
            )
            .expect("generated noise fact is unique");
    }
    world
}

fn main() {
    let program =
        ProgramIr::parse(SOURCE).expect("historical fixture parses to one experimental IR");
    let plan = IndexedPlan::compile(&program, program.law()).expect("law has an indexed plan");
    println!("plan_trace={:?}", plan.trace());
    let mut world = world(&program, 16_384);

    let reference_start = Instant::now();
    let (reference, scan_work) = reference_materialize(program.law(), &world);
    let reference_time = reference_start.elapsed();

    let indexed_start = Instant::now();
    let mut indexed = IndexedMaterialization::build(plan, &world).expect("indexes build");
    let indexed_build_time = indexed_start.elapsed();
    let indexed_build_work = indexed.work();
    assert_eq!(&reference, indexed.view());

    let old = program
        .fact(
            "spatial/viewer-cell",
            &[("observer", "viewer-0"), ("cell", "cell-0")],
        )
        .expect("old position fact");
    let new = program
        .fact(
            "spatial/viewer-cell",
            &[("observer", "viewer-0"), ("cell", "cell-1")],
        )
        .expect("new position fact");
    let changes = [Change::Withdraw(old), Change::Admit(new)];
    world.apply(&changes).expect("world update applies");
    indexed.reset_update_work();
    let update_start = Instant::now();
    indexed.apply(&changes).expect("indexed update applies");
    let indexed_update_time = update_start.elapsed();
    let reference_update_start = Instant::now();
    let (updated_reference, update_scan_work) = reference_materialize(program.law(), &world);
    let reference_update_time = reference_update_start.elapsed();
    assert_eq!(&updated_reference, indexed.view());

    println!("visible_fact_set_size={}", reference.len());
    println!(
        "reference_initial={reference_time:?} fact_checks={}",
        scan_work.fact_checks
    );
    println!(
        "indexed_build={indexed_build_time:?} fact_checks={}",
        indexed_build_work.build_fact_checks
    );
    println!(
        "reference_update={reference_update_time:?} fact_checks={}",
        update_scan_work.fact_checks
    );
    let indexed_work = indexed.work();
    println!(
        "indexed_update={indexed_update_time:?} counterpart_bucket_probes={} pair_visits={}",
        indexed_work.counterpart_bucket_probes, indexed_work.pair_visits
    );
}
