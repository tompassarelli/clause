//! Direct ES module lowering for the checked finite relational handler slice.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::*;

/// A generated module and its exact exported TypeScript interface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JavaScriptArtifactsV1 {
    pub module: String,
    pub declarations: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JavaScriptLoweringErrorV1(pub String);

impl fmt::Display for JavaScriptLoweringErrorV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for JavaScriptLoweringErrorV1 {}

type Result<T> = std::result::Result<T, JavaScriptLoweringErrorV1>;
fn unsupported<T>(message: impl Into<String>) -> Result<T> {
    Err(JavaScriptLoweringErrorV1(message.into()))
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValueType {
    Sequence(Box<Self>),
    Record(BTreeMap<Vec<u8>, Self>),
    Number,
    Boolean,
    Text,
    Referent(u32),
}
impl ValueType {
    fn descriptor(&self) -> String {
        match self {
            Self::Sequence(element) => format!("[\"sequence\",{}]", element.descriptor()),
            Self::Record(fields) => format!("[\"record\",[{}]]", fields.iter().map(|(key, value)| format!("[{},{}]", quote(std::str::from_utf8(key).expect("checked field")), value.descriptor())).collect::<Vec<_>>().join(",")),
            Self::Number => "[\"number\"]".into(),
            Self::Boolean => "[\"boolean\"]".into(),
            Self::Text => "[\"text\"]".into(),
            Self::Referent(domain) => format!("[\"referent\",{domain}]"),
        }
    }
    fn declaration(&self) -> String {
        match self {
            Self::Sequence(element) => format!("ReadonlyArray<{}>", element.declaration()),
            Self::Record(fields) => format!("{{ {} }}", fields.iter().map(|(key,value)| format!("readonly {}: {}", quote(std::str::from_utf8(key).expect("checked field")), value.declaration())).collect::<Vec<_>>().join("; ")),
            Self::Number => "number".into(),
            Self::Boolean => "boolean".into(),
            Self::Text => "string".into(),
            Self::Referent(domain) => format!("Referent<{domain}>"),
        }
    }
}
fn relation_type(kind: CanonicalRelationValueKindV1) -> Result<ValueType> {
    match kind {
        CanonicalRelationValueKindV1::Number => Ok(ValueType::Number),
        CanonicalRelationValueKindV1::Boolean => Ok(ValueType::Boolean),
        CanonicalRelationValueKindV1::Text => Ok(ValueType::Text),
        CanonicalRelationValueKindV1::Referent(domain) => Ok(ValueType::Referent(domain.get())),
        CanonicalRelationValueKindV1::Symbol => {
            unsupported("JavaScript lowering does not support Symbol values")
        }
    }
}
fn scalar_type(kind: CanonicalScalarValueKindV1) -> Result<ValueType> {
    match kind {
        CanonicalScalarValueKindV1::Number => Ok(ValueType::Number),
        CanonicalScalarValueKindV1::Boolean => Ok(ValueType::Boolean),
        CanonicalScalarValueKindV1::Text => Ok(ValueType::Text),
        _ => unsupported("JavaScript callables require Text, Number, or Boolean types"),
    }
}
fn callable_type(kind: &CanonicalValueTypeV1) -> Result<ValueType> {
    match kind {
        CanonicalValueTypeV1::Delayed { .. } | CanonicalValueTypeV1::OpaqueForeign { .. } => unsupported("JavaScript does not execute delayed target construction"),
        CanonicalValueTypeV1::Scalar(kind) => scalar_type(*kind),
        CanonicalValueTypeV1::Sequence(element) => Ok(ValueType::Sequence(Box::new(callable_type(element)?))),
        CanonicalValueTypeV1::Record(fields) => Ok(ValueType::Record(fields.iter().map(|(k,v)| Ok((k.clone(), callable_type(v)?))).collect::<Result<_>>()?)),
    }
}
fn foreign_name(module: &str) -> String {
    format!("ffi{}", module.as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>())
}
fn foreign_modules(expression: &CanonicalExecutableExpressionV1, modules: &mut BTreeSet<String>) {
    use CanonicalExecutableExpressionV1 as E;
    match expression {
        E::Foreign { binding, arguments } => {
            modules.insert(binding.module.clone());
            for value in arguments { foreign_modules(value, modules); }
        }
        E::SequenceFold { source, initial, body, .. } => { foreign_modules(source, modules); foreign_modules(initial, modules); foreign_modules(body, modules); }
        E::Let { value, body, .. } | E::SequenceMap { source: value, body, .. } => { foreign_modules(value, modules); foreign_modules(body, modules); }
        E::Sequence(values) => { for value in values { foreign_modules(value, modules); } }
        E::Record(fields) => { for value in fields.values() { foreign_modules(value, modules); } }
        E::SequenceAppend(a,b) | E::SequenceJoin(a,b) | E::SequenceDrop(a,b) | E::Concatenate(a,b) | E::Equal(a,b) | E::GreaterThan(a,b) | E::LessThanOrEqual(a,b) | E::Add(a,b) | E::Subtract(a,b) | E::Multiply(a,b) | E::Divide(a,b) | E::ContainsText(a,b) | E::StartsWith(a,b) => { foreign_modules(a,modules); foreign_modules(b,modules); }
        E::Require(a,b,c) | E::Conditional(a,b,c) => { foreign_modules(a,modules); foreign_modules(b,modules); foreign_modules(c,modules); }
        E::SequenceSort(value) | E::SequenceCount(value) | E::ScalarText(value) | E::Field(value,_) | E::SquareRoot(value) | E::TextTransform(_,value) => foreign_modules(value,modules),
        _ => {}
    }
}
fn name(bytes: &[u8]) -> Result<&str> {
    std::str::from_utf8(bytes)
        .map_err(|_| JavaScriptLoweringErrorV1("non-UTF-8 designation".into()))
}
fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch < ' ' || matches!(ch, '\u{2028}' | '\u{2029}') => {
                use std::fmt::Write;
                write!(&mut result, "\\u{:04x}", ch as u32).unwrap();
            }
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}
fn referent(value: CanonicalReferentV1) -> String {
    format!(
        "Object.freeze({{domain:{},identity:{}}})",
        value.domain.get(),
        value.identity.get()
    )
}
fn constant(value: &CanonicalScalarValueV1) -> Result<(String, ValueType)> {
    match value {
        CanonicalScalarValueV1::Number(bits) => {
            let number = f64::from_bits(*bits);
            if !number.is_finite() {
                return unsupported("non-finite Number constant");
            }
            Ok((number.to_string(), ValueType::Number))
        }
        CanonicalScalarValueV1::Boolean(value) => Ok((value.to_string(), ValueType::Boolean)),
        CanonicalScalarValueV1::Text(value) => {
            if value.len() > MAX_ATOM_FIELD_BYTES {
                return unsupported("Text constant exceeds canonical size limit");
            }
            Ok((quote(value), ValueType::Text))
        }
        CanonicalScalarValueV1::Referent(value) => {
            Ok((referent(*value), ValueType::Referent(value.domain.get())))
        }
        _ => unsupported("JavaScript lowering does not support this constant kind"),
    }
}

struct Lowerer<'a> {
    slots: BTreeMap<&'a CanonicalStateRefV1, usize>,
    tables: Vec<&'a CanonicalRelationTableV1>,
    arguments: Vec<Option<ValueType>>,
    bindings: BTreeMap<u16, ValueType>,
}
impl Lowerer<'_> {
    fn slot(&self, state: &CanonicalStateRefV1) -> Result<usize> {
        self.slots
            .get(state)
            .cloned()
            .ok_or_else(|| JavaScriptLoweringErrorV1("unresolved state coordinate".into()))
    }
    fn expression(
        &mut self,
        expression: &CanonicalExecutableExpressionV1,
        expected: Option<ValueType>,
    ) -> Result<(String, ValueType)> {
        use CanonicalExecutableExpressionV1 as E;
        let result = match expression {
            E::Sequence(values) => {
                let element = match &expected { Some(ValueType::Sequence(element)) => Some(element.as_ref().clone()), _ => None };
                let mut kind = element;
                let mut emitted = Vec::new();
                for value in values {
                    let (value, found) = self.expression(value, kind.clone())?;
                    kind = Some(found); emitted.push(value);
                }
                (format!("Object.freeze([{}])", emitted.join(",")), ValueType::Sequence(Box::new(kind.ok_or_else(|| JavaScriptLoweringErrorV1("untyped empty sequence".into()))?)))
            }
            E::Record(fields) => {
                let mut kinds = BTreeMap::new();
                let mut emitted = Vec::new();
                for (key,value) in fields {
                    let wanted = match &expected { Some(ValueType::Record(fields)) => fields.get(key).cloned(), _ => None };
                    let (value,kind) = self.expression(value,wanted)?;
                    kinds.insert(key.clone(),kind);
                    emitted.push(format!("[{},{}]", quote(name(key)?), value));
                }
                (format!("Object.freeze(Object.fromEntries([{}]))", emitted.join(",")), ValueType::Record(kinds))
            }
            E::EmptySequence(element) => ("Object.freeze([])".into(), ValueType::Sequence(Box::new(callable_type(element)?))),
            E::SequenceAppend(sequence,item) => {
                let (sequence, ValueType::Sequence(element)) = self.expression(sequence, expected.clone())? else { return unsupported("append requires a sequence"); };
                let (item,_) = self.expression(item,Some(*element.clone()))?;
                (format!("Object.freeze([...({sequence}),({item})])"), ValueType::Sequence(element))
            }
            E::SequenceSort(sequence) => {
                let kind = ValueType::Sequence(Box::new(ValueType::Text));
                let (sequence,_) = self.expression(sequence,Some(kind.clone()))?;
                (format!("Object.freeze([...({sequence})].sort())"), kind)
            }
            E::SequenceFold { accumulator, item, source, initial, body } => {
                let (source, ValueType::Sequence(element)) = self.expression(source,None)? else { return unsupported("fold requires a sequence"); };
                let (initial,kind) = self.expression(initial,expected.clone())?;
                let prior_accumulator = self.bindings.insert(*accumulator,kind.clone());
                let prior_item = self.bindings.insert(*item,*element);
                let result = self.expression(body,Some(kind.clone()));
                for (binding,previous) in [(*accumulator,prior_accumulator),(*item,prior_item)] {
                    if let Some(previous) = previous { self.bindings.insert(binding,previous); } else { self.bindings.remove(&binding); }
                }
                let (body,_) = result?;
                (format!("((xs,initial)=>{{let b{accumulator}=initial;for(const b{item} of xs){{b{accumulator}=({body});}}return b{accumulator};}})(({source}),({initial}))"),kind)
            }
            E::SequenceMap { binding, source, body } => {
                let (source, ValueType::Sequence(element)) = self.expression(source, None)? else { return unsupported("mapping requires a sequence"); };
                let previous = self.bindings.insert(*binding, *element);
                let wanted = match &expected { Some(ValueType::Sequence(element)) => Some(element.as_ref().clone()), _ => None };
                let result = self.expression(body, wanted);
                if let Some(previous) = previous { self.bindings.insert(*binding, previous); }
                else { self.bindings.remove(binding); }
                let (body, kind) = result?;
                (format!("Object.freeze(({source}).map((b{binding})=>({body})))"), ValueType::Sequence(Box::new(kind)))
            }
            E::SequenceCount(value) => {
                let (value, kind) = self.expression(value, None)?;
                if !matches!(kind, ValueType::Sequence(_)) { return unsupported("count requires a sequence"); }
                (format!("({value}).length"), ValueType::Number)
            }
            E::SequenceJoin(value, separator) => {
                let (value, _) = self.expression(value, Some(ValueType::Sequence(Box::new(ValueType::Text))))?;
                let (separator, _) = self.expression(separator, Some(ValueType::Text))?;
                (format!("text(({value}).join({separator}))"), ValueType::Text)
            }
            E::ScalarText(value) => {
                let (value, kind) = self.expression(value, None)?;
                if !matches!(kind, ValueType::Text | ValueType::Boolean | ValueType::Number) { return unsupported("interpolation requires Text, Bool or F64"); }
                (format!("scalarText({value})"), ValueType::Text)
            }
            E::SequenceDrop(value,count) => {
                let (value,kind) = self.expression(value,expected.clone())?;
                if !matches!(kind, ValueType::Sequence(_)) { return unsupported("drop requires a sequence"); }
                let (count,_) = self.expression(count,Some(ValueType::Number))?;
                (format!("drop({value},{count})"),kind)
            }
            E::Field(value,field) => {
                let (value,ValueType::Record(fields)) = self.expression(value,None)? else { return unsupported("field requires a record"); };
                let kind=fields.get(field).cloned().ok_or_else(|| JavaScriptLoweringErrorV1("unknown record field".into()))?;
                (format!("{value}[{}]",quote(name(field)?)),kind)
            }
            E::Require(condition,value,message) => {
                let (condition,_) = self.expression(condition,Some(ValueType::Boolean))?;
                let (value,kind) = self.expression(value,expected.clone())?;
                let (message,_) = self.expression(message,Some(ValueType::Text))?;
                (format!("({condition}?{value}:fail({message}))"),kind)
            }
            E::Foreign { binding, arguments } => {
                if binding.evaluation != CanonicalForeignEvaluationV1::Attempt {
                    return unsupported("JavaScript does not execute delayed target construction");
                }
                binding.check().map_err(|e| JavaScriptLoweringErrorV1(e.into()))?;
                let values=arguments.iter().zip(&binding.arguments).map(|(value,kind)| self.expression(value,Some(callable_type(kind)?)).map(|v| v.0)).collect::<Result<Vec<_>>>()?;
                let member=format!("{}[{}]",foreign_name(&binding.module),quote(&binding.member));
                let value=match binding.operation {
                    CanonicalForeignOperationV1::Get => member,
                    CanonicalForeignOperationV1::Call => format!("(0,{member})({})",values.join(",")),
                };
                let kind=callable_type(&binding.result)?;
                (format!("crossing({value},{})",kind.descriptor()),kind)
            }
            E::Let { binding, value, body } => {
                let (value, kind) = self.expression(value, None)?;
                let previous = self.bindings.insert(*binding, kind);
                let result = self.expression(body, expected.clone());
                if let Some(previous) = previous { self.bindings.insert(*binding, previous); }
                else { self.bindings.remove(binding); }
                let (body, kind) = result?;
                (format!("((b{binding})=>({body}))({value})"), kind)
            }

            E::Constant(value) => constant(value)?,
            E::Argument(index) => {
                let entry = self.arguments.get_mut(usize::from(*index)).ok_or_else(|| {
                    JavaScriptLoweringErrorV1("argument ordinal exceeds handler arity".into())
                })?;
                let kind = match (entry.clone(), expected.clone()) {
                    (Some(a), Some(b)) if a != b => {
                        return unsupported("inconsistent argument type");
                    }
                    (Some(a), _) | (None, Some(a)) => a,
                    (None, None) => {
                        return unsupported(
                            "argument type cannot be derived from checked operands",
                        );
                    }
                };
                *entry = Some(kind.clone());
                (format!("args[{index}]"), kind)
            }
            E::Binding(index) => (
                format!("b{index}"),
                self.bindings.get(index).cloned().ok_or_else(|| JavaScriptLoweringErrorV1("unbound rule variable".into()))?,
            ),
            E::ReferentFacet {
                value,
                domain,
                members,
            } => {
                // The facet is justified by checked membership, not JavaScript coercion.
                let input_type = match value.as_ref() {
                    E::Argument(index) => {
                        self.arguments.get(usize::from(*index)).cloned().flatten()
                    }
                    E::Binding(index) => self.bindings.get(index).cloned(),
                    _ => None,
                }
                .unwrap_or(ValueType::Referent(domain.get()));
                let (value, kind) = self.expression(value, Some(input_type))?;
                if !matches!(kind, ValueType::Referent(_)) {
                    return unsupported("facet operand is not a referent");
                }
                (
                    format!("requireFacet({value},{},{})", domain.get(), ids(members)),
                    ValueType::Referent(domain.get()),
                )
            }
            E::Concatenate(a, b) => {
                self.binary(a, b, ValueType::Text, ValueType::Text, "concatenate")?
            }
            E::Add(a, b) => self.binary(a, b, ValueType::Number, ValueType::Number, "add")?,
            E::Subtract(a, b) => {
                self.binary(a, b, ValueType::Number, ValueType::Number, "subtract")?
            }
            E::Multiply(a, b) => {
                self.binary(a, b, ValueType::Number, ValueType::Number, "multiply")?
            }
            E::Divide(a, b) => self.binary(a, b, ValueType::Number, ValueType::Number, "divide")?,
            E::GreaterThan(a, b) => {
                self.binary(a, b, ValueType::Number, ValueType::Boolean, "greater")?
            }
            E::LessThanOrEqual(a, b) => {
                self.binary(a, b, ValueType::Number, ValueType::Boolean, "lessEqual")?
            }
            E::StartsWith(a, b) => {
                self.binary(a, b, ValueType::Text, ValueType::Boolean, "startsWith")?
            }
            E::ContainsText(a, b) => {
                self.binary(a, b, ValueType::Text, ValueType::Boolean, "containsText")?
            }
            E::Equal(a, b) => {
                let (a, kind) = self.expression(a, None)?;
                let (b, _) = self.expression(b, Some(kind.clone()))?;
                (format!("equal({a},{b})"), ValueType::Boolean)
            }
            E::Not(value) => {
                let (value, _) = self.expression(value, Some(ValueType::Boolean))?;
                (format!("(!{value})"), ValueType::Boolean)
            }
            E::Conditional(condition, yes, no) => {
                let (condition, _) = self.expression(condition, Some(ValueType::Boolean))?;
                let (yes, kind) = self.expression(yes, expected.clone())?;
                let (no, _) = self.expression(no, Some(kind.clone()))?;
                (format!("({condition}?{yes}:{no})"), kind)
            }
            E::SquareRoot(value) => {
                let (value, _) = self.expression(value, Some(ValueType::Number))?;
                (format!("finite(Math.sqrt({value}))"), ValueType::Number)
            }
            E::RelationRead(table, subject) | E::RelationPresent(table, subject) => {
                let E::State(state) = table.as_ref() else {
                    return unsupported("relation read requires an exact state table");
                };
                let slot = self.slot(state)?;
                let table = self.tables[slot];
                let (subject, _) = self.expression(
                    subject,
                    Some(ValueType::Referent(table.subject_domain.get())),
                )?;
                if matches!(expression, E::RelationPresent(..)) {
                    (
                        format!("present(pre[{slot}],{subject})"),
                        ValueType::Boolean,
                    )
                } else {
                    if table.cardinality == CanonicalRelationCardinalityV1::Many {
                        return unsupported("many-valued relation read is unsupported");
                    }
                    (
                        format!("readOne(pre[{slot}],{subject})"),
                        relation_type(table.value_kind)?,
                    )
                }
            }
            other => {
                return unsupported(format!(
                    "unsupported JavaScript expression: {}",
                    expression_name(other)
                ));
            }
        };
        if expected.is_some_and(|expected| expected != result.1) {
            return unsupported("checked operand type mismatch");
        }
        Ok(result)
    }
    fn binary(
        &mut self,
        a: &CanonicalExecutableExpressionV1,
        b: &CanonicalExecutableExpressionV1,
        input: ValueType,
        output: ValueType,
        function: &str,
    ) -> Result<(String, ValueType)> {
        let (a, _) = self.expression(a, Some(input.clone()))?;
        let (b, _) = self.expression(b, Some(input))?;
        Ok((format!("{function}({a},{b})"), output))
    }
    fn pattern(
        &mut self,
        pattern: &CanonicalExecutableExpressionV1,
        candidate: &str,
        kind: ValueType,
    ) -> Result<String> {
        use CanonicalExecutableExpressionV1 as E;
        if let E::Binding(index) = pattern {
            if !self.bindings.contains_key(index) {
                self.bindings.insert(*index, kind.clone());
                return Ok(format!("const b{index}={candidate};\n"));
            }
        }
        if let E::ReferentFacet {
            value,
            domain,
            members,
        } = pattern
        {
            let mut output = String::new();
            if let E::Binding(index) = value.as_ref() {
                if !self.bindings.contains_key(index) {
                    self.bindings.insert(*index, kind);
                    output.push_str(&format!("const b{index}={candidate};\n"));
                }
            }
            let input_type = match value.as_ref() {
                E::Binding(index) => self.bindings.get(index).cloned(),
                E::Argument(index) => self.arguments.get(usize::from(*index)).cloned().flatten(),
                _ => None,
            }
            .unwrap_or(ValueType::Referent(domain.get()));
            let (value, _) = self.expression(value, Some(input_type))?;
            output.push_str(&format!(
                "if(!equal(facet({value},{},{}),{candidate})) continue;\n",
                domain.get(),
                ids(members)
            ));
            return Ok(output);
        }
        let (value, _) = self.expression(pattern, Some(kind))?;
        Ok(format!("if(!equal({value},{candidate})) continue;\n"))
    }
    fn rule(&mut self, rule: &CanonicalExecutableRuleV1) -> Result<String> {
        use CanonicalExecutableExpressionV1 as E;
        use CanonicalExecutablePredicateV1 as P;
        self.bindings.clear();
        if !rule.law_origins.is_empty() || !rule.removals.is_empty() {
            return unsupported(
                "JavaScript lowering does not support law origins or whole-state removals",
            );
        }
        let mut output = String::from("{\n");
        let mut close = String::from("}\n");
        for state in &rule.required_present {
            let slot = self.slot(state)?;
            output.push_str(&format!("if(pre[{slot}]!==undefined){{\n"));
            close.push_str("}\n");
        }
        for state in &rule.required_absent {
            let slot = self.slot(state)?;
            output.push_str(&format!("if(pre[{slot}]===undefined){{\n"));
            close.push_str("}\n");
        }
        for (ordinal, predicate) in rule.predicates.iter().enumerate() {
            match predicate {
                P::RelationMatch(state, subject, value) => {
                    let slot = self.slot(state)?;
                    let table = self.tables[slot];
                    output.push_str(&format!(
                        "for(const [s{ordinal},vs{ordinal}] of pre[{slot}].rows){{\n"
                    ));
                    output.push_str(&self.pattern(
                        subject,
                        &format!("s{ordinal}"),
                        ValueType::Referent(table.subject_domain.get()),
                    )?);
                    output.push_str(&format!("for(const v{ordinal} of vs{ordinal}){{\n"));
                    output.push_str(&self.pattern(
                        value,
                        &format!("v{ordinal}"),
                        relation_type(table.value_kind)?,
                    )?);
                    close.push_str("}\n}\n");
                }
                P::Equal(a, b) | P::GreaterThan(a, b) | P::LessThanOrEqual(a, b) => {
                    let expression = match predicate {
                        P::Equal(..) => E::Equal(Box::new(a.clone()), Box::new(b.clone())),
                        P::GreaterThan(..) => {
                            E::GreaterThan(Box::new(a.clone()), Box::new(b.clone()))
                        }
                        _ => E::LessThanOrEqual(Box::new(a.clone()), Box::new(b.clone())),
                    };
                    let (value, _) = self.expression(&expression, Some(ValueType::Boolean))?;
                    output.push_str(&format!("if({value}){{\n"));
                    close.push_str("}\n");
                }
                P::Contains(..) => return unsupported("unsupported JavaScript Contains predicate"),
            }
        }
        for assignment in &rule.assignments {
            let slot = self.slot(&assignment.target)?;
            let E::RelationEffects(effects) = &assignment.value else {
                return unsupported(
                    "JavaScript lowering currently requires relation-row assignments",
                );
            };
            let table = self.tables[slot];
            for effect in effects {
                let (mode, subject, value) = match effect {
                    CanonicalRelationEffectV1::Put(s, v) => (0, s, v),
                    CanonicalRelationEffectV1::Insert(s, v) => (1, s, v),
                    CanonicalRelationEffectV1::Remove(s, v) => (2, s, v),
                    CanonicalRelationEffectV1::Accumulate(..) => {
                        return unsupported("unsupported JavaScript Accumulate effect");
                    }
                };
                let (subject, _) = self.expression(
                    subject,
                    Some(ValueType::Referent(table.subject_domain.get())),
                )?;
                let (value, _) = self.expression(value, Some(relation_type(table.value_kind)?))?;
                output.push_str(&format!(
                    "stage(pending,pre,{slot},{mode},{subject},{value});\n"
                ));
            }
        }
        output.push_str(&close);
        Ok(output)
    }
}
fn ids(values: &[FormationLocalId]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| value.get().to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn expression_name(expression: &CanonicalExecutableExpressionV1) -> &'static str {
    use CanonicalExecutableExpressionV1 as E;
    match expression {
        E::State(_) => "State",
        E::TextTransform(..) => "TextTransform",
        E::Sum { .. } => "Sum",
        E::RelationEffects(_) => "RelationEffects",
        E::Accumulate(_) => "Accumulate",
        E::MatchesAny(_) => "MatchesAny",
        E::FreshReferent { .. } => "FreshReferent",
        E::RelationPut(..) => "RelationPut",
        E::RelationInsert(..) => "RelationInsert",
        E::RelationRemoveRow(..) => "RelationRemoveRow",
        E::RelationRemoveValue(..) => "RelationRemoveValue",
        E::Insert(..) => "Insert",
        E::Remove(..) => "Remove",
        _ => "invalid expression context",
    }
}

/// Compile checked canonical IR to a self-contained ES module.
///
/// Each session owns its state. External handlers validate arguments, read one
/// pre-state, and commit all row effects only after conflict and contract checks.
/// Returns an error for unsupported source productions, triggers, values, or IR.
pub fn lower_javascript_v1(
    package: &CanonicalSourcePackageSliceV1,
) -> Result<JavaScriptArtifactsV1> {
    if !package.unsupported.is_empty() {
        return unsupported("package contains unsupported source productions");
    }
    if !package.keyboard_bindings.is_empty()
        || !package.scalar_input_bindings.is_empty()
        || !package.referent_input_bindings.is_empty()
    {
        return unsupported("JavaScript lowering does not support physical input bindings");
    }
    let mut lowerer = Lowerer {
        slots: BTreeMap::new(),
        tables: Vec::new(),
        arguments: vec![],
        bindings: BTreeMap::new(),
    };
    let mut initial = Vec::new();
    for (slot, cell) in package.state_cells.iter().enumerate() {
        let Some(CanonicalScalarValueV1::RelationTable(table)) = &cell.initial_value else {
            return unsupported(
                "JavaScript lowering currently requires initialized relation-table state",
            );
        };
        if lowerer.slots.insert(&cell.state, slot).is_some() {
            return unsupported("duplicate state coordinate");
        }
        lowerer.tables.push(table);
        let kind = relation_type(table.value_kind)?;
        let cardinality = match table.cardinality {
            CanonicalRelationCardinalityV1::One => "one",
            CanonicalRelationCardinalityV1::Maybe => "maybe",
            CanonicalRelationCardinalityV1::Many => "many",
        };
        let mut rows = Vec::new();
        for (subject, values) in &table.rows {
            if subject.domain != table.subject_domain {
                return unsupported("initial relation subject type mismatch");
            }
            if table.cardinality != CanonicalRelationCardinalityV1::Many && values.len() != 1 {
                return unsupported("initial relation cardinality mismatch");
            }
            let values = values
                .iter()
                .map(|value| {
                    let (value, actual) = constant(value)?;
                    if actual != kind {
                        return unsupported("initial relation value type mismatch");
                    }
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()?;
            rows.push(format!("[{},[{}]]", referent(*subject), values.join(",")));
        }
        initial.push(format!(
            "{{domain:{},kind:{},cardinality:{},total:{},rows:[{}]}}",
            table.subject_domain.get(),
            kind.descriptor(),
            quote(cardinality),
            table.total,
            rows.join(",")
        ));
    }
    let mut refs = BTreeMap::new();
    let mut projections = BTreeMap::new();
    let mut read_declarations = BTreeSet::new();
    for projection in &package.relational_projection {
        let subject = name(&projection.subject)?;
        if refs
            .insert(subject, projection.referent)
            .is_some_and(|prior| prior != projection.referent)
        {
            return unsupported(
                "multiple domain facets for a projected subject are not yet supported",
            );
        }
        let slot = lowerer.slot(&projection.state)?;
        let table = lowerer.tables[slot];
        let relation = name(&projection.state.relation_designation)?;
        if !matches!(projection.state.path, CanonicalStatePathV1::Rows) {
            return unsupported("structured projection is not yet supported");
        }
        let key = format!("[{},{}]", quote(subject), quote(relation));
        if projections
            .insert(key, (slot, projection.referent))
            .is_some()
        {
            return unsupported("ambiguous projection coordinate");
        }
        let mut kind = relation_type(table.value_kind)?.declaration();
        if table.cardinality == CanonicalRelationCardinalityV1::Many {
            kind = format!("ReadonlyArray<{kind}>");
        } else if !table.total {
            kind.push_str(" | undefined");
        }
        read_declarations.insert(format!(
            "  read(subject: {}, relation: {}): {kind};\n",
            quote(subject),
            quote(relation)
        ));
    }
    let mut handlers = Vec::new();
    let mut handler_declarations = Vec::new();
    let mut handler_names = BTreeSet::new();
    for handler in &package.executable_handlers {
        if handler.trigger != CanonicalHandlerTriggerV1::External {
            return unsupported(format!(
                "unsupported JavaScript handler trigger: {:?}",
                handler.trigger
            ));
        }
        let designation = quote(name(&handler.designation)?);
        if !handler_names.insert(designation.clone()) {
            return unsupported("duplicate handler designation");
        }
        lowerer.arguments = vec![None; usize::from(handler.argument_count)];
        let body = handler
            .rules
            .iter()
            .map(|rule| lowerer.rule(rule))
            .collect::<Result<Vec<_>>>()?
            .join("");
        let types = lowerer
            .arguments
            .iter()
            .map(|kind| {
                kind.clone().ok_or_else(|| {
                    JavaScriptLoweringErrorV1("unused or unresolved handler argument type".into())
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let validation = types
            .iter()
            .enumerate()
            .map(|(i, kind)| format!("validate(args[{i}],{});\n", kind.descriptor()))
            .collect::<String>();
        handlers.push(format!("[{designation},(...args)=>{{\nif(args.length!=={})fail(\"ArgumentCount\");\n{validation}const pre=state;const pending=[];\n{body}state=commit(pre,pending);\n}}]",handler.argument_count));
        handler_declarations.push(format!(
            "    {designation}: ({}) => void;\n",
            types
                .iter()
                .enumerate()
                .map(|(i, kind)| format!("arg{i}: {}", kind.declaration()))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let reference_entries = refs
        .iter()
        .map(|(name, value)| format!("[{},{}]", quote(name), referent(*value)))
        .collect::<Vec<_>>()
        .join(",");
    let projection_entries = projections
        .iter()
        .map(|(key, (slot, subject))| format!("[{},[{slot},{}]]", quote(key), referent(*subject)))
        .collect::<Vec<_>>()
        .join(",");
    let mut module = format!(
        "// Generated from checked Clause canonical executable IR.\n{RUNTIME}\nexport function createSession(){{\nlet state=[{}];\nvalidateContracts(state);\nconst referents=Object.freeze(Object.fromEntries([{reference_entries}]));\nconst projections=new Map([{projection_entries}]);\nconst handlers=Object.freeze(Object.fromEntries([{}]));\nreturn Object.freeze({{referents,handlers,read(subject,relation){{\nconst projection=projections.get(JSON.stringify([subject,relation]));\nif(!projection)fail(\"UnknownProjection\");\nconst [slot,subjectValue]=projection;\nconst table=state[slot];\nconst values=row(table,subjectValue);\nreturn table.cardinality===\"many\"?Object.freeze([...(values??[])]):values?.[0];\n}}}});\n}}\n",
        initial.join(","),
        handlers.join(",")
    );
    let mut declarations = format!(
        "export interface Referent<Domain extends number> {{ readonly domain: Domain; readonly identity: number; }}\nexport interface Session {{\n  readonly referents: {{ {} }};\n  readonly handlers: {{\n{}  }};\n{}}}\nexport declare function createSession(): Session;\n",
        refs.iter()
            .map(|(name, value)| format!(
                "readonly {}: Referent<{}>;",
                quote(name),
                value.domain.get()
            ))
            .collect::<Vec<_>>()
            .join(" "),
        handler_declarations.join(""),
        read_declarations.into_iter().collect::<String>()
    );
    let mut export_names = BTreeSet::new();
    if package.state_cells.is_empty() && package.executable_handlers.is_empty() {
        module = format!("// Generated from checked Clause canonical executable IR.\n{RUNTIME}\n");
        declarations.clear();
    } else {
        export_names.insert("createSession".to_string());
    }
    let mut imports = BTreeSet::new();
    for (ordinal, callable) in package.callables.iter().enumerate() {
        check_canonical_callable_v1(callable).map_err(|e| JavaScriptLoweringErrorV1(format!("{e:?}")))?;
        foreign_modules(&callable.expression, &mut imports);
        let designation = name(&callable.designation)?;
        if callable.exported && !export_names.insert(designation.to_string()) {
            return unsupported("duplicate JavaScript export name");
        }
        let types = callable
            .arguments
            .iter()
            .map(|argument| callable_type(&argument.value_kind))
            .collect::<Result<Vec<_>>>()?;
        let result = callable_type(&callable.result_kind)?;
        // Pure callables cannot access session state or rule-local bindings.
        let mut pure = Lowerer {
            slots: BTreeMap::new(),
            tables: Vec::new(),
            bindings: BTreeMap::new(),
            arguments: types.iter().cloned().map(Some).collect(),
        };
        let (expression, _) = pure.expression(&callable.expression, Some(result.clone()))?;
        let validation = types
            .iter()
            .enumerate()
            .map(|(index, kind)| format!("args[{index}]=crossing(args[{index}],{});\n", kind.descriptor()))
            .collect::<String>();
        module.push_str(&format!(
            "function callable{ordinal}(...args){{\nif(args.length!=={})fail(\"ArgumentCount\");\n{validation}const result={expression};\nvalidate(result,{});\nreturn result;\n}}\n",
            types.len(), result.descriptor(),
        ));
        if callable.exported {
            let export = format!(
                "export {{ callable{ordinal} as {} }};\n",
                quote(designation)
            );
            module.push_str(&export);
            declarations.push_str(&format!(
                "declare function callable{ordinal}({}): {};\n{export}",
                types
                    .iter()
                    .enumerate()
                    .map(|(index, kind)| format!("arg{index}: {}", kind.declaration()))
                    .collect::<Vec<_>>()
                    .join(", "),
                result.declaration()
            ));
        }
    }
    let imports = imports.iter().map(|source| format!("import * as {} from {};\n", foreign_name(source), quote(source))).collect::<String>();
    module.insert_str(0, &imports);
    Ok(JavaScriptArtifactsV1 {
        module,
        declarations,
    })
}

const RUNTIME: &str = r#"
function fail(code){throw new Error(code);}
function equal(a,b){
 if(a===b)return true;
 if(a===null||b===null||typeof a!=='object'||typeof b!=='object'||Array.isArray(a)!==Array.isArray(b))return false;
 if(Array.isArray(a))return a.length===b.length&&a.every((v,i)=>equal(v,b[i]));
 const keys=Object.keys(a);return keys.length===Object.keys(b).length&&keys.every(k=>Object.hasOwn(b,k)&&equal(a[k],b[k]));
}
function finite(value){if(typeof value!=='number'||!Number.isFinite(value))fail('NumericDomain');return value===0?0:value;}
function scalarText(value){
 if(typeof value!=='number')return String(value);
 const rendered=String(finite(value));
 if(!rendered.includes('e'))return rendered;
 const negative=rendered.startsWith('-');
 const [mantissa,exponent]=rendered.replace(/^-/, '').split('e');
 const digits=mantissa.replace('.', '');
 const point=(mantissa.includes('.')?mantissa.indexOf('.'):mantissa.length)+Number(exponent);
 const decimal=point<=0?'0.'+'0'.repeat(-point)+digits:point>=digits.length?digits+'0'.repeat(point-digits.length):digits.slice(0,point)+'.'+digits.slice(point);
 return (negative?'-':'')+decimal;
}
function text(value){if(typeof value!=='string'||!value.isWellFormed()||new TextEncoder().encode(value).length>16777216)fail('TextDomain');return value;}
function validate(value,kind){
 switch(kind[0]){
 case 'number':finite(value);break;
 case 'text':text(value);break;
 case 'boolean':if(typeof value!=='boolean')fail('TypeMismatch');break;
 case 'referent':if(value===null||typeof value!=='object'||value.domain!==kind[1]||!Number.isInteger(value.identity)||value.identity<=0||value.identity>4294967295)fail('TypeMismatch');break;
 case 'sequence':if(!Array.isArray(value))fail('TypeMismatch');for(let i=0;i<value.length;i++){if(!Object.hasOwn(value,i))fail('TypeMismatch');validate(value[i],kind[1]);}break;
 case 'record':if(value===null||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==kind[1].length)fail('TypeMismatch');for(const [key,type] of kind[1]){if(!Object.hasOwn(value,key))fail('TypeMismatch');validate(value[key],type);}break;
 default:fail('TypeMismatch');
 }
}
function crossing(value,kind){
 validate(value,kind);
 if(kind[0]==='sequence')return Object.freeze(value.map(v=>crossing(v,kind[1])));
 if(kind[0]==='record')return Object.freeze(Object.fromEntries(kind[1].map(([key,type])=>[key,crossing(value[key],type)])));
 return value;
}
function drop(value,count){if(!Number.isFinite(count)||!Number.isInteger(count)||count<0)fail('NumericDomain');return Object.freeze(value.slice(count));}
function facet(value,domain,members){if(value===null||typeof value!=='object')return undefined;if(value.domain===domain)return value;if(members.includes(value.identity))return Object.freeze({domain,identity:value.identity});return undefined;}
function requireFacet(value,domain,members){const result=facet(value,domain,members);if(result===undefined)fail('TypeMismatch');return result;}
function concatenate(a,b){return text(text(a)+text(b));}
function add(a,b){return finite(finite(a)+finite(b));}
function subtract(a,b){return finite(finite(a)-finite(b));}
function multiply(a,b){return finite(finite(a)*finite(b));}
function divide(a,b){return finite(finite(a)/finite(b));}
function greater(a,b){return finite(a)>finite(b);}
function lessEqual(a,b){return finite(a)<=finite(b);}
function startsWith(a,b){return text(a).startsWith(text(b));}
function containsText(a,b){return text(a).includes(text(b));}
function row(table,subject){return table.rows.find(([key])=>equal(key,subject))?.[1];}
function present(table,subject){validate(subject,['referent',table.domain]);return row(table,subject)!==undefined;}
function readOne(table,subject){const values=row(table,subject);if(values?.length!==1)fail('MissingState');return values[0];}
function stage(pending,pre,slot,mode,subject,value){
 const table=pre[slot];validate(subject,['referent',table.domain]);validate(value,table.kind);
 if(table.cardinality==='many'){
  if(mode!==1&&mode!==2)fail('TypeMismatch');
 }else if(mode===1)fail('TypeMismatch');
 if(pending.some(effect=>effect.slot===slot&&equal(effect.subject,subject)&&(table.cardinality!=='many'||equal(effect.value,value))))fail('ConflictingStateEffects');
 pending.push({slot,mode,subject,value});
}
function commit(pre,pending){
 const next=pre.map(table=>({...table,rows:table.rows.map(([subject,values])=>[subject,[...values]])}));
 for(const {slot,mode,subject,value} of pending){
  const table=next[slot];let index=table.rows.findIndex(([key])=>equal(key,subject));
  if(mode===0){if(index<0)table.rows.push([subject,[value]]);else table.rows[index]=[subject,[value]];}
  else if(mode===1){if(index<0)table.rows.push([subject,[value]]);else if(!table.rows[index][1].some(prior=>equal(prior,value)))table.rows[index][1].push(value);}
  else{
   if(index<0)fail('MissingState');
   const values=table.rows[index][1];const at=values.findIndex(prior=>equal(prior,value));
   if(at<0)fail('MissingState');values.splice(at,1);if(values.length===0)table.rows.splice(index,1);
  }
 }
 validateContracts(next);return next;
}
function validateContracts(tables){
 const participants=[];
 for(const table of tables)for(const [subject,values] of table.rows){participants.push(subject);if(table.kind[0]==='referent')participants.push(...values);}
 for(const table of tables)if(table.total){if(table.cardinality!=='one')fail('TypeMismatch');for(const subject of participants)if(subject.domain===table.domain&&row(table,subject)?.length!==1)fail('MissingState');}
}
"#;
