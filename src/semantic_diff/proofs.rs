//! Chosen-proof comparison for consequences shared by two closures.

use crate::derive::Closure;

use super::ProofChange;

pub(super) fn changes(base: &Closure, successor: &Closure) -> Vec<ProofChange> {
    base.assertions()
        .iter()
        .filter_map(|consequence| {
            let successor_proof = successor.proof(consequence)?;
            let base_proof = base
                .proof(consequence)
                .expect("closure clauses always have selected proofs");
            (base_proof != successor_proof).then(|| ProofChange {
                consequence: consequence.clone(),
                base: base_proof.clone(),
                successor: successor_proof.clone(),
            })
        })
        .collect()
}
