use std::error::Error;
use std::fmt;

use clause_package::{
    AuthorityStore, CheckedProcessPackage, ProcessCarrier, ProcessError, ProcessIngressError,
    ProcessRecordV2,
};

mod executable;
mod persistent_session;
mod wasm_boundary;

pub use executable::*;
pub use persistent_session::*;
pub use wasm_boundary::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeInitError {
    NonSerialStepBatch {
        record_index: usize,
        step_count: usize,
    },
    Carrier(ProcessError),
}

impl fmt::Display for RuntimeInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonSerialStepBatch {
                record_index,
                step_count,
            } => write!(
                formatter,
                "package record {record_index} contains {step_count} Steps; the serial runtime requires exactly one"
            ),
            Self::Carrier(error) => write!(formatter, "process carrier rejected package: {error}"),
        }
    }
}

impl Error for RuntimeInitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Carrier(error) => Some(error),
            Self::NonSerialStepBatch { .. } => None,
        }
    }
}

impl From<ProcessError> for RuntimeInitError {
    fn from(error: ProcessError) -> Self {
        Self::Carrier(error)
    }
}

/// Progressive execution of one exact checked process package.
///
/// The package owns every executable proposal. Advancing dispatches only on
/// the universal `ProcessRecordV2` wire structure retained by the checked
/// package; it has no callback or semantic-identifier selection surface.
pub struct ProcessRuntime {
    package: CheckedProcessPackage,
    authority: AuthorityStore,
    carrier: ProcessCarrier,
}

impl ProcessRuntime {
    /// Instantiate after rejecting every unsupported package surface. This
    /// happens before any package record becomes visible in the carrier.
    pub fn instantiate(
        package: &CheckedProcessPackage,
        authority: &AuthorityStore,
    ) -> Result<Self, RuntimeInitError> {
        Self::instantiate_owned(package.clone(), authority.clone())
    }

    /// Instantiate while transferring the exact checked package and authority
    /// store into the runtime. Persistent hosts use this path so neither input
    /// is cloned for each command.
    pub fn instantiate_owned(
        package: CheckedProcessPackage,
        authority: AuthorityStore,
    ) -> Result<Self, RuntimeInitError> {
        for (record_index, record) in package.records().iter().enumerate() {
            if let ProcessRecordV2::Steps(steps) = record
                && steps.len() != 1
            {
                return Err(RuntimeInitError::NonSerialStepBatch {
                    record_index,
                    step_count: steps.len(),
                });
            }
        }
        let carrier = ProcessCarrier::instantiate(&package, &authority)?;
        Ok(Self {
            package,
            authority,
            carrier,
        })
    }

    /// Apply exactly the next package-owned record. `None` means the checked
    /// package is complete; rejection preserves the current record cursor.
    pub fn advance(&mut self) -> Result<Option<ProcessRecordV2>, ProcessError> {
        self.carrier.advance_package(&self.package, &self.authority)
    }

    /// Apply live records through the same checked carrier that owns package
    /// replay. The runtime retains its original package and authority binding;
    /// ingress cannot replace either one.
    pub fn apply_ingress(
        &mut self,
        records: &[ProcessRecordV2],
    ) -> Result<(), ProcessIngressError> {
        self.carrier.apply_ingress(records, &self.authority)
    }

    #[must_use]
    pub const fn carrier(&self) -> &ProcessCarrier {
        &self.carrier
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.remaining_record_count() == 0
    }

    #[must_use]
    pub fn remaining_record_count(&self) -> usize {
        self.package
            .records()
            .len()
            .saturating_sub(self.carrier.applied_package_record_count())
    }

    #[must_use]
    pub fn into_carrier(self) -> ProcessCarrier {
        self.carrier
    }
}
