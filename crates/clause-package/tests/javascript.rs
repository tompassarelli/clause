use std::path::Path;
use std::process::Command;

use clause_package::*;

const COMMAND_TEXT: &[u8] = include_bytes!("../../../test-vectors/authoring/command-text.clause");

fn checked(source: &[u8]) -> CanonicalSourcePackageSliceV1 {
    let cst = read_canonical_source_v1(source).unwrap();
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst,
        ProgramChangeOccurrenceId::from_bytes([3; IDENTITY_BYTES]),
    )
    .unwrap();
    elaborate_canonical_source_package_v1(
        &cst,
        CanonicalSourceContextV1 {
            universe: UniverseId::from_bytes([1; IDENTITY_BYTES]),
            semantics: ClauseSemanticsId::from_bytes([2; IDENTITY_BYTES]),
        },
        &plan,
    )
    .unwrap()
}

fn write_module(
    directory: &Path,
    name: &str,
    package: &CanonicalSourcePackageSliceV1,
) -> JavaScriptArtifactsV1 {
    let artifacts = lower_javascript_v1(package).unwrap();
    std::fs::write(directory.join(name), &artifacts.module).unwrap();
    artifacts
}

#[test]
fn generated_command_text_runs_in_bun_with_typed_rejection_and_atomic_pre_state() {
    let directory = std::env::temp_dir().join(format!("clause-js-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let mut package = checked(COMMAND_TEXT);
    let artifacts = write_module(&directory, "command.mjs", &package);
    assert!(
        artifacts
            .declarations
            .contains("\"describe\": (arg0: string, arg1: string) => void")
    );
    assert!(
        artifacts
            .declarations
            .contains("read(subject: \"response\", relation: \"output\"): string")
    );

    let original = package.executable_handlers[0].rules[0].clone();
    package.executable_handlers[0].rules.push(original.clone());
    write_module(&directory, "conflict.mjs", &package);
    package.executable_handlers[0].rules.pop();

    let output_state = original.assignments[0].target.clone();
    let summary_state = package
        .state_cells
        .iter()
        .find(|cell| cell.state.relation_designation == b"summary")
        .unwrap()
        .state
        .clone();
    let summary_binding = original
        .predicates
        .iter()
        .find_map(|predicate| match predicate {
            CanonicalExecutablePredicateV1::RelationMatch(state, subject, _)
                if *state == summary_state =>
            {
                Some(subject.clone())
            }
            _ => None,
        })
        .unwrap();
    let response = package
        .relational_projection
        .iter()
        .find(|projection| projection.subject == b"response")
        .unwrap()
        .referent;
    package.executable_handlers[0].rules[0]
        .assignments
        .push(CanonicalExecutableAssignmentV1 {
            target: summary_state,
            value: CanonicalExecutableExpressionV1::RelationEffects(vec![
                CanonicalRelationEffectV1::Put(
                    summary_binding,
                    CanonicalExecutableExpressionV1::RelationRead(
                        Box::new(CanonicalExecutableExpressionV1::State(output_state)),
                        Box::new(CanonicalExecutableExpressionV1::Constant(
                            CanonicalScalarValueV1::Referent(response),
                        )),
                    ),
                ),
            ]),
        });
    write_module(&directory, "prestate.mjs", &package);

    let script = r#"
import { createSession } from './command.mjs';
import { createSession as conflicting } from './conflict.mjs';
import { createSession as prestate } from './prestate.mjs';
function assert(value){if(!value)throw Error('assertion failed');}
function rejects(run,code){try{run();}catch(error){assert(error.message===code);return;}throw Error('expected '+code);}
const session=createSession();
assert(session.read('response','output')==='Unknown command\n');
session.handlers.describe('module','list');
assert(session.read('response','output')==='firn module list: list modules\n');
session.handlers.describe('module','status');
assert(session.read('response','output')==='firn module status: show module status\n');
session.handlers.describe('module','missing');
assert(session.read('response','output')==='firn module status: show module status\n');
rejects(()=>session.handlers.describe(7,'list'),'TextDomain');
rejects(()=>session.handlers.describe('module'),'ArgumentCount');
rejects(()=>session.handlers.describe('\ud800','list'),'TextDomain');
assert(session.read('response','output')==='firn module status: show module status\n');
const conflict=conflicting();
rejects(()=>conflict.handlers.describe('module','list'),'ConflictingStateEffects');
assert(conflict.read('response','output')==='Unknown command\n');
const simultaneous=prestate();
simultaneous.handlers.describe('module','list');
assert(simultaneous.read('response','output')==='firn module list: list modules\n');
assert(simultaneous.read('list','summary')==='Unknown command\n');
console.log('command-text: exact output, typed rejection, conflict atomicity, pre-state reads');
"#;
    std::fs::write(directory.join("run.mjs"), script).unwrap();
    let output = Command::new(std::env::var_os("CLAUSE_BUN").unwrap_or_else(|| "bun".into()))
        .arg(directory.join("run.mjs"))
        .output()
        .expect("Bun must be available for the JavaScript backend check");
    for name in ["command.mjs", "conflict.mjs", "prestate.mjs", "run.mjs"] {
        std::fs::remove_file(directory.join(name)).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "command-text: exact output, typed rejection, conflict atomicity, pre-state reads\n"
    );
}

#[test]
fn unsupported_executable_ir_is_rejected_before_emitting_an_artifact() {
    let mut package = checked(COMMAND_TEXT);
    package.executable_handlers[0].rules[0].assignments[0].value =
        CanonicalExecutableExpressionV1::Accumulate(Box::new(
            CanonicalExecutableExpressionV1::Constant(CanonicalScalarValueV1::Number(
                1.0_f64.to_bits(),
            )),
        ));
    assert_eq!(
        lower_javascript_v1(&package).unwrap_err().0,
        "JavaScript lowering currently requires relation-row assignments"
    );
    package.executable_handlers[0].trigger = CanonicalHandlerTriggerV1::RelationClosure;
    assert!(
        lower_javascript_v1(&package)
            .unwrap_err()
            .0
            .contains("RelationClosure")
    );
}

#[test]
fn pure_callable_exports_source_name_and_checks_finite_arguments_and_results() {
    let mut package = checked(include_bytes!(
        "../../../test-vectors/authoring/pure-callable.clause"
    ));
    let mut divide = package.callables[0].clone();
    divide.designation = b"quotient".to_vec();
    divide.arguments.truncate(2);
    for argument in &mut divide.arguments {
        argument.value_kind = CanonicalScalarValueKindV1::Number;
    }
    divide.result_kind = CanonicalScalarValueKindV1::Number;
    divide.expression = CanonicalExecutableExpressionV1::Divide(
        Box::new(CanonicalExecutableExpressionV1::Argument(0)),
        Box::new(CanonicalExecutableExpressionV1::Argument(1)),
    );
    package.callables.push(divide);
    let directory = std::env::temp_dir().join(format!("clause-js-pure-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let artifacts = write_module(&directory, "pure.mjs", &package);
    assert!(
        artifacts
            .declarations
            .contains("export { callable0 as \"missing-leaf\" }")
    );
    assert!(
        artifacts
            .declarations
            .contains("(arg0: string, arg1: string, arg2: string, arg3: string): string")
    );
    assert!(
        artifacts
            .declarations
            .contains("(arg0: number, arg1: number): number")
    );
    assert!(!artifacts.declarations.contains("createSession"));
    std::fs::write(directory.join("pure.d.ts"), artifacts.declarations).unwrap();
    std::fs::write(directory.join("run.mjs"), r#"
import { 'missing-leaf' as missingLeaf, quotient } from './pure.mjs';
function assert(value){if(!value)throw Error('assertion failed');}
function rejects(run,code){try{run();}catch(error){assert(error.message===code);return;}throw Error('expected '+code);}
const actual=missingLeaf('module','enable','NAME','Enable a module');
assert(actual==="firn: 'module enable' requires a leaf node\nUsage: firn module enable NAME\n  Enable a module\n");
rejects(()=>missingLeaf('module',7,'NAME','Enable a module'),'TextDomain');
rejects(()=>missingLeaf('module'),'ArgumentCount');
assert(quotient(6,2)===3);
rejects(()=>quotient(Infinity,2),'NumericDomain');
rejects(()=>quotient(1,0),'NumericDomain');
new Bun.Transpiler({loader:'ts'}).transformSync(await Bun.file(new URL('./pure.d.ts',import.meta.url)).text());
console.log(actual);
"#).unwrap();
    let output = Command::new(std::env::var_os("CLAUSE_BUN").unwrap_or_else(|| "bun".into()))
        .arg(directory.join("run.mjs"))
        .output()
        .expect("Bun must be available for the JavaScript backend check");
    for name in ["pure.mjs", "pure.d.ts", "run.mjs"] {
        std::fs::remove_file(directory.join(name)).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "firn: 'module enable' requires a leaf node\nUsage: firn module enable NAME\n  Enable a module\n\n"
    );
}
