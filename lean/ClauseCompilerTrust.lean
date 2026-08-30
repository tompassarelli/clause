import Lean
import ClauseCompiler

/-!
# CLCP-v3 Lean trust gate

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
    `ClauseCompiler.Replay.verifyEvalReceipt,
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
  let allowedPartialRuntimeHelpers : Array String := #[]
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
