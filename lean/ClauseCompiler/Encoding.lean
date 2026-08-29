import ClauseCompiler.Model

/-! Canonical CLCP-v2 encoders and the fixed FIPS-180-4 SHA-256 mechanic. -/

namespace ClauseCompiler

def ascii (value : String) : Bytes := value.toUTF8.toList

namespace Encoding

def u32Maximum : Nat := 4294967295
def u64Maximum : Nat := 18446744073709551615

def u32 (value : Nat) : Option Bytes :=
  if value ≤ u32Maximum then
    some [UInt8.ofNat (value / 16777216),
      UInt8.ofNat ((value / 65536) % 256),
      UInt8.ofNat ((value / 256) % 256), UInt8.ofNat (value % 256)]
  else none

def u64 (value : Nat) : Option Bytes :=
  if value ≤ u64Maximum then
    some [UInt8.ofNat (value / 72057594037927936),
      UInt8.ofNat ((value / 281474976710656) % 256),
      UInt8.ofNat ((value / 1099511627776) % 256),
      UInt8.ofNat ((value / 4294967296) % 256),
      UInt8.ofNat ((value / 16777216) % 256),
      UInt8.ofNat ((value / 65536) % 256),
      UInt8.ofNat ((value / 256) % 256), UInt8.ofNat (value % 256)]
  else none

def many : List (Option Bytes) → Option Bytes
  | [] => some []
  | head :: tail => do
      let encodedHead ← head
      let encodedTail ← many tail
      pure (encodedHead ++ encodedTail)

def blob (value : Bytes) : Option Bytes := do
  let size ← u32 value.length
  pure (size ++ value)

def fixed32 (value : Bytes) : Option Bytes :=
  if value.length = 32 then some value else none

def seq (encoder : α → Option Bytes) (values : List α) : Option Bytes := do
  let count ← u32 values.length
  let payload ← many (values.map encoder)
  pure (count ++ payload)

def frame (tag : UInt8) (payload : Bytes) : Option Bytes := do
  let size ← u32 payload.length
  pure (tag :: size ++ payload)

def term : Term → Option Bytes
  | .atom kind payload equality => do
      let fields ← many [blob kind, blob payload, blob equality]
      pure (0x00 :: fields)
  | .triple first second third => do
      let fields ← many [term first, term second, term third]
      pure (0x01 :: fields)
termination_by value => sizeOf value

def sort : KSort → Option Bytes
  | .bytes => some [0x00]
  | .term => some [0x01]

mutual
  def expr : KExpr → Option Bytes
  | .bytesLiteral value => (0x00 :: ·) <$> blob value
  | .termLiteral value => (0x01 :: ·) <$> term value
  | .var index => (0x02 :: ·) <$> u32 index
  | .makeAtom kind payload equality => do
      let fields ← many [expr kind, expr payload, expr equality]
      pure (0x03 :: fields)
  | .makeTriple first second third => do
      let fields ← many [expr first, expr second, expr third]
      pure (0x04 :: fields)
  | .letValue value body => do
      let fields ← many [expr value, expr body]
      pure (0x05 :: fields)
  | .caseTerm scrutinee atomBody tripleBody => do
      let fields ← many [expr scrutinee, expr atomBody, expr tripleBody]
      pure (0x06 :: fields)
  | .caseBytes scrutinee emptyBody consBody => do
      let fields ← many [expr scrutinee, expr emptyBody, expr consBody]
      pure (0x07 :: fields)
  | .concatBytes parts => do
      let count ← u32 parts.length
      let payload ← exprSeqPayload parts
      pure (0x08 :: count ++ payload)
  | .caseBytesEqual left right equalBody unequalBody => do
      let fields ← many [expr left, expr right, expr equalBody, expr unequalBody]
      pure (0x09 :: fields)
  | .call definitionId arguments => do
      let count ← u32 arguments.length
      let payload ← exprSeqPayload arguments
      let fields ← many [fixed32 definitionId, some (count ++ payload)]
      pure (0x0a :: fields)
  | .request operationId arguments => do
      let count ← u32 arguments.length
      let payload ← exprSeqPayload arguments
      let fields ← many [fixed32 operationId, some (count ++ payload)]
      pure (0x0b :: fields)

  def exprSeqPayload : KExprSeq → Option Bytes
    | .nil => some []
    | .cons head tail => do
        pure ((← expr head) ++ (← exprSeqPayload tail))
end

def namedSignature (value : NamedSignature) : Option Bytes := do
  pure (value.tag :: (← blob value.signature))

def ruleSignature (value : RuleSignature) : Option Bytes := do
  pure (value.tag :: value.premisePolicy :: (← blob value.clause))

def physicalOperation (value : PhysicalOperation) : Option Bytes := do
  many [fixed32 value.operationId, seq sort value.arguments, sort value.result]

def physicalProfile (value : PhysicalProfile) : Option Bytes := do
  let operations ← seq physicalOperation value.operations
  pure (value.profileVersion :: value.observationPolicy :: operations)

def manifest (value : CoreManifest) : Option Bytes := do
  let fields ← many [
    some [value.manifestVersion], seq (fun x => some [x]) value.frameTags,
    seq (fun x => some [x]) value.termTags,
    seq (fun x => some [x]) value.sortTags,
    seq namedSignature value.expressionForms, seq namedSignature value.abiForms,
    seq (fun x => some [x]) value.premisePolicyTags,
    seq (fun x => some [x]) value.lineageTags,
    seq (fun x => some [x]) value.nominalDeclarationTags,
    seq (fun x => some [x]) value.compilerEvidenceTags,
    seq (fun x => some [x]) value.valueTags,
    seq (fun x => some [x]) value.evalOutcomeTags,
    seq (fun x => some [x]) value.decodeVerdictTags,
    seq (fun x => some [x]) value.decodeCodeTags,
    seq (fun x => some [x]) value.authorizationStageTags,
    seq (fun x => some [x]) value.authorizationCodeTags,
    seq ruleSignature value.staticRules,
    seq ruleSignature value.evaluationRules,
    some [value.certificateFormatVersion], blob value.certificateSignature,
    seq blob value.contractClauses, physicalProfile value.physicalProfile]
  pure fields

def lineage : CompilerLineage → Option Bytes
  | .genesis => some [0x00]
  | .successor locator change => do
      pure (0x01 :: (← many [fixed32 locator, fixed32 change]))

def nominalDeclaration : NominalDeclaration → Option Bytes
  | .seed domain id => do
      pure (0x00 :: (← many [fixed32 domain, fixed32 id]))
  | .retainedSeed domain id revision => do
      pure (0x01 :: (← many [fixed32 domain, fixed32 id, fixed32 revision]))
  | .allocated domain id changeDomain changeId producerDomain producerId slot => do
      pure (0x02 :: (← many [fixed32 domain, fixed32 id,
        fixed32 changeDomain, fixed32 changeId,
        fixed32 producerDomain, fixed32 producerId, u64 slot]))

def interface (value : CompilerInterface) : Option Bytes :=
  many [fixed32 value.compile, fixed32 value.admitPropose]

def definition (value : Definition) : Option Bytes :=
  many [fixed32 value.id, seq sort value.arguments, sort value.result, expr value.body]

def subject (value : CompilerSubject) : Option Bytes :=
  many [lineage value.lineage, seq nominalDeclaration value.nominalDeclarations,
    interface value.interface, seq definition value.program, term value.buildRequest]

def kvalue : KValue → Option Bytes
  | .bytes value => (0x00 :: ·) <$> blob value
  | .term value => (0x01 :: ·) <$> term value

def evalOutcome (value : EvalOutcome) : Option Bytes := do
  pure (0x00 :: (← many [kvalue value.value, u64 value.remainingFuel,
    term value.observations]))

def evalStatement (value : EvalStatement) : Option Bytes :=
  many [blob value.exactAcceptedPredecessor, fixed32 value.coreContractId,
    fixed32 value.physicalProfileId, fixed32 value.entrypoint,
    seq kvalue value.arguments, u64 value.fuelLimit, evalOutcome value.expected]

def evalJudgment (value : EvalJudgment) : Option Bytes :=
  many [expr value.expression, seq kvalue value.environment,
    u64 value.fuelBefore, term value.observationsBefore,
    kvalue value.value, u64 value.fuelAfter, term value.observationsAfter]

def evalNode (value : EvalNode) : Option Bytes := do
  pure (value.ruleTag :: (← many [seq u32 value.premises,
    evalJudgment value.conclusion]))

def evalCertificate (value : EvalCertificate) : Option Bytes := do
  pure (value.formatVersion :: (← many [evalStatement value.statement,
    seq evalNode value.nodes]))

def evidence : CompilerEvidence → Option Bytes
  | .genesis => some [0x00]
  | .successor compile admission => do
      pure (0x01 :: (← many [evalCertificate compile, evalCertificate admission]))

def package (value : CompilerPackage) : Option Bytes := do
  let manifestPayload ← manifest value.manifest
  let subjectPayload ← subject value.subject
  let evidencePayload ← evidence value.evidence
  let frames ← many [frame 0x01 manifestPayload, frame 0x02 subjectPayload,
    frame 0x03 evidencePayload]
  pure ([0x43, 0x4c, 0x43, 0x50, 0x02] ++ frames)

end Encoding

namespace SHA256

def rotr (value : UInt32) (distance : Nat) : UInt32 :=
  (value >>> distance.toUInt32) ||| (value <<< (32 - distance).toUInt32)

def choose (x y z : UInt32) : UInt32 := (x &&& y) ^^^ (~~~x &&& z)
def majority (x y z : UInt32) : UInt32 := (x &&& y) ^^^ (x &&& z) ^^^ (y &&& z)
def bigSigma0 (x : UInt32) : UInt32 := rotr x 2 ^^^ rotr x 13 ^^^ rotr x 22
def bigSigma1 (x : UInt32) : UInt32 := rotr x 6 ^^^ rotr x 11 ^^^ rotr x 25
def smallSigma0 (x : UInt32) : UInt32 := rotr x 7 ^^^ rotr x 18 ^^^ (x >>> 3)
def smallSigma1 (x : UInt32) : UInt32 := rotr x 17 ^^^ rotr x 19 ^^^ (x >>> 10)

def constants : List UInt32 := [
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
  0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
  0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
  0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
  0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
  0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
  0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
  0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
  0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2]

def initial : List UInt32 := [0x6a09e667, 0xbb67ae85, 0x3c6ef372,
  0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]

def word (bytes : Bytes) : UInt32 :=
  match bytes with
  | a :: b :: c :: d :: _ =>
      (a.toUInt32 <<< 24) ||| (b.toUInt32 <<< 16) |||
        (c.toUInt32 <<< 8) ||| d.toUInt32
  | _ => 0

def initialWords (block : Bytes) : List UInt32 :=
  (List.range 16).map (fun i => word ((block.drop (i * 4)).take 4))

def expandWords : Nat → List UInt32 → List UInt32
  | 0, words => words
  | remaining + 1, words =>
      let i := words.length
      let next := smallSigma1 (words.getD (i - 2) 0) +
        words.getD (i - 7) 0 + smallSigma0 (words.getD (i - 15) 0) +
        words.getD (i - 16) 0
      expandWords remaining (words ++ [next])

structure Working where
  a : UInt32
  b : UInt32
  c : UInt32
  d : UInt32
  e : UInt32
  f : UInt32
  g : UInt32
  h : UInt32

def rounds : List UInt32 → List UInt32 → Working → Working
  | [], _, state | _, [], state => state
  | k :: ks, w :: ws, state =>
      let t1 := state.h + bigSigma1 state.e +
        choose state.e state.f state.g + k + w
      let t2 := bigSigma0 state.a + majority state.a state.b state.c
      rounds ks ws {
        a := t1 + t2, b := state.a, c := state.b, d := state.c,
        e := state.d + t1, f := state.e, g := state.f, h := state.g }

def compress (hash : List UInt32) (block : Bytes) : List UInt32 :=
  match hash with
  | a :: b :: c :: d :: e :: f :: g :: h :: _ =>
      let result := rounds constants (expandWords 48 (initialWords block))
        { a := a, b := b, c := c, d := d, e := e, f := f, g := g, h := h }
      [a + result.a, b + result.b, c + result.c, d + result.d,
        e + result.e, f + result.f, g + result.g, h + result.h]
  | _ => hash

def padZeros (count : Nat) : Bytes := List.replicate count 0

def padded (input : Bytes) : Bytes :=
  let withMarker := input ++ [0x80]
  let zeroCount := (56 + 64 - (withMarker.length % 64)) % 64
  let bitLength := input.length * 8
  withMarker ++ padZeros zeroCount ++ (Encoding.u64 bitLength).getD []

def blocks : Nat → Bytes → List Bytes
  | 0, _ => []
  | remaining + 1, input => input.take 64 :: blocks remaining (input.drop 64)

def wordBytes (value : UInt32) : Bytes := [
  (value >>> 24).toUInt8, (value >>> 16).toUInt8,
  (value >>> 8).toUInt8, value.toUInt8]

def hash (input : Bytes) : Hash32 :=
  let data := padded input
  let count := data.length / 64
  (blocks count data).foldl
    (fun state block => compress state block) initial
  |>.flatMap wordBytes

end SHA256

theorem sha256_empty_vector : SHA256.hash [] = [
    0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
    0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
    0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
    0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55] := by
  set_option maxRecDepth 100000 in
    decide

theorem sha256_abc_vector : SHA256.hash [0x61, 0x62, 0x63] = [
    0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
    0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
    0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
    0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad] := by
  set_option maxRecDepth 100000 in
    decide

def domainHash (domain : String) (components : List Bytes) : Hash32 :=
  let domainBytes := ascii domain
  let domainPrefix := (Encoding.u32 domainBytes.length).getD [] ++ domainBytes
  let fields := components.flatMap (fun component =>
    (Encoding.u64 component.length).getD [] ++ component)
  SHA256.hash (domainPrefix ++ fields)

namespace Fixed

def named (tag : UInt8) (signature : String) : NamedSignature :=
  { tag := tag, signature := ascii signature }

def rule (tag premise : UInt8) (clause : String) : RuleSignature :=
  { tag := tag, premisePolicy := premise, clause := ascii clause }

def sha256OperationId : Id32 :=
  domainHash "clause/physical-op/v1" [ascii "sha256"]

def physicalProfile : PhysicalProfile := {
  profileVersion := 0x00
  observationPolicy := 0x00
  operations := [{
    operationId := sha256OperationId
    arguments := [.bytes]
    result := .bytes
  }]
}

def expressionForms : List NamedSignature := [
  named 0x00 "BytesLiteral(value:Blob)->Bytes",
  named 0x01 "TermLiteral(value:Term)->Term",
  named 0x02 "Var(index:U32)->EnvironmentSort",
  named 0x03 "MakeAtom(kind:Bytes,payload:Bytes,equality:Bytes)->Term",
  named 0x04 "MakeTriple(first:Term,second:Term,third:Term)->Term",
  named 0x05 "Let(value:Any,body:(bind Any) Same)->Same",
  named 0x06 "CaseTerm(scrutinee:Term,atomBody:(bind Bytes Bytes Bytes) Same,tripleBody:(bind Term Term Term) Same)->Same",
  named 0x07 "CaseBytes(scrutinee:Bytes,emptyBody:Same,consBody:(bind Bytes Bytes) Same)->Same",
  named 0x08 "ConcatBytes(parts:Seq<Bytes>)->Bytes",
  named 0x09 "CaseBytesEqual(left:Bytes,right:Bytes,equalBody:Same,unequalBody:Same)->Same",
  named 0x0a "Call(definition:Id32,arguments:DefinitionArguments)->DefinitionResult",
  named 0x0b "Request(operation:Id32,arguments:PhysicalArguments)->PhysicalResult"]

def abiForms : List NamedSignature := [
  named 0x00 "ListNil()", named 0x01 "ListCons(head:Term,tail:List)",
  named 0x02 "ValueBytes(value:Bytes)", named 0x03 "ValueTerm(value:Term)",
  named 0x04 "NominalRef(domain:Id32,id:Id32)",
  named 0x05 "FixedId(domain:Id32,id:Id32)",
  named 0x06 "ContentId(domain:Id32,id:Id32)",
  named 0x07 "DerivedId(domain:Id32,id:Id32)",
  named 0x08 "IdentityPlan(retained:List<Retain>,seedInputs:List<SeedInput>)",
  named 0x09 "Retain(ref:NominalRef)", named 0x0a "SeedInput(ref:NominalRef)",
  named 0x10 "GenesisBase()",
  named 0x11 "AcceptedBase(packageHash:Hash32,revisionId:Id32)",
  named 0x12 "SourceUnit(unitId:Id32,artifactId:Hash32,bytes:Bytes)",
  named 0x13 "BuildRequest(base:GenesisBase|AcceptedBase,coreContractId:Hash32,physicalProfileId:Hash32,targetProfile:Term,sourceUnits:List<SourceUnit>,baseInputs:Term,identityRetentions:IdentityPlan,changeOccurrenceId:Id32,options:Term,compileFuel:U64,admissionFuel:U64,declaredPhysicalInputs:List<Term>)",
  named 0x14 "Built(subjectBytes:Bytes)",
  named 0x15 "Rejected(diagnostics:List<Term>)",
  named 0x16 "AdmissionRequest(buildRequest:BuildRequest,subjectBytes:Bytes,compileObservations:Observations)",
  named 0x17 "Propose(subjectBytes:Bytes)",
  named 0x18 "Reject(diagnostics:List<Term>)",
  named 0x19 "Observation(index:U64,operationId:Id32,arguments:List<KValue>,result:KValue)",
  named 0x1a "Observations(items:List<Observation>)",
  named 0x1b "Authorized(packageBytes:Bytes)",
  named 0x1c "Unauthorized(stage:U8,code:U8)"]

def staticRules : List RuleSignature := [
  rule 0x20 0x00 "Delta;Gamma |- BytesLiteral(b):Bytes",
  rule 0x21 0x00 "Delta;Gamma |- TermLiteral(t):Term",
  rule 0x22 0x00 "Delta;Gamma |- Var(i):Gamma[i] iff i<len(Gamma)",
  rule 0x23 0x03 "Delta;Gamma |- MakeAtom(k,p,q):Term iff k:Bytes and p:Bytes and q:Bytes",
  rule 0x24 0x03 "Delta;Gamma |- MakeTriple(a,b,c):Term iff a:Term and b:Term and c:Term",
  rule 0x25 0x02 "Delta;Gamma |- Let(v,b):r iff Delta;Gamma |- v:s and Delta;[s]++Gamma |- b:r",
  rule 0x26 0x03 "Delta;Gamma |- CaseTerm(s,a,t):r iff s:Term and Delta;[Bytes,Bytes,Bytes]++Gamma |- a:r and Delta;[Term,Term,Term]++Gamma |- t:r",
  rule 0x27 0x03 "Delta;Gamma |- CaseBytes(s,e,c):r iff s:Bytes and Delta;Gamma |- e:r and Delta;[Bytes,Bytes]++Gamma |- c:r",
  rule 0x28 0x05 "Delta;Gamma |- ConcatBytes(es):Bytes iff every es[i]:Bytes in encoded order",
  rule 0x29 0x04 "Delta;Gamma |- CaseBytesEqual(a,b,e,n):r iff a:Bytes and b:Bytes and e:r and n:r",
  rule 0x2a 0x06 "Delta;Gamma |- Call(d,args):r iff Delta contains exactly d:(ss)->r and len(args)=len(ss) and every args[i]:ss[i] in encoded order",
  rule 0x2b 0x06 "Delta;Gamma |- Request(op,args):r iff physicalProfile contains exactly op:(ss)->r and len(args)=len(ss) and every args[i]:ss[i] in encoded order"]

def evaluationRules : List RuleSignature := [
  rule 0x30 0x00 "J(BytesLiteral(b),g,f,o)=>(BytesValue(b),f-1,o) iff f>0",
  rule 0x31 0x00 "J(TermLiteral(t),g,f,o)=>(TermValue(t),f-1,o) iff f>0",
  rule 0x32 0x00 "J(Var(i),g,f,o)=>(g[i],f-1,o) iff f>0 and i<len(g)",
  rule 0x33 0x03 "after charge evaluate k,p,q left-to-right as BytesValue(kb),BytesValue(pb),BytesValue(qb); return TermValue(Atom(kb,pb,qb)) with final fuel and observations",
  rule 0x34 0x03 "after charge evaluate a,b,c left-to-right as TermValue(av),TermValue(bv),TermValue(cv); return TermValue(Triple(av,bv,cv)) with final fuel and observations",
  rule 0x35 0x02 "after charge evaluate v to x, then evaluate b under [x]++g; return the body value, fuel, and observations",
  rule 0x36 0x02 "after charge evaluate s to TermValue(Atom(k,p,q)), then evaluate atomBody under [BytesValue(k),BytesValue(p),BytesValue(q)]++g; return the selected body outcome",
  rule 0x37 0x02 "after charge evaluate s to TermValue(Triple(a,b,c)), then evaluate tripleBody under [TermValue(a),TermValue(b),TermValue(c)]++g; return the selected body outcome",
  rule 0x38 0x02 "after charge evaluate s to BytesValue(empty), then evaluate emptyBody under g; return the selected body outcome",
  rule 0x39 0x02 "after charge evaluate s to BytesValue(head++tail) with len(head)=1, then evaluate consBody under [BytesValue(head),BytesValue(tail)]++g; return the selected body outcome",
  rule 0x3a 0x05 "after charge evaluate es left-to-right as BytesValue parts and return BytesValue(concat(parts)); empty es returns empty Bytes with post-charge fuel and unchanged observations",
  rule 0x3b 0x03 "after charge evaluate a then b as BytesValue and iff lengths and octets are equal evaluate equalBody under g; return the selected body outcome",
  rule 0x3c 0x03 "after charge evaluate a then b as BytesValue and iff lengths or octets differ evaluate unequalBody under g; return the selected body outcome",
  rule 0x3d 0x07 "after charge resolve exactly d, evaluate args left-to-right, then evaluate its body under exactly the argument values with no caller environment; return the body outcome",
  rule 0x3e 0x01 "after charge evaluate the sole argument as BytesValue(input), compute FIPS-180-4 SHA-256(input), return BytesValue(H0||H1||H2||H3||H4||H5||H6||H7), and append exactly Observation(len(o),Sha256OpId,[Value(Bytes,input)],Value(Bytes,digest))"]

def certificateSignature : Bytes := ascii "EvalCertificate(formatVersion:CertificateFormatVersion,statement:EvalStatement,nodes:Seq<EvalNode>); CertificateFormatVersion=00; EvalStatement(exactAcceptedPredecessor:Blob,coreContractId:Hash32,physicalProfileId:Hash32,entrypoint:Id32,arguments:Seq<KValue>,fuelLimit:U64,expected:Returned(value:KValue,remainingFuel:U64,observations:Term)); KValue=00 BytesValue(Blob)|01 TermValue(Term); EvalNode(ruleTag:EvaluationRuleTag,premises:Seq<U32>,conclusion:EvalJudgment); EvaluationRuleTag=30|31|32|33|34|35|36|37|38|39|3a|3b|3c|3d|3e; EvalJudgment(expression:KExpr,environment:Seq<KValue>,fuelBefore:U64,observationsBefore:Term,value:KValue,fuelAfter:U64,observationsAfter:Term)"

def contractClauses : List Bytes := [
  ascii "C00: U8=one octet;U32=four-octet unsigned big-endian;U64=eight-octet unsigned big-endian;Blob=U32 length||octets[length];Seq<X>=U32 count||X[count];Frame<X>=U8 tag||U32 payloadLength||X;Id32 and Hash32 are exactly 32 octets;Span=Id32 sourceArtifactId||U64 start||U64 end with start<=end<=source length;record fields concatenate in displayed order;sum variants begin with displayed U8;all arithmetic is checked before cursor change conversion iteration or allocation;every bounded value consumes exactly;no padding trailing bytes or alternate spelling",
  ascii "C01: Term=00 Atom(kind:Blob,payload:Blob,equality:Blob)|01 Triple(first:Term,second:Term,third:Term); KSort=00 Bytes|01 Term; frameTags,termTags,sortTags,expressionForms,abiForms,premisePolicyTags,lineageTags,nominalDeclarationTags,compilerEvidenceTags,valueTags,evalOutcomeTags,decodeVerdictTags,decodeCodeTags,authorizationStageTags,authorizationCodeTags,staticRules,evaluationRules,certificateFormatVersion and physical profile values above are the complete closed tag sets and signatures",
  ascii "C02: KTag=clause/core-abi/tag/v1; KBytes=clause/core-abi/bytes/v1; KId32=clause/core-abi/id32/v1; KU64=clause/core-abi/u64/v1; KEq=clause/core/bytes-equal/v1; Tag(t)=Atom(KTag,U8(t),KEq); Bytes(b)=Atom(KBytes,b,KEq); Id(id)=Atom(KId32,id,KEq) iff len(id)=32; Nat64(n)=Atom(KU64,U64(n),KEq); List([])=Tag(00); List(x::xs)=Triple(Tag(01),x,List(xs)); Record(t,xs)=Triple(Tag(t),List(xs),Tag(00)); Core ABI constructors and field counts are exactly abiForms in tag order; wrong Atom kind field count wrapper fixed width list shape or trailing field is invalid",
  ascii "C03: CompilerSubject=lineage,nominalDeclarations,interface,program,buildRequest; lineage=00 Genesis|01 Successor(predecessorLocator:Hash32,changeOccurrenceId:Id32); interface=compile:Id32,admitPropose:Id32; Definition=id:Id32,arguments:Seq<KSort>,result:KSort,body:KExpr; definitions are sorted unique by id",
  ascii "C04: NominalDeclaration=00 Seed(domain,id)|01 RetainedSeed(domain,id,predecessorRevisionId)|02 Allocated(domain,id,changeInput:NominalWireRef,producerInput:NominalWireRef,localSlot:U64); NominalWireRef=domain:Id32||id:Id32; declarations are sorted unique by domain||id and every nominal reference resolves exactly one declaration in its required domain",
  ascii "C05: Seed is literal primitive provenance; RetainedSeed must match predecessor Seed or RetainedSeed and exact predecessor revision and cannot relabel Allocated; Allocated.id=DH(clause/new-nominal/v1,domain,wire(changeInput),wire(producerInput),U64(localSlot)); allocation inputs resolve and form an acyclic graph; dependency order then domain||id is the unique recomputation order; collision is invalid",
  ascii "C06: IdentityPlan has separately sorted unique Retain(NominalRef) and SeedInput(NominalRef) lists; every successor RetainedSeed appears only in retained; every newly introduced successor Seed appears only in seedInputs; each row matches declaration provenance; genesis retained is empty; no reference appears in both lists",
  ascii "C07: Delta is the canonical sorted unique definition table; Gamma and runtime environments use index-zero-first Var order; a definition is well formed iff its body has its declared result under its declared argument sorts and all transitive Call and Request references resolve; there is no subsorting, coercion, implicit argument, host value, fallback rule, or package-defined rule",
  ascii "C08: J(expression,environment,fuelBefore,observationsBefore)=>(value,fuelAfter,observationsAfter) is the sole successful evaluation judgment; values are only BytesValue or TermValue; fuel is U64; every rule consumes one unit before premises; zero fuel has no judgment; premises run strictly left-to-right and thread exact fuel and observations; integer overflow, bad value sort, unresolved definition, malformed observation, physical failure, or out-of-fuel has no successful judgment",
  ascii "C09: observationPolicy 00 appends exactly one canonical observation for each successful physical Request and none otherwise; observation indices are 0..n-1; the sole operation is Sha256OpId:[Bytes]->Bytes; SHA-256 is FIPS 180-4 over successive eight-bit message units and returns big-endian H0||H1||H2||H3||H4||H5||H6||H7; every other operation or signature is invalid",
  ascii "C10: Certificate format is certificateSignature above with formatVersion 00; nodes are indexed in encoded order; the last node is root; nodes is nonempty; every premise index is earlier than its consumer, appears in execution order, and is unique; every node is reachable from root; unknown ruleTag is DecodeRejected(06,ruleTagOffset), while a known tag with wrong expression, premise, state, value, fuel, environment, digest, or observation semantics is Unauthorized(stage,7f)",
  ascii "C11: A certificate node uses exactly one evaluation rule 30..3e and that rule's premisePolicy; the first premise begins after the parent's one-unit charge at observationsBefore; later premises begin at the prior premise's fuel and observations; Call has argument premises then one body premise under exactly argument values; RequestSha256 has one argument premise then fixed digest and one append; certificate nodes prove neither static well-formedness nor authority",
  ascii "C12: EvalStatement contains complete exact already-accepted predecessor bytes, derived manifest/profile IDs, exact entrypoint, canonical arguments, exact nonzero fuel limit, and Returned value,remainingFuel,Observations; its independently constructed root is Call(entrypoint,map(ValueLiteral,arguments)) under empty environment, statement fuel, and Observations([]); faults have no certificate form",
  ascii "C13: CompilerEvidence=00 GenesisEvidence with no payload|01 SuccessorEvidence(compileCertificate:EvalCertificate,admissionCertificate:EvalCertificate); evidence is never executable compiler meaning and cannot add a Core or certificate rule",
  ascii "V01: VerifyEvalCertificate first requires certificate formatVersion 00 and canonical byte equality of certificate.statement and the required EvalStatement",
  ascii "V02: VerifyEvalCertificate next strictly decodes required.exactAcceptedPredecessor, requires caller-supplied acceptance of those exact bytes, requires predecessor Frame01 byte-equal exactCoreManifestBytes, and independently derives CoreContractId and PhysicalProfileId",
  ascii "V03: VerifyEvalCertificate next requires both derived IDs equal the statement fields, statically checks the predecessor under rules 20..2b, resolves the entrypoint exactly once, and requires argument sorts equal its signature",
  ascii "V04: VerifyEvalCertificate next constructs Call(entrypoint,map(ValueLiteral,arguments)) without certificate input, where only BytesValue maps to BytesLiteral and TermValue maps to TermLiteral",
  ascii "V05: VerifyEvalCertificate next constructs the required root judgment with empty environment, fuelLimit, Observations([]), and exactly the value,remainingFuel,observations in required.expected",
  ascii "V06: VerifyEvalCertificate next scans nodes in encoded order and validates every exact known local rule, premise index, state transition, environment, value, fuel, and observation chain",
  ascii "V07: VerifyEvalCertificate finally requires every node reachable and the final conclusion canonical-byte-equal to the independently constructed root; success requires every prior step and uses no callback, theorem name, host rule registry, Boolean evaluator, or package rule",
  ascii "D00: StrictDecode returns only Decoded(exactInput,candidate) or DecodeRejected(code,offset); codes in precedence order are 00 WrongMagic,01 UnknownVersion,02 FrameTagOrderOrCount,03 Truncated,04 LengthOrCountOverflow,05 InvalidFixedWidth,06 UnknownSumTag,07 BoundedValueUnderConsumed,08 BoundedValueOverConsumed,09 TrailingBytes; fields are read depth-first in encoded order and equal-offset ties use lower code",
  ascii "D01: StrictDecode handles only closed byte grammar; order, uniqueness, exact manifest equality, reference bounds, ABI meaning, entrypoint signature, identity derivation, lineage/evidence consistency, known certificate-rule semantics, and profile conformance are authorization checks; malformed bytes never produce Unauthorized",
  ascii "A00: Authorization starts only after Decoded(exactInput,Q) and requires exactly one explicit request: GenesisAuthorizationRequest(ownerAnchor,R,E,Gc,Ga,I) or SuccessorAuthorizationRequest(P,R,E,I), where ownerAnchor=Missing|Supplied(OwnerAnchorWitness), OwnerAnchorWitness is an opaque non-package-wire capability created only by the external human-owner selection act, observe(witness)=OwnerAnchorObservation(exactSelectedBytes:Blob,selectedByteLength:U64,selectedPackageHash:Hash32), Gc and Ga are U64, and I=FinalPackageIdentityInput(packageHash:Hash32,exactPackageBytes:Blob); no owner-anchor variant, witness, or observation is encoded in Q; the request variant, never candidate data, selects the route; stages run 40..48; successor skips 42; genesis skips 43,45,46,47; both run 44 and 48; each row condition belongs to exactly one stage and route; rows run left-to-right and collection failures use encoded item order; failure at position i means every earlier condition passed and condition i is false, so first-failure predicates are pairwise disjoint and the first false condition is the only verdict",
  ascii "A40: CoreManifest rows=[manifest bytes differ exactCoreManifestBytes->(40,60)]",
  ascii "A41: CoreWellFormedness rows=[subject or ABI semantic structure->(41,61),nominal provenance allocation retention or reference->(41,62),definition order or duplicate->(41,63),compile then admitPropose resolution->(41,64),entrypoints equal->(41,65),compile then admitPropose signature not [Term]->Term->(41,66),other static rule 20..2b->(41,67),Request outside exact profile->(41,68)]",
  ascii "A42: GenesisAnchor rows=[lineage not Genesis->(42,69),supplied E not byte-identical Q.evidence or E not empty GenesisEvidence->(42,6a),ownerAnchor=Missing->(42,6b),ownerAnchor=Supplied(w) and observe(w) is not a self-consistent observation of the complete exact candidate because selectedByteLength!=byteLength(exactSelectedBytes) or selectedPackageHash!=CompilerPackageHash(exactSelectedBytes) or exactSelectedBytes is not octet-for-octet equal exactInput->(42,6c)]; length and hash checks never substitute for the final exact-byte equality or create authority",
  ascii "A43: ExactPredecessor rows=[lineage not Successor->(43,6d),candidate self candidate-basis or candidate-rule authority->(43,6f),supplied predecessor not already accepted including stale revision->(43,6e),locator differs CompilerPackageHash(P)->(43,70),resolved bytes not byte-identical accepted P->(43,71)]",
  ascii "A44: BuildRequest rows=[wrong ABI shape->(44,72),R not byte-identical Q.subject.buildRequest->(44,73),base route or exact base mismatch->(44,74),core ID mismatch->(44,75),profile ID mismatch->(44,76),source order or duplicate->(44,77),source artifact derivation->(44,78),IdentityPlan order uniqueness provenance retention or seed binding->(44,79),request lineage or nominal change occurrence mismatch->(44,7a),declared physical inputs nonempty->(44,7b),on genesis Gc or Ga zero or R.compileFuel!=Gc or R.admissionFuel!=Ga; on successor either R fuel zero->(44,7c)]",
  ascii "A45: CompileEvaluation rows=[evidence or compile certificate shape->(45,7d),statement predecessor manifest profile entrypoint arguments or fuel->(45,7e),known node premise root rule state fuel or observation semantics->(45,7f),no successful judgment->(45,80),result not Built->(45,81),Built bytes differ Q.subject->(45,82),compile observations differ root->(45,83)]",
  ascii "A46: AdmissionEvaluation rows=[certificate shape->(46,7d),statement predecessor manifest profile entrypoint arguments fuel or compile observations->(46,7e),known node premise root rule state fuel or observation semantics->(46,7f),no successful judgment->(46,80),result not Propose->(46,81),proposed bytes differ Q.subject->(46,82),admission observations differ root->(46,83)]",
  ascii "A47: EvidenceAttachment rows=[E not byte-identical Q.evidence->(47,84),Frame02 differs certified subject->(47,85),attaching E does not reproduce exact Q->(47,86)]",
  ascii "A48: FinalAuthorization rows=[I.exactPackageBytes not byte-identical exactInput or I.packageHash!=DH(clause/compiler-package/v1,I.exactPackageBytes)->(48,87)]",
  ascii "H00: DH(d,xs)=SHA256(U32(len(d))||ASCII(d)||each(U64(len(x))||x)); CoreContractId=DH(clause/core-contract/v1,exactCoreManifestBytes); PhysicalProfileId=DH(clause/physical-profile/v1,exactPhysicalProfileBytes); CompilerSemanticsId=DH(clause/compiler-semantics/v1,canonical(interface||program)); CompilerRevisionId=DH(clause/compiler-revision/v1,exactCompilerSubjectBytes); CompilerPackageHash=DH(clause/compiler-package/v1,exactWholePackageBytes); SourceArtifactId=DH(clause/source-artifact/v1,exactSourceBytes); BuildRequestId=DH(clause/compiler-build-request/v1,canonicalTermBytes(BuildRequest)); OriginId=DH(clause/origin/v1,canonicalAcyclicOriginNode); hashes never grant compiler authority",
  ascii "P00: Package bytes are magic CLCP,version 02,Frame(01,CoreManifestV1),Frame(02,CompilerSubject),Frame(03,CompilerEvidence),EOF exactly once in order; Frame03 is excluded from subject and revision identities; successor evidence contains predecessor bytes but never candidate evidence or candidate whole-package identity; only exact genesis anchor or already-accepted exact predecessor can authorize"]

def coreManifest : CoreManifest := {
  manifestVersion := 0x00
  frameTags := [0x01, 0x02, 0x03]
  termTags := [0x00, 0x01]
  sortTags := [0x00, 0x01]
  expressionForms := expressionForms
  abiForms := abiForms
  premisePolicyTags := [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
  lineageTags := [0x00, 0x01]
  nominalDeclarationTags := [0x00, 0x01, 0x02]
  compilerEvidenceTags := [0x00, 0x01]
  valueTags := [0x00, 0x01]
  evalOutcomeTags := [0x00]
  decodeVerdictTags := [0x00, 0x01]
  decodeCodeTags := [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]
  authorizationStageTags := [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]
  authorizationCodeTags := List.range 40 |>.map (fun n => UInt8.ofNat (0x60 + n))
  staticRules := staticRules
  evaluationRules := evaluationRules
  certificateFormatVersion := 0x00
  certificateSignature := certificateSignature
  contractClauses := contractClauses
  physicalProfile := physicalProfile
}

def exactCoreManifestBytes : Bytes := (Encoding.manifest coreManifest).getD []
def exactPhysicalProfileBytes : Bytes := (Encoding.physicalProfile physicalProfile).getD []
def coreContractId : Hash32 :=
  domainHash "clause/core-contract/v1" [exactCoreManifestBytes]
def physicalProfileId : Hash32 :=
  domainHash "clause/physical-profile/v1" [exactPhysicalProfileBytes]

end Fixed

def compilerPackageHash (bytes : Bytes) : Hash32 :=
  domainHash "clause/compiler-package/v1" [bytes]

def compilerRevisionId (subjectBytes : Bytes) : Id32 :=
  domainHash "clause/compiler-revision/v1" [subjectBytes]

def sourceArtifactId (sourceBytes : Bytes) : Hash32 :=
  domainHash "clause/source-artifact/v1" [sourceBytes]

def newNominalId (domain changeDomain changeId producerDomain producerId : Id32)
    (localSlot : Nat) : Id32 :=
  domainHash "clause/new-nominal/v1" [domain, changeDomain ++ changeId,
    producerDomain ++ producerId, (Encoding.u64 localSlot).getD []]

end ClauseCompiler
