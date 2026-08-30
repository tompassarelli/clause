import ClauseCompiler.ReplayPlanModel

/-!
# Canonical checkpoint replay-segment codec

Replay segments carry exact evaluator states but never repeat expression
syntax.  Every expression position is a canonical U32 reference into the
context deterministically derived from the initial replay expression followed
by definition bodies in program order.  The decoder is total, consumes its
input exactly, and reconstructs only the existing evaluator and replay-plan
types.
-/

namespace ClauseCompiler.ReplaySegmentCodec

open ReplayPlanModel

abbrev Decoder := Codec.Decoder

mutual
  def expressionCatalog : KExpr → List KExpr
    | .bytesLiteral value => [.bytesLiteral value]
    | .termLiteral value => [.termLiteral value]
    | .var index => [.var index]
    | .makeAtom kind payload equality =>
        .makeAtom kind payload equality ::
          (expressionCatalog kind ++ expressionCatalog payload ++
            expressionCatalog equality)
    | .makeTriple first second third =>
        .makeTriple first second third ::
          (expressionCatalog first ++ expressionCatalog second ++
            expressionCatalog third)
    | .letValue value body =>
        .letValue value body ::
          (expressionCatalog value ++ expressionCatalog body)
    | .caseTerm scrutinee atomBody tripleBody =>
        .caseTerm scrutinee atomBody tripleBody ::
          (expressionCatalog scrutinee ++ expressionCatalog atomBody ++
            expressionCatalog tripleBody)
    | .caseBytes scrutinee emptyBody consBody =>
        .caseBytes scrutinee emptyBody consBody ::
          (expressionCatalog scrutinee ++ expressionCatalog emptyBody ++
            expressionCatalog consBody)
    | .concatBytes parts =>
        .concatBytes parts :: expressionSequenceCatalog parts
    | .caseBytesEqual left right equalBody unequalBody =>
        .caseBytesEqual left right equalBody unequalBody ::
          (expressionCatalog left ++ expressionCatalog right ++
            expressionCatalog equalBody ++ expressionCatalog unequalBody)
    | .call definitionId arguments =>
        .call definitionId arguments :: expressionSequenceCatalog arguments
    | .request operationId arguments =>
        .request operationId arguments :: expressionSequenceCatalog arguments

  def expressionSequenceCatalog : KExprSeq → List KExpr
    | .nil => []
    | .cons head tail => expressionCatalog head ++ expressionSequenceCatalog tail
end

structure Context where
  expressions : List KExpr

def deriveContext (program : List Definition) (initialExpression : KExpr) : Context := {
  expressions := expressionCatalog initialExpression ++
    program.flatMap (fun definition => expressionCatalog definition.body)
}

def findExpressionBytes (target : Bytes) : List KExpr → Nat → Option Nat
  | [], _ => none
  | expression :: remaining, index =>
      if Encoding.expr expression = some target then some index
      else findExpressionBytes target remaining (index + 1)

def expressionIndex (context : Context) (expression : KExpr) : Option Nat := do
  let target ← Encoding.expr expression
  findExpressionBytes target context.expressions 0

def encodeExpressionReference (context : Context) (expression : KExpr) : Option Bytes := do
  Encoding.u32 (← expressionIndex context expression)

def decodeExpressionReference (context : Context) : Decoder KExpr := fun cursor => do
  let offset := cursor.position
  let (index, after) ← Codec.u32 cursor
  match nth? context.expressions index with
  | none => .error { code := .invalidFixedWidth, offset := offset }
  | some expression =>
      if expressionIndex context expression = some index then
        pure (expression, after)
      else
        .error { code := .invalidFixedWidth, offset := offset }

def encodeExpressionSequence (context : Context) (values : KExprSeq) : Option Bytes :=
  Encoding.seq (encodeExpressionReference context) values.toList

def decodeExpressionSequence (context : Context) : Decoder KExprSeq := fun cursor => do
  let (values, after) ← Codec.counted (decodeExpressionReference context) cursor
  pure (KExprSeq.ofList values, after)

def encodeValues (values : List KValue) : Option Bytes :=
  Encoding.seq Encoding.kvalue values

def encodeTerms (values : List Term) : Option Bytes :=
  Encoding.seq Encoding.term values

def decodeValues : Decoder (List KValue) := Codec.counted Codec.kvalue

def decodeTerms : Decoder (List Term) := Codec.counted Codec.term

def encodeSequencePurpose (context : Context) :
    Evaluator.SequencePurpose → Option Bytes
  | .concatenate => some [0x00]
  | .callBody body => do
      pure (0x01 :: (← encodeExpressionReference context body))
  | .requestDigest operationId => do
      pure (0x02 :: (← Encoding.fixed32 operationId))

def decodeSequencePurpose (context : Context) : Decoder Evaluator.SequencePurpose :=
    fun cursor => do
  let offset := cursor.position
  let (tag, afterTag) ← Codec.readByte cursor
  match tag with
  | 0x00 => pure (.concatenate, afterTag)
  | 0x01 =>
      let (body, after) ← decodeExpressionReference context afterTag
      pure (.callBody body, after)
  | 0x02 =>
      let (operationId, after) ← Codec.fixed32 afterTag
      pure (.requestDigest operationId, after)
  | _ => .error { code := .unknownSumTag, offset := offset }

def encodeControl (context : Context) : Evaluator.Control → Option Bytes
  | .evaluate expression environment => do
      let fields ← Encoding.many [
        encodeExpressionReference context expression,
        encodeValues environment]
      pure (0x00 :: fields)
  | .returned value => do
      pure (0x01 :: (← Encoding.kvalue value))

def decodeControl (context : Context) : Decoder Evaluator.Control := fun cursor => do
  let offset := cursor.position
  let (tag, afterTag) ← Codec.readByte cursor
  match tag with
  | 0x00 =>
      let (expression, c1) ← decodeExpressionReference context afterTag
      let (environment, c2) ← decodeValues c1
      pure (.evaluate expression environment, c2)
  | 0x01 =>
      let (value, after) ← Codec.kvalue afterTag
      pure (.returned value, after)
  | _ => .error { code := .unknownSumTag, offset := offset }

def encodeContinuation (context : Context) :
    Evaluator.Continuation → Option Bytes
  | .atomPayload payload equality environment => do
      let fields ← Encoding.many [
        encodeExpressionReference context payload,
        encodeExpressionReference context equality,
        encodeValues environment]
      pure (0x00 :: fields)
  | .atomEquality kind equality environment => do
      let fields ← Encoding.many [Encoding.blob kind,
        encodeExpressionReference context equality, encodeValues environment]
      pure (0x01 :: fields)
  | .atomFinish kind payload => do
      pure (0x02 :: (← Encoding.many [Encoding.blob kind, Encoding.blob payload]))
  | .tripleSecond second third environment => do
      let fields ← Encoding.many [
        encodeExpressionReference context second,
        encodeExpressionReference context third,
        encodeValues environment]
      pure (0x03 :: fields)
  | .tripleThird first third environment => do
      let fields ← Encoding.many [Encoding.term first,
        encodeExpressionReference context third, encodeValues environment]
      pure (0x04 :: fields)
  | .tripleFinish first second => do
      pure (0x05 :: (← Encoding.many [Encoding.term first, Encoding.term second]))
  | .letBody body environment => do
      let fields ← Encoding.many [encodeExpressionReference context body,
        encodeValues environment]
      pure (0x06 :: fields)
  | .caseTerm atomBody tripleBody environment => do
      let fields ← Encoding.many [
        encodeExpressionReference context atomBody,
        encodeExpressionReference context tripleBody,
        encodeValues environment]
      pure (0x07 :: fields)
  | .caseBytes emptyBody consBody environment => do
      let fields ← Encoding.many [
        encodeExpressionReference context emptyBody,
        encodeExpressionReference context consBody,
        encodeValues environment]
      pure (0x08 :: fields)
  | .equalRight right equalBody unequalBody environment => do
      let fields ← Encoding.many [
        encodeExpressionReference context right,
        encodeExpressionReference context equalBody,
        encodeExpressionReference context unequalBody,
        encodeValues environment]
      pure (0x09 :: fields)
  | .equalBranch left equalBody unequalBody environment => do
      let fields ← Encoding.many [Encoding.blob left,
        encodeExpressionReference context equalBody,
        encodeExpressionReference context unequalBody,
        encodeValues environment]
      pure (0x0a :: fields)
  | .sequence purpose remaining environment valuesReversed => do
      let fields ← Encoding.many [encodeSequencePurpose context purpose,
        encodeExpressionSequence context remaining,
        encodeValues environment, encodeValues valuesReversed]
      pure (0x0b :: fields)

def decodeContinuation (context : Context) : Decoder Evaluator.Continuation :=
    fun cursor => do
  let offset := cursor.position
  let (tag, afterTag) ← Codec.readByte cursor
  match tag with
  | 0x00 =>
      let (payload, c1) ← decodeExpressionReference context afterTag
      let (equality, c2) ← decodeExpressionReference context c1
      let (environment, c3) ← decodeValues c2
      pure (.atomPayload payload equality environment, c3)
  | 0x01 =>
      let (kind, c1) ← Codec.blob afterTag
      let (equality, c2) ← decodeExpressionReference context c1
      let (environment, c3) ← decodeValues c2
      pure (.atomEquality kind equality environment, c3)
  | 0x02 =>
      let (kind, c1) ← Codec.blob afterTag
      let (payload, c2) ← Codec.blob c1
      pure (.atomFinish kind payload, c2)
  | 0x03 =>
      let (second, c1) ← decodeExpressionReference context afterTag
      let (third, c2) ← decodeExpressionReference context c1
      let (environment, c3) ← decodeValues c2
      pure (.tripleSecond second third environment, c3)
  | 0x04 =>
      let (first, c1) ← Codec.term afterTag
      let (third, c2) ← decodeExpressionReference context c1
      let (environment, c3) ← decodeValues c2
      pure (.tripleThird first third environment, c3)
  | 0x05 =>
      let (first, c1) ← Codec.term afterTag
      let (second, c2) ← Codec.term c1
      pure (.tripleFinish first second, c2)
  | 0x06 =>
      let (body, c1) ← decodeExpressionReference context afterTag
      let (environment, c2) ← decodeValues c1
      pure (.letBody body environment, c2)
  | 0x07 =>
      let (atomBody, c1) ← decodeExpressionReference context afterTag
      let (tripleBody, c2) ← decodeExpressionReference context c1
      let (environment, c3) ← decodeValues c2
      pure (.caseTerm atomBody tripleBody environment, c3)
  | 0x08 =>
      let (emptyBody, c1) ← decodeExpressionReference context afterTag
      let (consBody, c2) ← decodeExpressionReference context c1
      let (environment, c3) ← decodeValues c2
      pure (.caseBytes emptyBody consBody environment, c3)
  | 0x09 =>
      let (right, c1) ← decodeExpressionReference context afterTag
      let (equalBody, c2) ← decodeExpressionReference context c1
      let (unequalBody, c3) ← decodeExpressionReference context c2
      let (environment, c4) ← decodeValues c3
      pure (.equalRight right equalBody unequalBody environment, c4)
  | 0x0a =>
      let (left, c1) ← Codec.blob afterTag
      let (equalBody, c2) ← decodeExpressionReference context c1
      let (unequalBody, c3) ← decodeExpressionReference context c2
      let (environment, c4) ← decodeValues c3
      pure (.equalBranch left equalBody unequalBody environment, c4)
  | 0x0b =>
      let (purpose, c1) ← decodeSequencePurpose context afterTag
      let (remaining, c2) ← decodeExpressionSequence context c1
      let (environment, c3) ← decodeValues c2
      let (valuesReversed, c4) ← decodeValues c3
      pure (.sequence purpose remaining environment valuesReversed, c4)
  | _ => .error { code := .unknownSumTag, offset := offset }

def encodeMachine (context : Context) (machine : MachineState) : Option Bytes :=
  Encoding.many [encodeControl context machine.control,
    Encoding.seq (encodeContinuation context) machine.continuations,
    Encoding.u64 machine.fuel, encodeTerms machine.observations]

def decodeMachine (context : Context) : Decoder MachineState := fun cursor => do
  let (control, c1) ← decodeControl context cursor
  let (continuations, c2) ← Codec.counted (decodeContinuation context) c1
  let (fuel, c3) ← Codec.u64 c2
  let (observations, c4) ← decodeTerms c3
  pure ({
    control := control
    continuations := continuations
    fuel := fuel
    observations := observations
  }, c4)

def encodeResult (result : Evaluator.Result) : Option Bytes :=
  Encoding.many [Encoding.kvalue result.value, Encoding.u64 result.fuel,
    encodeTerms result.observations]

def decodeResult : Decoder Evaluator.Result := fun cursor => do
  let (value, c1) ← Codec.kvalue cursor
  let (fuel, c2) ← Codec.u64 c1
  let (observations, c3) ← decodeTerms c2
  pure ({ value := value, fuel := fuel, observations := observations }, c3)

def encodeCheckpoint (context : Context)
    (checkpoint : ReplayCheckpoint) : Option Bytes :=
  Encoding.many [Encoding.u64 checkpoint.transitionsFromPrevious,
    encodeMachine context checkpoint.exactState]

def decodeCheckpoint (context : Context) : Decoder ReplayCheckpoint := fun cursor => do
  let (transitions, c1) ← Codec.u64 cursor
  let (state, c2) ← decodeMachine context c1
  pure ({ transitionsFromPrevious := transitions, exactState := state }, c2)

def encodeCompletion (completion : ReplayCompletion) : Option Bytes :=
  Encoding.many [Encoding.u64 completion.transitionsFromPrevious,
    encodeResult completion.exactResult]

def decodeCompletion : Decoder ReplayCompletion := fun cursor => do
  let (transitions, c1) ← Codec.u64 cursor
  let (result, c2) ← decodeResult c1
  pure ({ transitionsFromPrevious := transitions, exactResult := result }, c2)

def encodePlan (context : Context) : ReplayPlan → Option Bytes
  | .complete completion => do
      pure (0x01 :: (← encodeCompletion completion))
  | .checkpoint checkpoint remaining => do
      let fields ← Encoding.many [encodeCheckpoint context checkpoint,
        encodePlan context remaining]
      pure (0x00 :: fields)

def decodePlanFuel (context : Context) : Nat → Decoder ReplayPlan
  | 0 => Codec.failure .lengthOrCountOverflow 0
  | fuel + 1 => fun cursor => do
      let offset := cursor.position
      let (tag, afterTag) ← Codec.readByte cursor
      match tag with
      | 0x00 =>
          let (checkpoint, c1) ← decodeCheckpoint context afterTag
          let (remaining, c2) ← decodePlanFuel context fuel c1
          pure (.checkpoint checkpoint remaining, c2)
      | 0x01 =>
          let (completion, after) ← decodeCompletion afterTag
          pure (.complete completion, after)
      | _ => .error { code := .unknownSumTag, offset := offset }

def decodePlan (context : Context) : Decoder ReplayPlan := fun cursor =>
  decodePlanFuel context (cursor.limit - cursor.position + 1) cursor

def initialCursor (input : Bytes) : Codec.Cursor := {
  input := input
  remaining := input
  position := 0
  limit := input.length
}

def decodeExactly (context : Context) (input : Bytes) :
    Except DecodeFailure ReplayPlan := do
  let (plan, after) ← decodePlan context (initialCursor input)
  if after.position ≠ input.length then
    .error { code := .trailingBytes, offset := after.position }
  else
    match encodePlan context plan with
    | some canonical =>
        if canonical = input then pure plan
        else .error { code := .invalidFixedWidth, offset := 0 }
    | none => .error { code := .invalidFixedWidth, offset := 0 }

def failure? (context : Context) (input : Bytes) : Option DecodeFailure :=
  match decodeExactly context input with
  | .ok _ => none
  | .error failure => some failure

namespace Regression

def initialExpression : KExpr := .bytesLiteral []

def context : Context := deriveContext [] initialExpression

def completionPlan : ReplayPlan := .complete {
  transitionsFromPrevious := 0
  exactResult := Evaluator.result (.bytes []) 0 []
}

def completionBytes : Bytes := (encodePlan context completionPlan).getD []

def checkpointPlan : ReplayPlan := .checkpoint {
  transitionsFromPrevious := 0
  exactState := Evaluator.evaluating initialExpression [] [] 1 []
} completionPlan

def checkpointBytes : Bytes := (encodePlan context checkpointPlan).getD []

def invalidExpressionReferenceBytes : Bytes :=
  checkpointBytes.take 10 ++ [0x00, 0x00, 0x00, 0x01] ++
    checkpointBytes.drop 14

theorem invalidTagRejects :
    failure? context (0xff :: completionBytes.drop 1) =
      some { code := .unknownSumTag, offset := 0 } := by
  decide

theorem truncationRejects :
    failure? context (completionBytes.take 8) =
      some { code := .truncated, offset := 8 } := by
  decide

theorem invalidExpressionReferenceRejects :
    failure? context invalidExpressionReferenceBytes =
      some { code := .invalidFixedWidth, offset := 10 } := by
  decide

theorem trailingBytesReject :
    failure? context (completionBytes ++ [0xff]) =
      some { code := .trailingBytes, offset := completionBytes.length } := by
  decide

end Regression

end ClauseCompiler.ReplaySegmentCodec
