//! Source-deletion-safe request-program emission.

#![allow(unexpected_cfgs)]

use std::fmt::Write;

use crate::{
    kernel::{KernelError, PatternId, ReferentId, RelationalContent, Result, RoleId, Term},
    render::{RenderItem, RenderPlan},
    request::{QueryColumn, QuerySelection, Request, ResolvedProgram, Selection},
    wire,
};

/// Emit import-free ESM containing only frozen canonical RenderPlan data and
/// an exact-StateRevision lookup.
#[cfg(not(clause_generated))]
pub fn emit_render_plan_javascript(plans: &[RenderPlan]) -> Result<String> {
    let Some(first) = plans.first() else {
        return Err(KernelError::new(
            "generated JavaScript requires at least one RenderPlan",
        ));
    };
    let mut states = std::collections::BTreeSet::new();
    for plan in plans {
        if plan.program_revision() != first.program_revision() {
            return Err(KernelError::new(
                "generated JavaScript RenderPlans must name one ProgramRevision",
            ));
        }
        if !states.insert(plan.state_revision()) {
            return Err(KernelError::new(
                "generated JavaScript RenderPlans repeat a StateRevision",
            ));
        }
    }

    let mut output = String::new();
    writeln!(
        output,
        "export const kind = \"clause-render-plan-javascript-v2\";"
    )
    .expect("writing generated JavaScript to a String cannot fail");
    writeln!(
        output,
        "export const programRevisionId = {:?};",
        first.program_revision().as_str()
    )
    .expect("writing generated JavaScript to a String cannot fail");
    for (index, plan) in plans.iter().enumerate() {
        writeln!(output, "const plan{index} = {};", frozen_plan_source(plan))
            .expect("writing generated JavaScript to a String cannot fail");
    }
    writeln!(output, "const plans = Object.freeze({{")
        .expect("writing generated JavaScript to a String cannot fail");
    for (index, plan) in plans.iter().enumerate() {
        writeln!(
            output,
            "  {:?}: plan{index},",
            plan.state_revision().as_str()
        )
        .expect("writing generated JavaScript to a String cannot fail");
    }
    writeln!(output, "}});").expect("writing generated JavaScript to a String cannot fail");
    output.push_str(
        "export function renderPlan(stateRevisionId, requestedProgramRevisionId = programRevisionId) {\n\
         \x20 if (requestedProgramRevisionId !== programRevisionId) throw new Error(\"render plan names the wrong ProgramRevision\");\n\
         \x20 const plan = plans[stateRevisionId];\n\
         \x20 if (plan === undefined) throw new Error(\"unknown exact StateRevision\");\n\
         \x20 return plan;\n\
         }\n",
    );
    Ok(output)
}

/// Emit one source-deletion-safe specialized ESM transition from an exact
/// runtime-v3 session prefix and its exact Rust RenderPlans.
#[cfg(not(clause_generated))]
pub fn emit_live_runtime_javascript(
    session: &crate::runtime::RuntimeSession,
    plans: &[RenderPlan],
) -> Result<String> {
    let [initial, collected] = session.states() else {
        return Err(KernelError::new(
            "live JavaScript currently requires exactly one runtime transition",
        ));
    };
    let [input] = session.inputs() else {
        return Err(KernelError::new(
            "live JavaScript currently requires exactly one runtime input",
        ));
    };
    let crate::runtime::RuntimeInput::Events(events) = input else {
        return Err(KernelError::new(
            "live JavaScript currently requires an authored event transition",
        ));
    };
    let [event] = events.as_slice() else {
        return Err(KernelError::new(
            "live JavaScript currently requires exactly one event occurrence",
        ));
    };
    if !event.payload().is_empty() {
        return Err(KernelError::new(
            "live JavaScript currently requires an empty event payload",
        ));
    }
    let [initial_plan, collected_plan] = plans else {
        return Err(KernelError::new(
            "live JavaScript requires exact plans for both runtime states",
        ));
    };
    for (plan, state) in [(initial_plan, initial), (collected_plan, collected)] {
        if plan.program_revision() != session.program_revision()
            || plan.state_revision() != state.identity()
        {
            return Err(KernelError::new(
                "live JavaScript RenderPlan does not match its runtime state",
            ));
        }
    }
    let Some(transition) = collected.transition_occurrence() else {
        return Err(KernelError::new(
            "live JavaScript successor lacks its transition occurrence",
        ));
    };

    let mut output = String::new();
    writeln!(output, "export const kind = \"clause-js-runtime-v3\";")
        .expect("writing generated JavaScript to a String cannot fail");
    for (name, value) in [
        ("programRevisionId", session.program_revision().as_str()),
        ("semanticsId", session.semantics().as_str()),
        ("runtimeSessionId", session.identity().as_str()),
        (
            "sessionStartOccurrenceId",
            session.start_occurrence().as_str(),
        ),
        ("initialStateRevisionId", initial.identity().as_str()),
        ("finalStateRevisionId", collected.identity().as_str()),
        ("eventOccurrenceId", event.id().as_str()),
        ("transitionOccurrenceId", transition.as_str()),
        ("eventName", event.event().as_str()),
    ] {
        writeln!(output, "export const {name} = {value:?};")
            .expect("writing generated JavaScript to a String cannot fail");
    }
    writeln!(
        output,
        "export const runtimeSessionCanonical = {:?};",
        session.canonical_bytes()
    )
    .expect("writing generated JavaScript to a String cannot fail");
    writeln!(
        output,
        "export const expectedEventPayload = Object.freeze([]);"
    )
    .expect("writing generated JavaScript to a String cannot fail");
    writeln!(
        output,
        "export const events = Object.freeze({{{:?}: eventName}});",
        event.event().as_str()
    )
    .expect("writing generated JavaScript to a String cannot fail");
    writeln!(
        output,
        "const plans = Object.freeze({{{:?}:{},{:?}:{}}});",
        initial.identity().as_str(),
        frozen_plan_source(initial_plan),
        collected.identity().as_str(),
        frozen_plan_source(collected_plan),
    )
    .expect("writing generated JavaScript to a String cannot fail");
    output.push_str(
        "export const initialState = initialStateRevisionId;\n\
         export const capabilities = Object.freeze([]);\n\
         export function renderPlan(stateRevisionId, requestedProgramRevisionId = programRevisionId) {\n\
         \x20 if (requestedProgramRevisionId !== programRevisionId) throw new Error(\"render plan names the wrong ProgramRevision\");\n\
         \x20 const plan = plans[stateRevisionId];\n\
         \x20 if (plan === undefined) throw new Error(\"unknown exact StateRevision\");\n\
         \x20 return plan;\n\
         }\n\
         const samePayload = (value) => Array.isArray(value) && value.length === 0;\n\
         const matchesEvent = (event, occurrence, revision) => event && revision === programRevisionId && event.programRevisionId === programRevisionId && event.name === eventName && event.event === eventName && event.id === eventOccurrenceId && event.order === 0 && samePayload(event.payload) && occurrence === transitionOccurrenceId;\n\
         export function createEvent(name, event, payload, occurrence, order, revision) {\n\
         \x20 if (name !== eventName || event !== eventName || occurrence !== eventOccurrenceId || order !== 0 || revision !== programRevisionId || !samePayload(payload)) throw new Error(\"event does not match the sealed runtime input\");\n\
         \x20 return Object.freeze({ id: occurrence, event, name, payload: expectedEventPayload, order, programRevisionId });\n\
         }\n\
         export function createRuntime() {\n\
         \x20 let state = initialStateRevisionId;\n\
         \x20 return Object.freeze({\n\
         \x20   state: () => state,\n\
         \x20   transition(event, occurrence, revision) {\n\
         \x20     if (state !== initialStateRevisionId || !matchesEvent(event, occurrence, revision)) throw new Error(\"transition does not match the sealed runtime-v3 edge\");\n\
         \x20     state = finalStateRevisionId;\n\
         \x20     return Object.freeze({ programRevisionId, semanticsId, runtimeSessionId, sessionStartOccurrenceId, transitionOccurrenceId, state, runtimeSessionCanonical, effects: Object.freeze([]) });\n\
         \x20   },\n\
         \x20 });\n\
         }\n\
         export function validateTransitionResult(result, event, occurrence, revision) {\n\
         \x20 return matchesEvent(event, occurrence, revision) && result?.programRevisionId === programRevisionId && result?.semanticsId === semanticsId && result?.runtimeSessionId === runtimeSessionId && result?.sessionStartOccurrenceId === sessionStartOccurrenceId && result?.transitionOccurrenceId === transitionOccurrenceId && result?.state === finalStateRevisionId && result?.runtimeSessionCanonical === runtimeSessionCanonical;\n\
         }\n\
         export function validateEffectTrace() { return false; }\n",
    );
    Ok(output)
}

#[cfg(not(clause_generated))]
fn frozen_plan_source(plan: &RenderPlan) -> String {
    frozen_array(&[
        format!("{:?}", crate::render::RENDER_PLAN_TAG),
        frozen_array(&[
            "\"program-revision\"".to_owned(),
            format!("{:?}", plan.program_revision().as_str()),
        ]),
        frozen_array(&[
            "\"state-revision\"".to_owned(),
            format!("{:?}", plan.state_revision().as_str()),
        ]),
        frozen_array(&[
            "\"items\"".to_owned(),
            frozen_array(
                &plan
                    .items()
                    .iter()
                    .map(frozen_item_source)
                    .collect::<Vec<_>>(),
            ),
        ]),
    ])
}

#[cfg(not(clause_generated))]
fn frozen_item_source(item: &RenderItem) -> String {
    frozen_array(&[
        "\"item\"".to_owned(),
        format!("{:?}", item.id().as_str()),
        frozen_array(&[
            "\"position-f32x2\"".to_owned(),
            format!("\"{:08x}\"", item.position()[0].bits()),
            format!("\"{:08x}\"", item.position()[1].bits()),
        ]),
    ])
}

#[cfg(not(clause_generated))]
fn frozen_array(values: &[String]) -> String {
    format!("Object.freeze([{}])", values.join(","))
}

#[cfg(not(clause_generated))]
struct ChildModule {
    name: &'static str,
    declaration: &'static str,
    source: &'static str,
}

#[cfg(not(clause_generated))]
const NO_CHILDREN: &[ChildModule] = &[];

#[cfg(not(clause_generated))]
const KERNEL_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "clause",
        declaration: "mod clause;",
        source: include_str!("kernel/clause.rs"),
    },
    ChildModule {
        name: "error",
        declaration: "mod error;",
        source: include_str!("kernel/error.rs"),
    },
    ChildModule {
        name: "find",
        declaration: "mod find;",
        source: include_str!("kernel/find.rs"),
    },
    ChildModule {
        name: "identity",
        declaration: "mod identity;",
        source: include_str!("kernel/identity.rs"),
    },
    ChildModule {
        name: "matching",
        declaration: "pub(crate) mod matching;",
        source: include_str!("kernel/matching.rs"),
    },
    ChildModule {
        name: "model",
        declaration: "mod model;",
        source: include_str!("kernel/model.rs"),
    },
    ChildModule {
        name: "query",
        declaration: "mod query;",
        source: include_str!("kernel/query.rs"),
    },
    ChildModule {
        name: "revision",
        declaration: "mod revision;",
        source: include_str!("kernel/revision.rs"),
    },
    ChildModule {
        name: "schema",
        declaration: "mod schema;",
        source: include_str!("kernel/schema.rs"),
    },
];

#[cfg(not(clause_generated))]
const WIRE_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "canonical",
        declaration: "mod canonical;",
        source: include_str!("wire/canonical.rs"),
    },
    ChildModule {
        name: "decode",
        declaration: "mod decode;",
        source: include_str!("wire/decode.rs"),
    },
    ChildModule {
        name: "json",
        declaration: "mod json;",
        source: include_str!("wire/json.rs"),
    },
    ChildModule {
        name: "sha256",
        declaration: "mod sha256;",
        source: include_str!("wire/sha256.rs"),
    },
];

#[cfg(not(clause_generated))]
const DERIVE_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "closure",
        declaration: "mod closure;",
        source: include_str!("derive/closure.rs"),
    },
    ChildModule {
        name: "support",
        declaration: "mod support;",
        source: include_str!("derive/support.rs"),
    },
];

#[cfg(not(clause_generated))]
const EXECUTION_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "evaluate",
        declaration: "mod evaluate;",
        source: include_str!("execution/evaluate.rs"),
    },
    ChildModule {
        name: "explain",
        declaration: "mod explain;",
        source: include_str!("execution/explain.rs"),
    },
    ChildModule {
        name: "query",
        declaration: "mod query;",
        source: include_str!("execution/query.rs"),
    },
];

#[cfg(not(clause_generated))]
const INTERVENTION_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "all",
        declaration: "mod all;",
        source: include_str!("intervention/all.rs"),
    },
    ChildModule {
        name: "basis",
        declaration: "mod basis;",
        source: include_str!("intervention/basis.rs"),
    },
    ChildModule {
        name: "closure",
        declaration: "mod closure;",
        source: include_str!("intervention/closure.rs"),
    },
    ChildModule {
        name: "one",
        declaration: "mod one;",
        source: include_str!("intervention/one.rs"),
    },
    ChildModule {
        name: "search",
        declaration: "mod search;",
        source: include_str!("intervention/search.rs"),
    },
];

#[cfg(not(clause_generated))]
const SEMANTIC_DIFF_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "entailment",
        declaration: "mod entailment;",
        source: include_str!("semantic_diff/entailment.rs"),
    },
    ChildModule {
        name: "proofs",
        declaration: "mod proofs;",
        source: include_str!("semantic_diff/proofs.rs"),
    },
    ChildModule {
        name: "supports",
        declaration: "mod supports;",
        source: include_str!("semantic_diff/supports.rs"),
    },
];

#[cfg(not(clause_generated))]
const REQUEST_CHILDREN: &[ChildModule] = &[
    ChildModule {
        name: "canonical_rendering",
        declaration: "mod canonical_rendering;",
        source: include_str!("request/canonical_rendering.rs"),
    },
    ChildModule {
        name: "ordered_execution",
        declaration: "mod ordered_execution;",
        source: include_str!("request/ordered_execution.rs"),
    },
];

/// Emit a standalone program that reloads the referenced Revisions and
/// invokes the same ordered request evaluator as the interpreter.
#[cfg(not(clause_generated))]
pub fn emit_rust(program: &ResolvedProgram, limits: crate::request::RunLimits) -> Result<String> {
    let modules = target_neutral_modules(false)?;
    let mut body = String::new();
    let revisions = revision_order(program)?;
    let indices = revisions
        .iter()
        .enumerate()
        .map(|(index, revision)| (revision.identity().clone(), index))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (index, revision) in revisions.iter().enumerate() {
        let serialized = wire::serialize(revision);
        match revision.predecessor() {
            Some(predecessor) => {
                let base = indices
                    .get(predecessor)
                    .expect("lineage order contains each predecessor");
                writeln!(
                    body,
                    "let r{index} = wire::reload_successor({serialized:?}, &r{base}).expect(\"sealed successor reloads\");"
                )
                .expect("writing generated Revision reloads to a String cannot fail");
            }
            None => {
                writeln!(
                    body,
                    "let r{index} = wire::reload({serialized:?}).expect(\"sealed root reloads\");"
                )
                .expect("writing generated Revision reloads to a String cannot fail");
            }
        }
    }
    writeln!(body, "let program = request::ResolvedProgram::new(std::collections::BTreeMap::from([{}]), vec![{}]).expect(\"generated requests resolve\");", revisions.iter().enumerate().map(|(index, _)| format!("(r{index}.identity().clone(), r{index}.clone())")).collect::<Vec<_>>().join(","), program.requests().iter().map(|request| request_source(request, &indices)).collect::<Vec<_>>().join(",")).expect("writing the generated request registry to a String cannot fail");
    writeln!(body, "let limits = {};", run_limits_source(limits))
        .expect("writing generated request limits to a String cannot fail");
    writeln!(body, "let output = request::run(&program, limits)?;")
        .expect("writing generated request execution to a String cannot fail");
    writeln!(body, "print!(\"{{}}\", output.canonical_bytes());")
        .expect("writing the generated request transcript to a String cannot fail");
    Ok(format!(
        "{modules}\nfn run() -> kernel::Result<()> {{ {body} Ok(()) }}\nfn main() -> std::process::ExitCode {{ match run() {{ Ok(()) => std::process::ExitCode::SUCCESS, Err(error) => {{ eprintln!(\"{{error}}\"); std::process::ExitCode::FAILURE }} }} }}"
    ))
}

#[cfg(not(clause_generated))]
fn limits_source(limits: crate::derive::Limits) -> String {
    format!(
        "derive::Limits::new({}, {}, {})",
        limits.max_assertions, limits.max_rounds, limits.max_join_attempts
    )
}

#[cfg(not(clause_generated))]
fn support_limits_source(limits: crate::derive::SupportLimits) -> String {
    format!(
        "derive::SupportLimits::new({}, {}, {})",
        limits_source(limits.closure),
        limits.max_expansions,
        limits.max_supports_per_clause
    )
}

#[cfg(not(clause_generated))]
fn run_limits_source(limits: crate::request::RunLimits) -> String {
    let intervention = limits.intervention;
    format!(
        "request::RunLimits {{ closure: {}, support: {}, intervention: intervention::InterventionLimits::new({}, {}, {}).with_support_limits({}) }}",
        limits_source(limits.closure),
        support_limits_source(limits.support),
        limits_source(intervention.closure()),
        intervention.max_candidates(),
        intervention.max_solutions(),
        support_limits_source(intervention.support())
    )
}

/// Emit a standalone program that strictly reloads one root Revision and
/// evaluates the requested definitions in caller-supplied order.
#[cfg(not(clause_generated))]
pub fn emit_evaluation_rust(
    revision: &crate::kernel::Revision,
    definitions: &[ReferentId],
) -> Result<String> {
    if revision.predecessor().is_some() {
        return Err(KernelError::new(
            "generated evaluation requires a root Revision",
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    for definition in definitions {
        if !unique.insert(definition) {
            return Err(KernelError::new(format!(
                "generated evaluation requested duplicate definition '{}'",
                definition.as_str()
            )));
        }
        if revision.model().definition(definition).is_none() {
            return Err(KernelError::new(format!(
                "generated evaluation requested missing definition '{}'",
                definition.as_str()
            )));
        }
    }

    let modules = target_neutral_modules(false)?;
    let serialized = wire::serialize(revision);
    let requested = definitions
        .iter()
        .map(definition_source)
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "let revision = wire::reload({serialized:?}).expect(\"sealed root reloads\");\n\
         let requested = vec![{requested}];\n\
         let mut results = Vec::with_capacity(requested.len());\n\
         for definition_id in requested {{\n\
             let definition = revision.model().definition(&definition_id).expect(\"validated definition survives strict reload\");\n\
             let result = execution::evaluate(&revision, definition.denotation()).expect(\"sealed pure definition evaluates\");\n\
             results.push((definition_id, result));\n\
         }}\n\
         let output = request::EvaluationOutput::new(revision.identity().clone(), results).expect(\"validated evaluation requests remain unique\");\n\
         print!(\"{{}}\", output.canonical_bytes());"
    );
    Ok(format!("{modules}\nfn main() {{ {body} }}"))
}

/// Emit a standalone program that strictly reloads one checked Model Revision
/// and replays caller-supplied event occurrences through the canonical fold.
#[cfg(not(clause_generated))]
pub fn emit_runtime_rust(
    journey: &crate::elaborate::RuntimeJourney,
    policy: crate::runtime::RuntimePolicy,
    start: crate::runtime::SessionStartOccurrenceId,
    steps: Vec<crate::runtime::ReplayStep>,
) -> Result<String> {
    if journey.revision().predecessor().is_some() {
        return Err(KernelError::new(
            "generated runtime requires a root Model Revision",
        ));
    }
    crate::runtime::validate_policy(journey.revision().model(), &policy)?;
    let modules = target_neutral_modules(true)?;
    let revision = wire::serialize(journey.revision());
    let steps = steps
        .iter()
        .map(|step| {
            let crate::runtime::RuntimeInput::Events(events) = &step.input else {
                return Err(KernelError::new(
                    "generated runtime currently requires event replay steps",
                ));
            };
            Ok(format!(
                "runtime::ReplayStep {{ occurrence: runtime::TransitionOccurrenceId::new({:?}.into()).expect(\"sealed transition occurrence reloads\"), input: runtime::RuntimeInput::Events(vec![{}]) }}",
                step.occurrence.as_str(),
                events.iter().map(event_source).collect::<Vec<_>>().join(",")
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(",");
    let policy = format!(
        "runtime::RuntimePolicy::new(kernel::ReferentId::new({:?}.into()).expect(\"checked policy referent\"), {}, {}).expect(\"checked runtime policy reloads\")",
        policy.referent().as_str(),
        policy.max_supports(),
        policy.max_join_attempts(),
    );
    let Some(program) = journey.program_revision() else {
        return Err(KernelError::new(
            "generated runtime requires bound ProgramRevision",
        ));
    };
    let pin = crate::runtime::ProgramRevisionPin::from_revision(program);
    let predecessor = pin
        .predecessor
        .as_ref()
        .map(|identity| {
            format!(
                "Some(kernel::ProgramRevisionId::new({:?}.into()).expect(\"sealed predecessor ProgramRevision identity reloads\"))",
                identity.as_str()
            )
        })
        .unwrap_or_else(|| "None".to_owned());
    let body = format!(
        "let revision = wire::reload({revision:?}).expect(\"sealed runtime Revision reloads\");\n\
         let pin = runtime::ProgramRevisionPin {{ identity: kernel::ProgramRevisionId::new({:?}.into()).expect(\"sealed ProgramRevision identity reloads\"), program: kernel::ProgramId::from_referent(kernel::ReferentId::new({:?}.into()).expect(\"sealed Program identity reloads\")), semantics: kernel::ClauseSemanticsId::new({:?}.into()).expect(\"sealed semantics reloads\"), predecessor: {predecessor}, snapshot: kernel::ProgramSnapshotId::new({:?}.into()).expect(\"sealed snapshot identity reloads\"), change_occurrence: kernel::ProgramChangeOccurrenceId::from_referent(kernel::ReferentId::new({:?}.into()).expect(\"sealed change occurrence reloads\")) }};\n\
         let typed = runtime::RuntimeProgramRevision::from_pin(&revision, pin.clone()).expect(\"validated ProgramRevision pin\");\n\
         let policy = {policy};\n\
         let session = runtime::RuntimeSession::replay_with_occurrences(typed, policy, runtime::SessionStartOccurrenceId::new({:?}.into()).expect(\"sealed session start reloads\"), vec![{steps}]).expect(\"authored runtime journey replays\");\n\
         let _diffs = session.states().windows(2).map(|states| runtime::StateDiff::between(&states[0], &states[1], &revision).expect(\"generated runtime state diff is checked\")).collect::<Vec<_>>();\n\
         let canonical = session.canonical_bytes();\n\
         let typed = runtime::RuntimeProgramRevision::from_pin(&revision, pin).expect(\"validated ProgramRevision pin reloads\");\n\
         let replayed = runtime::reload_session_with_program(&canonical, &typed).expect(\"generated runtime history strictly reloads\");\n\
         print!(\"{{}}\", replayed.canonical_bytes());",
        pin.identity.as_str(),
        pin.program.as_str(),
        pin.semantics.as_str(),
        pin.snapshot.as_str(),
        pin.change_occurrence.as_str(),
        start.as_str()
    );
    Ok(format!("{modules}\nfn main() {{ {body} }}"))
}

#[cfg(not(clause_generated))]
fn event_source(event: &crate::runtime::TransitionEvent) -> String {
    format!(
        "runtime::TransitionEvent::new({}, {}, vec![{}])",
        relation_source(event.id()),
        relation_source(event.event()),
        event
            .payload()
            .iter()
            .map(term_source)
            .collect::<Vec<_>>()
            .join(","),
    )
}

#[cfg(not(clause_generated))]
fn target_neutral_modules(include_runtime: bool) -> Result<String> {
    let mut modules = String::new();
    for (name, source, children) in [
        ("kernel", include_str!("kernel.rs"), KERNEL_CHILDREN),
        ("wire", include_str!("wire.rs"), WIRE_CHILDREN),
        ("intrinsic", include_str!("intrinsic.rs"), NO_CHILDREN),
        ("derive", include_str!("derive.rs"), DERIVE_CHILDREN),
        ("delta", include_str!("delta.rs"), NO_CHILDREN),
        (
            "execution",
            include_str!("execution.rs"),
            EXECUTION_CHILDREN,
        ),
        (
            "intervention",
            include_str!("intervention.rs"),
            INTERVENTION_CHILDREN,
        ),
        (
            "semantic_diff",
            include_str!("semantic_diff.rs"),
            SEMANTIC_DIFF_CHILDREN,
        ),
        ("request", include_str!("request.rs"), REQUEST_CHILDREN),
    ] {
        let body = production_module(name, source, children)?;
        writeln!(modules, "mod {name} {{\n{body}\n}}")
            .expect("writing generated modules to a String cannot fail");
    }
    if include_runtime {
        let body = production_module("runtime", include_str!("runtime.rs"), NO_CHILDREN)?;
        writeln!(modules, "mod runtime {{\n{body}\n}}")
            .expect("writing generated runtime module to a String cannot fail");
    }
    Ok(modules)
}

#[cfg(not(clause_generated))]
fn revision_order(program: &ResolvedProgram) -> Result<Vec<&crate::kernel::Revision>> {
    let mut ordered = Vec::with_capacity(program.revisions().len());
    let mut admitted = std::collections::BTreeSet::new();
    while ordered.len() < program.revisions().len() {
        let before = ordered.len();
        for revision in program.revisions().values() {
            if admitted.contains(revision.identity())
                || revision
                    .predecessor()
                    .is_some_and(|predecessor| !admitted.contains(predecessor))
            {
                continue;
            }
            admitted.insert(revision.identity().clone());
            ordered.push(revision);
        }
        if ordered.len() == before {
            return Err(KernelError::new(
                "resolved Revision registry has incomplete or cyclic lineage",
            ));
        }
    }
    Ok(ordered)
}

#[cfg(not(clause_generated))]
fn production_module(name: &str, source: &str, children: &[ChildModule]) -> Result<String> {
    let mut source = source.to_owned();
    for child in children {
        let occurrences = source.match_indices(child.declaration).count();
        if occurrences != 1 {
            return Err(KernelError::new(format!(
                "generated module '{name}' expected exactly one declaration for child '{}', found {occurrences}",
                child.name
            )));
        }
        source = source.replacen(
            child.declaration,
            &format!(
                "{} {{\n{}\n}}",
                child.declaration.trim_end_matches(';'),
                child.source
            ),
            1,
        );
    }
    Ok(source)
}

#[cfg(not(clause_generated))]
fn revision_source(
    identity: &crate::kernel::RevisionId,
    indices: &std::collections::BTreeMap<crate::kernel::RevisionId, usize>,
) -> String {
    let index = indices.get(identity).expect("request revision is embedded");
    format!("r{index}.identity().clone()")
}

#[cfg(not(clause_generated))]
fn request_source(
    request: &Request,
    indices: &std::collections::BTreeMap<crate::kernel::RevisionId, usize>,
) -> String {
    match request {
        Request::Any {
            revision,
            pattern,
            dependencies,
        } => format!(
            "request::Request::Any {{ revision: {}, pattern: {}, dependencies: vec![{}] }}",
            revision_source(revision, indices),
            clause_source(pattern),
            dependencies
                .iter()
                .map(clause_source)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Request::Select {
            revision,
            pattern,
            dependencies,
            columns,
            selection,
        } => format!(
            "request::Request::Select {{ revision: {}, pattern: {}, dependencies: vec![{}], columns: vec![{}], selection: {} }}",
            revision_source(revision, indices),
            clause_source(pattern),
            dependencies
                .iter()
                .map(clause_source)
                .collect::<Vec<_>>()
                .join(","),
            columns
                .iter()
                .map(query_column_source)
                .collect::<Vec<_>>()
                .join(","),
            query_selection_source(*selection),
        ),
        Request::Find {
            revision,
            pattern,
            sought,
        } => format!(
            "request::Request::Find {{ revision: {}, pattern: {}, sought: {} }}",
            revision_source(revision, indices),
            clause_source(pattern),
            variable_source(sought)
        ),
        Request::Why {
            revision,
            target,
            all,
        } => format!(
            "request::Request::Why {{ revision: {}, target: {}, all: {all} }}",
            revision_source(revision, indices),
            clause_source(target)
        ),
        Request::Prevent {
            revision,
            target,
            selection,
            using,
        } => format!(
            "request::Request::Prevent {{ revision: {}, target: {}, selection: {}, using: vec![{}] }}",
            revision_source(revision, indices),
            clause_source(target),
            selection_source(*selection),
            using
                .iter()
                .map(relation_source)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Request::Achieve {
            revision,
            target,
            selection,
            using,
        } => format!(
            "request::Request::Achieve {{ revision: {}, target: {}, selection: {}, using: vec![{}] }}",
            revision_source(revision, indices),
            clause_source(target),
            selection_source(*selection),
            using
                .iter()
                .map(relation_source)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Request::Diff { base, successor } => format!(
            "request::Request::Diff {{ base: {}, successor: {} }}",
            revision_source(base, indices),
            revision_source(successor, indices)
        ),
    }
}

#[cfg(not(clause_generated))]
fn query_column_source(column: &QueryColumn) -> String {
    let label = column
        .label()
        .map(|label| format!("Some({label:?}.to_owned())"))
        .unwrap_or_else(|| "None".to_owned());
    format!(
        "request::QueryColumn::new({label}, {}, vec![{}])",
        variable_source(column.binder()),
        column
            .origins()
            .iter()
            .map(role_source)
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(not(clause_generated))]
fn selection_source(selection: Selection) -> &'static str {
    match selection {
        Selection::OneMinimal => "request::Selection::OneMinimal",
        Selection::AllMinimal => "request::Selection::AllMinimal",
    }
}

#[cfg(not(clause_generated))]
fn query_selection_source(selection: QuerySelection) -> &'static str {
    match selection {
        QuerySelection::All => "request::QuerySelection::All",
        QuerySelection::ExactlyOne => "request::QuerySelection::ExactlyOne",
        QuerySelection::CanonicalFirst => "request::QuerySelection::CanonicalFirst",
    }
}
#[cfg(not(clause_generated))]
fn relation_source(value: &ReferentId) -> String {
    format!(
        "kernel::ReferentId::new({:?}.into()).expect(\"generated relation\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn definition_source(value: &ReferentId) -> String {
    format!(
        "kernel::ReferentId::new({:?}.into()).expect(\"generated definition identity\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn role_source(value: &RoleId) -> String {
    format!(
        "kernel::RoleId::new({:?}.into()).expect(\"generated role\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn variable_source(value: &PatternId) -> String {
    format!(
        "kernel::PatternId::new({:?}.into()).expect(\"generated variable\")",
        value.as_str()
    )
}
#[cfg(not(clause_generated))]
fn term_source(value: &Term) -> String {
    match value {
        Term::Referent(id) => format!(
            "kernel::Term::referent(kernel::ReferentId::new({:?}.into()).expect(\"generated referent\"))",
            id.as_str()
        ),
        Term::Pattern(id) => format!("kernel::Term::pattern({})", variable_source(id)),
        Term::Application(id) => format!(
            "kernel::Term::application(kernel::ContentId::new({:?}.into()).expect(\"generated content\"))",
            id.as_str()
        ),
        Term::F32(value) => format!(
            "kernel::Term::f32_bits({}).expect(\"generated finite F32\")",
            value.bits()
        ),
        Term::Int(value) => format!("kernel::Term::int({value})"),
        Term::Bool(value) => format!("kernel::Term::boolean({value})"),
        Term::Product { shape, fields } => {
            let fields = fields
                .iter()
                .map(|(label, field)| {
                    format!(
                        "(kernel::Name::new({:?}.into()).expect(\"generated product label\"), kernel::ProductField::new(kernel::ReferentId::new({:?}.into()).expect(\"generated product domain\"), {}))",
                        label.as_str(),
                        field.domain().as_str(),
                        term_source(field.value())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "kernel::Term::product(kernel::ReferentId::new({:?}.into()).expect(\"generated product shape\"), std::collections::BTreeMap::from([{fields}])).expect(\"generated structural product\")",
                shape.as_str()
            )
        }
        Term::LabelledProduct { shape, fields } => {
            let fields = fields
                .iter()
                .map(|(field, value)| format!(
                    "(kernel::ReferentId::new({:?}.into()).expect(\"generated product field\"), {})",
                    field.as_str(),
                    term_source(value)
                ))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "kernel::Term::labelled_product(kernel::ReferentId::new({:?}.into()).expect(\"generated labelled product shape\"), std::collections::BTreeMap::from([{fields}])).expect(\"generated labelled product\")",
                shape.as_str()
            )
        }
        Term::Sum { tag, value } => format!(
            "kernel::Term::sum(kernel::Name::new({:?}.into()).expect(\"generated sum tag\"), {}).expect(\"generated structural sum\")",
            tag.as_str(),
            term_source(value)
        ),
        Term::Sequence {
            shape,
            element,
            values,
        } => format!(
            "kernel::Term::sequence(kernel::ReferentId::new({:?}.into()).expect(\"generated sequence shape\"), kernel::ReferentId::new({:?}.into()).expect(\"generated sequence element domain\"), vec![{}]).expect(\"generated structural sequence\")",
            shape.as_str(),
            element.as_str(),
            values.iter().map(term_source).collect::<Vec<_>>().join(",")
        ),
    }
}
#[cfg(not(clause_generated))]
fn clause_source(value: &RelationalContent) -> String {
    format!(
        "kernel::RelationalContent::new({}, std::collections::BTreeMap::from([{}])).expect(\"generated clause\")",
        relation_source(value.relation()),
        value
            .roles()
            .iter()
            .map(|(role, term)| format!("({}, {})", role_source(role), term_source(term)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(test)]
mod tests;
