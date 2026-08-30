use std::collections::TryReserveError;

use sha2::{Digest, Sha256};

use super::codec::{
    canonical_term_bytes, canonical_value_bytes, encode_core_manifest_value,
    encode_physical_profile_value,
};
use super::{
    CoreManifest, EncodeError, Hash32, Id32, KSort, KValue, NamedSignature, PhysicalOperation,
    PhysicalProfile, ResourceLimit, RuleSignature, Term, try_copy_bytes,
};

fn reserve(result: Result<(), TryReserveError>) -> Result<(), EncodeError> {
    match result {
        Ok(()) => Ok(()),
        Err(_) => Err(EncodeError::ResourceExhausted),
    }
}

macro_rules! try_results_vec {
    ($($value:expr),* $(,)?) => {{
        let mut values = Vec::new();
        let count = <[()]>::len(&[$(try_results_vec!(@unit $value)),*]);
        reserve(values.try_reserve_exact(count))?;
        $(values.push($value?);)*
        values
    }};
    (@unit $value:expr) => {{
        let _ = stringify!($value);
    }};
}

fn copy_bytes(value: &[u8]) -> Result<Vec<u8>, EncodeError> {
    match try_copy_bytes(value) {
        Ok(bytes) => Ok(bytes),
        Err(ResourceLimit) => Err(EncodeError::ResourceExhausted),
    }
}

fn named(tag: u8, signature: &str) -> Result<NamedSignature, EncodeError> {
    Ok(NamedSignature {
        tag,
        signature: copy_bytes(signature.as_bytes())?,
    })
}

fn rule(tag: u8, premise_policy: u8, clause: &str) -> Result<RuleSignature, EncodeError> {
    Ok(RuleSignature {
        tag,
        premise_policy,
        clause: copy_bytes(clause.as_bytes())?,
    })
}

fn clauses(values: &[&str]) -> Result<Vec<Vec<u8>>, EncodeError> {
    let mut clauses = Vec::new();
    reserve(clauses.try_reserve_exact(values.len()))?;
    for value in values {
        clauses.push(copy_bytes(value.as_bytes())?);
    }
    Ok(clauses)
}

fn tag_range(first: u8, last: u8) -> Result<Vec<u8>, EncodeError> {
    let width = last
        .checked_sub(first)
        .and_then(|width| usize::from(width).checked_add(1))
        .ok_or(EncodeError::ResourceExhausted)?;
    let mut tags = Vec::new();
    reserve(tags.try_reserve_exact(width))?;
    for tag in first..=last {
        tags.push(tag);
    }
    Ok(tags)
}

fn singleton<T>(value: T) -> Result<Vec<T>, EncodeError> {
    let mut values = Vec::new();
    reserve(values.try_reserve_exact(1))?;
    values.push(value);
    Ok(values)
}

impl CoreManifest {
    /// The one exact `CoreManifestV1` value fixed by the accepted P1 contract.
    #[must_use]
    pub fn canonical_v1() -> Self {
        Self::try_canonical_v1().expect("fixed CoreManifestV1 allocation")
    }

    pub(crate) fn try_canonical_v1() -> Result<Self, EncodeError> {
        let expression_forms = try_results_vec![
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
        let abi_forms = try_results_vec![
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
        let static_rules = try_results_vec![
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
        let evaluation_rules = try_results_vec![
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
            "C01: Term=00 Atom(kind:Blob,payload:Blob,equality:Blob)|01 Triple(first:Term,second:Term,third:Term); KSort=00 Bytes|01 Term; frameTags,termTags,sortTags,expressionForms,abiForms,premisePolicyTags,lineageTags,nominalDeclarationTags,compilerEvidenceTags,valueTags,decodeVerdictTags,decodeCodeTags,authorizationStageTags,authorizationCodeTags,staticRules,evaluationRules,receiptFormatVersion and physical profile values above are the complete closed tag sets and signatures",
            "C02: KTag=clause/core-abi/tag/v1; KBytes=clause/core-abi/bytes/v1; KId32=clause/core-abi/id32/v1; KU64=clause/core-abi/u64/v1; KEq=clause/core/bytes-equal/v1; Tag(t)=Atom(KTag,U8(t),KEq); Bytes(b)=Atom(KBytes,b,KEq); Id(id)=Atom(KId32,id,KEq) iff len(id)=32; Nat64(n)=Atom(KU64,U64(n),KEq); List([])=Tag(00); List(x::xs)=Triple(Tag(01),x,List(xs)); Record(t,xs)=Triple(Tag(t),List(xs),Tag(00)); Core ABI constructors and field counts are exactly abiForms in tag order; wrong Atom kind field count wrapper fixed width list shape or trailing field is invalid",
            "C03: CompilerSubject=lineage,nominalDeclarations,interface,program,buildRequest; lineage=00 Genesis|01 Successor(predecessorLocator:Hash32,changeOccurrenceId:Id32); interface=compile:Id32,admitPropose:Id32; Definition=id:Id32,arguments:Seq<KSort>,result:KSort,body:KExpr; definitions are sorted unique by id",
            "C04: NominalDeclaration=00 Seed(domain,id)|01 RetainedSeed(domain,id,predecessorRevisionId)|02 Allocated(domain,id,changeInput:NominalWireRef,producerInput:NominalWireRef,localSlot:U64); NominalWireRef=domain:Id32||id:Id32; declarations are sorted unique by domain||id and every nominal reference resolves exactly one declaration in its required domain",
            "C05: Seed is literal primitive provenance; RetainedSeed must match predecessor Seed or RetainedSeed and exact predecessor revision and cannot relabel Allocated; Allocated.id=DH(clause/new-nominal/v1,domain,wire(changeInput),wire(producerInput),U64(localSlot)); allocation inputs resolve and form an acyclic graph; dependency order then domain||id is the unique recomputation order; collision is invalid",
            "C06: IdentityPlan has separately sorted unique Retain(NominalRef) and SeedInput(NominalRef) lists; every successor RetainedSeed appears only in retained; every newly introduced successor Seed appears only in seedInputs; each row matches declaration provenance; genesis retained is empty; no reference appears in both lists",
            "C07: Delta is the canonical sorted unique definition table; Gamma and runtime environments use index-zero-first Var order; a definition is well formed iff its body has its declared result under its declared argument sorts and all transitive Call and Request references resolve; there is no subsorting, coercion, implicit argument, host value, fallback rule, or package-defined rule",
            "C08: J(expression,environment,fuelBefore,observationsBefore)=>(value,fuelAfter,observationsAfter) is the sole successful evaluation judgment; values are only BytesValue or TermValue; fuel is U64; every rule consumes one unit before premises; zero fuel has no judgment; premises run strictly left-to-right and thread exact fuel and observations; integer overflow, bad value sort, unresolved definition, malformed observation, physical failure, or out-of-fuel has no successful judgment",
            "C09: observationPolicy 00 appends exactly one canonical observation for each successful physical Request and none otherwise; observation indices are 0..n-1; the sole operation is Sha256OpId:[Bytes]->Bytes; SHA-256 is FIPS 180-4 over successive eight-bit message units and returns big-endian H0||H1||H2||H3||H4||H5||H6||H7; every other operation or signature is invalid",
            "C10: EvalReceipt is receiptSignature above and exactly 73 bytes: formatVersion 00, expectedValueHash:Hash32, expectedRemainingFuel:U64, expectedObservationsHash:Hash32; it contains no returned value, observations, request, predecessor package, expression, environment, rule, premise, node, graph, trace, or authority; unknown formatVersion is DecodeRejected(06,formatVersionOffset)",
            "C11: EvalRequest is checker-constructed and never encoded; it binds acceptedPredecessorPackageHash=CompilerPackageHash(exact already-accepted predecessor bytes), derived CoreContractId and PhysicalProfileId, exact entrypoint, canonical arguments, and exact nonzero fuel; its expression is Call(entrypoint,map(ValueLiteral,arguments)) under empty environment and Observations([])",
            "C12: Complete deterministic replay under evaluation rules 30..3e is the only receipt verification; success requires DH(clause/eval-receipt-value/v1,canonical actual KValue bytes)=expectedValueHash, actual remaining fuel=expectedRemainingFuel, and DH(clause/eval-receipt-observations/v1,canonical actual Observations Term bytes)=expectedObservationsHash; an unencodable actual value is replay failure and unencodable observations are observation mismatch; faults have no receipt form; an optional trace is diagnostic only and never admission authority",
            "C13: CompilerEvidence=00 GenesisEvidence with no payload|01 SuccessorEvidence(compileReceipt:EvalReceipt,admissionReceipt:EvalReceipt); evidence is never executable compiler meaning and cannot add a Core, evaluation rule, request, trace, or authority",
            "V01: VerifyEvalReceipt first requires receipt formatVersion 00",
            "V02: VerifyEvalReceipt strictly decodes the separately supplied exact predecessor bytes, requires caller-supplied acceptance of those exact bytes, requires request.acceptedPredecessorPackageHash=CompilerPackageHash(exact bytes), requires predecessor Frame01 byte-equal exactCoreManifestBytes, and independently derives CoreContractId and PhysicalProfileId",
            "V03: VerifyEvalReceipt requires both derived IDs equal the checker-constructed request fields, statically checks the predecessor under rules 20..2b, resolves the request entrypoint exactly once, and requires argument sorts equal its signature",
            "V04: VerifyEvalReceipt constructs Call(entrypoint,map(ValueLiteral,arguments)) without receipt input, where only BytesValue maps to BytesLiteral and TermValue maps to TermLiteral",
            "V05: VerifyEvalReceipt completely evaluates that call under empty environment, request fuelLimit, and Observations([]) using only fixed rules 30..3e and the carried physical profile",
            "V06: VerifyEvalReceipt canonicalizes and domain-hashes the actual replayed value and Observations Term and requires exact expectedValueHash, expectedRemainingFuel, and expectedObservationsHash equality; it never uses receipt data to construct either replay",
            "V07: success requires every prior step and uses no graph, trace, callback, theorem name, host rule registry, Boolean evaluator, or package rule",
            "D00: StrictDecode returns only Decoded(exactInput,candidate) or DecodeRejected(code,offset); codes in precedence order are 00 WrongMagic,01 UnknownVersion,02 FrameTagOrderOrCount,03 Truncated,04 LengthOrCountOverflow,05 InvalidFixedWidth,06 UnknownSumTag,07 BoundedValueUnderConsumed,08 BoundedValueOverConsumed,09 TrailingBytes; fields are read depth-first in encoded order and equal-offset ties use lower code",
            "D01: StrictDecode handles only closed byte grammar; order, uniqueness, exact manifest equality, reference bounds, ABI meaning, entrypoint signature, identity derivation, lineage/evidence consistency, receipt replay semantics, and profile conformance are authorization checks; malformed bytes never produce Unauthorized",
            "A00: Authorization starts only after Decoded(exactInput,Q) and requires exactly one explicit request: GenesisAuthorizationRequest(ownerAnchor,R,E,Gc,Ga,I) or SuccessorAuthorizationRequest(P,R,E,I), where ownerAnchor=Missing|Supplied(OwnerAnchorWitness), OwnerAnchorWitness is an opaque non-package-wire capability created only by the external human-owner selection act, observe(witness)=OwnerAnchorObservation(exactSelectedBytes:Blob,selectedByteLength:U64,selectedPackageHash:Hash32), Gc and Ga are U64, and I=FinalPackageIdentityInput(packageHash:Hash32,exactPackageBytes:Blob); no owner-anchor variant, witness, or observation is encoded in Q; the request variant, never candidate data, selects the route; stages run 40..48; successor skips 42; genesis skips 43,45,46,47; both run 44 and 48; each row condition belongs to exactly one stage and route; rows run left-to-right and collection failures use encoded item order; failure at position i means every earlier condition passed and condition i is false, so first-failure predicates are pairwise disjoint and the first false condition is the only verdict",
            "A40: CoreManifest rows=[manifest bytes differ exactCoreManifestBytes->(40,60)]",
            "A41: CoreWellFormedness rows=[subject or ABI semantic structure->(41,61),nominal provenance allocation retention or reference->(41,62),definition order or duplicate->(41,63),compile then admitPropose resolution->(41,64),entrypoints equal->(41,65),compile then admitPropose signature not [Term]->Term->(41,66),other static rule 20..2b->(41,67),Request outside exact profile->(41,68)]",
            "A42: GenesisAnchor rows=[lineage not Genesis->(42,69),supplied E not byte-identical Q.evidence or E not empty GenesisEvidence->(42,6a),ownerAnchor=Missing->(42,6b),ownerAnchor=Supplied(w) and observe(w) is not a self-consistent observation of the complete exact candidate because selectedByteLength!=byteLength(exactSelectedBytes) or selectedPackageHash!=CompilerPackageHash(exactSelectedBytes) or exactSelectedBytes is not octet-for-octet equal exactInput->(42,6c)]; length and hash checks never substitute for the final exact-byte equality or create authority",
            "A43: ExactPredecessor rows=[lineage not Successor->(43,6d),candidate self candidate-basis or candidate-rule authority->(43,6f),supplied predecessor not already accepted including stale revision->(43,6e),locator differs CompilerPackageHash(P)->(43,70),resolved bytes not byte-identical accepted P->(43,71)]",
            "A44: BuildRequest rows=[wrong ABI shape->(44,72),R not byte-identical Q.subject.buildRequest->(44,73),base route or exact base mismatch->(44,74),core ID mismatch->(44,75),profile ID mismatch->(44,76),source order or duplicate->(44,77),source artifact derivation->(44,78),IdentityPlan order uniqueness provenance retention or seed binding->(44,79),request lineage or nominal change occurrence mismatch->(44,7a),declared physical inputs nonempty->(44,7b),on genesis Gc or Ga zero or R.compileFuel!=Gc or R.admissionFuel!=Ga; on successor either R fuel zero->(44,7c)]",
            "A45: CompileEvaluation rows=[evidence or compile receipt shape->(45,7d),no successful complete replay or actual KValue has no canonical encoding->(45,80),DH(clause/eval-receipt-value/v1,canonical actual KValue bytes) differs expectedValueHash->(45,7e),remaining fuel differs expectedRemainingFuel->(45,7f),actual result not Built->(45,81),Built bytes differ Q.subject->(45,82),canonical actual compile Observations Term has no encoding or its DH(clause/eval-receipt-observations/v1,bytes) differs expectedObservationsHash->(45,83)]",
            "A46: AdmissionEvaluation rows=[admission receipt shape->(46,7d),construct admission request from verified actual compile observations then no successful complete replay or actual KValue has no canonical encoding->(46,80),DH(clause/eval-receipt-value/v1,canonical actual KValue bytes) differs expectedValueHash->(46,7e),remaining fuel differs expectedRemainingFuel->(46,7f),actual result not Propose->(46,81),Propose bytes differ Q.subject->(46,82),canonical actual admission Observations Term has no encoding or its DH(clause/eval-receipt-observations/v1,bytes) differs expectedObservationsHash->(46,83)]",
            "A47: EvidenceAttachment rows=[E not byte-identical Q.evidence->(47,84),Frame02 differs certified subject->(47,85),attaching E does not reproduce exact Q->(47,86)]",
            "A48: FinalAuthorization rows=[I.exactPackageBytes not byte-identical exactInput or I.packageHash!=DH(clause/compiler-package/v1,I.exactPackageBytes)->(48,87)]",
            "H00: DH(d,xs)=SHA256(U32(len(d))||ASCII(d)||each(U64(len(x))||x)); CoreContractId=DH(clause/core-contract/v1,exactCoreManifestBytes); PhysicalProfileId=DH(clause/physical-profile/v1,exactPhysicalProfileBytes); EvalReceiptValueHash=DH(clause/eval-receipt-value/v1,canonicalKValueBytes); EvalReceiptObservationsHash=DH(clause/eval-receipt-observations/v1,canonicalObservationsTermBytes); CompilerSemanticsId=DH(clause/compiler-semantics/v1,canonical(interface||program)); CompilerRevisionId=DH(clause/compiler-revision/v1,exactCompilerSubjectBytes); CompilerPackageHash=DH(clause/compiler-package/v1,exactWholePackageBytes); SourceArtifactId=DH(clause/source-artifact/v1,exactSourceBytes); BuildRequestId=DH(clause/compiler-build-request/v1,canonicalTermBytes(BuildRequest)); OriginId=DH(clause/origin/v1,canonicalAcyclicOriginNode); hashes never grant compiler authority",
            "P00: Package bytes are magic CLCP,version 03,Frame(01,CoreManifestV1),Frame(02,CompilerSubject),Frame(03,CompilerEvidence),EOF exactly once in order; Frame03 is excluded from subject and revision identities; successor Frame03 payload is exactly 147 bytes: tag 01 then two ordered 73-byte trace-free receipts; it contains no predecessor bytes, candidate evidence, returned value, observations, or candidate whole-package identity; only exact genesis anchor or separately supplied already-accepted exact predecessor can authorize",
        ])?;

        Ok(Self {
            manifest_version: 0x00,
            frame_tags: copy_bytes(&[0x01, 0x02, 0x03])?,
            term_tags: copy_bytes(&[0x00, 0x01])?,
            sort_tags: copy_bytes(&[0x00, 0x01])?,
            expression_forms,
            abi_forms,
            premise_policy_tags: tag_range(0x00, 0x07)?,
            lineage_tags: copy_bytes(&[0x00, 0x01])?,
            nominal_declaration_tags: copy_bytes(&[0x00, 0x01, 0x02])?,
            compiler_evidence_tags: copy_bytes(&[0x00, 0x01])?,
            value_tags: copy_bytes(&[0x00, 0x01])?,
            decode_verdict_tags: copy_bytes(&[0x00, 0x01])?,
            decode_code_tags: tag_range(0x00, 0x09)?,
            authorization_stage_tags: tag_range(0x40, 0x48)?,
            authorization_code_tags: tag_range(0x60, 0x87)?,
            static_rules,
            evaluation_rules,
            receipt_format_version: 0x00,
            receipt_signature: copy_bytes(b"EvalReceipt(formatVersion:ReceiptFormatVersion,expectedValueHash:Hash32,expectedRemainingFuel:U64,expectedObservationsHash:Hash32); ReceiptFormatVersion=00; expectedValueHash=DH(clause/eval-receipt-value/v1,canonical KValue bytes); expectedObservationsHash=DH(clause/eval-receipt-observations/v1,canonical Term bytes); KValue=00 BytesValue(Blob)|01 TermValue(Term)")?,
            contract_clauses,
            physical_profile: PhysicalProfile::try_sealed_sha256()?,
        })
    }
}

impl PhysicalProfile {
    /// The only admitted compiler physical profile.
    #[must_use]
    pub fn sealed_sha256() -> Self {
        Self::try_sealed_sha256().expect("fixed sealed SHA-256 profile allocation")
    }

    pub(crate) fn try_sealed_sha256() -> Result<Self, EncodeError> {
        Ok(Self {
            profile_version: 0x00,
            observation_policy: 0x00,
            operations: singleton(PhysicalOperation {
                operation_id: sha256_operation_id(),
                arguments: singleton(KSort::Bytes)?,
                result: KSort::Bytes,
            })?,
        })
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
    encode_core_manifest_value(&CoreManifest::try_canonical_v1()?)
}

pub fn exact_physical_profile_bytes() -> Result<Vec<u8>, EncodeError> {
    encode_physical_profile_value(&PhysicalProfile::try_sealed_sha256()?)
}

pub fn core_contract_id() -> Result<Hash32, EncodeError> {
    let bytes = exact_core_manifest_bytes()?;
    Ok(domain_hash("clause/core-contract/v1", &[&bytes]))
}

pub fn physical_profile_id() -> Result<Hash32, EncodeError> {
    let bytes = exact_physical_profile_bytes()?;
    Ok(domain_hash("clause/physical-profile/v1", &[&bytes]))
}

pub fn eval_receipt_value_hash(value: &KValue) -> Result<Hash32, EncodeError> {
    let bytes = canonical_value_bytes(value)?;
    Ok(domain_hash("clause/eval-receipt-value/v1", &[&bytes]))
}

pub fn eval_receipt_observations_hash(observations: &Term) -> Result<Hash32, EncodeError> {
    let bytes = canonical_term_bytes(observations)?;
    Ok(domain_hash(
        "clause/eval-receipt-observations/v1",
        &[&bytes],
    ))
}

#[must_use]
pub fn compiler_package_hash(exact_package_bytes: &[u8]) -> Hash32 {
    domain_hash("clause/compiler-package/v1", &[exact_package_bytes])
}

#[must_use]
pub fn source_artifact_id(exact_source_bytes: &[u8]) -> Hash32 {
    domain_hash("clause/source-artifact/v1", &[exact_source_bytes])
}
