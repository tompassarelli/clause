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
  let roots : Array Name := #[
    `ClauseCompiler.Authorization.authorizeBytesGenesis,
    `ClauseCompiler.Authorization.authorizeBytesSuccessor,
    `ClauseCompiler.Certificate.verifyEvalCertificate,
    `ClauseCompiler.Codec.strictDecode,
    `ClauseCompiler.Encoding.package]
  for root in roots do
    unless environment.contains root do
      throwError "ClauseCompiler trust audit is missing root {root}"

  /- Init is the pinned kernel/core-library boundary.  Every reachable
  declaration outside that one module family is traversed through both its
  type and body and audited, regardless of its source module. -/
  let trustedBoundaryModules : Array Name := #[`Init]
  let isTrustedBoundary (name : Name) : Bool :=
    match environment.getModuleIdxFor? name with
    | none => false
    | some moduleIndex =>
        match environment.header.moduleNames[moduleIndex]? with
        | none => false
        | some moduleName =>
            trustedBoundaryModules.any (fun boundaryPrefix =>
              boundaryPrefix.isPrefixOf moduleName)
  let allowedPartialRuntimeHelpers : Array String := #[
    "ClauseCompiler.Authorization.findNominal._unsafe_rec",
    "ClauseCompiler.Authorization.termNominalReferencesValid._unsafe_rec",
    "ClauseCompiler.Authorization.expressionNominalReferencesValid._unsafe_rec",
    "ClauseCompiler.Authorization.expressionSeqNominalReferencesValid._unsafe_rec",
    "ClauseCompiler.Static.infer._unsafe_rec",
    "ClauseCompiler.Static.inferSeqAny._unsafe_rec",
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
    "ClauseCompiler.Static.requestSeqConform._unsafe_rec",
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
    "ClauseCompiler.Static.requestsConform._unsafe_rec",
    "ClauseCompiler.KExprSeq.ofList._unsafe_rec",
    "ClauseCompiler.bytesLt._unsafe_rec",
    "ClauseCompiler.Encoding.term._unsafe_rec"]
  let mut pending := roots.toList
  let mut visited : NameSet := {}
  let mut checked := 0
  let mut boundaryDeclarations := 0
  while let name :: tail := pending do
    pending := tail
    if visited.contains name then continue
    visited := visited.insert name
    if isTrustedBoundary name then
      boundaryDeclarations := boundaryDeclarations + 1
      continue
    let some info := environment.find? name
      | throwError "reachable ClauseCompiler declaration is absent: {name}"
    checked := checked + 1
    if info.isUnsafe then
      throwError "unsafe declaration in ClauseCompiler trust closure: {name}"
    if info.isPartial then
      unless allowedPartialRuntimeHelpers.contains name.toString do
        throwError "unexpected partial declaration in ClauseCompiler trust closure: {name}"
    if (Compiler.getImplementedBy? environment name).isSome then
      throwError "implemented_by declaration in ClauseCompiler trust closure: {name}"
    if (getExternAttrData? environment name).isSome then
      throwError "extern declaration in ClauseCompiler trust closure: {name}"
    for dependency in info.getUsedConstantsAsSet do
      unless visited.contains dependency do
        pending := dependency :: pending
  if checked == 0 || boundaryDeclarations == 0 then
    throwError "ClauseCompiler trust audit found an empty root closure or boundary"
  for root in roots do
    let axioms ← collectAxioms root
    for axiomName in axioms do
      unless axiomName == `ClauseCompiler.AcceptedExact || axiomName == ``propext ||
          axiomName == ``Quot.sound do
        throwError "ClauseCompiler root {root} depends on disallowed axiom: {axiomName}"
