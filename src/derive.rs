//! Deterministic finite closure for admitted positive laws.

mod closure;
mod matching;
mod support;

pub use closure::{Closure, Limits, Proof, Witness, saturate};
pub use support::{
    Support, SupportFrontier, SupportLimits, SupportProof, SupportStatus, SupportWitness,
    support_frontier,
};

#[cfg(test)]
mod tests;
