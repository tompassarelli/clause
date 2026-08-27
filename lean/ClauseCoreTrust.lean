import Lean
import ClauseCore

/-!
# Clause Core trust gate

This build-time audit inspects declarations originating in the `ClauseCore`
module. It is verification machinery, not part of Clause's semantic model.

Lean emits partial internal runtime helpers for the total recursive
`Term.sameRepresentation` and finite premise-reference matcher. The audit
allows those exact generated names and requires both to be present; every other
partial declaration and every unsafe, foreign, or replacement implementation
is rejected.

The initial axiom policy admits only `propext`, which Lean uses in generated
injectivity support for the dependent Term constructors. `Quot.sound`,
`Classical.choice`, and every other axiom remain disallowed.
-/

open Lean Elab Command

run_cmd do
  let environment ← getEnv
  let some clauseModuleIndex := environment.getModuleIdx? `ClauseCore
    | throwError "ClauseCore module is absent from the imported environment"
  let allowedPartialRuntimeHelpers : Array Name :=
    #[`ClauseCore.Term.sameRepresentation._unsafe_rec,
      `ClauseCore.DerivationCertificate.referencesMatch._unsafe_rec]
  let mut observedPartialRuntimeHelpers : Array Name := #[]
  let mut checkedDeclarations := 0
  for (name, info) in environment.constants do
    if environment.getModuleIdxFor? name == some clauseModuleIndex then
      checkedDeclarations := checkedDeclarations + 1
      if info.isUnsafe then
        throwError "unsafe ClauseCore declaration: {name}"
      if info.isPartial then
        unless allowedPartialRuntimeHelpers.contains name do
          throwError "unexpected partial ClauseCore declaration: {name}"
        observedPartialRuntimeHelpers := observedPartialRuntimeHelpers.push name
      if (Compiler.getImplementedBy? environment name).isSome then
        throwError "implemented_by ClauseCore declaration: {name}"
      if (getExternAttrData? environment name).isSome then
        throwError "extern ClauseCore declaration: {name}"
      let axioms ← collectAxioms name
      for axiomName in axioms do
        unless axiomName == ``propext do
          throwError "ClauseCore declaration {name} depends on disallowed axiom: {axiomName}"
  if checkedDeclarations == 0 then
    throwError "ClauseCore trust audit found no declarations"
  for allowedName in allowedPartialRuntimeHelpers do
    unless observedPartialRuntimeHelpers.contains allowedName do
      throwError "expected generated runtime helper is absent: {allowedName}"
