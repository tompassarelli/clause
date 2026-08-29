import Lean
import ClauseCompiler

/-!
# CLCP-v2 Lean trust gate

The audit admits the one explicitly named external predecessor-acceptance
premise.  It rejects unsafe, foreign, replacement, or any other axiomatic
implementation in the construct-blind decoder/checker/evaluator closure.
-/

open Lean Elab Command

run_cmd do
  let environment ← getEnv
  let modules : Array Name := #[
    `ClauseCompiler.Model,
    `ClauseCompiler.Encoding,
    `ClauseCompiler.Codec,
    `ClauseCompiler.Checker,
    `ClauseCompiler.Authorization,
    `ClauseCompiler]
  let moduleIndices := modules.filterMap environment.getModuleIdx?
  if moduleIndices.size != modules.size then
    throwError "ClauseCompiler trust audit is missing an imported module"
  let allowedPartialRuntimeHelpers : Array String := #[
    "ClauseCompiler.Authorization.findNominal._unsafe_rec",
    "ClauseCompiler.Static.infer._unsafe_rec",
    "ClauseCompiler.Codec.consumeMagic.loop._unsafe_rec",
    "ClauseCompiler.nth?._unsafe_rec",
    "ClauseCompiler.SHA256.expandWords._unsafe_rec",
    "ClauseCompiler.ABI.asListFuel._unsafe_rec",
    "ClauseCompiler.sortedUniqueBy._unsafe_rec",
    "ClauseCompiler.Encoding.many._unsafe_rec",
    "ClauseCompiler.Static.findOperation._unsafe_rec",
    "ClauseCompiler.ABI.list._unsafe_rec",
    "ClauseCompiler.instDecidableEqTerm.decEq._unsafe_rec",
    "ClauseCompiler.Evaluator.evaluate.evaluateSequence._unsafe_rec",
    "ClauseCompiler.Codec.decodeExprFuel._unsafe_rec",
    "ClauseCompiler.Codec.decodeTermFuel._unsafe_rec",
    "ClauseCompiler.Static.findDefinition._unsafe_rec",
    "ClauseCompiler.instReprTerm.repr._unsafe_rec",
    "ClauseCompiler.Authorization.requestSeqConform._unsafe_rec",
    "ClauseCompiler.Certificate.nodesValid.loop._unsafe_rec",
    "ClauseCompiler.Certificate.traceExpressions._unsafe_rec",
    "ClauseCompiler.Static.inferSeq._unsafe_rec",
    "ClauseCompiler.KExprSeq.length._unsafe_rec",
    "ClauseCompiler.contains._unsafe_rec",
    "ClauseCompiler.KExprSeq.toList._unsafe_rec",
    "ClauseCompiler.ABI.termBudget._unsafe_rec",
    "ClauseCompiler.all₂._unsafe_rec",
    "ClauseCompiler.Evaluator.evaluate._unsafe_rec",
    "ClauseCompiler.Evaluator.evaluate.concatenate._unsafe_rec",
    "ClauseCompiler.SHA256.rounds._unsafe_rec",
    "ClauseCompiler.Static.inferSeqAgainst._unsafe_rec",
    "ClauseCompiler.ABI.decodeAll._unsafe_rec",
    "ClauseCompiler.SHA256.blocks._unsafe_rec",
    "ClauseCompiler.all._unsafe_rec",
    "ClauseCompiler.Encoding.expr._unsafe_rec",
    "ClauseCompiler.Authorization.allocationAcyclic.visit._unsafe_rec",
    "ClauseCompiler.Encoding.exprSeqPayload._unsafe_rec",
    "ClauseCompiler.unique._unsafe_rec",
    "ClauseCompiler.Codec.counted.loop._unsafe_rec",
    "ClauseCompiler.Certificate.markReachable._unsafe_rec",
    "ClauseCompiler.Codec.readBytes._unsafe_rec",
    "ClauseCompiler.Authorization.requestsConform._unsafe_rec",
    "ClauseCompiler.KExprSeq.ofList._unsafe_rec",
    "ClauseCompiler.bytesLt._unsafe_rec",
    "ClauseCompiler.Encoding.term._unsafe_rec"]
  let mut observedPartialRuntimeHelpers : Array String := #[]
  let mut checked := 0
  for (name, info) in environment.constants do
    let inScope := match environment.getModuleIdxFor? name with
      | some sourceModule => moduleIndices.contains sourceModule
      | none => false
    if inScope then
      checked := checked + 1
      if info.isUnsafe then
        throwError "unsafe ClauseCompiler declaration: {name}"
      if info.isPartial then
        unless allowedPartialRuntimeHelpers.contains name.toString do
          throwError "unexpected partial ClauseCompiler declaration: {name}"
        observedPartialRuntimeHelpers := observedPartialRuntimeHelpers.push name.toString
      if (Compiler.getImplementedBy? environment name).isSome then
        throwError "implemented_by ClauseCompiler declaration: {name}"
      if (getExternAttrData? environment name).isSome then
        throwError "extern ClauseCompiler declaration: {name}"
      let axioms ← collectAxioms name
      for axiomName in axioms do
        unless axiomName == `ClauseCompiler.AcceptedExact || axiomName == ``propext ||
            axiomName == ``Quot.sound do
          throwError "ClauseCompiler declaration {name} depends on disallowed axiom: {axiomName}"
  if checked == 0 then
    throwError "ClauseCompiler trust audit found no declarations"
  for allowedName in allowedPartialRuntimeHelpers do
    unless observedPartialRuntimeHelpers.contains allowedName do
      throwError "expected generated runtime helper is absent: {allowedName}"
