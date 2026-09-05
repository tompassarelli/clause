//! Build the browser identity-input specimen through the actual resident compiler.
fn main() {
    let source = include_bytes!("../../../test-vectors/authoring/referent-input-transition.clause");
    let workbench = clause_workbench::ResidentSourceWorkbenchV1::open(source).unwrap();
    let destination =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/referent-input.cwr1");
    std::fs::write(&destination, &workbench.generation().cwr1).unwrap();
    println!("{}", destination.display());
}
