//! Build the browser identity-input specimen through the actual resident compiler.
fn main() {
    for (name, source) in [
        (
            "referent-input",
            include_bytes!("../../../test-vectors/authoring/referent-input-transition.clause")
                .as_slice(),
        ),
        (
            "account-contributions",
            include_bytes!("../../../test-vectors/authoring/selected-account-contributions.clause")
                .as_slice(),
        ),
        (
            "party-contributions",
            include_bytes!("../../../test-vectors/authoring/targeted-party-contributions.clause")
                .as_slice(),
        ),
        (
            "cross-subject-target",
            include_bytes!("../../../test-vectors/authoring/cross-subject-referent-target.clause")
                .as_slice(),
        ),
    ] {
        let workbench = clause_workbench::ResidentSourceWorkbenchV1::open(source).unwrap();
        let destination = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../target/{name}.cwr1"));
        std::fs::write(&destination, &workbench.generation().cwr1).unwrap();
        println!("{}", destination.display());
    }
}
