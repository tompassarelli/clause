use clause_package::*;
use clause_runtime::{ExecutableValueV1 as V, lower_canonical_callable_v1};

/// Exercises source checking, callable lowering, and evaluation in the Wasm target.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn check_text_decomposition() -> u32 {
    let cst = read_canonical_source_v1(include_bytes!(
        "../../../test-vectors/authoring/text-decomposition.clause"
    )).unwrap();
    let plan = plan_independent_canonical_source_allocations_v1(
        &cst, ProgramChangeOccurrenceId::from_bytes([3; 32]),
    ).unwrap();
    let checked = CheckedCanonicalSourceAnalysisV1::new(cst, plan, CanonicalSourceContextV1 {
        universe: UniverseId::from_bytes([1; 32]),
        semantics: ClauseSemanticsId::from_bytes([2; 32]),
    }).unwrap();
    let text = |value: &str| V::text(value).unwrap();
    let texts = |values: &[&str]| V::Sequence(values.iter().map(|v| text(v)).collect());
    let cases = [
        ("letters", vec![text("a🚀e\u{301}\0")], texts(&["a", "🚀", "e", "\u{301}", "\0"])),
        ("parts", vec![text("/a//"), text("/")], texts(&["", "a", "", ""])),
        ("parts", vec![text("a.*b.*"), text(".*")], texts(&["a", "b", ""])),
        ("parts", vec![text("🚀a"), text("")], texts(&["🚀", "a"])),
        ("integer", vec![text("\u{feff} +42tail")], V::number(42.0).unwrap()),
        ("integer", vec![text("not a number")], text("not a number")),
        ("command-argv", vec![text("firn\0\0host\0")], texts(&["firn", "", "host"])),
    ];
    for (name, arguments, expected) in &cases {
        let definition = checked.package().callables.iter().find(|c| c.designation == name.as_bytes()).unwrap();
        let callable = lower_canonical_callable_v1(definition).unwrap();
        assert_eq!(callable.invoke(arguments).unwrap(), *expected, "{name}");
    }
    cases.len() as u32
}

fn main() {
    println!("{} checked text cases passed", check_text_decomposition());
}
