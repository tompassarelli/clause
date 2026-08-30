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
deriving DecidableEq, Repr

def result (value : KValue) (fuel : Fuel) (observations : List Term) : Result := {
  value := value
  fuel := fuel
  observations := observations
}

inductive SequencePurpose where
  | concatenate
  | callBody (body : KExpr)
  | requestDigest (operationId : Id32)

inductive Continuation where
  | atomPayload (payload equality : KExpr) (environment : List KValue)
  | atomEquality (kind : Bytes) (equality : KExpr) (environment : List KValue)
  | atomFinish (kind payload : Bytes)
  | tripleSecond (second third : KExpr) (environment : List KValue)
  | tripleThird (first : Term) (third : KExpr) (environment : List KValue)
  | tripleFinish (first second : Term)
  | letBody (body : KExpr) (environment : List KValue)
  | caseTerm (atomBody tripleBody : KExpr) (environment : List KValue)
  | caseBytes (emptyBody consBody : KExpr) (environment : List KValue)
  | equalRight (right equalBody unequalBody : KExpr) (environment : List KValue)
  | equalBranch (left : Bytes) (equalBody unequalBody : KExpr)
      (environment : List KValue)
  | sequence (purpose : SequencePurpose) (remaining : KExprSeq)
      (environment : List KValue) (valuesReversed : List KValue)

inductive Control where
  | evaluate (expression : KExpr) (environment : List KValue)
  | returned (value : KValue)

structure Machine where
  control : Control
  continuations : List Continuation
  fuel : Fuel
  observations : List Term

def evaluating (expression : KExpr) (environment : List KValue)
    (continuations : List Continuation) (fuel : Fuel)
    (observations : List Term) : Machine := {
  control := .evaluate expression environment
  continuations := continuations
  fuel := fuel
  observations := observations
}

def returning (value : KValue) (continuations : List Continuation)
    (fuel : Fuel) (observations : List Term) : Machine := {
  control := .returned value
  continuations := continuations
  fuel := fuel
  observations := observations
}

def concatenate (values : List KValue) : Option Bytes :=
  let rec loop : List KValue → Bytes → Option Bytes
    | [], reversed => some reversed.reverse
    | .bytes bytes :: tail, reversed =>
        loop tail (bytes.reverse ++ reversed)
    | _ :: _, _ => none
  loop values []

def finishSequence (purpose : SequencePurpose) (valuesReversed : List KValue)
    (continuations : List Continuation) (fuel : Fuel)
    (observations : List Term) : Option Machine :=
  let values := valuesReversed.reverse
  match purpose with
  | .concatenate => do
      pure (returning (.bytes (← concatenate values))
        continuations fuel observations)
  | .callBody body =>
      pure (evaluating body values continuations fuel observations)
  | .requestDigest operationId =>
      match values with
      | [.bytes input] =>
          let digest := SHA256.hash input
          let item := ABI.observation observations.length operationId
            [.bytes input] (.bytes digest)
          pure (returning (.bytes digest) continuations fuel
            (observations ++ [item]))
      | _ => none

def step (program : List Definition) (profile : PhysicalProfile)
    (machine : Machine) : Option (Sum Result Machine) :=
  match machine.control with
  | .evaluate expression environment =>
      match machine.fuel with
      | 0 => none
      | charged + 1 =>
        let continuations := machine.continuations
        let observations := machine.observations
        match expression with
        | .bytesLiteral value =>
            some (.inr (returning (.bytes value) continuations charged observations))
        | .termLiteral value =>
            some (.inr (returning (.term value) continuations charged observations))
        | .var index => do
            let value ← nth? environment index
            pure (.inr (returning value continuations charged observations))
        | .makeAtom kind payload equality =>
            some (.inr (evaluating kind environment
              (.atomPayload payload equality environment :: continuations)
              charged observations))
        | .makeTriple first second third =>
            some (.inr (evaluating first environment
              (.tripleSecond second third environment :: continuations)
              charged observations))
        | .letValue value body =>
            some (.inr (evaluating value environment
              (.letBody body environment :: continuations)
              charged observations))
        | .caseTerm scrutinee atomBody tripleBody =>
            some (.inr (evaluating scrutinee environment
              (.caseTerm atomBody tripleBody environment :: continuations)
              charged observations))
        | .caseBytes scrutinee emptyBody consBody =>
            some (.inr (evaluating scrutinee environment
              (.caseBytes emptyBody consBody environment :: continuations)
              charged observations))
        | .concatBytes parts =>
            match parts with
            | .nil =>
                some (.inr (returning (.bytes []) continuations charged observations))
            | .cons head tail =>
                some (.inr (evaluating head environment
                  (.sequence .concatenate tail environment [] :: continuations)
                  charged observations))
        | .caseBytesEqual left right equalBody unequalBody =>
            some (.inr (evaluating left environment
              (.equalRight right equalBody unequalBody environment :: continuations)
              charged observations))
        | .call definitionId arguments => do
            let definition ← Static.findDefinition definitionId program
            match arguments with
            | .nil =>
                pure (.inr (evaluating definition.body [] continuations
                  charged observations))
            | .cons head tail =>
                pure (.inr (evaluating head environment
                  (.sequence (.callBody definition.body) tail environment [] ::
                    continuations)
                  charged observations))
        | .request operationId arguments => do
            let operation ← Static.findOperation operationId profile.operations
            if operation.operationId ≠ Fixed.sha256OperationId ||
                operation.arguments ≠ [.bytes] || operation.result ≠ .bytes then none
            else
              match arguments with
              | .nil => none
              | .cons head tail =>
                  pure (.inr (evaluating head environment
                    (.sequence (.requestDigest operationId) tail environment [] ::
                      continuations)
                    charged observations))
  | .returned value =>
      match machine.continuations with
      | [] => some (.inl (result value machine.fuel machine.observations))
      | continuation :: remaining =>
          match continuation with
          | .atomPayload payload equality environment =>
              match value with
              | .bytes kind =>
                  some (.inr (evaluating payload environment
                    (.atomEquality kind equality environment :: remaining)
                    machine.fuel machine.observations))
              | _ => none
          | .atomEquality kind equality environment =>
              match value with
              | .bytes payload =>
                  some (.inr (evaluating equality environment
                    (.atomFinish kind payload :: remaining)
                    machine.fuel machine.observations))
              | _ => none
          | .atomFinish kind payload =>
              match value with
              | .bytes equality =>
                  some (.inr (returning (.term (.atom kind payload equality))
                    remaining machine.fuel machine.observations))
              | _ => none
          | .tripleSecond second third environment =>
              match value with
              | .term first =>
                  some (.inr (evaluating second environment
                    (.tripleThird first third environment :: remaining)
                    machine.fuel machine.observations))
              | _ => none
          | .tripleThird first third environment =>
              match value with
              | .term second =>
                  some (.inr (evaluating third environment
                    (.tripleFinish first second :: remaining)
                    machine.fuel machine.observations))
              | _ => none
          | .tripleFinish first second =>
              match value with
              | .term third =>
                  some (.inr (returning (.term (.triple first second third))
                    remaining machine.fuel machine.observations))
              | _ => none
          | .letBody body environment =>
              some (.inr (evaluating body (value :: environment) remaining
                machine.fuel machine.observations))
          | .caseTerm atomBody tripleBody environment =>
              match value with
              | .term (.atom kind payload equality) =>
                  some (.inr (evaluating atomBody
                    ([.bytes kind, .bytes payload, .bytes equality] ++ environment)
                    remaining machine.fuel machine.observations))
              | .term (.triple first second third) =>
                  some (.inr (evaluating tripleBody
                    ([.term first, .term second, .term third] ++ environment)
                    remaining machine.fuel machine.observations))
              | _ => none
          | .caseBytes emptyBody consBody environment =>
              match value with
              | .bytes [] =>
                  some (.inr (evaluating emptyBody environment remaining
                    machine.fuel machine.observations))
              | .bytes (head :: tail) =>
                  some (.inr (evaluating consBody
                    ([.bytes [head], .bytes tail] ++ environment)
                    remaining machine.fuel machine.observations))
              | _ => none
          | .equalRight right equalBody unequalBody environment =>
              match value with
              | .bytes left =>
                  some (.inr (evaluating right environment
                    (.equalBranch left equalBody unequalBody environment :: remaining)
                    machine.fuel machine.observations))
              | _ => none
          | .equalBranch left equalBody unequalBody environment =>
              match value with
              | .bytes right =>
                  some (.inr (evaluating
                    (if left = right then equalBody else unequalBody)
                    environment remaining machine.fuel machine.observations))
              | _ => none
          | .sequence purpose remainingExpressions sequenceEnvironment values =>
              let values := value :: values
              match remainingExpressions with
              | .nil => do
                  let next ← finishSequence purpose values remaining
                    machine.fuel machine.observations
                  pure (.inr next)
              | .cons head tail =>
                  some (.inr (evaluating head sequenceEnvironment
                    (.sequence purpose tail sequenceEnvironment values :: remaining)
                    machine.fuel machine.observations))

def loop (program : List Definition) (profile : PhysicalProfile) :
    Nat → Machine → Option Result
  | 0, _ => none
  | budget + 1, machine =>
      match step program profile machine with
      | none => none
      | some (.inl complete) => some complete
      | some (.inr next) => loop program profile budget next

def transitionBudget (fuel : Fuel) : Nat := fuel * 2 + 1

def run (program : List Definition) (profile : PhysicalProfile)
    (expression : KExpr) (environment : List KValue) (fuel : Fuel)
    (observations : List Term) : Option Result :=
  loop program profile (transitionBudget fuel)
    (evaluating expression environment [] fuel observations)

namespace Regression

def atomA : Term := .atom [0xa1] [0xa2] [0xa3]
def atomB : Term := .atom [0xb1] [0xb2] [0xb3]
def atomC : Term := .atom [0xc1] [0xc2] [0xc3]

def runClosed (expression : KExpr) (fuel : Fuel) : Option Result :=
  run [] Fixed.physicalProfile expression [] fuel []

theorem rule30BytesLiteral :
    runClosed (.bytesLiteral [0x30]) 1 =
      some (result (.bytes [0x30]) 0 []) := by
  decide

theorem rule31TermLiteral :
    runClosed (.termLiteral atomA) 1 =
      some (result (.term atomA) 0 []) := by
  decide

theorem rule32Var :
    run [] Fixed.physicalProfile (.var 1)
      [.bytes [0x00], .bytes [0x32]] 1 [] =
      some (result (.bytes [0x32]) 0 []) := by
  decide

theorem rule33MakeAtom :
    runClosed (.makeAtom (.bytesLiteral [0x01]) (.bytesLiteral [0x02])
      (.bytesLiteral [0x03])) 4 =
      some (result (.term (.atom [0x01] [0x02] [0x03])) 0 []) := by
  decide

theorem rule34MakeTriple :
    runClosed (.makeTriple (.termLiteral atomA) (.termLiteral atomB)
      (.termLiteral atomC)) 4 =
      some (result (.term (.triple atomA atomB atomC)) 0 []) := by
  decide

theorem rule35Let :
    runClosed (.letValue (.bytesLiteral [0x35]) (.var 0)) 3 =
      some (result (.bytes [0x35]) 0 []) := by
  decide

theorem rule36CaseTermAtom :
    runClosed (.caseTerm (.termLiteral atomA) (.var 1)
      (.termLiteral atomB)) 3 =
      some (result (.bytes [0xa2]) 0 []) := by
  decide

theorem rule37CaseTermTriple :
    runClosed (.caseTerm (.termLiteral (.triple atomA atomB atomC))
      (.termLiteral atomA) (.var 2)) 3 =
      some (result (.term atomC) 0 []) := by
  decide

theorem rule38CaseBytesEmpty :
    runClosed (.caseBytes (.bytesLiteral []) (.bytesLiteral [0x38])
      (.bytesLiteral [0xff])) 3 =
      some (result (.bytes [0x38]) 0 []) := by
  decide

theorem rule39CaseBytesCons :
    runClosed (.caseBytes (.bytesLiteral [0x39, 0x3a])
      (.bytesLiteral []) (.concatBytes (KExprSeq.ofList [.var 0, .var 1]))) 5 =
      some (result (.bytes [0x39, 0x3a]) 0 []) := by
  decide

theorem rule3aConcatLeftToRight :
    runClosed (.concatBytes (KExprSeq.ofList [
      .bytesLiteral [0x3a], .bytesLiteral [0x00]])) 3 =
      some (result (.bytes [0x3a, 0x00]) 0 []) := by
  decide

theorem rule3bEqualBranch :
    runClosed (.caseBytesEqual (.bytesLiteral [0x3b])
      (.bytesLiteral [0x3b]) (.bytesLiteral [0x01])
      (.bytesLiteral [0x00])) 4 =
      some (result (.bytes [0x01]) 0 []) := by
  decide

theorem rule3cUnequalBranch :
    runClosed (.caseBytesEqual (.bytesLiteral [0x3c])
      (.bytesLiteral [0xff]) (.bytesLiteral [0x00])
      (.bytesLiteral [0x01])) 4 =
      some (result (.bytes [0x01]) 0 []) := by
  decide

def rule3dId : Id32 := List.replicate 31 0x00 ++ [0x3d]

def rule3dProgram : List Definition := [{
  id := rule3dId
  arguments := [.bytes, .bytes]
  result := .bytes
  body := .concatBytes (KExprSeq.ofList [.var 0, .var 1])
}]

theorem rule3dCallArgumentsLeftToRightAndIsolated :
    run rule3dProgram Fixed.physicalProfile
      (.call rule3dId (KExprSeq.ofList [
        .bytesLiteral [0x3d], .bytesLiteral [0x00]]))
      [.bytes [0xff]] 6 [] =
      some (result (.bytes [0x3d, 0x00]) 0 []) := by
  decide

def rule3ePriorObservation : Term := .atom [0x3e] [] []
def rule3eInput : Bytes := [0x61, 0x62, 0x63]
def rule3eDigest : Bytes := SHA256.hash rule3eInput

def rule3eMachine0 : Machine :=
  evaluating (.request Fixed.sha256OperationId
    (KExprSeq.ofList [.bytesLiteral rule3eInput])) [] [] 2
    [rule3ePriorObservation]

def rule3eMachine1 : Machine :=
  evaluating (.bytesLiteral rule3eInput) []
    [.sequence (.requestDigest Fixed.sha256OperationId) .nil [] []] 1
    [rule3ePriorObservation]

def rule3eMachine2 : Machine :=
  returning (.bytes rule3eInput)
    [.sequence (.requestDigest Fixed.sha256OperationId) .nil [] []] 0
    [rule3ePriorObservation]

def rule3eMachine3 : Machine :=
  returning (.bytes rule3eDigest) [] 0 [rule3ePriorObservation,
    ABI.observation 1 Fixed.sha256OperationId
      [.bytes rule3eInput] (.bytes rule3eDigest)]

theorem fixedSha256OperationLookup :
    Static.findOperation Fixed.sha256OperationId
      Fixed.physicalProfile.operations = some {
        operationId := Fixed.sha256OperationId
        arguments := [.bytes]
        result := .bytes
      } := by
  simp [Fixed.physicalProfile, Static.findOperation]

theorem rule3eStep0 :
    step [] Fixed.physicalProfile rule3eMachine0 = some (.inr rule3eMachine1) := by
  simp [rule3eMachine0, rule3eMachine1, step, evaluating,
    fixedSha256OperationLookup, KExprSeq.ofList]

theorem rule3eStep1 :
    step [] Fixed.physicalProfile rule3eMachine1 = some (.inr rule3eMachine2) := by
  rfl

theorem rule3eStep2 :
    step [] Fixed.physicalProfile rule3eMachine2 = some (.inr rule3eMachine3) := by
  simp [rule3eMachine2, rule3eMachine3, step, returning,
    finishSequence, rule3eDigest]

theorem rule3eStep3 :
    step [] Fixed.physicalProfile rule3eMachine3 = some (.inl
      (result (.bytes rule3eDigest) 0 [rule3ePriorObservation,
        ABI.observation 1 Fixed.sha256OperationId
          [.bytes rule3eInput] (.bytes rule3eDigest)])) := by
  rfl

theorem rule3eRequestAppendsObservation :
    run [] Fixed.physicalProfile
      (.request Fixed.sha256OperationId
        (KExprSeq.ofList [.bytesLiteral rule3eInput])) [] 2
      [rule3ePriorObservation] =
      some (result (.bytes rule3eDigest) 0 [rule3ePriorObservation,
        ABI.observation 1 Fixed.sha256OperationId
          [.bytes rule3eInput] (.bytes rule3eDigest)]) := by
  change loop [] Fixed.physicalProfile 5 rule3eMachine0 = _
  simp only [loop, rule3eStep0, rule3eStep1, rule3eStep2, rule3eStep3]

def recursiveBytesId : Id32 := List.replicate 31 0x00 ++ [0x90]

def recursiveBytesProgram : List Definition := [{
  id := recursiveBytesId
  arguments := [.bytes]
  result := .bytes
  body := .caseBytes (.var 0)
    (.bytesLiteral [])
    (.call recursiveBytesId (.cons (.var 1) .nil))
}]

def recursiveBytesExpression (size : Nat) : KExpr :=
  .call recursiveBytesId
    (.cons (.bytesLiteral (List.replicate size 0x61)) .nil)

def recursiveBytesFuel (size : Nat) : Fuel := size * 4 + 5

def recursiveBytesRun (size : Nat) : Option Result :=
  run recursiveBytesProgram Fixed.physicalProfile
    (recursiveBytesExpression size) [] (recursiveBytesFuel size) []

theorem recursiveBytesThree :
    recursiveBytesRun 3 = some (result (.bytes []) 0 []) := by
  decide

end Regression

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

def matchesReceipt (result : Evaluator.Result) (receipt : EvalReceipt) : Bool :=
  match evalReceiptValueHash result.value,
      evalReceiptObservationsHash (ABI.observations result.observations) with
  | some valueHash, some observationsHash =>
      valueHash = receipt.expectedValueHash &&
      result.fuel = receipt.expectedRemainingFuel &&
      observationsHash = receipt.expectedObservationsHash
  | _, _ => false

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
      | some result => matchesReceipt result receipt

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
  expectedValueHash := (evalReceiptValueHash (.term profileEscapeTerm)).getD []
  expectedRemainingFuel := 0
  expectedObservationsHash :=
    (evalReceiptObservationsHash ABI.emptyObservations).getD []
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
