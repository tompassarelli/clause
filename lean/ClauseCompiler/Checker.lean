import ClauseCompiler.Codec

/-! Fixed Core ABI, static checking, generic evaluation, and certificate checking. -/

namespace ClauseCompiler

def nth? : List α → Nat → Option α
  | [], _ => none
  | head :: _, 0 => some head
  | _ :: tail, n + 1 => nth? tail n

def all (predicate : α → Bool) : List α → Bool
  | [] => true
  | head :: tail => predicate head && all predicate tail

def all₂ (predicate : α → β → Bool) : List α → List β → Bool
  | [], [] => true
  | a :: as, b :: bs => predicate a b && all₂ predicate as bs
  | _, _ => false

def contains (value : α) [DecidableEq α] : List α → Bool
  | [] => false
  | head :: tail => decide (head = value) || contains value tail

def unique [DecidableEq α] : List α → Bool
  | [] => true
  | head :: tail => !contains head tail && unique tail

def bytesLt : Bytes → Bytes → Bool
  | [], [] => false
  | [], _ :: _ => true
  | _ :: _, [] => false
  | a :: as, b :: bs =>
      if a.toNat < b.toNat then true
      else if a = b then bytesLt as bs
      else false

def sortedUniqueBy (key : α → Bytes) : List α → Bool
  | [] | [_] => true
  | first :: second :: tail =>
      bytesLt (key first) (key second) && sortedUniqueBy key (second :: tail)

namespace ABI

def kTag := ascii "clause/core-abi/tag/v1"
def kBytes := ascii "clause/core-abi/bytes/v1"
def kId32 := ascii "clause/core-abi/id32/v1"
def kU64 := ascii "clause/core-abi/u64/v1"
def kEq := ascii "clause/core/bytes-equal/v1"

def tag (value : UInt8) : Term := .atom kTag [value] kEq
def bytes (value : Bytes) : Term := .atom kBytes value kEq
def id (value : Id32) : Term := .atom kId32 value kEq
def nat64 (value : Nat) : Term :=
  .atom kU64 (Encoding.u64 value |>.getD []) kEq

def list : List Term → Term
  | [] => tag 0x00
  | head :: tail => .triple (tag 0x01) head (list tail)

def record (tagValue : UInt8) (fields : List Term) : Term :=
  .triple (tag tagValue) (list fields) (tag 0x00)

def value : KValue → Term
  | .bytes data => record 0x02 [bytes data]
  | .term data => record 0x03 [data]

def asTag : Term → Option UInt8
  | .atom kind [value] equality =>
      if kind = kTag && equality = kEq then some value else none
  | _ => none

def asBytes : Term → Option Bytes
  | .atom kind payload equality =>
      if kind = kBytes && equality = kEq then some payload else none
  | _ => none

def asId : Term → Option Id32
  | .atom kind payload equality =>
      if kind = kId32 && equality = kEq && payload.length = 32 then some payload else none
  | _ => none

def asNat64 : Term → Option Nat
  | .atom kind payload equality =>
      if kind = kU64 && equality = kEq && payload.length = 8 then
        match payload with
        | [a,b,c,d,e,f,g,h] => some (a.toNat * 72057594037927936 +
            b.toNat * 281474976710656 + c.toNat * 1099511627776 +
            d.toNat * 4294967296 + e.toNat * 16777216 + f.toNat * 65536 +
            g.toNat * 256 + h.toNat)
        | _ => none
      else none
  | _ => none

def termBudget : Term → Nat
  | .atom _ _ _ => 1
  | .triple first second third =>
      termBudget first + termBudget second + termBudget third + 1

def asListFuel : Nat → Term → Option (List Term)
  | 0, _ => none
  | fuel + 1, candidate =>
      match asTag candidate with
      | some 0x00 => some []
      | _ =>
          match candidate with
          | .triple marker head tail =>
              if asTag marker = some 0x01 then
                (head :: ·) <$> asListFuel fuel tail
              else none
          | _ => none

def asList (candidate : Term) : Option (List Term) :=
  asListFuel (termBudget candidate + 1) candidate

def asRecord (expected : UInt8) : Term → Option (List Term)
  | .triple marker fields trailer =>
      if asTag marker = some expected && asTag trailer = some 0x00 then
        asList fields
      else none
  | _ => none

inductive Base where
  | genesis
  | accepted (packageHash revisionId : Id32)
deriving DecidableEq

structure SourceUnit where
  unitId : Id32
  artifactId : Hash32
  bytes : Bytes
deriving DecidableEq

structure NominalRef where
  domain : Id32
  id : Id32
deriving DecidableEq

structure IdentityPlan where
  retained : List NominalRef
  seedInputs : List NominalRef
deriving DecidableEq

structure BuildRequest where
  base : Base
  coreContractId : Hash32
  physicalProfileId : Hash32
  targetProfile : Term
  sourceUnits : List SourceUnit
  baseInputs : Term
  identityPlan : IdentityPlan
  changeOccurrenceId : Id32
  options : Term
  compileFuel : Fuel
  admissionFuel : Fuel
  declaredPhysicalInputs : List Term
deriving DecidableEq

def decodeBase (value : Term) : Option Base :=
  match asRecord 0x10 value with
  | some [] => some .genesis
  | _ =>
      match asRecord 0x11 value with
      | some [hash, revision] => .accepted <$> asId hash <*> asId revision
      | _ => none

def decodeSourceUnit (value : Term) : Option SourceUnit := do
  let fields ← asRecord 0x12 value
  match fields with
  | [unit, artifact, source] => pure {
      unitId := (← asId unit)
      artifactId := (← asId artifact)
      bytes := (← asBytes source)
    }
  | _ => none

def decodeNominalRef (value : Term) : Option NominalRef := do
  let fields ← asRecord 0x04 value
  match fields with
  | [domain, id] => pure { domain := (← asId domain), id := (← asId id) }
  | _ => none

def decodeRefWrapper (tagValue : UInt8) (value : Term) : Option NominalRef := do
  let fields ← asRecord tagValue value
  match fields with
  | [reference] => decodeNominalRef reference
  | _ => none

def decodeAll (decoder : α → Option β) : List α → Option (List β)
  | [] => some []
  | head :: tail => do pure ((← decoder head) :: (← decodeAll decoder tail))

def decodeIdentityPlan (value : Term) : Option IdentityPlan := do
  let fields ← asRecord 0x08 value
  match fields with
  | [retained, seeds] => pure {
      retained := (← decodeAll (decodeRefWrapper 0x09) (← asList retained))
      seedInputs := (← decodeAll (decodeRefWrapper 0x0a) (← asList seeds))
    }
  | _ => none

def decodeBuildRequest (value : Term) : Option BuildRequest := do
  let fields ← asRecord 0x13 value
  match fields with
  | [base, coreId, profileId, target, sources, inputs, identities,
      change, options, compileFuel, admissionFuel, physicalInputs] => pure {
      base := (← decodeBase base)
      coreContractId := (← asId coreId)
      physicalProfileId := (← asId profileId)
      targetProfile := target
      sourceUnits := (← decodeAll decodeSourceUnit (← asList sources))
      baseInputs := inputs
      identityPlan := (← decodeIdentityPlan identities)
      changeOccurrenceId := (← asId change)
      options := options
      compileFuel := (← asNat64 compileFuel)
      admissionFuel := (← asNat64 admissionFuel)
      declaredPhysicalInputs := (← asList physicalInputs)
    }
  | _ => none

def observations (items : List Term) : Term := record 0x1a [list items]
def emptyObservations : Term := observations []

def observation (index : Nat) (operationId : Id32)
    (arguments : List KValue) (result : KValue) : Term :=
  record 0x19 [nat64 index, id operationId,
    list (arguments.map value), value result]

def decodeObservations (value : Term) : Option (List Term) := do
  let fields ← asRecord 0x1a value
  match fields with
  | [items] => asList items
  | _ => none

def builtBytes (value : Term) : Option Bytes := do
  let fields ← asRecord 0x14 value
  match fields with | [subject] => asBytes subject | _ => none

def proposedBytes (value : Term) : Option Bytes := do
  let fields ← asRecord 0x17 value
  match fields with | [subject] => asBytes subject | _ => none

end ABI

namespace Static

def findDefinition (id : Id32) : List Definition → Option Definition
  | [] => none
  | definition :: tail => if definition.id = id then some definition else findDefinition id tail

def findOperation (id : Id32) : List PhysicalOperation → Option PhysicalOperation
  | [] => none
  | operation :: tail => if operation.operationId = id then some operation else findOperation id tail

mutual
  def infer (program : List Definition) (profile : PhysicalProfile)
      (environment : List KSort) : KExpr → Option KSort
    | .bytesLiteral _ => some .bytes
    | .termLiteral _ => some .term
    | .var index => nth? environment index
    | .makeAtom kind payload equality => do
        if (← infer program profile environment kind) = .bytes &&
            (← infer program profile environment payload) = .bytes &&
            (← infer program profile environment equality) = .bytes then
          pure .term
        else none
    | .makeTriple first second third => do
        if (← infer program profile environment first) = .term &&
            (← infer program profile environment second) = .term &&
            (← infer program profile environment third) = .term then
          pure .term
        else none
    | .letValue value body => do
        let valueSort ← infer program profile environment value
        infer program profile (valueSort :: environment) body
    | .caseTerm scrutinee atomBody tripleBody => do
        if (← infer program profile environment scrutinee) ≠ .term then none else
        let atomSort ← infer program profile
          ([.bytes, .bytes, .bytes] ++ environment) atomBody
        let tripleSort ← infer program profile
          ([.term, .term, .term] ++ environment) tripleBody
        if atomSort = tripleSort then pure atomSort else none
    | .caseBytes scrutinee emptyBody consBody => do
        if (← infer program profile environment scrutinee) ≠ .bytes then none else
        let emptySort ← infer program profile environment emptyBody
        let consSort ← infer program profile ([.bytes, .bytes] ++ environment) consBody
        if emptySort = consSort then pure emptySort else none
    | .concatBytes parts => do
        if ← inferSeq program profile environment parts .bytes then pure .bytes else none
    | .caseBytesEqual left right equalBody unequalBody => do
        if (← infer program profile environment left) ≠ .bytes ||
            (← infer program profile environment right) ≠ .bytes then none else
        let equalSort ← infer program profile environment equalBody
        let unequalSort ← infer program profile environment unequalBody
        if equalSort = unequalSort then pure equalSort else none
    | .call id arguments => do
        let definition ← findDefinition id program
        if ← inferSeqAgainst program profile environment arguments definition.arguments then
          pure definition.result
        else none
    | .request _ arguments => do
        if ← inferSeqAny program profile environment arguments then pure .bytes else none

  def inferSeq (program : List Definition) (profile : PhysicalProfile)
      (environment : List KSort) : KExprSeq → KSort → Option Bool
    | .nil, _ => some true
    | .cons head tail, expected => do
        if (← infer program profile environment head) = expected then
          inferSeq program profile environment tail expected
        else pure false

  def inferSeqAgainst (program : List Definition) (profile : PhysicalProfile)
      (environment : List KSort) : KExprSeq → List KSort → Option Bool
    | .nil, [] => some true
    | .cons head tail, expected :: remaining => do
        if (← infer program profile environment head) = expected then
          inferSeqAgainst program profile environment tail remaining
        else pure false
    | _, _ => some false

  /- Static checking proves that request arguments are themselves well typed,
  but deliberately does not own the operation, arity, argument-signature, or
  result-signature verdict.  Those closed-profile checks are the later 41/68
  row in authorization. -/
  def inferSeqAny (program : List Definition) (profile : PhysicalProfile)
      (environment : List KSort) : KExprSeq → Option Bool
    | .nil => some true
    | .cons head tail => do
        let _ ← infer program profile environment head
        inferSeqAny program profile environment tail
end

def definitionsSortedUnique (program : List Definition) : Bool :=
  sortedUniqueBy Definition.id program

def definitionsWellTyped (program : List Definition) (profile : PhysicalProfile) : Bool :=
  definitionsSortedUnique program && all (fun definition =>
    infer program profile definition.arguments definition.body = some definition.result) program

def requestSignatureConforms (program : List Definition) (profile : PhysicalProfile)
    (environment : List KSort) (operationId : Id32) (arguments : KExprSeq) : Bool :=
  match findOperation operationId profile.operations with
  | none => false
  | some operation =>
      operation.result = .bytes &&
      inferSeqAgainst program profile environment arguments operation.arguments = some true

mutual
  def requestsConform (program : List Definition) (profile : PhysicalProfile)
      (environment : List KSort) : KExpr → Bool
    | .bytesLiteral _ | .termLiteral _ | .var _ => true
    | .makeAtom a b c | .makeTriple a b c =>
        requestsConform program profile environment a &&
        requestsConform program profile environment b &&
        requestsConform program profile environment c
    | .letValue value body =>
        requestsConform program profile environment value &&
        match infer program profile environment value with
        | none => false
        | some sort => requestsConform program profile (sort :: environment) body
    | .caseTerm scrutinee atomBody tripleBody =>
        requestsConform program profile environment scrutinee &&
        requestsConform program profile
          ([.bytes, .bytes, .bytes] ++ environment) atomBody &&
        requestsConform program profile
          ([.term, .term, .term] ++ environment) tripleBody
    | .caseBytes scrutinee emptyBody consBody =>
        requestsConform program profile environment scrutinee &&
        requestsConform program profile environment emptyBody &&
        requestsConform program profile ([.bytes, .bytes] ++ environment) consBody
    | .concatBytes parts => requestSeqConform program profile environment parts
    | .caseBytesEqual a b c d =>
        requestsConform program profile environment a &&
        requestsConform program profile environment b &&
        requestsConform program profile environment c &&
        requestsConform program profile environment d
    | .call _ arguments => requestSeqConform program profile environment arguments
    | .request operation arguments =>
        requestSignatureConforms program profile environment operation arguments &&
        requestSeqConform program profile environment arguments

  def requestSeqConform (program : List Definition) (profile : PhysicalProfile)
      (environment : List KSort) : KExprSeq → Bool
    | .nil => true
    | .cons head tail => requestsConform program profile environment head &&
        requestSeqConform program profile environment tail
end

def definitionsConformToProfile (program : List Definition)
    (profile : PhysicalProfile) : Bool :=
  all (fun definition =>
    requestsConform program profile definition.arguments definition.body) program

def definitionsWellFormed (program : List Definition) (profile : PhysicalProfile) : Bool :=
  definitionsWellTyped program profile && definitionsConformToProfile program profile

def entrypointsWellFormed (subject : CompilerSubject) : Bool :=
  if subject.interface.compile = subject.interface.admitPropose then false else
  match findDefinition subject.interface.compile subject.program,
      findDefinition subject.interface.admitPropose subject.program with
  | some compile, some admit =>
      compile.arguments = [.term] && compile.result = .term &&
        admit.arguments = [.term] && admit.result = .term
  | _, _ => false

end Static

namespace Evaluator

structure Result where
  value : KValue
  fuel : Fuel
  observations : List Term

def result (value : KValue) (fuel : Fuel) (observations : List Term) : Result := {
  value := value
  fuel := fuel
  observations := observations
}

def evaluate (budget : Nat) (program : List Definition) (profile : PhysicalProfile)
    (expression : KExpr) (environment : List KValue) (fuel : Fuel)
    (observations : List Term) : Option Result :=
  match budget, fuel with
  | 0, _ | _, 0 => none
  | budget + 1, fuel + 1 =>
    let charged := fuel
    let rec evaluateSequence (currentFuel : Fuel) (currentObservations : List Term) :
        KExprSeq → Option (List KValue × Fuel × List Term)
      | .nil => some ([], currentFuel, currentObservations)
      | .cons head tail => do
          let first ← evaluate budget program profile head environment
            currentFuel currentObservations
          let (remaining, finalFuel, finalObservations) ←
            evaluateSequence first.fuel first.observations tail
          pure (first.value :: remaining, finalFuel, finalObservations)
    match expression with
    | .bytesLiteral value => some (result (.bytes value) charged observations)
    | .termLiteral value => some (result (.term value) charged observations)
    | .var index => do
        let value ← nth? environment index
        pure (result value charged observations)
    | .makeAtom kind payload equality => do
        let k ← evaluate budget program profile kind environment charged observations
        let p ← evaluate budget program profile payload environment k.fuel k.observations
        let e ← evaluate budget program profile equality environment p.fuel p.observations
        match k.value, p.value, e.value with
        | .bytes kb, .bytes pb, .bytes eb =>
            pure (result (.term (.atom kb pb eb)) e.fuel e.observations)
        | _, _, _ => none
    | .makeTriple first second third => do
        let a ← evaluate budget program profile first environment charged observations
        let b ← evaluate budget program profile second environment a.fuel a.observations
        let c ← evaluate budget program profile third environment b.fuel b.observations
        match a.value, b.value, c.value with
        | .term av, .term bv, .term cv =>
            pure (result (.term (.triple av bv cv)) c.fuel c.observations)
        | _, _, _ => none
    | .letValue value body => do
        let result ← evaluate budget program profile value environment charged observations
        evaluate budget program profile body (result.value :: environment)
          result.fuel result.observations
    | .caseTerm scrutinee atomBody tripleBody => do
        let selected ← evaluate budget program profile scrutinee environment charged observations
        match selected.value with
        | .term (.atom kind payload equality) =>
            evaluate budget program profile atomBody
              ([KValue.bytes kind, KValue.bytes payload, KValue.bytes equality] ++ environment)
              selected.fuel selected.observations
        | .term (.triple first second third) =>
            evaluate budget program profile tripleBody
              ([KValue.term first, KValue.term second, KValue.term third] ++ environment)
              selected.fuel selected.observations
        | _ => none
    | .caseBytes scrutinee emptyBody consBody => do
        let selected ← evaluate budget program profile scrutinee environment charged observations
        match selected.value with
        | .bytes [] =>
            evaluate budget program profile emptyBody environment
              selected.fuel selected.observations
        | .bytes (head :: tail) =>
            evaluate budget program profile consBody
              ([KValue.bytes [head], KValue.bytes tail] ++ environment)
              selected.fuel selected.observations
        | _ => none
    | .concatBytes parts => do
        let (values, finalFuel, finalObservations) ←
          evaluateSequence charged observations parts
        let rec concatenate : List KValue → Option Bytes
          | [] => some []
          | .bytes bytes :: tail => do pure (bytes ++ (← concatenate tail))
          | _ => none
        pure (result (.bytes (← concatenate values)) finalFuel finalObservations)
    | .caseBytesEqual left right equalBody unequalBody => do
        let a ← evaluate budget program profile left environment charged observations
        let b ← evaluate budget program profile right environment a.fuel a.observations
        match a.value, b.value with
        | KValue.bytes av, KValue.bytes bv =>
            evaluate budget program profile
              (if av = bv then equalBody else unequalBody)
              environment b.fuel b.observations
        | _, _ => none
    | .call definitionId arguments => do
        let definition ← Static.findDefinition definitionId program
        let (values, finalFuel, finalObservations) ←
          evaluateSequence charged observations arguments
        evaluate budget program profile definition.body values finalFuel finalObservations
    | .request operationId arguments => do
        let operation ← Static.findOperation operationId profile.operations
        if operation.operationId ≠ Fixed.sha256OperationId ||
            operation.arguments ≠ [.bytes] || operation.result ≠ .bytes then none else
        let (values, finalFuel, finalObservations) ←
          evaluateSequence charged observations arguments
        match values with
        | [.bytes input] =>
            let digest := SHA256.hash input
            let item := ABI.observation finalObservations.length operationId
              [.bytes input] (.bytes digest)
            pure (result (.bytes digest) finalFuel (finalObservations ++ [item]))
        | _ => none
def run (program : List Definition) (profile : PhysicalProfile)
    (expression : KExpr) (environment : List KValue) (fuel : Fuel)
    (observations : List Term) : Option Result :=
  evaluate (fuel + 1) program profile expression environment fuel observations

end Evaluator

namespace Replay

def valueLiteral : KValue → KExpr
  | .bytes value => .bytesLiteral value
  | .term value => .termLiteral value

def requestExpression (request : EvalRequest) : KExpr :=
  .call request.entrypoint
    (KExprSeq.ofList (request.arguments.map valueLiteral))

def requestWellFormed (exactAcceptedPredecessor : Bytes)
    (predecessor : DecodedPackage) (request : EvalRequest) : Bool :=
  predecessor.exactInput = exactAcceptedPredecessor &&
  request.acceptedPredecessorPackageHash =
    compilerPackageHash exactAcceptedPredecessor &&
  predecessor.exactManifestPayload = Fixed.exactCoreManifestBytes &&
  predecessor.package.manifest = Fixed.coreManifest &&
  request.coreContractId = Fixed.coreContractId &&
  request.physicalProfileId = Fixed.physicalProfileId &&
  request.fuelLimit > 0 &&
  Static.definitionsWellFormed predecessor.package.subject.program
    predecessor.package.manifest.physicalProfile &&
  Static.entrypointsWellFormed predecessor.package.subject &&
  match Static.findDefinition request.entrypoint
      predecessor.package.subject.program with
  | none => false
  | some entrypoint =>
      all₂ (fun value expected => value.sort = expected)
        request.arguments entrypoint.arguments

def run (predecessor : DecodedPackage) (request : EvalRequest) :
    Option Evaluator.Result :=
  Evaluator.run predecessor.package.subject.program
    predecessor.package.manifest.physicalProfile
    (requestExpression request) [] request.fuelLimit []

def outcome (result : Evaluator.Result) : EvalOutcome := {
  value := result.value
  remainingFuel := result.fuel
  observations := ABI.observations result.observations
}

def verifyEvalReceipt (exactAcceptedPredecessor : Bytes)
    (_accepted : AcceptedExact exactAcceptedPredecessor)
    (request : EvalRequest) (receipt : EvalReceipt) : Bool :=
  receipt.formatVersion = Fixed.coreManifest.receiptFormatVersion &&
  match Codec.strictDecode exactAcceptedPredecessor with
  | .rejected _ => false
  | .decoded predecessor =>
      requestWellFormed exactAcceptedPredecessor predecessor request &&
      match run predecessor request with
      | none => false
      | some result => outcome result = receipt.expected

theorem nonconformingDefinitionsRejectReceipt
    (exactAcceptedPredecessor : Bytes)
    (accepted : AcceptedExact exactAcceptedPredecessor)
    (request : EvalRequest)
    (receipt : EvalReceipt)
    (predecessor : DecodedPackage)
    (binding : Codec.strictDecode exactAcceptedPredecessor = .decoded predecessor)
    (profileFailure :
      Static.definitionsConformToProfile predecessor.package.subject.program
        predecessor.package.manifest.physicalProfile = false) :
    verifyEvalReceipt exactAcceptedPredecessor accepted request receipt = false := by
  simp [verifyEvalReceipt, binding, requestWellFormed,
    Static.definitionsWellFormed, profileFailure]

namespace Regression

/- The fixture's two entrypoints are well typed.  Its third, unused definition
is the exact profile escape; the public verifier rejects any strict-decoded
predecessor whose definitions fail that gate, independently of reachability. -/
def profileEscapeId (suffix : UInt8) : Id32 := List.replicate 31 0x00 ++ [suffix]

def profileEscapeCompileId : Id32 := profileEscapeId 0x01
def profileEscapeAdmitId : Id32 := profileEscapeId 0x02
def profileEscapeUnusedId : Id32 := profileEscapeId 0x03
def profileEscapeOperationId : Id32 :=
  if Fixed.sha256OperationId = List.replicate 32 0x00 then
    List.replicate 32 0xff
  else
    List.replicate 32 0x00

theorem profileEscapeOperationId_unknown :
    Fixed.sha256OperationId ≠ profileEscapeOperationId := by
  unfold profileEscapeOperationId
  split
  · rename_i isZero
    rw [isZero]
    decide
  · rename_i isNotZero
    exact isNotZero

theorem profileEscapeOperationId_length : profileEscapeOperationId.length = 32 := by
  unfold profileEscapeOperationId
  split <;> decide

def profileEscapeTerm : Term := .atom [] [] []

def profileEscapeProgram : List Definition := [
  {
    id := profileEscapeCompileId
    arguments := [.term]
    result := .term
    body := .var 0
  },
  {
    id := profileEscapeAdmitId
    arguments := [.term]
    result := .term
    body := .var 0
  },
  {
    id := profileEscapeUnusedId
    arguments := []
    result := .bytes
    body := .request profileEscapeOperationId .nil
  }
]

def profileEscapeSubject : CompilerSubject := {
  lineage := .genesis
  nominalDeclarations := []
  interface := {
    compile := profileEscapeCompileId
    admitPropose := profileEscapeAdmitId
  }
  program := profileEscapeProgram
  buildRequest := profileEscapeTerm
}

def profileEscapePackage : CompilerPackage := {
  manifest := Fixed.coreManifest
  subject := profileEscapeSubject
  evidence := .genesis
}

def profileEscapeBytes : Bytes := (Encoding.package profileEscapePackage).getD []

def profileEscapeRequest : EvalRequest := {
  acceptedPredecessorPackageHash := compilerPackageHash profileEscapeBytes
  coreContractId := Fixed.coreContractId
  physicalProfileId := Fixed.physicalProfileId
  entrypoint := profileEscapeCompileId
  arguments := [.term profileEscapeTerm]
  fuelLimit := 3
}

def profileEscapeReceipt : EvalReceipt := {
  formatVersion := 0x00
  expected := {
    value := .term profileEscapeTerm
    remainingFuel := 0
    observations := ABI.emptyObservations
  }
}

theorem profileEscapeProgramWellTyped :
    Static.definitionsWellTyped profileEscapeProgram Fixed.physicalProfile = true := by
  simp [Static.definitionsWellTyped, Static.definitionsSortedUnique, sortedUniqueBy,
    profileEscapeProgram, profileEscapeCompileId, profileEscapeAdmitId,
    profileEscapeUnusedId, profileEscapeId, bytesLt, all, Static.infer,
    Static.inferSeqAny, nth?]

theorem profileEscapeProgramRejectedByProfile :
    Static.definitionsConformToProfile profileEscapeProgram Fixed.physicalProfile = false := by
  simp [Static.definitionsConformToProfile, profileEscapeProgram, all,
    Static.requestsConform, Static.requestSignatureConforms, Static.findOperation,
    Static.requestSeqConform, Fixed.physicalProfile, profileEscapeOperationId_unknown]

theorem profileEscapeProgramNotWellFormed :
    Static.definitionsWellFormed profileEscapeProgram Fixed.physicalProfile = false := by
  simp [Static.definitionsWellFormed, profileEscapeProgramWellTyped,
    profileEscapeProgramRejectedByProfile]

theorem unusedOutOfProfileRequestRejected
    (accepted : AcceptedExact profileEscapeBytes)
    (predecessor : DecodedPackage)
    (binding : Codec.strictDecode profileEscapeBytes = .decoded predecessor)
    (profileFailure :
      Static.definitionsConformToProfile predecessor.package.subject.program
        predecessor.package.manifest.physicalProfile = false) :
    profileEscapeOperationId.length = 32 ∧
    Static.definitionsWellTyped profileEscapeProgram Fixed.physicalProfile = true ∧
    Static.definitionsConformToProfile profileEscapeProgram Fixed.physicalProfile = false ∧
    Static.definitionsWellFormed profileEscapeProgram Fixed.physicalProfile = false ∧
    verifyEvalReceipt profileEscapeBytes accepted
      profileEscapeRequest profileEscapeReceipt = false := by
  refine ⟨profileEscapeOperationId_length, profileEscapeProgramWellTyped,
    profileEscapeProgramRejectedByProfile, profileEscapeProgramNotWellFormed, ?_⟩
  exact nonconformingDefinitionsRejectReceipt
    profileEscapeBytes accepted profileEscapeRequest profileEscapeReceipt
    predecessor binding profileFailure

end Regression

end Replay

end ClauseCompiler
