import Lean
import ClauseCore
import ClauseCoreVectors

/-!
# Clause Core trust gate

This build-time audit inspects declarations originating in the `ClauseCore`
and `ClauseCoreVectors` modules. It is verification machinery, not part of
Clause's semantic model.

Lean emits partial internal runtime helpers for the total recursive
Term comparer, finite premise-reference matcher, structural list encoder,
count-bounded list decoder, structural Term encoder, fuel-bounded Term decoder,
and fuel-bounded nested-predecessor validator. The audit allows those exact
seven generated names and requires all seven to be present; every other partial
declaration and every unsafe, foreign, or replacement implementation is
rejected.

The initial axiom policy admits only `propext`, which Lean uses in generated
injectivity support for the dependent Term constructors. `Quot.sound`,
`Classical.choice`, and every other axiom remain disallowed.
-/

open Lean Elab Command

run_cmd do
  let environment ← getEnv
  let some clauseModuleIndex := environment.getModuleIdx? `ClauseCore
    | throwError "ClauseCore module is absent from the imported environment"
  let some vectorModuleIndex := environment.getModuleIdx? `ClauseCoreVectors
    | throwError "ClauseCoreVectors module is absent from the imported environment"
  let allowedPartialRuntimeHelpers : Array Name :=
    #[`ClauseCore.Term.sameRepresentation._unsafe_rec,
      `ClauseCore.DerivationCertificate.referencesMatch._unsafe_rec,
      `ClauseCore.Codec.encodeSequence._unsafe_rec,
      `ClauseCore.Codec.decodeSequence._unsafe_rec,
      `ClauseCore.Codec.encodeTerm._unsafe_rec,
      `ClauseCore.Codec.decodeTermWithFuel._unsafe_rec]
  let predecessorValidationHelper :=
    "_private.ClauseCore.0.ClauseCore.Codec.validateLineagePredecessors._unsafe_rec"
  let mut observedPartialRuntimeHelpers : Array Name := #[]
  let mut observedPredecessorValidationHelper := false
  let mut checkedCoreDeclarations := 0
  let mut checkedVectorDeclarations := 0
  for (name, info) in environment.constants do
    let sourceModule := environment.getModuleIdxFor? name
    if sourceModule == some clauseModuleIndex ||
        sourceModule == some vectorModuleIndex then
      if sourceModule == some clauseModuleIndex then
        checkedCoreDeclarations := checkedCoreDeclarations + 1
      else
        checkedVectorDeclarations := checkedVectorDeclarations + 1
      if info.isUnsafe then
        throwError "unsafe ClauseCore declaration: {name}"
      if info.isPartial then
        unless allowedPartialRuntimeHelpers.contains name ||
            name.toString == predecessorValidationHelper do
          throwError "unexpected partial ClauseCore declaration: {name}"
        if name.toString == predecessorValidationHelper then
          observedPredecessorValidationHelper := true
        else
          observedPartialRuntimeHelpers := observedPartialRuntimeHelpers.push name
      if (Compiler.getImplementedBy? environment name).isSome then
        throwError "implemented_by ClauseCore declaration: {name}"
      if (getExternAttrData? environment name).isSome then
        throwError "extern ClauseCore declaration: {name}"
      let axioms ← collectAxioms name
      for axiomName in axioms do
        unless axiomName == ``propext do
          throwError "ClauseCore declaration {name} depends on disallowed axiom: {axiomName}"
  if checkedCoreDeclarations == 0 then
    throwError "ClauseCore trust audit found no declarations"
  if checkedVectorDeclarations == 0 then
    throwError "ClauseCoreVectors trust audit found no declarations"
  for allowedName in allowedPartialRuntimeHelpers do
    unless observedPartialRuntimeHelpers.contains allowedName do
      throwError "expected generated runtime helper is absent: {allowedName}"
  unless observedPredecessorValidationHelper do
    throwError "expected generated runtime helper is absent: {predecessorValidationHelper}"
