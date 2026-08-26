//! Authored grounded scene content projects from exact runtime state.

use clause::{
    elaborate, frontend,
    kernel::ReferentId,
    render::{SceneProjectionSpec, project_render_plan},
    runtime::{RuntimePolicy, RuntimeSession, TransitionEvent},
};

const SOURCE: &str = r#"F32
Entity
State
Policy

Vec2
  x: F32
  y: F32

coin/state: RelationShape
  {coin: Entity} state {state: State}
  mode coin -> state: one

scene/placement: RelationShape
  {item: Entity} scene-position {point: Vec2}
  mode item -> point: one

game
  player ∈ Entity
  coin ∈ Entity
  active ∈ State
  collected ∈ State
  replay-policy ∈ Policy
  player scene-position Vec2 { x: 0.0, y: 0.0 }
  coin state active

coin scene-position Vec2 { x: 10.0, y: 0.0 } if
  coin state active

on collect
  coin state active ~>
    coin state collected
"#;

#[test]
fn authored_guarded_scene_projects_from_root_and_omits_withdrawn_coin() {
    let compiled = elaborate::compile(frontend::parse(SOURCE).expect("scene source parses"))
        .expect("scene source elaborates");
    let [journey] = compiled.runtime_journeys() else {
        panic!("one authored scene event produces one runtime journey");
    };
    let revision = journey.revision();
    let model = revision.model();
    let relation = compiled
        .designations()
        .global("scene/placement")
        .expect("scene relation resolves");
    let item_role = compiled
        .designations()
        .role(&relation, "item")
        .expect("scene item role resolves");
    let position_role = compiled
        .designations()
        .role(&relation, "point")
        .expect("scene position role resolves");
    let vec2 = compiled
        .designations()
        .global("Vec2")
        .expect("Vec2 shape resolves");
    let x = compiled
        .designations()
        .scoped(&vec2, "x")
        .expect("Vec2.x resolves");
    let y = compiled
        .designations()
        .scoped(&vec2, "y")
        .expect("Vec2.y resolves");
    let spec = SceneProjectionSpec::new(
        relation.clone(),
        item_role.clone(),
        position_role.clone(),
        vec2.clone(),
        x.clone(),
        y.clone(),
    )
    .expect("scene projection spec is exact");

    let player = compiled
        .designations()
        .scoped(model.id(), "player")
        .expect("player resolves");
    let coin = compiled
        .designations()
        .scoped(model.id(), "coin")
        .expect("coin resolves");
    let policy = compiled
        .designations()
        .scoped(model.id(), "replay-policy")
        .expect("runtime policy resolves");
    let event = compiled
        .designations()
        .scoped(model.id(), "collect")
        .expect("collect event resolves");
    let root = RuntimeSession::start(
        revision,
        RuntimePolicy::new(policy, 128, 512).expect("runtime policy is bounded"),
    )
    .expect("scene runtime starts");

    let initial = project_render_plan(revision, root.latest(), &spec)
        .expect("root scene projects from supported content");
    assert_eq!(initial.items().len(), 2);
    let initial_item = |id: &ReferentId| {
        initial
            .items()
            .iter()
            .find(|item| item.id() == id)
            .expect("expected scene item is present")
    };
    assert_eq!(initial_item(&player).position()[0].bits(), 0x0000_0000);
    assert_eq!(initial_item(&player).position()[1].bits(), 0x0000_0000);
    assert_eq!(initial_item(&coin).position()[0].bits(), 0x4120_0000);
    assert_eq!(initial_item(&coin).position()[1].bits(), 0x0000_0000);

    let committed = root
        .transition(
            revision,
            vec![TransitionEvent::new(
                ReferentId::from_digest([0xa7; 32]),
                event,
                Vec::new(),
            )],
        )
        .expect("coin collection commits");
    let collected = project_render_plan(revision, committed.latest(), &spec)
        .expect("successor scene projects from supported content");
    assert_eq!(collected.items().len(), 1);
    assert_eq!(collected.items()[0].id(), &player);
    assert!(collected.items().iter().all(|item| item.id() != &coin));

    let coin_role = compiled
        .designations()
        .role(
            &compiled
                .designations()
                .global("coin/state")
                .expect("coin state relation resolves"),
            "coin",
        )
        .expect("coin state item role resolves");
    let wrong_roles = SceneProjectionSpec::new(
        relation.clone(),
        coin_role,
        position_role.clone(),
        vec2.clone(),
        x.clone(),
        y.clone(),
    )
    .expect("syntactically distinct wrong roles construct");
    assert_eq!(
        project_render_plan(revision, committed.latest(), &wrong_roles)
            .expect_err("foreign role identity must fail closed")
            .to_string(),
        "scene projection relation roles do not match"
    );

    let scalar_shape = compiled
        .designations()
        .global("F32")
        .expect("F32 structural shape resolves");
    let wrong_shape = SceneProjectionSpec::new(
        relation.clone(),
        item_role.clone(),
        position_role.clone(),
        scalar_shape,
        x.clone(),
        y.clone(),
    )
    .expect("syntactically valid scalar position shape constructs");
    assert_eq!(
        project_render_plan(revision, committed.latest(), &wrong_shape)
            .expect_err("a scalar cannot masquerade as Vec2")
            .to_string(),
        "scene projection position shape is not a labelled product"
    );

    let wrong_fields = SceneProjectionSpec::new(
        relation.clone(),
        item_role.clone(),
        position_role.clone(),
        vec2.clone(),
        x.clone(),
        player.clone(),
    )
    .expect("syntactically distinct wrong position fields construct");
    assert_eq!(
        project_render_plan(revision, committed.latest(), &wrong_fields)
            .expect_err("foreign field identity must fail closed")
            .to_string(),
        "scene projection position shape does not match x/y fields"
    );

    let unknown = SceneProjectionSpec::new(
        ReferentId::from_digest([0xfe; 32]),
        item_role,
        position_role,
        vec2,
        x,
        y,
    )
    .expect("syntactically valid unknown relation spec constructs");
    assert!(project_render_plan(revision, committed.latest(), &unknown).is_err());
}
