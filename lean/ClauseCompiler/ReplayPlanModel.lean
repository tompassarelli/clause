import ClauseCompiler.Checker

/-!
# Closed checkpoint replay-plan model

This module defines only the data and representability boundary for a replay
plan.  A checkpoint retains the evaluator's exact machine state; it does not
summarize, normalize, or reconstruct one.  Plan checking and any artifact
encoding are deliberately outside this module.
-/

namespace ClauseCompiler.ReplayPlanModel

abbrev MachineState := Evaluator.Machine

structure ReplayCheckpoint where
  transitionsFromPrevious : Nat
  exactState : MachineState

structure ReplayCompletion where
  transitionsFromPrevious : Nat
  exactResult : Evaluator.Result

inductive ReplayPlan where
  | complete (completion : ReplayCompletion)
  | checkpoint (checkpoint : ReplayCheckpoint) (remaining : ReplayPlan)

inductive ListFailureLocator where
  | count
  | element (index : Nat)
deriving DecidableEq, Repr

inductive SequencePurposeFailureLocator where
  | callBody
  | requestOperationId
deriving DecidableEq, Repr

inductive ControlFailureLocator where
  | expression
  | environment (location : ListFailureLocator)
  | returnedValue
deriving DecidableEq, Repr

inductive ContinuationFailureLocator where
  | atomPayloadPayload
  | atomPayloadEquality
  | atomPayloadEnvironment (location : ListFailureLocator)
  | atomEqualityKind
  | atomEqualityEquality
  | atomEqualityEnvironment (location : ListFailureLocator)
  | atomFinishKind
  | atomFinishPayload
  | tripleSecondSecond
  | tripleSecondThird
  | tripleSecondEnvironment (location : ListFailureLocator)
  | tripleThirdFirst
  | tripleThirdThird
  | tripleThirdEnvironment (location : ListFailureLocator)
  | tripleFinishFirst
  | tripleFinishSecond
  | letBodyBody
  | letBodyEnvironment (location : ListFailureLocator)
  | caseTermAtomBody
  | caseTermTripleBody
  | caseTermEnvironment (location : ListFailureLocator)
  | caseBytesEmptyBody
  | caseBytesConsBody
  | caseBytesEnvironment (location : ListFailureLocator)
  | equalRightRight
  | equalRightEqualBody
  | equalRightUnequalBody
  | equalRightEnvironment (location : ListFailureLocator)
  | equalBranchLeft
  | equalBranchEqualBody
  | equalBranchUnequalBody
  | equalBranchEnvironment (location : ListFailureLocator)
  | sequencePurpose (location : SequencePurposeFailureLocator)
  | sequenceRemainingExpressions (location : ListFailureLocator)
  | sequenceEnvironment (location : ListFailureLocator)
  | sequenceValuesReversed (location : ListFailureLocator)
deriving DecidableEq, Repr

inductive MachineFailureLocator where
  | control (location : ControlFailureLocator)
  | continuationCount
  | continuation (index : Nat) (location : ContinuationFailureLocator)
  | fuel
  | observationCount
  | observation (index : Nat)
deriving DecidableEq, Repr

inductive ResultFailureLocator where
  | value
  | fuel
  | observationCount
  | observation (index : Nat)
deriving DecidableEq, Repr

inductive PlanFailureLocator where
  | checkpointTransitions (artifactIndex : Nat)
  | checkpointState (artifactIndex : Nat) (location : MachineFailureLocator)
  | completionTransitions (artifactIndex : Nat)
  | completionResult (artifactIndex : Nat) (location : ResultFailureLocator)
deriving DecidableEq, Repr

def optionSucceeded : Option α → Bool
  | some _ => true
  | none => false

def noFailure : Option α → Bool
  | none => true
  | some _ => false

def u32Representable (value : Nat) : Bool :=
  optionSucceeded (Encoding.u32 value)

def u64Representable (value : Nat) : Bool :=
  optionSucceeded (Encoding.u64 value)

def bytesRepresentable (value : Bytes) : Bool :=
  optionSucceeded (Encoding.blob value)

def id32Representable (value : Id32) : Bool :=
  optionSucceeded (Encoding.fixed32 value)

def termRepresentable (value : Term) : Bool :=
  optionSucceeded (Encoding.term value)

def expressionRepresentable (value : KExpr) : Bool :=
  optionSucceeded (Encoding.expr value)

def valueRepresentable (value : KValue) : Bool :=
  optionSucceeded (Encoding.kvalue value)

def failureUnless (condition : Bool) (failure : α) : Option α :=
  if condition then none else some failure

def firstFailure : List (Unit → Option α) → Option α
  | [] => none
  | candidate :: remaining =>
      match candidate () with
      | some failure => some failure
      | none => firstFailure remaining

def firstIndexedFailure (locate : α → Option β) : List α → Nat → Option (Nat × β)
  | [], _ => none
  | value :: remaining, index =>
      match locate value with
      | some failure => some (index, failure)
      | none => firstIndexedFailure locate remaining (index + 1)

def firstElementFailure (representable : α → Bool) (values : List α) : Option Nat :=
  (firstIndexedFailure
    (fun value => failureUnless (representable value) ()) values 0).map Prod.fst

def firstListFailure (representable : α → Bool)
    (values : List α) : Option ListFailureLocator :=
  if !u32Representable values.length then some .count
  else (firstElementFailure representable values).map .element

def expressionSequenceRepresentable (values : KExprSeq) : Bool :=
  optionSucceeded (do
    let count <- Encoding.u32 values.length
    let payload <- Encoding.exprSeqPayload values
    pure (count ++ payload))

def firstExpressionSequenceFailure
    (values : KExprSeq) : Option ListFailureLocator :=
  firstListFailure expressionRepresentable values.toList

def firstSequencePurposeFailure :
    Evaluator.SequencePurpose → Option SequencePurposeFailureLocator
  | .concatenate => none
  | .callBody body =>
      failureUnless (expressionRepresentable body) .callBody
  | .requestDigest operationId =>
      failureUnless (id32Representable operationId) .requestOperationId

def sequencePurposeRepresentable (purpose : Evaluator.SequencePurpose) : Bool :=
  noFailure (firstSequencePurposeFailure purpose)

def firstControlFailure : Evaluator.Control → Option ControlFailureLocator
  | .evaluate expression environment => firstFailure [
      fun _ => failureUnless (expressionRepresentable expression) .expression,
      fun _ => (firstListFailure valueRepresentable environment).map .environment
    ]
  | .returned value =>
      failureUnless (valueRepresentable value) .returnedValue

def controlRepresentable (control : Evaluator.Control) : Bool :=
  noFailure (firstControlFailure control)

def firstContinuationFailure :
    Evaluator.Continuation → Option ContinuationFailureLocator
  | .atomPayload payload equality environment => firstFailure [
      fun _ => failureUnless (expressionRepresentable payload) .atomPayloadPayload,
      fun _ => failureUnless (expressionRepresentable equality) .atomPayloadEquality,
      fun _ => (firstListFailure valueRepresentable environment).map
        .atomPayloadEnvironment
    ]
  | .atomEquality kind equality environment => firstFailure [
      fun _ => failureUnless (bytesRepresentable kind) .atomEqualityKind,
      fun _ => failureUnless (expressionRepresentable equality) .atomEqualityEquality,
      fun _ => (firstListFailure valueRepresentable environment).map
        .atomEqualityEnvironment
    ]
  | .atomFinish kind payload => firstFailure [
      fun _ => failureUnless (bytesRepresentable kind) .atomFinishKind,
      fun _ => failureUnless (bytesRepresentable payload) .atomFinishPayload
    ]
  | .tripleSecond second third environment => firstFailure [
      fun _ => failureUnless (expressionRepresentable second) .tripleSecondSecond,
      fun _ => failureUnless (expressionRepresentable third) .tripleSecondThird,
      fun _ => (firstListFailure valueRepresentable environment).map
        .tripleSecondEnvironment
    ]
  | .tripleThird first third environment => firstFailure [
      fun _ => failureUnless (termRepresentable first) .tripleThirdFirst,
      fun _ => failureUnless (expressionRepresentable third) .tripleThirdThird,
      fun _ => (firstListFailure valueRepresentable environment).map
        .tripleThirdEnvironment
    ]
  | .tripleFinish first second => firstFailure [
      fun _ => failureUnless (termRepresentable first) .tripleFinishFirst,
      fun _ => failureUnless (termRepresentable second) .tripleFinishSecond
    ]
  | .letBody body environment => firstFailure [
      fun _ => failureUnless (expressionRepresentable body) .letBodyBody,
      fun _ => (firstListFailure valueRepresentable environment).map
        .letBodyEnvironment
    ]
  | .caseTerm atomBody tripleBody environment => firstFailure [
      fun _ => failureUnless (expressionRepresentable atomBody) .caseTermAtomBody,
      fun _ => failureUnless (expressionRepresentable tripleBody) .caseTermTripleBody,
      fun _ => (firstListFailure valueRepresentable environment).map
        .caseTermEnvironment
    ]
  | .caseBytes emptyBody consBody environment => firstFailure [
      fun _ => failureUnless (expressionRepresentable emptyBody) .caseBytesEmptyBody,
      fun _ => failureUnless (expressionRepresentable consBody) .caseBytesConsBody,
      fun _ => (firstListFailure valueRepresentable environment).map
        .caseBytesEnvironment
    ]
  | .equalRight right equalBody unequalBody environment => firstFailure [
      fun _ => failureUnless (expressionRepresentable right) .equalRightRight,
      fun _ => failureUnless (expressionRepresentable equalBody) .equalRightEqualBody,
      fun _ => failureUnless (expressionRepresentable unequalBody)
        .equalRightUnequalBody,
      fun _ => (firstListFailure valueRepresentable environment).map
        .equalRightEnvironment
    ]
  | .equalBranch left equalBody unequalBody environment => firstFailure [
      fun _ => failureUnless (bytesRepresentable left) .equalBranchLeft,
      fun _ => failureUnless (expressionRepresentable equalBody) .equalBranchEqualBody,
      fun _ => failureUnless (expressionRepresentable unequalBody)
        .equalBranchUnequalBody,
      fun _ => (firstListFailure valueRepresentable environment).map
        .equalBranchEnvironment
    ]
  | .sequence purpose remaining environment valuesReversed => firstFailure [
      fun _ => (firstSequencePurposeFailure purpose).map .sequencePurpose,
      fun _ => (firstExpressionSequenceFailure remaining).map
        .sequenceRemainingExpressions,
      fun _ => (firstListFailure valueRepresentable environment).map
        .sequenceEnvironment,
      fun _ => (firstListFailure valueRepresentable valuesReversed).map
        .sequenceValuesReversed
    ]

def continuationRepresentable (continuation : Evaluator.Continuation) : Bool :=
  noFailure (firstContinuationFailure continuation)

def firstMachineFailure (machine : MachineState) : Option MachineFailureLocator :=
  firstFailure [
    fun _ => (firstControlFailure machine.control).map .control,
    fun _ => failureUnless (u32Representable machine.continuations.length)
      .continuationCount,
    fun _ => (firstIndexedFailure firstContinuationFailure
      machine.continuations 0).map (fun (index, failure) =>
        .continuation index failure),
    fun _ => failureUnless (u64Representable machine.fuel) .fuel,
    fun _ => failureUnless (u32Representable machine.observations.length)
      .observationCount,
    fun _ => (firstElementFailure termRepresentable machine.observations).map
      .observation
  ]

def firstResultFailure (result : Evaluator.Result) : Option ResultFailureLocator :=
  firstFailure [
    fun _ => failureUnless (valueRepresentable result.value) .value,
    fun _ => failureUnless (u64Representable result.fuel) .fuel,
    fun _ => failureUnless (u32Representable result.observations.length)
      .observationCount,
    fun _ => (firstElementFailure termRepresentable result.observations).map
      .observation
  ]

def firstPlanFailureFrom : ReplayPlan → Nat → Option PlanFailureLocator
  | .complete completion, index => firstFailure [
      fun _ => failureUnless
        (u64Representable completion.transitionsFromPrevious)
        (.completionTransitions index),
      fun _ => (firstResultFailure completion.exactResult).map
        (.completionResult index)
    ]
  | .checkpoint checkpoint remaining, index => firstFailure [
      fun _ => failureUnless
        (u64Representable checkpoint.transitionsFromPrevious)
        (.checkpointTransitions index),
      fun _ => (firstMachineFailure checkpoint.exactState).map
        (.checkpointState index),
      fun _ => firstPlanFailureFrom remaining (index + 1)
    ]

def firstPlanFailure (plan : ReplayPlan) : Option PlanFailureLocator :=
  firstPlanFailureFrom plan 0

def machineRepresentable (machine : MachineState) : Bool :=
  noFailure (firstMachineFailure machine)

def resultRepresentable (result : Evaluator.Result) : Bool :=
  noFailure (firstResultFailure result)

def checkpointRepresentable (checkpoint : ReplayCheckpoint) : Bool :=
  u64Representable checkpoint.transitionsFromPrevious &&
    machineRepresentable checkpoint.exactState

def completionRepresentable (completion : ReplayCompletion) : Bool :=
  u64Representable completion.transitionsFromPrevious &&
    resultRepresentable completion.exactResult

def planRepresentable (plan : ReplayPlan) : Bool :=
  noFailure (firstPlanFailure plan)

namespace Regression

def emptyReturningMachine : MachineState :=
  Evaluator.returning (.bytes []) [] 0 []

def invalidPurposeBeforeFuelMachine : MachineState :=
  Evaluator.returning (.bytes []) [
    .sequence (.requestDigest []) .nil [] []
  ] (Encoding.u64Maximum + 1) []

def invalidCheckpointBeforeStatePlan : ReplayPlan :=
  .checkpoint {
    transitionsFromPrevious := Encoding.u64Maximum + 1
    exactState := invalidPurposeBeforeFuelMachine
  } (.complete {
    transitionsFromPrevious := 0
    exactResult := Evaluator.result (.bytes []) 0 []
  })

theorem exactStateIsEvaluatorState (machine : Evaluator.Machine) :
    (machine : MachineState) = machine := rfl

theorem emptyReturningMachineIsRepresentable :
    machineRepresentable emptyReturningMachine = true := by
  decide

theorem continuationFailurePrecedesFuelFailure :
    firstMachineFailure invalidPurposeBeforeFuelMachine = some
      (.continuation 0 (.sequencePurpose .requestOperationId)) := by
  decide

theorem checkpointTransitionFailurePrecedesStateFailure :
    firstPlanFailure invalidCheckpointBeforeStatePlan =
      some (.checkpointTransitions 0) := by
  decide

end Regression

end ClauseCompiler.ReplayPlanModel
