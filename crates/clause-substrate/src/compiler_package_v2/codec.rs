use super::*;

const MAGIC: &[u8; 4] = b"CLCP";
const VERSION: u8 = 0x02;

struct EncodeBudget {
    items: usize,
    term_nodes: usize,
    expression_nodes: usize,
}

impl EncodeBudget {
    const fn new() -> Self {
        Self {
            items: 0,
            term_nodes: 0,
            expression_nodes: 0,
        }
    }
}

struct Encoder<'a> {
    bytes: Vec<u8>,
    budget: &'a mut EncodeBudget,
}

impl<'a> Encoder<'a> {
    fn new(budget: &'a mut EncodeBudget) -> Result<Self, EncodeError> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(MAX_WIRE_BYTES)
            .map_err(|_| EncodeError::ResourceExhausted)?;
        Ok(Self { bytes, budget })
    }

    fn reserve(&self, additional: usize) -> Result<(), EncodeError> {
        let end = self
            .bytes
            .len()
            .checked_add(additional)
            .ok_or(EncodeError::ResourceExhausted)?;
        if end > MAX_WIRE_BYTES {
            Err(EncodeError::ResourceExhausted)
        } else {
            Ok(())
        }
    }

    fn term_node(&mut self) -> Result<(), EncodeError> {
        self.budget.term_nodes = self
            .budget
            .term_nodes
            .checked_add(1)
            .ok_or(EncodeError::ResourceExhausted)?;
        if self.budget.term_nodes > MAX_TERM_NODES {
            Err(EncodeError::ResourceExhausted)
        } else {
            Ok(())
        }
    }

    fn expression_node(&mut self) -> Result<(), EncodeError> {
        self.budget.expression_nodes = self
            .budget
            .expression_nodes
            .checked_add(1)
            .ok_or(EncodeError::ResourceExhausted)?;
        if self.budget.expression_nodes > MAX_EXPRESSION_NODES {
            Err(EncodeError::ResourceExhausted)
        } else {
            Ok(())
        }
    }

    fn items(&mut self, additional: usize) -> Result<(), EncodeError> {
        self.budget.items = self
            .budget
            .items
            .checked_add(additional)
            .ok_or(EncodeError::ResourceExhausted)?;
        if self.budget.items > MAX_WIRE_ITEMS {
            Err(EncodeError::ResourceExhausted)
        } else {
            Ok(())
        }
    }

    fn u8(&mut self, value: u8) -> Result<(), EncodeError> {
        self.reserve(1)?;
        self.bytes.push(value);
        Ok(())
    }

    fn u32(&mut self, value: u32) -> Result<(), EncodeError> {
        self.reserve(4)?;
        self.bytes.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    fn u64(&mut self, value: u64) -> Result<(), EncodeError> {
        self.reserve(8)?;
        self.bytes.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    fn fixed(&mut self, value: &[u8]) -> Result<(), EncodeError> {
        self.reserve(value.len())?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn length(&mut self, field: &'static str, length: usize) -> Result<(), EncodeError> {
        let length =
            u32::try_from(length).map_err(|_| EncodeError::LengthExceedsU32 { field, length })?;
        self.u32(length)
    }

    fn blob(&mut self, field: &'static str, value: &[u8]) -> Result<(), EncodeError> {
        self.length(field, value.len())?;
        self.fixed(value)
    }

    fn sequence<T>(
        &mut self,
        field: &'static str,
        values: &[T],
        mut encode: impl FnMut(&mut Self, &T) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.length(field, values.len())?;
        if values.len() > MAX_WIRE_ITEMS {
            return Err(EncodeError::ResourceExhausted);
        }
        self.items(values.len())?;
        for value in values {
            encode(self, value)?;
        }
        Ok(())
    }

    fn frame(&mut self, tag: u8, field: &'static str, payload: Vec<u8>) -> Result<(), EncodeError> {
        self.u8(tag)?;
        self.length(field, payload.len())?;
        self.fixed(&payload)
    }
}

pub fn encode(package: &CompilerPackage) -> Result<Vec<u8>, EncodeError> {
    let mut budget = EncodeBudget::new();
    let mut encoder = Encoder::new(&mut budget)?;
    encoder.fixed(MAGIC)?;
    encoder.u8(VERSION)?;
    let core_manifest =
        encode_core_manifest_value_with_budget(&package.core_manifest, &mut *encoder.budget)?;
    encoder.frame(0x01, "core manifest frame", core_manifest)?;
    let subject = encode_subject_value(&package.subject, &mut *encoder.budget)?;
    encoder.frame(0x02, "compiler subject frame", subject)?;
    let evidence = encode_evidence_value(&package.evidence, &mut *encoder.budget)?;
    encoder.frame(0x03, "compiler evidence frame", evidence)?;
    Ok(encoder.bytes)
}

pub(crate) fn encode_core_manifest_value(manifest: &CoreManifest) -> Result<Vec<u8>, EncodeError> {
    let mut budget = EncodeBudget::new();
    encode_core_manifest_value_with_budget(manifest, &mut budget)
}

fn encode_core_manifest_value_with_budget(
    manifest: &CoreManifest,
    budget: &mut EncodeBudget,
) -> Result<Vec<u8>, EncodeError> {
    let mut encoder = Encoder::new(budget)?;
    encoder.u8(manifest.manifest_version)?;
    encode_u8_sequence(&mut encoder, "frame tags", &manifest.frame_tags)?;
    encode_u8_sequence(&mut encoder, "term tags", &manifest.term_tags)?;
    encode_u8_sequence(&mut encoder, "sort tags", &manifest.sort_tags)?;
    encoder.sequence(
        "expression forms",
        &manifest.expression_forms,
        encode_named_signature,
    )?;
    encoder.sequence("ABI forms", &manifest.abi_forms, encode_named_signature)?;
    encode_u8_sequence(
        &mut encoder,
        "premise policy tags",
        &manifest.premise_policy_tags,
    )?;
    encode_u8_sequence(&mut encoder, "lineage tags", &manifest.lineage_tags)?;
    encode_u8_sequence(
        &mut encoder,
        "nominal declaration tags",
        &manifest.nominal_declaration_tags,
    )?;
    encode_u8_sequence(
        &mut encoder,
        "compiler evidence tags",
        &manifest.compiler_evidence_tags,
    )?;
    encode_u8_sequence(&mut encoder, "value tags", &manifest.value_tags)?;
    encode_u8_sequence(
        &mut encoder,
        "evaluation outcome tags",
        &manifest.eval_outcome_tags,
    )?;
    encode_u8_sequence(
        &mut encoder,
        "decode verdict tags",
        &manifest.decode_verdict_tags,
    )?;
    encode_u8_sequence(&mut encoder, "decode code tags", &manifest.decode_code_tags)?;
    encode_u8_sequence(
        &mut encoder,
        "authorization stage tags",
        &manifest.authorization_stage_tags,
    )?;
    encode_u8_sequence(
        &mut encoder,
        "authorization code tags",
        &manifest.authorization_code_tags,
    )?;
    encoder.sequence(
        "static rules",
        &manifest.static_rules,
        encode_rule_signature,
    )?;
    encoder.sequence(
        "evaluation rules",
        &manifest.evaluation_rules,
        encode_rule_signature,
    )?;
    encoder.u8(manifest.certificate_format_version)?;
    encoder.blob("certificate signature", &manifest.certificate_signature)?;
    encoder.sequence(
        "contract clauses",
        &manifest.contract_clauses,
        |encoder, clause| encoder.blob("contract clause", clause),
    )?;
    encode_physical_profile(&mut encoder, &manifest.physical_profile)?;
    Ok(encoder.bytes)
}

pub(crate) fn encode_physical_profile_value(
    profile: &PhysicalProfile,
) -> Result<Vec<u8>, EncodeError> {
    let mut budget = EncodeBudget::new();
    let mut encoder = Encoder::new(&mut budget)?;
    encode_physical_profile(&mut encoder, profile)?;
    Ok(encoder.bytes)
}

fn encode_u8_sequence(
    encoder: &mut Encoder<'_>,
    field: &'static str,
    values: &[u8],
) -> Result<(), EncodeError> {
    encoder.sequence(field, values, |encoder, value| encoder.u8(*value))
}

fn encode_named_signature(
    encoder: &mut Encoder<'_>,
    value: &NamedSignature,
) -> Result<(), EncodeError> {
    encoder.u8(value.tag)?;
    encoder.blob("named signature", &value.signature)
}

fn encode_rule_signature(
    encoder: &mut Encoder<'_>,
    value: &RuleSignature,
) -> Result<(), EncodeError> {
    encoder.u8(value.tag)?;
    encoder.u8(value.premise_policy)?;
    encoder.blob("rule clause", &value.clause)
}

fn encode_physical_profile(
    encoder: &mut Encoder<'_>,
    value: &PhysicalProfile,
) -> Result<(), EncodeError> {
    encoder.u8(value.profile_version)?;
    encoder.u8(value.observation_policy)?;
    encoder.sequence(
        "physical operations",
        &value.operations,
        |encoder, operation| {
            encoder.fixed(operation.operation_id.as_bytes())?;
            encoder.sequence(
                "physical operation arguments",
                &operation.arguments,
                |encoder, sort| encode_sort(encoder, *sort),
            )?;
            encode_sort(encoder, operation.result)
        },
    )
}

fn encode_sort(encoder: &mut Encoder<'_>, sort: KSort) -> Result<(), EncodeError> {
    encoder.u8(match sort {
        KSort::Bytes => 0x00,
        KSort::Term => 0x01,
    })
}

fn encode_term(
    encoder: &mut Encoder<'_>,
    value: &Term,
    current_depth: usize,
) -> Result<(), EncodeError> {
    let next_depth = encode_term_depth(current_depth)?;
    encoder.term_node()?;
    match value {
        Term::Atom {
            kind,
            canonical_payload,
            equality_contract,
        } => {
            encoder.u8(0x00)?;
            encoder.blob("Atom kind", kind)?;
            encoder.blob("Atom payload", canonical_payload)?;
            encoder.blob("Atom equality contract", equality_contract)
        }
        Term::Triple(first, second, third) => {
            encoder.u8(0x01)?;
            encode_term(encoder, first, next_depth)?;
            encode_term(encoder, second, next_depth)?;
            encode_term(encoder, third, next_depth)
        }
    }
}

struct EncodeExpressionTask<'a> {
    expression: &'a KExpr,
    depth: usize,
}

fn encode_expr(
    encoder: &mut Encoder<'_>,
    value: &KExpr,
    current_depth: usize,
) -> Result<(), EncodeError> {
    let mut tasks = Vec::new();
    push_encode_expression(
        &mut tasks,
        EncodeExpressionTask {
            expression: value,
            depth: current_depth,
        },
    )?;
    while let Some(task) = tasks.pop() {
        let next_depth = encode_expression_depth(task.depth)?;
        encoder.expression_node()?;
        match task.expression {
            KExpr::BytesLiteral(value) => {
                encoder.u8(0x00)?;
                encoder.blob("BytesLiteral", value)?;
            }
            KExpr::TermLiteral(value) => {
                encoder.u8(0x01)?;
                encode_term(encoder, value, 0)?;
            }
            KExpr::Var(index) => {
                encoder.u8(0x02)?;
                encoder.u32(*index)?;
            }
            KExpr::MakeAtom {
                kind,
                payload,
                equality,
            } => {
                encoder.u8(0x03)?;
                reserve_encode_expressions(&mut tasks, 3)?;
                tasks.push(EncodeExpressionTask {
                    expression: equality,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: payload,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: kind,
                    depth: next_depth,
                });
            }
            KExpr::MakeTriple {
                first,
                second,
                third,
            } => {
                encoder.u8(0x04)?;
                reserve_encode_expressions(&mut tasks, 3)?;
                tasks.push(EncodeExpressionTask {
                    expression: third,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: second,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: first,
                    depth: next_depth,
                });
            }
            KExpr::Let { value, body } => {
                encoder.u8(0x05)?;
                reserve_encode_expressions(&mut tasks, 2)?;
                tasks.push(EncodeExpressionTask {
                    expression: body,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: value,
                    depth: next_depth,
                });
            }
            KExpr::CaseTerm {
                scrutinee,
                atom_body,
                triple_body,
            } => {
                encoder.u8(0x06)?;
                reserve_encode_expressions(&mut tasks, 3)?;
                tasks.push(EncodeExpressionTask {
                    expression: triple_body,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: atom_body,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: scrutinee,
                    depth: next_depth,
                });
            }
            KExpr::CaseBytes {
                scrutinee,
                empty_body,
                cons_body,
            } => {
                encoder.u8(0x07)?;
                reserve_encode_expressions(&mut tasks, 3)?;
                tasks.push(EncodeExpressionTask {
                    expression: cons_body,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: empty_body,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: scrutinee,
                    depth: next_depth,
                });
            }
            KExpr::ConcatBytes(parts) => {
                encoder.u8(0x08)?;
                encoder.length("ConcatBytes parts", parts.len())?;
                if parts.len() > MAX_WIRE_ITEMS {
                    return Err(EncodeError::ResourceExhausted);
                }
                encoder.items(parts.len())?;
                reserve_encode_expressions(&mut tasks, parts.len())?;
                for part in parts.iter().rev() {
                    tasks.push(EncodeExpressionTask {
                        expression: part,
                        depth: next_depth,
                    });
                }
            }
            KExpr::CaseBytesEqual {
                left,
                right,
                equal_body,
                unequal_body,
            } => {
                encoder.u8(0x09)?;
                reserve_encode_expressions(&mut tasks, 4)?;
                tasks.push(EncodeExpressionTask {
                    expression: unequal_body,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: equal_body,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: right,
                    depth: next_depth,
                });
                tasks.push(EncodeExpressionTask {
                    expression: left,
                    depth: next_depth,
                });
            }
            KExpr::Call {
                definition_id,
                arguments,
            } => {
                encoder.u8(0x0a)?;
                encoder.fixed(definition_id.as_bytes())?;
                encoder.length("Call arguments", arguments.len())?;
                if arguments.len() > MAX_WIRE_ITEMS {
                    return Err(EncodeError::ResourceExhausted);
                }
                encoder.items(arguments.len())?;
                reserve_encode_expressions(&mut tasks, arguments.len())?;
                for argument in arguments.iter().rev() {
                    tasks.push(EncodeExpressionTask {
                        expression: argument,
                        depth: next_depth,
                    });
                }
            }
            KExpr::Request {
                physical_operation_id,
                arguments,
            } => {
                encoder.u8(0x0b)?;
                encoder.fixed(physical_operation_id.as_bytes())?;
                encoder.length("Request arguments", arguments.len())?;
                if arguments.len() > MAX_WIRE_ITEMS {
                    return Err(EncodeError::ResourceExhausted);
                }
                encoder.items(arguments.len())?;
                reserve_encode_expressions(&mut tasks, arguments.len())?;
                for argument in arguments.iter().rev() {
                    tasks.push(EncodeExpressionTask {
                        expression: argument,
                        depth: next_depth,
                    });
                }
            }
        }
    }
    Ok(())
}

fn reserve_encode_expressions(
    tasks: &mut Vec<EncodeExpressionTask<'_>>,
    additional: usize,
) -> Result<(), EncodeError> {
    let final_len = tasks
        .len()
        .checked_add(additional)
        .ok_or(EncodeError::ResourceExhausted)?;
    if final_len > MAX_EXPRESSION_NODES {
        return Err(EncodeError::ResourceExhausted);
    }
    tasks
        .try_reserve(additional)
        .map_err(|_| EncodeError::ResourceExhausted)
}

fn push_encode_expression<'a>(
    tasks: &mut Vec<EncodeExpressionTask<'a>>,
    task: EncodeExpressionTask<'a>,
) -> Result<(), EncodeError> {
    reserve_encode_expressions(tasks, 1)?;
    tasks.push(task);
    Ok(())
}

fn encode_term_depth(current_depth: usize) -> Result<usize, EncodeError> {
    if current_depth >= MAX_TERM_DEPTH {
        Err(EncodeError::ResourceExhausted)
    } else {
        Ok(current_depth + 1)
    }
}

fn encode_expression_depth(current_depth: usize) -> Result<usize, EncodeError> {
    if current_depth >= MAX_EXPRESSION_DEPTH {
        Err(EncodeError::ResourceExhausted)
    } else {
        Ok(current_depth + 1)
    }
}

fn encode_subject_value(
    subject: &CompilerSubject,
    budget: &mut EncodeBudget,
) -> Result<Vec<u8>, EncodeError> {
    let mut encoder = Encoder::new(budget)?;
    match &subject.lineage {
        CompilerLineage::Genesis => encoder.u8(0x00)?,
        CompilerLineage::Successor {
            predecessor_locator,
            change_occurrence_id,
        } => {
            encoder.u8(0x01)?;
            encoder.fixed(predecessor_locator.as_bytes())?;
            encoder.fixed(change_occurrence_id.as_bytes())?;
        }
    }
    encoder.sequence(
        "nominal declarations",
        &subject.nominal_declarations,
        encode_nominal_declaration,
    )?;
    encoder.fixed(subject.interface.compile.as_bytes())?;
    encoder.fixed(subject.interface.admit_propose.as_bytes())?;
    encoder.sequence("definitions", &subject.program, encode_definition)?;
    encode_term(&mut encoder, &subject.build_request, 0)?;
    Ok(encoder.bytes)
}

fn encode_nominal_declaration(
    encoder: &mut Encoder<'_>,
    declaration: &NominalDeclaration,
) -> Result<(), EncodeError> {
    match declaration {
        NominalDeclaration::Seed { domain, id } => {
            encoder.u8(0x00)?;
            encoder.fixed(domain.as_bytes())?;
            encoder.fixed(id.as_bytes())?;
        }
        NominalDeclaration::RetainedSeed {
            domain,
            id,
            predecessor_revision_id,
        } => {
            encoder.u8(0x01)?;
            encoder.fixed(domain.as_bytes())?;
            encoder.fixed(id.as_bytes())?;
            encoder.fixed(predecessor_revision_id.as_bytes())?;
        }
        NominalDeclaration::Allocated {
            domain,
            id,
            change_input,
            producer_input,
            local_slot,
        } => {
            encoder.u8(0x02)?;
            encoder.fixed(domain.as_bytes())?;
            encoder.fixed(id.as_bytes())?;
            encode_nominal_ref(encoder, change_input)?;
            encode_nominal_ref(encoder, producer_input)?;
            encoder.u64(*local_slot)?;
        }
    }
    Ok(())
}

fn encode_nominal_ref(
    encoder: &mut Encoder<'_>,
    reference: &NominalWireRef,
) -> Result<(), EncodeError> {
    encoder.fixed(reference.domain.as_bytes())?;
    encoder.fixed(reference.id.as_bytes())
}

fn encode_definition(
    encoder: &mut Encoder<'_>,
    definition: &Definition,
) -> Result<(), EncodeError> {
    encoder.fixed(definition.id.as_bytes())?;
    encoder.sequence(
        "definition arguments",
        &definition.arguments,
        |encoder, sort| encode_sort(encoder, *sort),
    )?;
    encode_sort(encoder, definition.result)?;
    encode_expr(encoder, &definition.body, 0)
}

fn encode_evidence_value(
    evidence: &CompilerEvidence,
    budget: &mut EncodeBudget,
) -> Result<Vec<u8>, EncodeError> {
    let mut encoder = Encoder::new(budget)?;
    match evidence {
        CompilerEvidence::Genesis => encoder.u8(0x00)?,
        CompilerEvidence::Successor {
            compile_certificate,
            admission_certificate,
        } => {
            encoder.u8(0x01)?;
            encode_certificate(&mut encoder, compile_certificate)?;
            encode_certificate(&mut encoder, admission_certificate)?;
        }
    }
    Ok(encoder.bytes)
}

fn encode_certificate(
    encoder: &mut Encoder<'_>,
    certificate: &EvalCertificate,
) -> Result<(), EncodeError> {
    if certificate.format_version != 0x00 {
        return Err(EncodeError::InvalidClosedTag {
            field: "certificate format version",
            tag: certificate.format_version,
        });
    }
    encoder.u8(certificate.format_version)?;
    encode_statement(encoder, &certificate.statement)?;
    encoder.sequence("evaluation nodes", &certificate.nodes, |encoder, node| {
        if !(0x30..=0x3e).contains(&node.rule_tag) {
            return Err(EncodeError::InvalidClosedTag {
                field: "evaluation rule",
                tag: node.rule_tag,
            });
        }
        encoder.u8(node.rule_tag)?;
        encoder.sequence("node premises", &node.premises, |encoder, premise| {
            encoder.u32(*premise)
        })?;
        encode_judgment(encoder, &node.conclusion)
    })
}

fn encode_statement(
    encoder: &mut Encoder<'_>,
    statement: &EvalStatement,
) -> Result<(), EncodeError> {
    encoder.blob(
        "accepted predecessor",
        &statement.exact_accepted_predecessor,
    )?;
    encoder.fixed(statement.core_contract_id.as_bytes())?;
    encoder.fixed(statement.physical_profile_id.as_bytes())?;
    encoder.fixed(statement.entrypoint.as_bytes())?;
    encoder.sequence("statement arguments", &statement.arguments, encode_value)?;
    encoder.u64(statement.fuel_limit)?;
    encode_outcome(encoder, &statement.expected)
}

fn encode_value(encoder: &mut Encoder<'_>, value: &KValue) -> Result<(), EncodeError> {
    match value {
        KValue::Bytes(bytes) => {
            encoder.u8(0x00)?;
            encoder.blob("BytesValue", bytes)
        }
        KValue::Term(term) => {
            encoder.u8(0x01)?;
            encode_term(encoder, term, 0)
        }
    }
}

fn encode_outcome(encoder: &mut Encoder<'_>, outcome: &EvalOutcome) -> Result<(), EncodeError> {
    match outcome {
        EvalOutcome::Returned {
            value,
            remaining_fuel,
            observations,
        } => {
            encoder.u8(0x00)?;
            encode_value(encoder, value)?;
            encoder.u64(*remaining_fuel)?;
            encode_term(encoder, observations, 0)
        }
    }
}

fn encode_judgment(encoder: &mut Encoder<'_>, judgment: &EvalJudgment) -> Result<(), EncodeError> {
    encode_expr(encoder, &judgment.expression, 0)?;
    encoder.sequence("judgment environment", &judgment.environment, encode_value)?;
    encoder.u64(judgment.fuel_before)?;
    encode_term(encoder, &judgment.observations_before, 0)?;
    encode_value(encoder, &judgment.value)?;
    encoder.u64(judgment.fuel_after)?;
    encode_term(encoder, &judgment.observations_after, 0)
}

#[derive(Clone, Copy)]
struct DecodeBudget {
    items: usize,
    term_nodes: usize,
    expression_nodes: usize,
}

impl DecodeBudget {
    const fn new() -> Self {
        Self {
            items: 0,
            term_nodes: 0,
            expression_nodes: 0,
        }
    }

    fn item(&mut self) -> Result<(), DecodeFailure> {
        self.items = self
            .items
            .checked_add(1)
            .ok_or(DecodeFailure::ResourceExhausted)?;
        if self.items > MAX_WIRE_ITEMS {
            Err(DecodeFailure::ResourceExhausted)
        } else {
            Ok(())
        }
    }

    fn term_node(&mut self) -> Result<(), DecodeFailure> {
        self.term_nodes = self
            .term_nodes
            .checked_add(1)
            .ok_or(DecodeFailure::ResourceExhausted)?;
        if self.term_nodes > MAX_TERM_NODES {
            Err(DecodeFailure::ResourceExhausted)
        } else {
            Ok(())
        }
    }

    fn expression_node(&mut self) -> Result<(), DecodeFailure> {
        self.expression_nodes = self
            .expression_nodes
            .checked_add(1)
            .ok_or(DecodeFailure::ResourceExhausted)?;
        if self.expression_nodes > MAX_EXPRESSION_NODES {
            Err(DecodeFailure::ResourceExhausted)
        } else {
            Ok(())
        }
    }
}

struct Cursor<'a> {
    input: &'a [u8],
    offset: usize,
    limit: usize,
    budget: DecodeBudget,
}

impl<'a> Cursor<'a> {
    fn top(input: &'a [u8]) -> Self {
        Self {
            input,
            offset: 0,
            limit: input.len(),
            budget: DecodeBudget::new(),
        }
    }

    fn rejection(&self, code: DecodeCode, offset: usize) -> DecodeFailure {
        DecodeFailure::Rejected(DecodeRejection {
            code,
            offset: u64::try_from(offset).expect("slice offset fits U64"),
        })
    }

    fn read(&mut self, length: usize) -> Result<&'a [u8], DecodeFailure> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| self.rejection(DecodeCode::LengthOrCountOverflow, self.offset))?;
        if end > self.limit {
            let code = if end <= self.input.len() {
                DecodeCode::BoundedValueOverConsumed
            } else {
                DecodeCode::Truncated
            };
            let offset = if code == DecodeCode::Truncated {
                self.input.len()
            } else {
                self.limit
            };
            return Err(self.rejection(code, offset));
        }
        let value = &self.input[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, DecodeFailure> {
        Ok(self.read(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, DecodeFailure> {
        let bytes: [u8; 4] = self
            .read(4)?
            .try_into()
            .expect("read returned exact U32 width");
        Ok(u32::from_be_bytes(bytes))
    }

    fn u64(&mut self) -> Result<u64, DecodeFailure> {
        let bytes: [u8; 8] = self
            .read(8)?
            .try_into()
            .expect("read returned exact U64 width");
        Ok(u64::from_be_bytes(bytes))
    }

    fn id32(&mut self) -> Result<Id32, DecodeFailure> {
        let bytes: [u8; 32] = self
            .read(32)?
            .try_into()
            .expect("read returned exact Id32 width");
        Ok(Id32(bytes))
    }

    fn hash32(&mut self) -> Result<Hash32, DecodeFailure> {
        let bytes: [u8; 32] = self
            .read(32)?
            .try_into()
            .expect("read returned exact Hash32 width");
        Ok(Hash32(bytes))
    }

    fn blob(&mut self) -> Result<Vec<u8>, DecodeFailure> {
        let length_offset = self.offset;
        let length = usize::try_from(self.u32()?)
            .map_err(|_| self.rejection(DecodeCode::LengthOrCountOverflow, length_offset))?;
        let source = self.read(length)?;
        let mut value = Vec::new();
        value
            .try_reserve_exact(length)
            .map_err(|_| DecodeFailure::ResourceExhausted)?;
        value.extend_from_slice(source);
        Ok(value)
    }

    fn sequence<T>(
        &mut self,
        mut decode: impl FnMut(&mut Self) -> Result<T, DecodeFailure>,
    ) -> Result<Vec<T>, DecodeFailure> {
        let count = self.sequence_count()?;
        let mut values = Vec::new();
        for _ in 0..count {
            let value = decode(self)?;
            self.budget.item()?;
            values
                .try_reserve(1)
                .map_err(|_| DecodeFailure::ResourceExhausted)?;
            values.push(value);
        }
        Ok(values)
    }

    fn sequence_count(&mut self) -> Result<usize, DecodeFailure> {
        let count_offset = self.offset;
        usize::try_from(self.u32()?)
            .map_err(|_| self.rejection(DecodeCode::LengthOrCountOverflow, count_offset))
    }

    fn frame<T>(
        &mut self,
        expected_tag: u8,
        decode: impl FnOnce(&mut Cursor<'a>) -> Result<T, DecodeFailure>,
    ) -> Result<(T, std::ops::Range<usize>), DecodeFailure> {
        let tag_offset = self.offset;
        let tag = self.u8()?;
        if tag != expected_tag {
            return Err(self.rejection(DecodeCode::FrameTagOrderOrCount, tag_offset));
        }
        let length_offset = self.offset;
        let length = usize::try_from(self.u32()?)
            .map_err(|_| self.rejection(DecodeCode::LengthOrCountOverflow, length_offset))?;
        let start = self.offset;
        let end = start
            .checked_add(length)
            .ok_or_else(|| self.rejection(DecodeCode::LengthOrCountOverflow, length_offset))?;
        if end > self.limit {
            return Err(self.rejection(DecodeCode::Truncated, self.limit));
        }
        let mut bounded = Cursor {
            input: self.input,
            offset: start,
            limit: end,
            budget: self.budget,
        };
        let value = decode(&mut bounded)?;
        if bounded.offset != end {
            return Err(self.rejection(DecodeCode::BoundedValueUnderConsumed, end));
        }
        self.budget = bounded.budget;
        self.offset = end;
        Ok((value, start..end))
    }
}

pub fn decode(input: &[u8]) -> Result<DecodedCompilerPackage, DecodeFailure> {
    if input.len() > MAX_WIRE_BYTES {
        return Err(DecodeFailure::ResourceExhausted);
    }
    let mut cursor = Cursor::top(input);
    for expected in MAGIC {
        let offset = cursor.offset;
        let found = cursor.u8()?;
        if found != *expected {
            return Err(cursor.rejection(DecodeCode::WrongMagic, offset));
        }
    }
    let version_offset = cursor.offset;
    let version = cursor.u8()?;
    if version != VERSION {
        return Err(cursor.rejection(DecodeCode::UnknownVersion, version_offset));
    }
    let (core_manifest, exact_core_manifest) = cursor.frame(0x01, decode_core_manifest_value)?;
    let (subject, exact_subject) = cursor.frame(0x02, decode_subject_value)?;
    let (evidence, exact_evidence) = cursor.frame(0x03, decode_evidence_value)?;
    if cursor.offset != cursor.limit {
        return Err(cursor.rejection(DecodeCode::TrailingBytes, cursor.offset));
    }
    let mut exact_input = Vec::new();
    exact_input
        .try_reserve_exact(input.len())
        .map_err(|_| DecodeFailure::ResourceExhausted)?;
    exact_input.extend_from_slice(input);
    Ok(DecodedCompilerPackage::new(
        exact_input,
        exact_core_manifest,
        exact_subject,
        exact_evidence,
        CompilerPackage {
            core_manifest,
            subject,
            evidence,
        },
    ))
}

fn decode_core_manifest_value(cursor: &mut Cursor<'_>) -> Result<CoreManifest, DecodeFailure> {
    Ok(CoreManifest {
        manifest_version: cursor.u8()?,
        frame_tags: cursor.sequence(|cursor| cursor.u8())?,
        term_tags: cursor.sequence(|cursor| cursor.u8())?,
        sort_tags: cursor.sequence(|cursor| cursor.u8())?,
        expression_forms: cursor.sequence(decode_named_signature)?,
        abi_forms: cursor.sequence(decode_named_signature)?,
        premise_policy_tags: cursor.sequence(|cursor| cursor.u8())?,
        lineage_tags: cursor.sequence(|cursor| cursor.u8())?,
        nominal_declaration_tags: cursor.sequence(|cursor| cursor.u8())?,
        compiler_evidence_tags: cursor.sequence(|cursor| cursor.u8())?,
        value_tags: cursor.sequence(|cursor| cursor.u8())?,
        eval_outcome_tags: cursor.sequence(|cursor| cursor.u8())?,
        decode_verdict_tags: cursor.sequence(|cursor| cursor.u8())?,
        decode_code_tags: cursor.sequence(|cursor| cursor.u8())?,
        authorization_stage_tags: cursor.sequence(|cursor| cursor.u8())?,
        authorization_code_tags: cursor.sequence(|cursor| cursor.u8())?,
        static_rules: cursor.sequence(decode_rule_signature)?,
        evaluation_rules: cursor.sequence(decode_rule_signature)?,
        certificate_format_version: cursor.u8()?,
        certificate_signature: cursor.blob()?,
        contract_clauses: cursor.sequence(|cursor| cursor.blob())?,
        physical_profile: decode_physical_profile(cursor)?,
    })
}

fn decode_named_signature(cursor: &mut Cursor<'_>) -> Result<NamedSignature, DecodeFailure> {
    Ok(NamedSignature {
        tag: cursor.u8()?,
        signature: cursor.blob()?,
    })
}

fn decode_rule_signature(cursor: &mut Cursor<'_>) -> Result<RuleSignature, DecodeFailure> {
    Ok(RuleSignature {
        tag: cursor.u8()?,
        premise_policy: cursor.u8()?,
        clause: cursor.blob()?,
    })
}

fn decode_physical_profile(cursor: &mut Cursor<'_>) -> Result<PhysicalProfile, DecodeFailure> {
    Ok(PhysicalProfile {
        profile_version: cursor.u8()?,
        observation_policy: cursor.u8()?,
        operations: cursor.sequence(|cursor| {
            Ok(PhysicalOperation {
                operation_id: cursor.id32()?,
                arguments: cursor.sequence(decode_sort)?,
                result: decode_sort(cursor)?,
            })
        })?,
    })
}

fn unknown(cursor: &Cursor<'_>, offset: usize) -> DecodeFailure {
    cursor.rejection(DecodeCode::UnknownSumTag, offset)
}

fn decode_sort(cursor: &mut Cursor<'_>) -> Result<KSort, DecodeFailure> {
    let offset = cursor.offset;
    match cursor.u8()? {
        0x00 => Ok(KSort::Bytes),
        0x01 => Ok(KSort::Term),
        _ => Err(unknown(cursor, offset)),
    }
}

fn term_depth(next: usize) -> Result<usize, DecodeFailure> {
    if next >= MAX_TERM_DEPTH {
        Err(DecodeFailure::ResourceExhausted)
    } else {
        Ok(next + 1)
    }
}

fn expression_depth(next: usize) -> Result<usize, DecodeFailure> {
    if next >= MAX_EXPRESSION_DEPTH {
        Err(DecodeFailure::ResourceExhausted)
    } else {
        Ok(next + 1)
    }
}

fn decode_term(cursor: &mut Cursor<'_>, current_depth: usize) -> Result<Term, DecodeFailure> {
    let next_depth = term_depth(current_depth)?;
    cursor.budget.term_node()?;
    let offset = cursor.offset;
    match cursor.u8()? {
        0x00 => Ok(Term::Atom {
            kind: cursor.blob()?,
            canonical_payload: cursor.blob()?,
            equality_contract: cursor.blob()?,
        }),
        0x01 => Ok(Term::Triple(
            Box::new(decode_term(cursor, next_depth)?),
            Box::new(decode_term(cursor, next_depth)?),
            Box::new(decode_term(cursor, next_depth)?),
        )),
        _ => Err(unknown(cursor, offset)),
    }
}

enum FixedExpression {
    MakeAtom,
    MakeTriple,
    Let,
    CaseTerm,
    CaseBytes,
    CaseBytesEqual,
}

enum SequenceExpression {
    ConcatBytes,
    Call(Id32),
    Request(Id32),
}

enum DecodeExpressionTask {
    Read {
        depth: usize,
    },
    BuildFixed(FixedExpression),
    Sequence {
        kind: SequenceExpression,
        remaining: usize,
        depth: usize,
        values: Vec<KExpr>,
    },
    SequenceItem {
        kind: SequenceExpression,
        remaining: usize,
        depth: usize,
        values: Vec<KExpr>,
    },
}

fn decode_expr(cursor: &mut Cursor<'_>, current_depth: usize) -> Result<KExpr, DecodeFailure> {
    let mut tasks = Vec::new();
    push_decode_expression(
        &mut tasks,
        DecodeExpressionTask::Read {
            depth: current_depth,
        },
    )?;
    let mut results = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            DecodeExpressionTask::Read { depth } => {
                let next_depth = expression_depth(depth)?;
                cursor.budget.expression_node()?;
                let offset = cursor.offset;
                match cursor.u8()? {
                    0x00 => {
                        push_decoded_expression(&mut results, KExpr::BytesLiteral(cursor.blob()?))?
                    }
                    0x01 => push_decoded_expression(
                        &mut results,
                        KExpr::TermLiteral(decode_term(cursor, 0)?),
                    )?,
                    0x02 => {
                        push_decoded_expression(&mut results, KExpr::Var(cursor.u32()?))?;
                    }
                    0x03 => schedule_fixed_expression(
                        &mut tasks,
                        FixedExpression::MakeAtom,
                        3,
                        next_depth,
                    )?,
                    0x04 => schedule_fixed_expression(
                        &mut tasks,
                        FixedExpression::MakeTriple,
                        3,
                        next_depth,
                    )?,
                    0x05 => {
                        schedule_fixed_expression(&mut tasks, FixedExpression::Let, 2, next_depth)?
                    }
                    0x06 => schedule_fixed_expression(
                        &mut tasks,
                        FixedExpression::CaseTerm,
                        3,
                        next_depth,
                    )?,
                    0x07 => schedule_fixed_expression(
                        &mut tasks,
                        FixedExpression::CaseBytes,
                        3,
                        next_depth,
                    )?,
                    0x08 => push_decode_expression(
                        &mut tasks,
                        DecodeExpressionTask::Sequence {
                            kind: SequenceExpression::ConcatBytes,
                            remaining: cursor.sequence_count()?,
                            depth: next_depth,
                            values: Vec::new(),
                        },
                    )?,
                    0x09 => schedule_fixed_expression(
                        &mut tasks,
                        FixedExpression::CaseBytesEqual,
                        4,
                        next_depth,
                    )?,
                    0x0a => {
                        let definition_id = cursor.id32()?;
                        push_decode_expression(
                            &mut tasks,
                            DecodeExpressionTask::Sequence {
                                kind: SequenceExpression::Call(definition_id),
                                remaining: cursor.sequence_count()?,
                                depth: next_depth,
                                values: Vec::new(),
                            },
                        )?;
                    }
                    0x0b => {
                        let physical_operation_id = cursor.id32()?;
                        push_decode_expression(
                            &mut tasks,
                            DecodeExpressionTask::Sequence {
                                kind: SequenceExpression::Request(physical_operation_id),
                                remaining: cursor.sequence_count()?,
                                depth: next_depth,
                                values: Vec::new(),
                            },
                        )?;
                    }
                    _ => return Err(unknown(cursor, offset)),
                }
            }
            DecodeExpressionTask::BuildFixed(kind) => {
                let expression = match kind {
                    FixedExpression::MakeAtom => {
                        let equality = pop_decoded_expression(&mut results)?;
                        let payload = pop_decoded_expression(&mut results)?;
                        let kind = pop_decoded_expression(&mut results)?;
                        KExpr::MakeAtom {
                            kind: Box::new(kind),
                            payload: Box::new(payload),
                            equality: Box::new(equality),
                        }
                    }
                    FixedExpression::MakeTriple => {
                        let third = pop_decoded_expression(&mut results)?;
                        let second = pop_decoded_expression(&mut results)?;
                        let first = pop_decoded_expression(&mut results)?;
                        KExpr::MakeTriple {
                            first: Box::new(first),
                            second: Box::new(second),
                            third: Box::new(third),
                        }
                    }
                    FixedExpression::Let => {
                        let body = pop_decoded_expression(&mut results)?;
                        let value = pop_decoded_expression(&mut results)?;
                        KExpr::Let {
                            value: Box::new(value),
                            body: Box::new(body),
                        }
                    }
                    FixedExpression::CaseTerm => {
                        let triple_body = pop_decoded_expression(&mut results)?;
                        let atom_body = pop_decoded_expression(&mut results)?;
                        let scrutinee = pop_decoded_expression(&mut results)?;
                        KExpr::CaseTerm {
                            scrutinee: Box::new(scrutinee),
                            atom_body: Box::new(atom_body),
                            triple_body: Box::new(triple_body),
                        }
                    }
                    FixedExpression::CaseBytes => {
                        let cons_body = pop_decoded_expression(&mut results)?;
                        let empty_body = pop_decoded_expression(&mut results)?;
                        let scrutinee = pop_decoded_expression(&mut results)?;
                        KExpr::CaseBytes {
                            scrutinee: Box::new(scrutinee),
                            empty_body: Box::new(empty_body),
                            cons_body: Box::new(cons_body),
                        }
                    }
                    FixedExpression::CaseBytesEqual => {
                        let unequal_body = pop_decoded_expression(&mut results)?;
                        let equal_body = pop_decoded_expression(&mut results)?;
                        let right = pop_decoded_expression(&mut results)?;
                        let left = pop_decoded_expression(&mut results)?;
                        KExpr::CaseBytesEqual {
                            left: Box::new(left),
                            right: Box::new(right),
                            equal_body: Box::new(equal_body),
                            unequal_body: Box::new(unequal_body),
                        }
                    }
                };
                push_decoded_expression(&mut results, expression)?;
            }
            DecodeExpressionTask::Sequence {
                kind,
                remaining,
                depth,
                values,
            } => {
                if remaining == 0 {
                    let expression = match kind {
                        SequenceExpression::ConcatBytes => KExpr::ConcatBytes(values),
                        SequenceExpression::Call(definition_id) => KExpr::Call {
                            definition_id,
                            arguments: values,
                        },
                        SequenceExpression::Request(physical_operation_id) => KExpr::Request {
                            physical_operation_id,
                            arguments: values,
                        },
                    };
                    push_decoded_expression(&mut results, expression)?;
                } else {
                    reserve_decode_expressions(&mut tasks, 2)?;
                    tasks.push(DecodeExpressionTask::SequenceItem {
                        kind,
                        remaining,
                        depth,
                        values,
                    });
                    tasks.push(DecodeExpressionTask::Read { depth });
                }
            }
            DecodeExpressionTask::SequenceItem {
                kind,
                remaining,
                depth,
                mut values,
            } => {
                let value = pop_decoded_expression(&mut results)?;
                cursor.budget.item()?;
                values
                    .try_reserve(1)
                    .map_err(|_| DecodeFailure::ResourceExhausted)?;
                values.push(value);
                push_decode_expression(
                    &mut tasks,
                    DecodeExpressionTask::Sequence {
                        kind,
                        remaining: remaining - 1,
                        depth,
                        values,
                    },
                )?;
            }
        }
    }
    if results.len() != 1 {
        return Err(DecodeFailure::ResourceExhausted);
    }
    pop_decoded_expression(&mut results)
}

fn schedule_fixed_expression(
    tasks: &mut Vec<DecodeExpressionTask>,
    kind: FixedExpression,
    children: usize,
    depth: usize,
) -> Result<(), DecodeFailure> {
    reserve_decode_expressions(tasks, children + 1)?;
    tasks.push(DecodeExpressionTask::BuildFixed(kind));
    for _ in 0..children {
        tasks.push(DecodeExpressionTask::Read { depth });
    }
    Ok(())
}

fn reserve_decode_expressions(
    tasks: &mut Vec<DecodeExpressionTask>,
    additional: usize,
) -> Result<(), DecodeFailure> {
    let final_len = tasks
        .len()
        .checked_add(additional)
        .ok_or(DecodeFailure::ResourceExhausted)?;
    if final_len > MAX_EXPRESSION_NODES {
        return Err(DecodeFailure::ResourceExhausted);
    }
    tasks
        .try_reserve(additional)
        .map_err(|_| DecodeFailure::ResourceExhausted)
}

fn push_decode_expression(
    tasks: &mut Vec<DecodeExpressionTask>,
    task: DecodeExpressionTask,
) -> Result<(), DecodeFailure> {
    reserve_decode_expressions(tasks, 1)?;
    tasks.push(task);
    Ok(())
}

fn push_decoded_expression(
    results: &mut Vec<KExpr>,
    expression: KExpr,
) -> Result<(), DecodeFailure> {
    results
        .try_reserve(1)
        .map_err(|_| DecodeFailure::ResourceExhausted)?;
    results.push(expression);
    Ok(())
}

fn pop_decoded_expression(results: &mut Vec<KExpr>) -> Result<KExpr, DecodeFailure> {
    results.pop().ok_or(DecodeFailure::ResourceExhausted)
}

fn decode_subject_value(cursor: &mut Cursor<'_>) -> Result<CompilerSubject, DecodeFailure> {
    let lineage_offset = cursor.offset;
    let lineage = match cursor.u8()? {
        0x00 => CompilerLineage::Genesis,
        0x01 => CompilerLineage::Successor {
            predecessor_locator: cursor.hash32()?,
            change_occurrence_id: cursor.id32()?,
        },
        _ => return Err(unknown(cursor, lineage_offset)),
    };
    let nominal_declarations = cursor.sequence(decode_nominal_declaration)?;
    let interface = CompilerInterface {
        compile: cursor.id32()?,
        admit_propose: cursor.id32()?,
    };
    let program = cursor.sequence(decode_definition)?;
    let build_request = decode_term(cursor, 0)?;
    Ok(CompilerSubject {
        lineage,
        nominal_declarations,
        interface,
        program,
        build_request,
    })
}

fn decode_nominal_declaration(
    cursor: &mut Cursor<'_>,
) -> Result<NominalDeclaration, DecodeFailure> {
    let offset = cursor.offset;
    match cursor.u8()? {
        0x00 => Ok(NominalDeclaration::Seed {
            domain: cursor.id32()?,
            id: cursor.id32()?,
        }),
        0x01 => Ok(NominalDeclaration::RetainedSeed {
            domain: cursor.id32()?,
            id: cursor.id32()?,
            predecessor_revision_id: cursor.id32()?,
        }),
        0x02 => Ok(NominalDeclaration::Allocated {
            domain: cursor.id32()?,
            id: cursor.id32()?,
            change_input: decode_nominal_ref(cursor)?,
            producer_input: decode_nominal_ref(cursor)?,
            local_slot: cursor.u64()?,
        }),
        _ => Err(unknown(cursor, offset)),
    }
}

fn decode_nominal_ref(cursor: &mut Cursor<'_>) -> Result<NominalWireRef, DecodeFailure> {
    Ok(NominalWireRef {
        domain: cursor.id32()?,
        id: cursor.id32()?,
    })
}

fn decode_definition(cursor: &mut Cursor<'_>) -> Result<Definition, DecodeFailure> {
    Ok(Definition {
        id: cursor.id32()?,
        arguments: cursor.sequence(decode_sort)?,
        result: decode_sort(cursor)?,
        body: decode_expr(cursor, 0)?,
    })
}

fn decode_evidence_value(cursor: &mut Cursor<'_>) -> Result<CompilerEvidence, DecodeFailure> {
    let offset = cursor.offset;
    match cursor.u8()? {
        0x00 => Ok(CompilerEvidence::Genesis),
        0x01 => Ok(CompilerEvidence::Successor {
            compile_certificate: Box::new(decode_certificate(cursor)?),
            admission_certificate: Box::new(decode_certificate(cursor)?),
        }),
        _ => Err(unknown(cursor, offset)),
    }
}

fn decode_certificate(cursor: &mut Cursor<'_>) -> Result<EvalCertificate, DecodeFailure> {
    let format_offset = cursor.offset;
    let format_version = cursor.u8()?;
    if format_version != 0x00 {
        return Err(unknown(cursor, format_offset));
    }
    Ok(EvalCertificate {
        format_version,
        statement: decode_statement(cursor)?,
        nodes: cursor.sequence(decode_node)?,
    })
}

fn decode_statement(cursor: &mut Cursor<'_>) -> Result<EvalStatement, DecodeFailure> {
    Ok(EvalStatement {
        exact_accepted_predecessor: cursor.blob()?,
        core_contract_id: cursor.hash32()?,
        physical_profile_id: cursor.hash32()?,
        entrypoint: cursor.id32()?,
        arguments: cursor.sequence(decode_value)?,
        fuel_limit: cursor.u64()?,
        expected: decode_outcome(cursor)?,
    })
}

fn decode_value(cursor: &mut Cursor<'_>) -> Result<KValue, DecodeFailure> {
    let offset = cursor.offset;
    match cursor.u8()? {
        0x00 => Ok(KValue::Bytes(cursor.blob()?)),
        0x01 => Ok(KValue::Term(decode_term(cursor, 0)?)),
        _ => Err(unknown(cursor, offset)),
    }
}

fn decode_outcome(cursor: &mut Cursor<'_>) -> Result<EvalOutcome, DecodeFailure> {
    let offset = cursor.offset;
    match cursor.u8()? {
        0x00 => Ok(EvalOutcome::Returned {
            value: decode_value(cursor)?,
            remaining_fuel: cursor.u64()?,
            observations: decode_term(cursor, 0)?,
        }),
        _ => Err(unknown(cursor, offset)),
    }
}

fn decode_node(cursor: &mut Cursor<'_>) -> Result<EvalNode, DecodeFailure> {
    let rule_offset = cursor.offset;
    let rule_tag = cursor.u8()?;
    if !(0x30..=0x3e).contains(&rule_tag) {
        return Err(unknown(cursor, rule_offset));
    }
    Ok(EvalNode {
        rule_tag,
        premises: cursor.sequence(|cursor| cursor.u32())?,
        conclusion: decode_judgment(cursor)?,
    })
}

fn decode_judgment(cursor: &mut Cursor<'_>) -> Result<EvalJudgment, DecodeFailure> {
    Ok(EvalJudgment {
        expression: decode_expr(cursor, 0)?,
        environment: cursor.sequence(decode_value)?,
        fuel_before: cursor.u64()?,
        observations_before: decode_term(cursor, 0)?,
        value: decode_value(cursor)?,
        fuel_after: cursor.u64()?,
        observations_after: decode_term(cursor, 0)?,
    })
}
