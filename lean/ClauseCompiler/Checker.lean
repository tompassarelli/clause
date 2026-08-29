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

def definitionsWellFormed (program : List Definition) (profile : PhysicalProfile) : Bool :=
  definitionsSortedUnique program && all (fun definition =>
    infer program profile definition.arguments definition.body = some definition.result) program

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

namespace Certificate

def sameExpr (left right : KExpr) : Bool :=
  Encoding.expr left = Encoding.expr right

def sameJudgment (left right : EvalJudgment) : Bool :=
  Encoding.evalJudgment left = Encoding.evalJudgment right

def premiseCount : KExpr → KValue → Option Nat
  | .bytesLiteral _, _ | .termLiteral _, _ | .var _, _ => some 0
  | .makeAtom _ _ _, .term (.atom ..) => some 3
  | .makeTriple _ _ _, .term (.triple ..) => some 3
  | .letValue _ _, _ => some 2
  | .caseTerm _ _ _, _ => some 2
  | .caseBytes _ _ _, _ => some 2
  | .concatBytes parts, .bytes _ => some parts.length
  | .caseBytesEqual _ _ _ _, _ => some 3
  | .call _ arguments, _ => some (arguments.length + 1)
  | .request _ arguments, _ => some arguments.length
  | _, _ => none

def premiseValue (prior : List EvalNode) (indices : List Nat) (position : Nat) :
    Option KValue := do
  let index ← nth? indices position
  let node ← nth? prior index
  pure node.conclusion.value

def expectedRuleTag (prior : List EvalNode) (node : EvalNode) : Option UInt8 :=
  match node.conclusion.expression, node.conclusion.value with
  | .bytesLiteral _, .bytes _ => some 0x30
  | .termLiteral _, .term _ => some 0x31
  | .var _, _ => some 0x32
  | .makeAtom _ _ _, .term (.atom ..) => some 0x33
  | .makeTriple _ _ _, .term (.triple ..) => some 0x34
  | .letValue _ _, _ => some 0x35
  | .caseTerm _ _ _, _ =>
      match premiseValue prior node.premises 0 with
      | some (.term (.atom ..)) => some 0x36
      | some (.term (.triple ..)) => some 0x37
      | _ => none
  | .caseBytes _ _ _, _ =>
      match premiseValue prior node.premises 0 with
      | some (.bytes []) => some 0x38
      | some (.bytes (_ :: _)) => some 0x39
      | _ => none
  | .concatBytes _, .bytes _ => some 0x3a
  | .caseBytesEqual _ _ _ _, _ =>
      match premiseValue prior node.premises 0, premiseValue prior node.premises 1 with
      | some (.bytes left), some (.bytes right) =>
          if left = right then some 0x3b else some 0x3c
      | _, _ => none
  | .call _ _, _ => some 0x3d
  | .request _ _, .bytes _ => some 0x3e
  | _, _ => none

def indicesEarlierUnique (index : Nat) (indices : List Nat) : Bool :=
  unique indices && all (fun premise => premise < index) indices

structure PremiseTrace where
  judgments : List EvalJudgment
  fuel : Fuel
  observations : Term

def directJudgment (program : List Definition) (profile : PhysicalProfile)
    (expression : KExpr) (environment : List KValue) (fuel : Fuel)
    (observations : Term) : Option EvalJudgment := do
  let observationItems ← ABI.decodeObservations observations
  let result ← Evaluator.run program profile expression environment fuel observationItems
  pure {
    expression := expression
    environment := environment
    fuelBefore := fuel
    observationsBefore := observations
    value := result.value
    fuelAfter := result.fuel
    observationsAfter := ABI.observations result.observations
  }

def traceExpressions (program : List Definition) (profile : PhysicalProfile)
    (environment : List KValue) :
    List KExpr → Fuel → Term → Option PremiseTrace
  | [], fuel, observations => some {
      judgments := []
      fuel := fuel
      observations := observations
    }
  | expression :: tail, fuel, observations => do
      let judgment ← directJudgment program profile expression environment fuel observations
      let remaining ← traceExpressions program profile environment tail
        judgment.fuelAfter judgment.observationsAfter
      pure {
        judgments := judgment :: remaining.judgments
        fuel := remaining.fuel
        observations := remaining.observations
      }

def appendJudgment (trace : PremiseTrace) (judgment : EvalJudgment) : PremiseTrace := {
  judgments := trace.judgments ++ [judgment]
  fuel := judgment.fuelAfter
  observations := judgment.observationsAfter
}

def expectedPremises (program : List Definition) (profile : PhysicalProfile)
    (conclusion : EvalJudgment) : Option (List EvalJudgment) :=
  match conclusion.fuelBefore with
  | 0 => none
  | charged + 1 => do
      let trace ← match conclusion.expression with
        | .bytesLiteral _ | .termLiteral _ | .var _ => some {
            judgments := []
            fuel := charged
            observations := conclusion.observationsBefore
          }
        | .makeAtom kind payload equality =>
            traceExpressions program profile conclusion.environment
              [kind, payload, equality] charged conclusion.observationsBefore
        | .makeTriple first second third =>
            traceExpressions program profile conclusion.environment
              [first, second, third] charged conclusion.observationsBefore
        | .letValue value body => do
            let valueTrace ← traceExpressions program profile conclusion.environment
              [value] charged conclusion.observationsBefore
            let valueJudgment ← valueTrace.judgments.head?
            let bodyJudgment ← directJudgment program profile body
              (valueJudgment.value :: conclusion.environment)
              valueTrace.fuel valueTrace.observations
            pure (appendJudgment valueTrace bodyJudgment)
        | .caseTerm scrutinee atomBody tripleBody => do
            let selectedTrace ← traceExpressions program profile conclusion.environment
              [scrutinee] charged conclusion.observationsBefore
            let selected ← selectedTrace.judgments.head?
            let (body, environment) ← match selected.value with
              | .term (.atom kind payload equality) => some (atomBody,
                  [.bytes kind, .bytes payload, .bytes equality] ++ conclusion.environment)
              | .term (.triple first second third) => some (tripleBody,
                  [.term first, .term second, .term third] ++ conclusion.environment)
              | _ => none
            let bodyJudgment ← directJudgment program profile body environment
              selectedTrace.fuel selectedTrace.observations
            pure (appendJudgment selectedTrace bodyJudgment)
        | .caseBytes scrutinee emptyBody consBody => do
            let selectedTrace ← traceExpressions program profile conclusion.environment
              [scrutinee] charged conclusion.observationsBefore
            let selected ← selectedTrace.judgments.head?
            let (body, environment) ← match selected.value with
              | .bytes [] => some (emptyBody, conclusion.environment)
              | .bytes (head :: tail) => some (consBody,
                  [.bytes [head], .bytes tail] ++ conclusion.environment)
              | _ => none
            let bodyJudgment ← directJudgment program profile body environment
              selectedTrace.fuel selectedTrace.observations
            pure (appendJudgment selectedTrace bodyJudgment)
        | .concatBytes parts =>
            traceExpressions program profile conclusion.environment parts.toList
              charged conclusion.observationsBefore
        | .caseBytesEqual left right equalBody unequalBody => do
            let operands ← traceExpressions program profile conclusion.environment
              [left, right] charged conclusion.observationsBefore
            let leftJudgment ← operands.judgments.head?
            let rightJudgment ← (operands.judgments.drop 1).head?
            let body ← match leftJudgment.value, rightJudgment.value with
              | .bytes leftBytes, .bytes rightBytes =>
                  some (if leftBytes = rightBytes then equalBody else unequalBody)
              | _, _ => none
            let bodyJudgment ← directJudgment program profile body conclusion.environment
              operands.fuel operands.observations
            pure (appendJudgment operands bodyJudgment)
        | .call definitionId arguments => do
            let definition ← Static.findDefinition definitionId program
            let argumentsTrace ← traceExpressions program profile conclusion.environment
              arguments.toList charged conclusion.observationsBefore
            let bodyJudgment ← directJudgment program profile definition.body
              (argumentsTrace.judgments.map (fun judgment => judgment.value))
              argumentsTrace.fuel argumentsTrace.observations
            pure (appendJudgment argumentsTrace bodyJudgment)
        | .request _ arguments =>
            traceExpressions program profile conclusion.environment arguments.toList
              charged conclusion.observationsBefore
      pure trace.judgments

def premisesMatch (program : List Definition) (profile : PhysicalProfile)
    (prior : List EvalNode) (node : EvalNode) : Bool :=
  match expectedPremises program profile node.conclusion,
      node.premises.mapM (fun premise => (nth? prior premise).map EvalNode.conclusion) with
  | some expected, some actual => all₂ sameJudgment expected actual
  | _, _ => false

def nodeValid (program : List Definition) (profile : PhysicalProfile)
    (prior : List EvalNode) (index : Nat) (node : EvalNode) : Bool :=
  node.ruleTag = expectedRuleTag prior node &&
  node.premises.length = premiseCount node.conclusion.expression node.conclusion.value &&
  indicesEarlierUnique index node.premises &&
  premisesMatch program profile prior node &&
  match ABI.decodeObservations node.conclusion.observationsBefore with
  | none => false
  | some observations =>
      match Evaluator.run program profile node.conclusion.expression
          node.conclusion.environment node.conclusion.fuelBefore observations with
      | none => false
      | some result => result.value = node.conclusion.value &&
          result.fuel = node.conclusion.fuelAfter &&
          ABI.observations result.observations = node.conclusion.observationsAfter &&
          all (fun premise => (nth? prior premise).isSome) node.premises

def nodesValid (program : List Definition) (profile : PhysicalProfile) :
    List EvalNode → Bool
  | nodes =>
      let rec loop (prior : List EvalNode) (index : Nat) : List EvalNode → Bool
        | [] => true
        | node :: tail => nodeValid program profile prior index node &&
            loop (prior ++ [node]) (index + 1) tail
      loop [] 0 nodes

def markReachable (nodes : List EvalNode) : Nat → Nat → List Nat
  | 0, root => [root]
  | budget + 1, root =>
      if let some node := nth? nodes root then
        root :: node.premises.flatMap (markReachable nodes budget)
      else []

def allReachable (nodes : List EvalNode) : Bool :=
  match nodes.length with
  | 0 => false
  | count + 1 =>
      let reached := markReachable nodes (count + 1) count
      all (fun index => contains index reached) (List.range (count + 1))

def valueLiteral : KValue → KExpr
  | .bytes value => .bytesLiteral value
  | .term value => .termLiteral value

def requiredRoot (statement : EvalStatement) : EvalJudgment := {
  expression := .call statement.entrypoint
    (KExprSeq.ofList (statement.arguments.map valueLiteral))
  environment := []
  fuelBefore := statement.fuelLimit
  observationsBefore := ABI.emptyObservations
  value := statement.expected.value
  fuelAfter := statement.expected.remainingFuel
  observationsAfter := statement.expected.observations
}

def checkGraph (program : List Definition) (profile : PhysicalProfile)
    (statement : EvalStatement) (nodes : List EvalNode) : Bool :=
  nodesValid program profile nodes && allReachable nodes &&
    match nodes.reverse with
    | [] => false
    | root :: _ => sameJudgment root.conclusion (requiredRoot statement)

def verifyEvalCertificate (required : EvalStatement)
    (_accepted : AcceptedExact required.exactAcceptedPredecessor)
    (certificate : EvalCertificate) : Bool :=
  certificate.formatVersion = 0x00 && certificate.statement = required &&
  match Codec.strictDecode required.exactAcceptedPredecessor with
  | .rejected _ => false
  | .decoded predecessor =>
      predecessor.exactManifestPayload = Fixed.exactCoreManifestBytes &&
      required.coreContractId = Fixed.coreContractId &&
      required.physicalProfileId = Fixed.physicalProfileId &&
      predecessor.package.manifest = Fixed.coreManifest &&
      Static.definitionsWellFormed predecessor.package.subject.program
        predecessor.package.manifest.physicalProfile &&
      Static.entrypointsWellFormed predecessor.package.subject &&
      match Static.findDefinition required.entrypoint predecessor.package.subject.program with
      | none => false
      | some entrypoint =>
          all₂ (fun value expected => value.sort = expected)
            required.arguments entrypoint.arguments &&
          match Evaluator.run predecessor.package.subject.program
              predecessor.package.manifest.physicalProfile
              (requiredRoot required).expression [] required.fuelLimit [] with
          | none => false
          | some result => result.value = required.expected.value &&
              result.fuel = required.expected.remainingFuel &&
              ABI.observations result.observations = required.expected.observations &&
              checkGraph predecessor.package.subject.program
                predecessor.package.manifest.physicalProfile required certificate.nodes

end Certificate

end ClauseCompiler
