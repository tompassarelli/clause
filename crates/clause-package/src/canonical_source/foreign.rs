use super::*;

/// Exact foreign member access. Module and member names are inert ABI data.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalForeignBindingV1 {
    pub module: String,
    pub member: String,
    pub operation: CanonicalForeignOperationV1,
    pub failure: CanonicalForeignFailureV1,
    pub arguments: Vec<CanonicalValueTypeV1>,
    pub result: CanonicalValueTypeV1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalForeignOperationV1 {
    /// Invoke the exported function without a receiver.
    Call,
    Get,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalForeignFailureV1 {
    Throw,
}

impl CanonicalForeignBindingV1 {
    /// Foreign reads and calls remain host obligations until a target binds
    /// this exact member and validates its typed crossing.
    pub fn check(&self) -> Result<(), &'static str> {
        if self.module.is_empty()
            || self.member.is_empty()
            || self.module.contains('\0')
            || self.member.contains('\0')
        {
            return Err("foreign module and member must be nonempty identifiers");
        }
        if self.operation == CanonicalForeignOperationV1::Get && !self.arguments.is_empty() {
            return Err("foreign property access takes no arguments");
        }
        for kind in self.arguments.iter().chain(std::iter::once(&self.result)) {
            kind.check()?;
        }
        Ok(())
    }
}

pub(super) fn read_abi(
    lines: &[SourceLine<'_>],
) -> Option<(
    CanonicalForeignOperationV1,
    CanonicalForeignFailureV1,
    String,
    String,
)> {
    let mut member = None;
    let mut module = None;
    let mut failure = None;
    for line in lines.iter().filter(|l| !l.text.trim().is_empty()) {
        let (key, value) = line.text.trim().split_once(": ")?;
        match key {
            "get" | "call" if member.is_none() => {
                member = Some((
                    if key == "get" {
                        CanonicalForeignOperationV1::Get
                    } else {
                        CanonicalForeignOperationV1::Call
                    },
                    parse_text_literal(value)?,
                ))
            }
            "from" if module.is_none() => module = Some(parse_text_literal(value)?),
            "failure" if value == "throw" && failure.is_none() => {
                failure = Some(CanonicalForeignFailureV1::Throw)
            }
            _ => return None,
        }
    }
    let (operation, member) = member?;
    Some((operation, failure?, module?, member))
}
