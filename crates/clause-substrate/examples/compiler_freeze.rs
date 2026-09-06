//! Construct-blind package invocation for the host-freeze experiment.
use std::{env, error::Error, fs};

use clause_substrate::{
    compiler_package_v3::{Id32, KValue, decode, decode_canonical_term, encode_canonical_term},
    evaluator::Evaluator,
};

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let [
        command,
        package_path,
        entrypoint_path,
        argument_path,
        fuel,
        output,
    ] = arguments.as_slice()
    else {
        return Err("expected evaluate PACKAGE ENTRYPOINT ARGUMENT FUEL OUTPUT".into());
    };
    if command != "evaluate" {
        return Err("expected evaluate".into());
    }
    let package = decode(&fs::read(package_path)?)?;
    let entrypoint = Id32(
        fs::read(entrypoint_path)?
            .try_into()
            .map_err(|_| "entrypoint must contain exactly 32 bytes")?,
    );
    let argument = decode_canonical_term(&fs::read(argument_path)?)?;
    let evaluator = Evaluator::new(&package.package().subject.program)?;
    let result =
        evaluator.invoke_entrypoint(entrypoint, &[KValue::Term(argument)], fuel.parse()?)?;
    let value = match result.value {
        KValue::Bytes(bytes) => {
            let mut value = vec![0];
            value.extend(u32::try_from(bytes.len())?.to_be_bytes());
            value.extend(bytes);
            value
        }
        KValue::Term(term) => {
            let mut value = vec![1];
            value.extend(encode_canonical_term(&term)?);
            value
        }
    };
    fs::write(output, value)?;
    fs::write(
        format!("{output}.observations"),
        encode_canonical_term(&result.observations.try_to_term()?)?,
    )?;
    fs::write(
        format!("{output}.fuel"),
        format!("{}\n", result.remaining_fuel),
    )?;
    Ok(())
}
