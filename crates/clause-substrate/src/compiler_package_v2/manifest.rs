use sha2::{Digest, Sha256};

use super::codec::{encode_core_manifest_value, encode_physical_profile_value};
use super::{
    CoreManifest, EncodeError, Hash32, Id32, KSort, NamedSignature, PhysicalOperation,
    PhysicalProfile, RuleSignature,
};

fn named(tag: u8, signature: &str) -> NamedSignature {
    NamedSignature {
        tag,
        signature: signature.as_bytes().to_vec(),
    }
}

fn rule(tag: u8, premise_policy: u8, clause: &str) -> RuleSignature {
    RuleSignature {
        tag,
        premise_policy,
        clause: clause.as_bytes().to_vec(),
    }
}

fn clauses(values: &[&str]) -> Vec<Vec<u8>> {
    values
        .iter()
        .map(|value| value.as_bytes().to_vec())
        .collect()
}

impl CoreManifest {
    /// The one exact `CoreManifestV1` value fixed by the accepted P1 contract.
    #[must_use]
    pub fn canonical_v1() -> Self {
        let expression_forms = vec![
            named(0x00, "BytesLiteral(value:Blob)->Bytes"),
            named(0x01, "TermLiteral(value:Term)->Term"),
            named(0x02, "Var(index:U32)->EnvironmentSort"),
            named(
                0x03,
                "MakeAtom(kind:Bytes,payload:Bytes,equality:Bytes)->Term",
            ),
            named(0x04, "MakeTriple(first:Term,second:Term,third:Term)->Term"),
            named(0x05, "Let(value:Any,body:(bind Any) Same)->Same"),
            named(
                0x06,
                "CaseTerm(scrutinee:Term,atomBody:(bind Bytes Bytes Bytes) Same,tripleBody:(bind Term Term Term) Same)->Same",
            ),
            named(
                0x07,
                "CaseBytes(scrutinee:Bytes,emptyBody:Same,consBody:(bind Bytes Bytes) Same)->Same",
            ),
            named(0x08, "ConcatBytes(parts:Seq<Bytes>)->Bytes"),
            named(
                0x09,
                "CaseBytesEqual(left:Bytes,right:Bytes,equalBody:Same,unequalBody:Same)->Same",
            ),
            named(
                0x0a,
                "Call(definition:Id32,arguments:DefinitionArguments)->DefinitionResult",
            ),
            named(
                0x0b,
                "Request(operation:Id32,arguments:PhysicalArguments)->PhysicalResult",
            ),
        ];
        let abi_forms = vec![
            named(0x00, "ListNil()"),
            named(0x01, "ListCons(head:Term,tail:List)"),
            named(0x02, "ValueBytes(value:Bytes)"),
            named(0x03, "ValueTerm(value:Term)"),
            named(0x04, "NominalRef(domain:Id32,id:Id32)"),
            named(0x05, "FixedId(domain:Id32,id:Id32)"),
            named(0x06, "ContentId(domain:Id32,id:Id32)"),
            named(0x07, "DerivedId(domain:Id32,id:Id32)"),
            named(
                0x08,
                "IdentityPlan(retained:List<Retain>,seedInputs:List<SeedInput>)",
            ),
            named(0x09, "Retain(ref:NominalRef)"),
            named(0x0a, "SeedInput(ref:NominalRef)"),
            named(0x10, "GenesisBase()"),
            named(0x11, "AcceptedBase(packageHash:Hash32,revisionId:Id32)"),
            named(
                0x12,
                "SourceUnit(unitId:Id32,artifactId:Hash32,bytes:Bytes)",
            ),
            named(
                0x13,
                "BuildRequest(base:GenesisBase|AcceptedBase,coreContractId:Hash32,physicalProfileId:Hash32,targetProfile:Term,sourceUnits:List<SourceUnit>,baseInputs:Term,identityRetentions:IdentityPlan,changeOccurrenceId:Id32,options:Term,compileFuel:U64,admissionFuel:U64,declaredPhysicalInputs:List<Term>)",
            ),
            named(0x14, "Built(subjectBytes:Bytes)"),
            named(0x15, "Rejected(diagnostics:List<Term>)"),
            named(
                0x16,
                "AdmissionRequest(buildRequest:BuildRequest,subjectBytes:Bytes,compileObservations:Observations)",
            ),
            named(0x17, "Propose(subjectBytes:Bytes)"),
            named(0x18, "Reject(diagnostics:List<Term>)"),
            named(
                0x19,
                "Observation(index:U64,operationId:Id32,arguments:List<KValue>,result:KValue)",
            ),
            named(0x1a, "Observations(items:List<Observation>)"),
            named(0x1b, "Authorized(packageBytes:Bytes)"),
            named(0x1c, "Unauthorized(stage:U8,code:U8)"),
        ];
        let static_rules = vec![
            rule(0x20, 0x00, "Delta;Gamma |- BytesLiteral(b):Bytes"),
            rule(0x21, 0x00, "Delta;Gamma |- TermLiteral(t):Term"),
            rule(
                0x22,
                0x00,
                "Delta;Gamma |- Var(i):Gamma[i] iff i<len(Gamma)",
            ),
            rule(
                0x23,
                0x03,
                "Delta;Gamma |- MakeAtom(k,p,q):Term iff k:Bytes and p:Bytes and q:Bytes",
            ),
            rule(
                0x24,
                0x03,
                "Delta;Gamma |- MakeTriple(a,b,c):Term iff a:Term and b:Term and c:Term",
            ),
            rule(
                0x25,
                0x02,
                "Delta;Gamma |- Let(v,b):r iff Delta;Gamma |- v:s and Delta;[s]++Gamma |- b:r",
            ),
            rule(
                0x26,
                0x03,
                "Delta;Gamma |- CaseTerm(s,a,t):r iff s:Term and Delta;[Bytes,Bytes,Bytes]++Gamma |- a:r and Delta;[Term,Term,Term]++Gamma |- t:r",
            ),
            rule(
                0x27,
                0x03,
                "Delta;Gamma |- CaseBytes(s,e,c):r iff s:Bytes and Delta;Gamma |- e:r and Delta;[Bytes,Bytes]++Gamma |- c:r",
            ),
            rule(
                0x28,
                0x05,
                "Delta;Gamma |- ConcatBytes(es):Bytes iff every es[i]:Bytes in encoded order",
            ),
            rule(
                0x29,
                0x04,
                "Delta;Gamma |- CaseBytesEqual(a,b,e,n):r iff a:Bytes and b:Bytes and e:r and n:r",
            ),
            rule(
                0x2a,
                0x06,
                "Delta;Gamma |- Call(d,args):r iff Delta contains exactly d:(ss)->r and len(args)=len(ss) and every args[i]:ss[i] in encoded order",
            ),
            rule(
                0x2b,
                0x06,
                "Delta;Gamma |- Request(op,args):r iff physicalProfile contains exactly op:(ss)->r and len(args)=len(ss) and every args[i]:ss[i] in encoded order",
            ),
        ];
        let evaluation_rules = vec![
            rule(
                0x30,
                0x00,
                "J(BytesLiteral(b),g,f,o)=>(BytesValue(b),f-1,o) iff f>0",
            ),
            rule(
                0x31,
                0x00,
                "J(TermLiteral(t),g,f,o)=>(TermValue(t),f-1,o) iff f>0",
            ),
            rule(
                0x32,
                0x00,
                "J(Var(i),g,f,o)=>(g[i],f-1,o) iff f>0 and i<len(g)",
            ),
            rule(
                0x33,
                0x03,
                "after charge evaluate k,p,q left-to-right as BytesValue(kb),BytesValue(pb),BytesValue(qb); return TermValue(Atom(kb,pb,qb)) with final fuel and observations",
            ),
            rule(
                0x34,
                0x03,
                "after charge evaluate a,b,c left-to-right as TermValue(av),TermValue(bv),TermValue(cv); return TermValue(Triple(av,bv,cv)) with final fuel and observations",
            ),
            rule(
                0x35,
                0x02,
                "after charge evaluate v to x, then evaluate b under [x]++g; return the body value, fuel, and observations",
            ),
            rule(
                0x36,
                0x02,
                "after charge evaluate s to TermValue(Atom(k,p,q)), then evaluate atomBody under [BytesValue(k),BytesValue(p),BytesValue(q)]++g; return the selected body outcome",
            ),
            rule(
                0x37,
                0x02,
                "after charge evaluate s to TermValue(Triple(a,b,c)), then evaluate tripleBody under [TermValue(a),TermValue(b),TermValue(c)]++g; return the selected body outcome",
            ),
            rule(
                0x38,
                0x02,
                "after charge evaluate s to BytesValue(empty), then evaluate emptyBody under g; return the selected body outcome",
            ),
            rule(
                0x39,
                0x02,
                "after charge evaluate s to BytesValue(head++tail) with len(head)=1, then evaluate consBody under [BytesValue(head),BytesValue(tail)]++g; return the selected body outcome",
            ),
            rule(
                0x3a,
                0x05,
                "after charge evaluate es left-to-right as BytesValue parts and return BytesValue(concat(parts)); empty es returns empty Bytes with post-charge fuel and unchanged observations",
            ),
            rule(
                0x3b,
                0x03,
                "after charge evaluate a then b as BytesValue and iff lengths and octets are equal evaluate equalBody under g; return the selected body outcome",
            ),
            rule(
                0x3c,
                0x03,
                "after charge evaluate a then b as BytesValue and iff lengths or octets differ evaluate unequalBody under g; return the selected body outcome",
            ),
            rule(
                0x3d,
                0x07,
                "after charge resolve exactly d, evaluate args left-to-right, then evaluate its body under exactly the argument values with no caller environment; return the body outcome",
            ),
            rule(
                0x3e,
                0x01,
                "after charge evaluate the sole argument as BytesValue(input), compute FIPS-180-4 SHA-256(input), return BytesValue(H0||H1||H2||H3||H4||H5||H6||H7), and append exactly Observation(len(o),Sha256OpId,[Value(Bytes,input)],Value(Bytes,digest))",
            ),
        ];

        let contract_clauses = clauses(&[
            "C00: U8=one octet;U32=four-octet unsigned big-endian;U64=eight-octet unsigned big-endian;Blob=U32 length||octets[length];Seq<X>=U32 count||X[count];Frame<X>=U8 tag||U32 payloadLength||X;Id32 and Hash32 are exactly 32 octets;Span=Id32 sourceArtifactId||U64 start||U64 end with start<=end<=source length;record fields concatenate in displayed order;sum variants begin with displayed U8;all arithmetic is checked before cursor change conversion iteration or allocation;every bounded value consumes exactly;no padding trailing bytes or alternate spelling",
            "C01: Term=00 Atom(kind:Blob,payload:Blob,equality:Blob)|01 Triple(first:Term,second:Term,third:Term); KSort=00 Bytes|01 Term; frameTags,termTags,sortTags,expressionForms,abiForms,premisePolicyTags,lineageTags,nominalDeclarationTags,compilerEvidenceTags,valueTags,evalOutcomeTags,decodeVerdictTags,decodeCodeTags,authorizationStageTags,authorizationCodeTags,staticRules,evaluationRules,certificateFormatVersion and physical profile values above are the complete closed tag sets and signatures",
            "C02: KTag=clause/core-abi/tag/v1; KBytes=clause/core-abi/bytes/v1; KId32=clause/core-abi/id32/v1; KU64=clause/core-abi/u64/v1; KEq=clause/core/bytes-equal/v1; Tag(t)=Atom(KTag,U8(t),KEq); Bytes(b)=Atom(KBytes,b,KEq); Id(id)=Atom(KId32,id,KEq) iff len(id)=32; Nat64(n)=Atom(KU64,U64(n),KEq); List([])=Tag(00); List(x::xs)=Triple(Tag(01),x,List(xs)); Record(t,xs)=Triple(Tag(t),List(xs),Tag(00)); Core ABI constructors and field counts are exactly abiForms in tag order; wrong Atom kind field count wrapper fixed width list shape or trailing field is invalid",
            "C03: CompilerSubject=lineage,nominalDeclarations,interface,program,buildRequest; lineage=00 Genesis|01 Successor(predecessorLocator:Hash32,changeOccurrenceId:Id32); interface=compile:Id32,admitPropose:Id32; Definition=id:Id32,arguments:Seq<KSort>,result:KSort,body:KExpr; definitions are sorted unique by id",
            "C04: NominalDeclaration=00 Seed(domain,id)|01 RetainedSeed(domain,id,predecessorRevisionId)|02 Allocated(domain,id,changeInput:NominalWireRef,producerInput:NominalWireRef,localSlot:U64); NominalWireRef=domain:Id32||id:Id32; declarations are sorted unique by domain||id and every nominal reference resolves exactly one declaration in its required domain",
            "C05: Seed is literal primitive provenance; RetainedSeed must match predecessor Seed or RetainedSeed and exact predecessor revision and cannot relabel Allocated; Allocated.id=DH(clause/new-nominal/v1,domain,wire(changeInput),wire(producerInput),U64(localSlot)); allocation inputs resolve and form an acyclic graph; dependency order then domain||id is the unique recomputation order; collision is invalid",
            "C06: IdentityPlan has separately sorted unique Retain(NominalRef) and SeedInput(NominalRef) lists; every successor RetainedSeed appears only in retained; every newly introduced successor Seed appears only in seedInputs; each row matches declaration provenance; genesis retained is empty; no reference appears in both lists",
            "C07: Delta is the canonical sorted unique definition table; Gamma and runtime environments use index-zero-first Var order; a definition is well formed iff its body has its declared result under its declared argument sorts and all transitive Call and Request references resolve; there is no subsorting, coercion, implicit argument, host value, fallback rule, or package-defined rule",
            "C08: J(expression,environment,fuelBefore,observationsBefore)=>(value,fuelAfter,observationsAfter) is the sole successful evaluation judgment; values are only BytesValue or TermValue; fuel is U64; every rule consumes one unit before premises; zero fuel has no judgment; premises run strictly left-to-right and thread exact fuel and observations; integer overflow, bad value sort, unresolved definition, malformed observation, physical failure, or out-of-fuel has no successful judgment",
            "C09: observationPolicy 00 appends exactly one canonical observation for each successful physical Request and none otherwise; observation indices are 0..n-1; the sole operation is Sha256OpId:[Bytes]->Bytes; SHA-256 is FIPS 180-4 over successive eight-bit message units and returns big-endian H0||H1||H2||H3||H4||H5||H6||H7; every other operation or signature is invalid",
            "C10: Certificate format is certificateSignature above with formatVersion 00; nodes are indexed in encoded order; the last node is root; nodes is nonempty; every premise index is earlier than its consumer, appears in execution order, and is unique; every node is reachable from root; unknown ruleTag is DecodeRejected(06,ruleTagOffset), while a known tag with wrong expression, premise, state, value, fuel, environment, digest, or observation semantics is Unauthorized(stage,7f)",
            "C11: A certificate node uses exactly one evaluation rule 30..3e and that rule's premisePolicy; the first premise begins after the parent's one-unit charge at observationsBefore; later premises begin at the prior premise's fuel and observations; Call has argument premises then one body premise under exactly argument values; RequestSha256 has one argument premise then fixed digest and one append; certificate nodes prove neither static well-formedness nor authority",
            "C12: EvalStatement contains complete exact already-accepted predecessor bytes, derived manifest/profile IDs, exact entrypoint, canonical arguments, exact nonzero fuel limit, and Returned value,remainingFuel,Observations; its independently constructed root is Call(entrypoint,map(ValueLiteral,arguments)) under empty environment, statement fuel, and Observations([]); faults have no certificate form",
            "C13: CompilerEvidence=00 GenesisEvidence with no payload|01 SuccessorEvidence(compileCertificate:EvalCertificate,admissionCertificate:EvalCertificate); evidence is never executable compiler meaning and cannot add a Core or certificate rule",
            "V01: VerifyEvalCertificate first requires certificate formatVersion 00 and canonical byte equality of certificate.statement and the required EvalStatement",
            "V02: VerifyEvalCertificate next strictly decodes required.exactAcceptedPredecessor, requires caller-supplied acceptance of those exact bytes, requires predecessor Frame01 byte-equal exactCoreManifestBytes, and independently derives CoreContractId and PhysicalProfileId",
            "V03: VerifyEvalCertificate next requires both derived IDs equal the statement fields, statically checks the predecessor under rules 20..2b, resolves the entrypoint exactly once, and requires argument sorts equal its signature",
            "V04: VerifyEvalCertificate next constructs Call(entrypoint,map(ValueLiteral,arguments)) without certificate input, where only BytesValue maps to BytesLiteral and TermValue maps to TermLiteral",
            "V05: VerifyEvalCertificate next constructs the required root judgment with empty environment, fuelLimit, Observations([]), and exactly the value,remainingFuel,observations in required.expected",
            "V06: VerifyEvalCertificate next scans nodes in encoded order and validates every exact known local rule, premise index, state transition, environment, value, fuel, and observation chain",
            "V07: VerifyEvalCertificate finally requires every node reachable and the final conclusion canonical-byte-equal to the independently constructed root; success requires every prior step and uses no callback, theorem name, host rule registry, Boolean evaluator, or package rule",
            "D00: StrictDecode returns only Decoded(exactInput,candidate) or DecodeRejected(code,offset); codes in precedence order are 00 WrongMagic,01 UnknownVersion,02 FrameTagOrderOrCount,03 Truncated,04 LengthOrCountOverflow,05 InvalidFixedWidth,06 UnknownSumTag,07 BoundedValueUnderConsumed,08 BoundedValueOverConsumed,09 TrailingBytes; fields are read depth-first in encoded order and equal-offset ties use lower code",
            "D01: StrictDecode handles only closed byte grammar; order, uniqueness, exact manifest equality, reference bounds, ABI meaning, entrypoint signature, identity derivation, lineage/evidence consistency, known certificate-rule semantics, and profile conformance are authorization checks; malformed bytes never produce Unauthorized",
            "A00: Authorization starts only after Decoded(exactInput,Q) and requires exactly one explicit request: GenesisAuthorizationRequest(ownerAnchor,R,E,Gc,Ga,I) or SuccessorAuthorizationRequest(P,R,E,I), where ownerAnchor=Missing|Supplied(OwnerAnchorWitness), OwnerAnchorWitness is an opaque non-package-wire capability created only by the external human-owner selection act, observe(witness)=OwnerAnchorObservation(exactSelectedBytes:Blob,selectedByteLength:U64,selectedPackageHash:Hash32), Gc and Ga are U64, and I=FinalPackageIdentityInput(packageHash:Hash32,exactPackageBytes:Blob); no owner-anchor variant, witness, or observation is encoded in Q; the request variant, never candidate data, selects the route; stages run 40..48; successor skips 42; genesis skips 43,45,46,47; both run 44 and 48; each row condition belongs to exactly one stage and route; rows run left-to-right and collection failures use encoded item order; failure at position i means every earlier condition passed and condition i is false, so first-failure predicates are pairwise disjoint and the first false condition is the only verdict",
            "A40: CoreManifest rows=[manifest bytes differ exactCoreManifestBytes->(40,60)]",
            "A41: CoreWellFormedness rows=[subject or ABI semantic structure->(41,61),nominal provenance allocation retention or reference->(41,62),definition order or duplicate->(41,63),compile then admitPropose resolution->(41,64),entrypoints equal->(41,65),compile then admitPropose signature not [Term]->Term->(41,66),other static rule 20..2b->(41,67),Request outside exact profile->(41,68)]",
            "A42: GenesisAnchor rows=[lineage not Genesis->(42,69),supplied E not byte-identical Q.evidence or E not empty GenesisEvidence->(42,6a),ownerAnchor=Missing->(42,6b),ownerAnchor=Supplied(w) and observe(w) is not a self-consistent observation of the complete exact candidate because selectedByteLength!=byteLength(exactSelectedBytes) or selectedPackageHash!=CompilerPackageHash(exactSelectedBytes) or exactSelectedBytes is not octet-for-octet equal exactInput->(42,6c)]; length and hash checks never substitute for the final exact-byte equality or create authority",
            "A43: ExactPredecessor rows=[lineage not Successor->(43,6d),candidate self candidate-basis or candidate-rule authority->(43,6f),supplied predecessor not already accepted including stale revision->(43,6e),locator differs CompilerPackageHash(P)->(43,70),resolved bytes not byte-identical accepted P->(43,71)]",
            "A44: BuildRequest rows=[wrong ABI shape->(44,72),R not byte-identical Q.subject.buildRequest->(44,73),base route or exact base mismatch->(44,74),core ID mismatch->(44,75),profile ID mismatch->(44,76),source order or duplicate->(44,77),source artifact derivation->(44,78),IdentityPlan order uniqueness provenance retention or seed binding->(44,79),request lineage or nominal change occurrence mismatch->(44,7a),declared physical inputs nonempty->(44,7b),on genesis Gc or Ga zero or R.compileFuel!=Gc or R.admissionFuel!=Ga; on successor either R fuel zero->(44,7c)]",
            "A45: CompileEvaluation rows=[evidence or compile certificate shape->(45,7d),statement predecessor manifest profile entrypoint arguments or fuel->(45,7e),known node premise root rule state fuel or observation semantics->(45,7f),no successful judgment->(45,80),result not Built->(45,81),Built bytes differ Q.subject->(45,82),compile observations differ root->(45,83)]",
            "A46: AdmissionEvaluation rows=[certificate shape->(46,7d),statement predecessor manifest profile entrypoint arguments fuel or compile observations->(46,7e),known node premise root rule state fuel or observation semantics->(46,7f),no successful judgment->(46,80),result not Propose->(46,81),proposed bytes differ Q.subject->(46,82),admission observations differ root->(46,83)]",
            "A47: EvidenceAttachment rows=[E not byte-identical Q.evidence->(47,84),Frame02 differs certified subject->(47,85),attaching E does not reproduce exact Q->(47,86)]",
            "A48: FinalAuthorization rows=[I.exactPackageBytes not byte-identical exactInput or I.packageHash!=DH(clause/compiler-package/v1,I.exactPackageBytes)->(48,87)]",
            "H00: DH(d,xs)=SHA256(U32(len(d))||ASCII(d)||each(U64(len(x))||x)); CoreContractId=DH(clause/core-contract/v1,exactCoreManifestBytes); PhysicalProfileId=DH(clause/physical-profile/v1,exactPhysicalProfileBytes); CompilerSemanticsId=DH(clause/compiler-semantics/v1,canonical(interface||program)); CompilerRevisionId=DH(clause/compiler-revision/v1,exactCompilerSubjectBytes); CompilerPackageHash=DH(clause/compiler-package/v1,exactWholePackageBytes); SourceArtifactId=DH(clause/source-artifact/v1,exactSourceBytes); BuildRequestId=DH(clause/compiler-build-request/v1,canonicalTermBytes(BuildRequest)); OriginId=DH(clause/origin/v1,canonicalAcyclicOriginNode); hashes never grant compiler authority",
            "P00: Package bytes are magic CLCP,version 02,Frame(01,CoreManifestV1),Frame(02,CompilerSubject),Frame(03,CompilerEvidence),EOF exactly once in order; Frame03 is excluded from subject and revision identities; successor evidence contains predecessor bytes but never candidate evidence or candidate whole-package identity; only exact genesis anchor or already-accepted exact predecessor can authorize",
        ]);

        Self {
            manifest_version: 0x00,
            frame_tags: vec![0x01, 0x02, 0x03],
            term_tags: vec![0x00, 0x01],
            sort_tags: vec![0x00, 0x01],
            expression_forms,
            abi_forms,
            premise_policy_tags: (0x00..=0x07).collect(),
            lineage_tags: vec![0x00, 0x01],
            nominal_declaration_tags: vec![0x00, 0x01, 0x02],
            compiler_evidence_tags: vec![0x00, 0x01],
            value_tags: vec![0x00, 0x01],
            eval_outcome_tags: vec![0x00],
            decode_verdict_tags: vec![0x00, 0x01],
            decode_code_tags: (0x00..=0x09).collect(),
            authorization_stage_tags: (0x40..=0x48).collect(),
            authorization_code_tags: (0x60..=0x87).collect(),
            static_rules,
            evaluation_rules,
            certificate_format_version: 0x00,
            certificate_signature: b"EvalCertificate(formatVersion:CertificateFormatVersion,statement:EvalStatement,nodes:Seq<EvalNode>); CertificateFormatVersion=00; EvalStatement(exactAcceptedPredecessor:Blob,coreContractId:Hash32,physicalProfileId:Hash32,entrypoint:Id32,arguments:Seq<KValue>,fuelLimit:U64,expected:Returned(value:KValue,remainingFuel:U64,observations:Term)); KValue=00 BytesValue(Blob)|01 TermValue(Term); EvalNode(ruleTag:EvaluationRuleTag,premises:Seq<U32>,conclusion:EvalJudgment); EvaluationRuleTag=30|31|32|33|34|35|36|37|38|39|3a|3b|3c|3d|3e; EvalJudgment(expression:KExpr,environment:Seq<KValue>,fuelBefore:U64,observationsBefore:Term,value:KValue,fuelAfter:U64,observationsAfter:Term)".to_vec(),
            contract_clauses,
            physical_profile: PhysicalProfile::sealed_sha256(),
        }
    }
}

impl PhysicalProfile {
    /// The only admitted compiler physical profile.
    #[must_use]
    pub fn sealed_sha256() -> Self {
        Self {
            profile_version: 0x00,
            observation_policy: 0x00,
            operations: vec![PhysicalOperation {
                operation_id: sha256_operation_id(),
                arguments: vec![KSort::Bytes],
                result: KSort::Bytes,
            }],
        }
    }
}

/// `DH(d, xs)` from the accepted P1 contract.
#[must_use]
pub fn domain_hash(domain: &str, components: &[&[u8]]) -> Hash32 {
    let mut hasher = Sha256::new();
    let domain_bytes = domain.as_bytes();
    let domain_length = u32::try_from(domain_bytes.len()).expect("fixed hash domain fits U32");
    hasher.update(domain_length.to_be_bytes());
    hasher.update(domain_bytes);
    for component in components {
        let length = u64::try_from(component.len()).expect("host slice length fits U64");
        hasher.update(length.to_be_bytes());
        hasher.update(component);
    }
    Hash32(hasher.finalize().into())
}

#[must_use]
pub fn sha256_operation_id() -> Id32 {
    Id32(domain_hash("clause/physical-op/v1", &[b"sha256"]).0)
}

pub fn exact_core_manifest_bytes() -> Result<Vec<u8>, EncodeError> {
    encode_core_manifest_value(&CoreManifest::canonical_v1())
}

pub fn exact_physical_profile_bytes() -> Result<Vec<u8>, EncodeError> {
    encode_physical_profile_value(&PhysicalProfile::sealed_sha256())
}

pub fn core_contract_id() -> Result<Hash32, EncodeError> {
    let bytes = exact_core_manifest_bytes()?;
    Ok(domain_hash("clause/core-contract/v1", &[&bytes]))
}

pub fn physical_profile_id() -> Result<Hash32, EncodeError> {
    let bytes = exact_physical_profile_bytes()?;
    Ok(domain_hash("clause/physical-profile/v1", &[&bytes]))
}

#[must_use]
pub fn compiler_package_hash(exact_package_bytes: &[u8]) -> Hash32 {
    domain_hash("clause/compiler-package/v1", &[exact_package_bytes])
}

#[must_use]
pub fn source_artifact_id(exact_source_bytes: &[u8]) -> Hash32 {
    domain_hash("clause/source-artifact/v1", &[exact_source_bytes])
}
