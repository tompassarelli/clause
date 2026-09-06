import ClauseCompiler

/-! Test-only exact-byte authority ingress for the host-freeze falsifier.

The one axiom supplies the isolated test's external predecessor premise. It
does not belong to the compiler constitution, its trust roots, or a real owner
selection. The selected bytes remain a separate checker input; candidate bytes
cannot replace them. All parsing, evaluation, replay, and verdicts below use
the ordinary construct-blind compiler implementation.
-/

open ClauseCompiler

private axiom testOnlyExactSelection (selectedBytes : Bytes) :
  AcceptedExact selectedBytes

private def readBytes (path : String) : IO Bytes :=
  return (← IO.FS.readBinFile path).toList

private def decoded (bytes : Bytes) : IO DecodedPackage :=
  match Codec.strictDecode bytes with
  | .decoded candidate => pure candidate
  | .rejected failure => throw (IO.userError s!"decode rejected: {repr failure}")

private def writeBytes (path : String) (bytes : Bytes) : IO Unit :=
  IO.FS.writeBinFile path (ByteArray.mk bytes.toArray)

private def checkedVerdict (exactBytes : Bytes)
    (verdict : DecodeVerdict × Option AuthorizationVerdict) : IO Unit :=
  match verdict with
  | (.decoded _, some (.authorized authorized)) =>
      if authorized == exactBytes then IO.println "Authorized exact candidate bytes"
      else throw (IO.userError "checker returned different bytes")
  | (_, some (.unauthorized failure)) =>
      throw (IO.userError s!"Unauthorized {failure.stage.tag} {failure.code.tag}")
  | (.rejected failure, _) =>
      throw (IO.userError s!"decode rejected: {repr failure}")
  | _ => throw (IO.userError "checker omitted its verdict")

private def checkGenesis (anchorPath candidatePath : String) : IO Unit := do
  let selected ← readBytes anchorPath
  let bytes ← readBytes candidatePath
  let candidate ← decoded bytes
  let some request := ABI.decodeBuildRequest candidate.package.subject.buildRequest
    | throw (IO.userError "invalid build request")
  let anchor := OwnerAnchorWitness.fromExternalSelection {
    exactSelectedBytes := selected
    selectedByteLength := selected.length
    selectedPackageHash := compilerPackageHash selected
  }
  checkedVerdict bytes (Authorization.authorizeBytesGenesis {
    ownerAnchor := .supplied anchor
    buildRequest := candidate.package.subject.buildRequest
    evidence := candidate.package.evidence
    compileFuelLimit := request.compileFuel
    admissionFuelLimit := request.admissionFuel
    finalIdentity := ⟨compilerPackageHash bytes, bytes⟩
  } bytes)

private def checkSuccessor (anchorPath offeredPath candidatePath : String) : IO Unit := do
  let selected ← readBytes anchorPath
  let offered ← readBytes offeredPath
  let bytes ← readBytes candidatePath
  let candidate ← decoded bytes
  checkedVerdict bytes (Authorization.authorizeBytesSuccessor {
    predecessor := .accepted selected (testOnlyExactSelection selected) offered
    buildRequest := candidate.package.subject.buildRequest
    evidence := candidate.package.evidence
    finalIdentity := ⟨compilerPackageHash bytes, bytes⟩
  } bytes)

private def readTerm (path : String) : IO Term := do
  let bytes ← readBytes path
  match Codec.term ⟨bytes, bytes, 0, bytes.length⟩ with
  | .ok (term, after) =>
      if after.position == bytes.length && Encoding.term term == some bytes then
        pure term
      else throw (IO.userError "noncanonical term or trailing bytes")
  | .error failure => throw (IO.userError s!"term decode rejected: {repr failure}")

private def evaluate (packagePath entrypointPath argumentPath fuelText outputPath : String) :
    IO Unit := do
  let exact ← readBytes packagePath
  let package ← decoded exact
  let entrypoint ← readBytes entrypointPath
  let argument ← readTerm argumentPath
  let some fuel := fuelText.toNat?
    | throw (IO.userError "invalid fuel")
  let request : EvalRequest := {
    acceptedPredecessorPackageHash := compilerPackageHash exact
    coreContractId := Fixed.coreContractId
    physicalProfileId := Fixed.physicalProfileId
    entrypoint := entrypoint
    arguments := [.term argument]
    fuelLimit := fuel
  }
  unless Replay.requestWellFormed exact package request do
    throw (IO.userError "invalid evaluation request")
  let some result := Replay.run package request
    | throw (IO.userError "evaluation rejected or exhausted fuel")
  let some value := Encoding.kvalue result.value
    | throw (IO.userError "result encoding failed")
  let some observations := Encoding.term (ABI.observations result.observations)
    | throw (IO.userError "observation encoding failed")
  writeBytes outputPath value
  writeBytes (outputPath ++ ".observations") observations
  IO.FS.writeFile (outputPath ++ ".fuel") (toString result.fuel ++ "\n")

def main (arguments : List String) : IO UInt32 := do
  try
    match arguments with
    | ["test-genesis", anchor, candidate] => checkGenesis anchor candidate
    | ["test-successor", anchor, offered, candidate] => checkSuccessor anchor offered candidate
    | ["evaluate", package, entrypoint, argument, fuel, output] =>
        evaluate package entrypoint argument fuel output
    | _ => throw (IO.userError "expected test-genesis, test-successor, or evaluate")
    pure 0
  catch error =>
    IO.eprintln error
    pure 1
