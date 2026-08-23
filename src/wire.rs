//! Canonical Clause semantic wire v2.  The semantic payload is intentionally
//! an ordered JSON array: its exact UTF-8 bytes are the identity preimage.

use crate::kernel::{
    Cardinality, ClaimResult, Clause, Intent, IntentResult, KernelError, Law, Mode, Model, Proof,
    Relation, RequireResult, Result, Revision, Role, Sentence, Term,
};

pub fn semantic_payload(model: &Model) -> String {
    let relations = model
        .relations()
        .values()
        .map(relation_json)
        .collect::<Vec<_>>()
        .join(",");
    let facts = model
        .facts()
        .iter()
        .map(|fact| clause_json("fact", fact))
        .collect::<Vec<_>>()
        .join(",");
    let laws = model
        .laws()
        .iter()
        .map(law_json)
        .collect::<Vec<_>>()
        .join(",");
    let query = clause_json("query", model.query());
    let intents = model
        .intents()
        .iter()
        .map(intent_json)
        .collect::<Vec<_>>()
        .join(",");
    let sought_role = model
        .query()
        .roles()
        .iter()
        .find_map(|(role, term)| term.is_variable().then_some(role.as_str()))
        .expect("model admission requires one query variable");
    format!(
        "[\"clause-semantic-v4\",[\"relations\",[{relations}]],[\"facts\",[{facts}]],[\"laws\",[{laws}]],[\"query\",{query}],[\"intents\",[{intents}]],[\"order\",\"{}\",\"{}\"]]",
        escape(model.order()),
        escape(sought_role)
    )
}

pub fn revision_id(model: &Model) -> String {
    format!(
        "rev-sha256-{}",
        sha256_hex(semantic_payload(model).as_bytes())
    )
}

pub fn serialize(revision: &Revision) -> String {
    format!(
        "[\"clause-revision-v2\",\"{}\",{}]",
        escape(revision.identity()),
        semantic_payload(revision.model())
    )
}

pub fn claim_output(result: &ClaimResult) -> String {
    match result {
        ClaimResult::Admitted { .. } => {
            let successor = result.successor().expect("admitted claim has successor");
            let fact = result.fact().expect("admitted claim has fact");
            format!(
                "[\"clause-claim-output-v1\",\"admitted\",[\"branch\",\"{}\"],[\"base\",\"{}\"],[\"revision\",\"{}\"],[\"fact\",{}]]",
                escape(result.branch().name()),
                escape(result.base_revision().identity()),
                escape(successor.revision().identity()),
                clause_json("fact", fact)
            )
        }
        ClaimResult::Duplicate { .. } => format!(
            "[\"clause-claim-output-v1\",\"duplicate\",[\"branch\",\"{}\"],[\"revision\",\"{}\"],[\"diagnostic\",\"claim.duplicate\"]]",
            escape(result.branch().name()),
            escape(result.base_revision().identity())
        ),
    }
}

pub fn require_output(result: &RequireResult) -> String {
    match result {
        RequireResult::Satisfied { .. } => format!(
            "[\"clause-require-output-v1\",\"satisfied\",[\"revision\",\"{}\"],[\"proof\",{}]]",
            escape(result.revision().identity()),
            proof_json(result.proof().expect("satisfied require has proof"))
        ),
        RequireResult::Unsatisfied { .. } => format!(
            "[\"clause-require-output-v1\",\"unsatisfied\",[\"revision\",\"{}\"],[\"clause\",{}],[\"diagnostic\",\"require.unsatisfied\"]]",
            escape(result.revision().identity()),
            clause_json(
                "clause",
                result.clause().expect("unsatisfied require has clause")
            )
        ),
    }
}

pub fn intent_output(result: &IntentResult) -> String {
    match result {
        IntentResult::Proposed { revision, intent } => {
            let desired = clause_json("clause", intent.desired());
            let fact = clause_json("fact", intent.desired());
            format!(
                "[\"clause-intent-output-v1\",\"proposed\",[\"revision\",\"{}\"],[\"intent\",\"{}\"],[\"desired\",{}],[\"plan\",[\"plan\",\"plan/{}/{}\",\"operation\",\"claim\",\"base\",\"{}\",\"fact\",{}]],[\"explanation\",[\"explanation\",\"desired-clause-is-absent\",\"revision\",\"{}\",\"clause\",{},\"diagnostic\",\"require.unsatisfied\"]]]",
                escape(revision.identity()),
                escape(intent.name()),
                desired,
                escape(revision.identity()),
                escape(intent.name()),
                escape(revision.identity()),
                fact,
                escape(revision.identity()),
                desired,
            )
        }
        IntentResult::AlreadySatisfied {
            revision,
            intent,
            proof,
        } => format!(
            "[\"clause-intent-output-v1\",\"already-satisfied\",[\"revision\",\"{}\"],[\"intent\",\"{}\"],[\"desired\",{}],[\"proof\",{}],[\"explanation\",[\"explanation\",\"desired-clause-is-claimed\",\"revision\",\"{}\"]]]",
            escape(revision.identity()),
            escape(intent.name()),
            clause_json("clause", intent.desired()),
            proof_json(proof),
            escape(revision.identity()),
        ),
        IntentResult::Rejected { revision, name } => format!(
            "[\"clause-intent-output-v1\",\"rejected\",[\"revision\",\"{}\"],[\"intent\",\"{}\"],[\"diagnostic\",\"intent.unknown\"]]",
            escape(revision.identity()),
            escape(name),
        ),
    }
}

pub fn reload(bytes: &str) -> Result<Revision> {
    let value = JsonParser::new(bytes).parse()?;
    if json(&value) != bytes {
        return Err(KernelError::new("revision wire is not canonical JSON"));
    }
    let envelope = list(&value, 3, "revision envelope")?;
    require_string(&envelope[0], "clause-revision-v2", "revision envelope tag")?;
    let claimed = string(&envelope[1], "revision identity")?;
    if !claimed.starts_with("rev-sha256-")
        || claimed.len() != 75
        || !claimed[11..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(KernelError::new("invalid revision identity"));
    }
    let model = decode_model(&envelope[2])?;
    let expected = revision_id(&model);
    if claimed != expected {
        return Err(KernelError::new(
            "revision identity does not match canonical semantic payload",
        ));
    }
    if serialize(&Revision::reloaded(claimed.to_owned(), model.clone())) != bytes {
        return Err(KernelError::new("revision payload is not canonical"));
    }
    Ok(Revision::reloaded(claimed.to_owned(), model))
}

fn relation_json(relation: &Relation) -> String {
    let roles = relation
        .roles()
        .values()
        .map(|role| format!("[\"{}\",\"{}\"]", escape(role.name()), escape(role.typ())))
        .collect::<Vec<_>>()
        .join(",");
    let sentence = relation.sentence();
    let modes = relation
        .modes()
        .iter()
        .map(mode_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"relation\",\"{}\",\"roles\",[{roles}],\"sentence\",[\"{}\",\"{}\",\"{}\"],\"modes\",[{modes}]]",
        escape(relation.name()),
        escape(sentence.left()),
        escape(sentence.literal()),
        escape(sentence.right())
    )
}

fn mode_json(mode: &Mode) -> String {
    let known = mode
        .known()
        .iter()
        .map(|name| format!("\"{}\"", escape(name)))
        .collect::<Vec<_>>()
        .join(",");
    let sought = mode
        .sought()
        .iter()
        .map(|name| format!("\"{}\"", escape(name)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"mode\",\"finite\",\"known\",[{known}],\"sought\",[{sought}],\"cardinality\",\"{}\"]",
        mode.cardinality().as_str()
    )
}

fn clause_json(kind: &str, clause: &Clause) -> String {
    let roles = clause
        .roles()
        .iter()
        .map(|(name, term)| format!("[\"{}\",{}]", escape(name), term_json(term)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"{kind}\",\"{}\",\"roles\",[{roles}]]",
        escape(clause.relation())
    )
}

fn law_json(law: &Law) -> String {
    let premises = law
        .premises()
        .iter()
        .map(|premise| clause_json("premise", premise))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"law\",\"{}\",\"premises\",[{premises}],\"conclusion\",{}]",
        escape(law.name()),
        clause_json("conclusion", law.conclusion())
    )
}

fn intent_json(intent: &Intent) -> String {
    format!(
        "[\"intent\",\"{}\",\"desired\",{}]",
        escape(intent.name()),
        clause_json("clause", intent.desired())
    )
}

fn term_json(term: &Term) -> String {
    format!(
        "[\"{}\",\"{}\"]",
        if term.is_variable() {
            "variable"
        } else {
            "literal"
        },
        escape(term.text())
    )
}

fn proof_json(proof: &Proof) -> String {
    let roles = proof
        .roles()
        .iter()
        .map(|(name, value)| format!("[\"{}\",\"{}\"]", escape(name), escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[\"proof\",\"{}\",\"relation\",\"{}\",\"roles\",[{roles}]]",
        escape(&proof.identity()),
        escape(proof.relation())
    )
}

fn decode_model(value: &Json) -> Result<Model> {
    let root = list(value, 7, "semantic payload")?;
    require_string(&root[0], "clause-semantic-v4", "semantic tag")?;
    let relations_group = list(&root[1], 2, "relations")?;
    require_string(&relations_group[0], "relations", "relations tag")?;
    let relation_values = array(&relations_group[1], "relations body")?;
    let mut relations = Vec::new();
    for relation in relation_values {
        relations.push(decode_relation(relation)?);
    }
    let facts_group = list(&root[2], 2, "facts")?;
    require_string(&facts_group[0], "facts", "facts tag")?;
    let fact_values = array(&facts_group[1], "facts body")?;
    let mut facts = Vec::new();
    for fact in fact_values {
        facts.push(decode_clause(fact, "fact")?);
    }
    let laws_group = list(&root[3], 2, "laws")?;
    require_string(&laws_group[0], "laws", "laws tag")?;
    let mut laws = Vec::new();
    for law in array(&laws_group[1], "laws body")? {
        laws.push(decode_law(law)?);
    }
    let query_group = list(&root[4], 2, "query")?;
    require_string(&query_group[0], "query", "query tag")?;
    let query = decode_clause(&query_group[1], "query")?;
    let intents_group = list(&root[5], 2, "intents")?;
    require_string(&intents_group[0], "intents", "intents tag")?;
    let mut intents = Vec::new();
    for intent in array(&intents_group[1], "intents body")? {
        intents.push(decode_intent(intent)?);
    }
    let order_group = list(&root[6], 3, "order")?;
    require_string(&order_group[0], "order", "order tag")?;
    let order = string(&order_group[1], "order value")?.to_owned();
    let model = Model::with_laws_and_intents(relations, facts, laws, query, intents, order)?;
    if semantic_payload(&model) != json(value) {
        return Err(KernelError::new("semantic payload is not canonical"));
    }
    Ok(model)
}

fn decode_law(value: &Json) -> Result<Law> {
    let item = list(value, 6, "law")?;
    require_string(&item[0], "law", "law tag")?;
    let name = string(&item[1], "law name")?;
    require_string(&item[2], "premises", "law premises tag")?;
    let mut premises = Vec::new();
    for premise in array(&item[3], "law premises")? {
        premises.push(decode_clause(premise, "premise")?);
    }
    require_string(&item[4], "conclusion", "law conclusion tag")?;
    Law::new(name, premises, decode_clause(&item[5], "conclusion")?)
}

fn decode_intent(value: &Json) -> Result<Intent> {
    let item = list(value, 4, "intent")?;
    require_string(&item[0], "intent", "intent tag")?;
    let name = string(&item[1], "intent name")?;
    require_string(&item[2], "desired", "intent desired tag")?;
    Intent::new(name, decode_clause(&item[3], "clause")?)
}

fn decode_relation(value: &Json) -> Result<Relation> {
    let item = list(value, 8, "relation")?;
    require_string(&item[0], "relation", "relation tag")?;
    let name = string(&item[1], "relation name")?;
    require_string(&item[2], "roles", "relation roles tag")?;
    let role_values = array(&item[3], "relation roles")?;
    let mut roles = Vec::new();
    for role in role_values {
        let pair = list(role, 2, "role")?;
        roles.push(Role::new(
            string(&pair[0], "role name")?,
            string(&pair[1], "role type")?,
        )?);
    }
    require_string(&item[4], "sentence", "sentence tag")?;
    let sentence_values = list(&item[5], 3, "sentence")?;
    let sentence = Sentence::new(
        string(&sentence_values[0], "sentence left")?,
        string(&sentence_values[1], "sentence literal")?,
        string(&sentence_values[2], "sentence right")?,
    )?;
    require_string(&item[6], "modes", "modes tag")?;
    let mut modes = Vec::new();
    for mode in array(&item[7], "modes")? {
        modes.push(decode_mode(mode)?);
    }
    Relation::new(name, roles, sentence, modes)
}

fn decode_mode(value: &Json) -> Result<Mode> {
    let item = list(value, 8, "mode")?;
    require_string(&item[0], "mode", "mode tag")?;
    require_string(&item[1], "finite", "mode finiteness")?;
    require_string(&item[2], "known", "mode known tag")?;
    let known = array(&item[3], "known roles")?
        .iter()
        .map(|value| string(value, "known role").map(str::to_owned))
        .collect::<Result<Vec<_>>>()?;
    require_string(&item[4], "sought", "mode sought tag")?;
    let sought = array(&item[5], "sought roles")?
        .iter()
        .map(|value| string(value, "sought role").map(str::to_owned))
        .collect::<Result<Vec<_>>>()?;
    require_string(&item[6], "cardinality", "mode cardinality tag")?;
    Mode::finite(
        known,
        sought,
        Cardinality::parse(string(&item[7], "mode cardinality")?)?,
    )
}

fn decode_clause(value: &Json, expected_kind: &str) -> Result<Clause> {
    let item = list(value, 4, "clause")?;
    require_string(&item[0], expected_kind, "clause tag")?;
    let relation = string(&item[1], "clause relation")?;
    require_string(&item[2], "roles", "clause roles tag")?;
    let mut roles = Vec::new();
    for role in array(&item[3], "clause roles")? {
        let pair = list(role, 2, "clause role")?;
        roles.push((
            string(&pair[0], "clause role name")?.to_owned(),
            decode_term(&pair[1])?,
        ));
    }
    Clause::new(relation, roles)
}

fn decode_term(value: &Json) -> Result<Term> {
    let item = list(value, 2, "term")?;
    let text = string(&item[1], "term text")?;
    match string(&item[0], "term kind")? {
        "literal" => Term::literal(text),
        "variable" => Term::variable(text),
        _ => Err(KernelError::new("invalid term kind")),
    }
}

fn escape(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{1f}' => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output
}

#[derive(Clone, Debug)]
enum Json {
    String(String),
    Array(Vec<Json>),
}
fn json(value: &Json) -> String {
    match value {
        Json::String(text) => format!("\"{}\"", escape(text)),
        Json::Array(values) => format!(
            "[{}]",
            values.iter().map(json).collect::<Vec<_>>().join(",")
        ),
    }
}
fn array<'a>(value: &'a Json, where_: &str) -> Result<&'a [Json]> {
    match value {
        Json::Array(values) => Ok(values),
        _ => Err(KernelError::new(format!("invalid {where_}"))),
    }
}
fn list<'a>(value: &'a Json, count: usize, where_: &str) -> Result<&'a [Json]> {
    let values = array(value, where_)?;
    if values.len() == count {
        Ok(values)
    } else {
        Err(KernelError::new(format!("invalid {where_}")))
    }
}
fn string<'a>(value: &'a Json, where_: &str) -> Result<&'a str> {
    match value {
        Json::String(text) => Ok(text),
        _ => Err(KernelError::new(format!("invalid {where_}"))),
    }
}
fn require_string(value: &Json, expected: &str, where_: &str) -> Result<()> {
    if string(value, where_)? == expected {
        Ok(())
    } else {
        Err(KernelError::new(format!("invalid {where_}")))
    }
}

struct JsonParser<'a> {
    input: &'a [u8],
    at: usize,
}
impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            at: 0,
        }
    }
    fn parse(mut self) -> Result<Json> {
        let value = self.value()?;
        if self.at == self.input.len() {
            Ok(value)
        } else {
            Err(KernelError::new("trailing data in revision wire"))
        }
    }
    fn value(&mut self) -> Result<Json> {
        match self.peek() {
            Some(b'\"') => self.string().map(Json::String),
            Some(b'[') => self.array(),
            _ => Err(KernelError::new(
                "revision wire admits only arrays and strings",
            )),
        }
    }
    fn array(&mut self) -> Result<Json> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Json::Array(values));
        }
        loop {
            values.push(self.value()?);
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Json::Array(values));
                }
                _ => return Err(KernelError::new("invalid JSON array")),
            }
        }
    }
    fn string(&mut self) -> Result<String> {
        self.expect(b'\"')?;
        let mut output = String::new();
        loop {
            let byte = self
                .next()
                .ok_or_else(|| KernelError::new("unterminated JSON string"))?;
            match byte {
                b'\"' => return Ok(output),
                b'\\' => match self
                    .next()
                    .ok_or_else(|| KernelError::new("truncated JSON escape"))?
                {
                    b'\"' => output.push('"'),
                    b'\\' => output.push('\\'),
                    b'n' => output.push('\n'),
                    b'r' => output.push('\r'),
                    b't' => output.push('\t'),
                    b'u' => {
                        let hex = self.take(4)?;
                        let text = std::str::from_utf8(hex)
                            .map_err(|_| KernelError::new("invalid JSON unicode escape"))?;
                        let scalar = u32::from_str_radix(text, 16)
                            .map_err(|_| KernelError::new("invalid JSON unicode escape"))?;
                        output.push(
                            char::from_u32(scalar)
                                .ok_or_else(|| KernelError::new("invalid JSON unicode escape"))?,
                        );
                    }
                    _ => return Err(KernelError::new("invalid JSON escape")),
                },
                0..=0x1f => return Err(KernelError::new("control character in JSON string")),
                _ if byte < 0x80 => output.push(byte as char),
                _ => {
                    let length = utf8_width(byte)
                        .ok_or_else(|| KernelError::new("invalid UTF-8 in JSON string"))?;
                    let mut encoded = vec![byte];
                    encoded.extend_from_slice(self.take(length - 1)?);
                    output.push_str(
                        std::str::from_utf8(&encoded)
                            .map_err(|_| KernelError::new("invalid UTF-8 in JSON string"))?,
                    );
                }
            }
        }
    }
    fn peek(&self) -> Option<u8> {
        self.input.get(self.at).copied()
    }
    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.at += 1;
        Some(byte)
    }
    fn expect(&mut self, expected: u8) -> Result<()> {
        if self.next() == Some(expected) {
            Ok(())
        } else {
            Err(KernelError::new("invalid JSON"))
        }
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(count)
            .ok_or_else(|| KernelError::new("truncated JSON"))?;
        let result = self
            .input
            .get(self.at..end)
            .ok_or_else(|| KernelError::new("truncated JSON"))?;
        self.at = end;
        Ok(result)
    }
}
fn utf8_width(first: u8) -> Option<usize> {
    match first {
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut data = bytes.to_vec();
    data.push(0x80);
    while (data.len() % 64) != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    for block in data.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, chunk) in block.chunks_exact(4).enumerate().take(16) {
            words[index] = u32::from_be_bytes(chunk.try_into().expect("four bytes"));
        }
        for index in 16..64 {
            words[index] = words[index - 16]
                .wrapping_add(
                    words[index - 15].rotate_right(7)
                        ^ words[index - 15].rotate_right(18)
                        ^ (words[index - 15] >> 3),
                )
                .wrapping_add(words[index - 7])
                .wrapping_add(
                    words[index - 2].rotate_right(17)
                        ^ words[index - 2].rotate_right(19)
                        ^ (words[index - 2] >> 10),
                );
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (
            state[0], state[1], state[2], state[3], state[4], state[5], state[6], state[7],
        );
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        claim_output, intent_output, reload, require_output, semantic_payload, serialize,
        sha256_hex,
    };
    use crate::kernel::{
        Cardinality, Clause, Intent, Law, Mode, Model, Relation, Role, Sentence, Term, claim,
        intent, require,
    };

    fn model(variable: &str) -> Model {
        let relation = Relation::new(
            "catalog/contains",
            vec![
                Role::new("set", "Text").unwrap(),
                Role::new("member", "Text").unwrap(),
            ],
            Sentence::new("set", "contains", "member").unwrap(),
            vec![
                Mode::finite(vec!["set".into()], vec!["member".into()], Cardinality::Many).unwrap(),
            ],
        )
        .unwrap();
        let fact = |member: &str| {
            Clause::new(
                "catalog/contains",
                vec![
                    ("set".into(), Term::literal("letters").unwrap()),
                    ("member".into(), Term::literal(member).unwrap()),
                ],
            )
            .unwrap()
        };
        let query = Clause::new(
            "catalog/contains",
            vec![
                ("set".into(), Term::literal("letters").unwrap()),
                ("member".into(), Term::variable(variable).unwrap()),
            ],
        )
        .unwrap();
        let desired = Clause::new(
            "catalog/contains",
            vec![
                ("set".into(), Term::literal("letters").unwrap()),
                ("member".into(), Term::literal("c").unwrap()),
            ],
        )
        .unwrap();
        Model::with_intents(
            vec![relation],
            vec![fact("b"), fact("a")],
            query,
            vec![Intent::new("catalog/restock", desired).unwrap()],
            "ascending",
        )
        .unwrap()
    }

    fn law_model() -> Model {
        let base = model("member");
        let pattern = |set: &str, member: &str| {
            Clause::new(
                "catalog/contains",
                vec![
                    ("set".into(), Term::variable(set).unwrap()),
                    ("member".into(), Term::variable(member).unwrap()),
                ],
            )
            .unwrap()
        };
        let law = |name: &str| {
            Law::new(
                name,
                vec![pattern("set", "member"), pattern("member", "set")],
                pattern("set", "member"),
            )
            .unwrap()
        };
        Model::with_laws_and_intents(
            base.relations().values().cloned().collect(),
            base.facts().to_vec(),
            vec![law("catalog/zeta"), law("catalog/alpha")],
            base.query().clone(),
            base.intents().to_vec(),
            base.order(),
        )
        .unwrap()
    }

    #[test]
    fn sha256_matches_the_standard_vector() {
        assert_eq!(
            super::sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sealed_wire_roundtrips_and_rejects_every_tamper_class() {
        let revision = crate::kernel::Revision::admit(model("member"));
        let semantic = semantic_payload(revision.model());
        let expected_semantic = "[\"clause-semantic-v4\",[\"relations\",[[\"relation\",\"catalog/contains\",\"roles\",[[\"member\",\"Text\"],[\"set\",\"Text\"]],\"sentence\",[\"set\",\"contains\",\"member\"],\"modes\",[[\"mode\",\"finite\",\"known\",[\"set\"],\"sought\",[\"member\"],\"cardinality\",\"many\"]]]]], [\"facts\",[[\"fact\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"a\"]],[\"set\",[\"literal\",\"letters\"]]]],[\"fact\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"b\"]],[\"set\",[\"literal\",\"letters\"]]]]]], [\"laws\",[]], [\"query\",[\"query\",\"catalog/contains\",\"roles\",[[\"member\",[\"variable\",\"member\"]],[\"set\",[\"literal\",\"letters\"]]]]], [\"intents\",[[\"intent\",\"catalog/restock\",\"desired\",[\"clause\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]]]], [\"order\",\"ascending\",\"member\"]]".replace("], [", "],[");
        assert_eq!(semantic, expected_semantic);
        assert_eq!(
            revision.identity(),
            "rev-sha256-48bd94194c0a888bbc2b373d1c0babffa6668650b4de860a2f7376ddefc426a8"
        );
        let wire = serialize(&revision);
        assert_eq!(reload(&wire).unwrap(), revision);
        assert!(reload(&(" ".to_owned() + &wire)).is_err(), "changed bytes");
        assert!(reload(&(wire.clone() + " ")).is_err(), "changed bytes");
        assert!(
            reload(&wire.replacen("\"a\"", "\"z\"", 1)).is_err(),
            "changed role binding"
        );
        assert!(
            reload(&wire.replacen("rev-sha256-", "rev-sha256-a", 1)).is_err(),
            "changed claimed identity"
        );
        assert!(
            reload(&wire.replacen("catalog/restock", "catalog/missing", 1)).is_err(),
            "changed intent name"
        );
        assert!(
            reload(&wire.replacen("[\"literal\",\"c\"]", "[\"literal\",\"d\"]", 1)).is_err(),
            "changed desired term"
        );
        assert!(
            reload(&wire.replacen("[\"relations\"", "[\"facts\"", 1)).is_err(),
            "changed canonical array order"
        );
    }

    #[test]
    fn laws_are_canonical_strictly_reloaded_hashed_and_preserved_by_claim() {
        let model_with_laws = law_model();
        assert_eq!(
            model_with_laws
                .laws()
                .iter()
                .map(Law::name)
                .collect::<Vec<_>>(),
            vec!["catalog/alpha", "catalog/zeta"]
        );
        assert_eq!(
            model_with_laws.laws()[0].premises()[0].roles()["set"].text(),
            "set",
            "law premise order is semantic"
        );
        assert_eq!(
            model_with_laws.laws()[0].premises()[1].roles()["set"].text(),
            "member",
            "law premise order is preserved"
        );

        let revision = crate::kernel::Revision::admit(model("member"));
        let law_revision = crate::kernel::Revision::admit(model_with_laws);
        assert_ne!(law_revision.identity(), revision.identity());
        let semantic = semantic_payload(law_revision.model());
        let alpha = super::law_json(&law_revision.model().laws()[0]);
        let zeta = super::law_json(&law_revision.model().laws()[1]);
        assert!(semantic.find(&alpha).unwrap() < semantic.find(&zeta).unwrap());

        let wire = serialize(&law_revision);
        assert!(wire.starts_with("[\"clause-revision-v2\""));
        assert_eq!(reload(&wire).unwrap(), law_revision);
        assert!(
            reload(&wire.replacen("catalog/alpha", "catalog/changed", 1)).is_err(),
            "law tampering changes revision identity"
        );
        assert!(
            reload(&wire.replacen("clause-revision-v2", "clause-revision-v1", 1)).is_err(),
            "only the current revision envelope is admitted"
        );

        let noncanonical = semantic.replacen(
            &format!("[\"laws\",[{alpha},{zeta}]]"),
            &format!("[\"laws\",[{zeta},{alpha}]]"),
            1,
        );
        let claimed = format!("rev-sha256-{}", sha256_hex(noncanonical.as_bytes()));
        assert!(
            reload(&format!(
                "[\"clause-revision-v2\",\"{claimed}\",{noncanonical}]"
            ))
            .is_err(),
            "reload rejects noncanonical law order even with a recomputed identity"
        );

        let branch = crate::kernel::Branch::new("catalog", law_revision.clone()).unwrap();
        let fact = Clause::new(
            "catalog/contains",
            vec![
                ("set".into(), Term::literal("letters").unwrap()),
                ("member".into(), Term::literal("c").unwrap()),
            ],
        )
        .unwrap();
        let claimed = claim(&branch, fact).unwrap();
        assert_eq!(
            claimed.successor().unwrap().revision().model().laws(),
            law_revision.model().laws(),
            "an immutable claim preserves every law"
        );
    }

    #[test]
    fn reload_rejects_recomputed_intent_outside_relation_namespace() {
        let revision = crate::kernel::Revision::admit(model("member"));
        let semantic =
            semantic_payload(revision.model()).replace("catalog/restock", "other/restock");
        let identity = format!("rev-sha256-{}", sha256_hex(semantic.as_bytes()));
        let wire = format!("[\"clause-revision-v2\",\"{identity}\",{semantic}]");

        assert!(
            reload(&wire).is_err(),
            "recomputed identity must not bypass intent namespace admission"
        );
    }

    #[test]
    fn order_names_the_sought_role_not_the_variable_token() {
        assert!(
            semantic_payload(&model("answer")).ends_with("[\"order\",\"ascending\",\"member\"]]")
        );
    }

    #[test]
    fn claim_and_require_are_immutable_and_emit_exact_arrays() {
        let revision = crate::kernel::Revision::admit(model("member"));
        let branch = crate::kernel::Branch::new("catalog", revision.clone()).unwrap();
        let fact = |member: &str| {
            Clause::new(
                "catalog/contains",
                vec![
                    ("set".into(), Term::literal("letters").unwrap()),
                    ("member".into(), Term::literal(member).unwrap()),
                ],
            )
            .unwrap()
        };
        let c = fact("c");

        let missing = require(&revision, c.clone()).unwrap();
        assert_eq!(
            require_output(&missing),
            format!(
                "[\"clause-require-output-v1\",\"unsatisfied\",[\"revision\",\"{}\"],[\"clause\",[\"clause\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]],[\"diagnostic\",\"require.unsatisfied\"]]",
                revision.identity()
            )
        );

        let admitted = claim(&branch, c.clone()).unwrap();
        let successor = admitted.successor().unwrap();
        assert_eq!(
            branch.revision(),
            &revision,
            "claim changed its input branch"
        );
        assert_eq!(
            revision.model().facts().len(),
            2,
            "claim changed its base revision"
        );
        assert_eq!(successor.revision().model().facts().len(), 3);
        assert_eq!(
            semantic_payload(successor.revision().model()),
            "[\"clause-semantic-v4\",[\"relations\",[[\"relation\",\"catalog/contains\",\"roles\",[[\"member\",\"Text\"],[\"set\",\"Text\"]],\"sentence\",[\"set\",\"contains\",\"member\"],\"modes\",[[\"mode\",\"finite\",\"known\",[\"set\"],\"sought\",[\"member\"],\"cardinality\",\"many\"]]]]],[\"facts\",[[\"fact\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"a\"]],[\"set\",[\"literal\",\"letters\"]]]],[\"fact\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"b\"]],[\"set\",[\"literal\",\"letters\"]]]],[\"fact\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]]],[\"laws\",[]],[\"query\",[\"query\",\"catalog/contains\",\"roles\",[[\"member\",[\"variable\",\"member\"]],[\"set\",[\"literal\",\"letters\"]]]]],[\"intents\",[[\"intent\",\"catalog/restock\",\"desired\",[\"clause\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]]]],[\"order\",\"ascending\",\"member\"]]"
        );
        assert_eq!(
            claim_output(&admitted),
            format!(
                "[\"clause-claim-output-v1\",\"admitted\",[\"branch\",\"catalog\"],[\"base\",\"{}\"],[\"revision\",\"{}\"],[\"fact\",[\"fact\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]]]",
                revision.identity(),
                successor.revision().identity()
            )
        );

        let duplicate = claim(&branch, fact("a")).unwrap();
        assert!(duplicate.successor().is_none());
        assert_eq!(
            claim_output(&duplicate),
            format!(
                "[\"clause-claim-output-v1\",\"duplicate\",[\"branch\",\"catalog\"],[\"revision\",\"{}\"],[\"diagnostic\",\"claim.duplicate\"]]",
                revision.identity()
            )
        );

        let satisfied = require(successor.revision(), c).unwrap();
        assert_eq!(
            require_output(&satisfied),
            format!(
                "[\"clause-require-output-v1\",\"satisfied\",[\"revision\",\"{}\"],[\"proof\",[\"proof\",\"proof/{}/catalog/contains/member=c,set=letters\",\"relation\",\"catalog/contains\",\"roles\",[[\"member\",\"c\"],[\"set\",\"letters\"]]]]]",
                successor.revision().identity(),
                successor.revision().identity()
            )
        );
    }

    #[test]
    fn intent_is_immutable_and_emits_exact_arrays() {
        const BASE: &str =
            "rev-sha256-48bd94194c0a888bbc2b373d1c0babffa6668650b4de860a2f7376ddefc426a8";
        const NEXT: &str =
            "rev-sha256-67569fe238751531a561f2202f263ce961a0a70c2f24fe1e2e9210c5414ee910";
        let revision = crate::kernel::Revision::admit(model("member"));
        let branch = crate::kernel::Branch::new("catalog", revision.clone()).unwrap();
        assert_eq!(revision.identity(), BASE);

        let proposed = intent(&branch, "catalog/restock");
        assert_eq!(
            branch.revision(),
            &revision,
            "intent changed its input branch"
        );
        assert_eq!(
            proposed.revision(),
            &revision,
            "proposal changed its revision"
        );
        assert_eq!(
            intent_output(&proposed),
            format!(
                "[\"clause-intent-output-v1\",\"proposed\",[\"revision\",\"{BASE}\"],[\"intent\",\"catalog/restock\"],[\"desired\",[\"clause\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]],[\"plan\",[\"plan\",\"plan/{BASE}/catalog/restock\",\"operation\",\"claim\",\"base\",\"{BASE}\",\"fact\",[\"fact\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]]],[\"explanation\",[\"explanation\",\"desired-clause-is-absent\",\"revision\",\"{BASE}\",\"clause\",[\"clause\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]],\"diagnostic\",\"require.unsatisfied\"]]]"
            )
        );
        let unknown = intent(&branch, "catalog/missing");
        assert_eq!(unknown.revision(), &revision);
        assert_eq!(
            intent_output(&unknown),
            format!(
                "[\"clause-intent-output-v1\",\"rejected\",[\"revision\",\"{BASE}\"],[\"intent\",\"catalog/missing\"],[\"diagnostic\",\"intent.unknown\"]]"
            )
        );

        let desired = proposed.intent().unwrap().desired().clone();
        let admitted = claim(&branch, desired).unwrap();
        let successor = admitted.successor().unwrap();
        assert_eq!(
            branch.revision(),
            &revision,
            "claim changed its input branch"
        );
        assert_eq!(
            revision.model().facts().len(),
            2,
            "claim changed its base revision"
        );
        assert_eq!(successor.revision().identity(), NEXT);
        assert_eq!(successor.revision().model().intents().len(), 1);

        let satisfied = intent(successor, "catalog/restock");
        assert_eq!(satisfied.revision(), successor.revision());
        assert_eq!(
            intent_output(&satisfied),
            format!(
                "[\"clause-intent-output-v1\",\"already-satisfied\",[\"revision\",\"{NEXT}\"],[\"intent\",\"catalog/restock\"],[\"desired\",[\"clause\",\"catalog/contains\",\"roles\",[[\"member\",[\"literal\",\"c\"]],[\"set\",[\"literal\",\"letters\"]]]]],[\"proof\",[\"proof\",\"proof/{NEXT}/catalog/contains/member=c,set=letters\",\"relation\",\"catalog/contains\",\"roles\",[[\"member\",\"c\"],[\"set\",\"letters\"]]]],[\"explanation\",[\"explanation\",\"desired-clause-is-claimed\",\"revision\",\"{NEXT}\"]]]"
            )
        );
    }
}
