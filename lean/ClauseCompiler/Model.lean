/-!
# Clause compiler constitutional model

This module is the fixed, construct-blind CLCP-v3 data model.  It deliberately
contains no Clause token, grammar, binder, type, effect, macro, diagnostic, or
compiler-version constructor.  Package programs can express those meanings
only as data evaluated by the twelve closed `KExpr` forms below.
-/

namespace ClauseCompiler

abbrev Bytes := List UInt8
abbrev Id32 := Bytes
abbrev Hash32 := Bytes
abbrev Fuel := Nat

def id32 (bytes : Bytes) : Option Id32 :=
  if bytes.length = 32 then some bytes else none

inductive Term where
  | atom (kind canonicalPayload equalityContract : Bytes)
  | triple (first second third : Term)
deriving DecidableEq, Repr

inductive KSort where
  | bytes
  | term
deriving DecidableEq, Repr

mutual
  inductive KExpr where
    | bytesLiteral (value : Bytes)
    | termLiteral (value : Term)
    | var (deBruijnIndex : Nat)
    | makeAtom (kind payload equality : KExpr)
    | makeTriple (first second third : KExpr)
    | letValue (value body : KExpr)
    | caseTerm (scrutinee atomBody tripleBody : KExpr)
    | caseBytes (scrutinee emptyBody consBody : KExpr)
    | concatBytes (parts : KExprSeq)
    | caseBytesEqual (left right equalBody unequalBody : KExpr)
    | call (definitionId : Id32) (arguments : KExprSeq)
    | request (physicalOperationId : Id32) (arguments : KExprSeq)

  inductive KExprSeq where
    | nil
    | cons (head : KExpr) (tail : KExprSeq)
end

namespace KExprSeq

def length : KExprSeq → Nat
  | .nil => 0
  | .cons _ tail => tail.length + 1

def toList : KExprSeq → List KExpr
  | .nil => []
  | .cons head tail => head :: tail.toList

def ofList : List KExpr → KExprSeq
  | [] => .nil
  | head :: tail => .cons head (ofList tail)

end KExprSeq

structure NamedSignature where
  tag : UInt8
  signature : Bytes
deriving DecidableEq, Repr

structure RuleSignature where
  tag : UInt8
  premisePolicy : UInt8
  clause : Bytes
deriving DecidableEq, Repr

structure PhysicalOperation where
  operationId : Id32
  arguments : List KSort
  result : KSort
deriving DecidableEq, Repr

structure PhysicalProfile where
  profileVersion : UInt8
  observationPolicy : UInt8
  operations : List PhysicalOperation
deriving DecidableEq, Repr

structure CoreManifest where
  manifestVersion : UInt8
  frameTags : List UInt8
  termTags : List UInt8
  sortTags : List UInt8
  expressionForms : List NamedSignature
  abiForms : List NamedSignature
  premisePolicyTags : List UInt8
  lineageTags : List UInt8
  nominalDeclarationTags : List UInt8
  compilerEvidenceTags : List UInt8
  valueTags : List UInt8
  decodeVerdictTags : List UInt8
  decodeCodeTags : List UInt8
  authorizationStageTags : List UInt8
  authorizationCodeTags : List UInt8
  staticRules : List RuleSignature
  evaluationRules : List RuleSignature
  receiptFormatVersion : UInt8
  receiptSignature : Bytes
  contractClauses : List Bytes
  physicalProfile : PhysicalProfile
deriving DecidableEq, Repr

inductive CompilerLineage where
  | genesis
  | successor (predecessorLocator : Hash32) (changeOccurrenceId : Id32)
deriving DecidableEq, Repr

inductive NominalDeclaration where
  | seed (domain id : Id32)
  | retainedSeed (domain id predecessorRevisionId : Id32)
  | allocated (domain id : Id32) (changeDomain changeId : Id32)
      (producerDomain producerId : Id32) (localSlot : Nat)
deriving DecidableEq, Repr

def NominalDeclaration.domain : NominalDeclaration → Id32
  | .seed domain _ | .retainedSeed domain _ _ | .allocated domain _ _ _ _ _ _ => domain

def NominalDeclaration.id : NominalDeclaration → Id32
  | .seed _ id | .retainedSeed _ id _ | .allocated _ id _ _ _ _ _ => id

structure CompilerInterface where
  compile : Id32
  admitPropose : Id32
deriving DecidableEq, Repr

structure Definition where
  id : Id32
  arguments : List KSort
  result : KSort
  body : KExpr

structure CompilerSubject where
  lineage : CompilerLineage
  nominalDeclarations : List NominalDeclaration
  interface : CompilerInterface
  program : List Definition
  buildRequest : Term

inductive KValue where
  | bytes (value : Bytes)
  | term (value : Term)
deriving DecidableEq, Repr

def KValue.sort : KValue → KSort
  | .bytes _ => .bytes
  | .term _ => .term

/- `EvalRequest` is checker-constructed replay context, never Frame03 data.  The
predecessor package is supplied separately with its canonical decode binding
and acceptance premise; this digest binds that exact input without recursively
embedding package bytes in successor evidence. -/
structure EvalRequest where
  acceptedPredecessorPackageHash : Hash32
  coreContractId : Hash32
  physicalProfileId : Hash32
  entrypoint : Id32
  arguments : List KValue
  fuelLimit : Fuel
deriving DecidableEq, Repr

structure EvalReceipt where
  formatVersion : UInt8
  expectedValueHash : Hash32
  expectedRemainingFuel : Fuel
  expectedObservationsHash : Hash32
deriving DecidableEq, Repr

inductive CompilerEvidence where
  | genesis
  | successor (compileReceipt admissionReceipt : EvalReceipt)
deriving DecidableEq, Repr

structure CompilerPackage where
  manifest : CoreManifest
  subject : CompilerSubject
  evidence : CompilerEvidence

structure DecodedPackage where
  exactInput : Bytes
  exactManifestPayload : Bytes
  exactSubjectPayload : Bytes
  exactEvidencePayload : Bytes
  package : CompilerPackage

inductive DecodeCode where
  | wrongMagic
  | unknownVersion
  | frameTagOrderOrCount
  | truncated
  | lengthOrCountOverflow
  | invalidFixedWidth
  | unknownSumTag
  | boundedValueUnderConsumed
  | boundedValueOverConsumed
  | trailingBytes
deriving DecidableEq, Repr

def DecodeCode.tag : DecodeCode → UInt8
  | .wrongMagic => 0x00
  | .unknownVersion => 0x01
  | .frameTagOrderOrCount => 0x02
  | .truncated => 0x03
  | .lengthOrCountOverflow => 0x04
  | .invalidFixedWidth => 0x05
  | .unknownSumTag => 0x06
  | .boundedValueUnderConsumed => 0x07
  | .boundedValueOverConsumed => 0x08
  | .trailingBytes => 0x09

structure DecodeFailure where
  code : DecodeCode
  offset : Nat
deriving DecidableEq, Repr

inductive DecodeVerdict where
  | decoded (value : DecodedPackage)
  | rejected (failure : DecodeFailure)

inductive AuthorizationStage where
  | coreManifest
  | coreWellFormedness
  | genesisAnchor
  | exactPredecessor
  | buildRequest
  | compileEvaluation
  | admissionEvaluation
  | evidenceAttachment
  | finalAuthorization
deriving DecidableEq, Repr

def AuthorizationStage.tag : AuthorizationStage → UInt8
  | .coreManifest => 0x40
  | .coreWellFormedness => 0x41
  | .genesisAnchor => 0x42
  | .exactPredecessor => 0x43
  | .buildRequest => 0x44
  | .compileEvaluation => 0x45
  | .admissionEvaluation => 0x46
  | .evidenceAttachment => 0x47
  | .finalAuthorization => 0x48

inductive AuthorizationCode where
  | manifestMismatch | subjectStructure | nominalTable
  | definitionOrderOrDuplicate | entrypointResolution | entrypointAliased
  | entrypointSignature | staticRule | physicalRequestSignature
  | genesisWrongLineage | genesisEvidenceNotEmpty | missingAnchor
  | anchorBytesMismatch | successorWrongLineage | predecessorNotAccepted
  | candidateOrSelfPredecessor | locatorMismatch | predecessorBytesMismatch
  | buildRequestShape | detachedBuildRequest | baseMismatch
  | coreContractMismatch | physicalProfileMismatch | sourceOrderOrDuplicate
  | sourceArtifactMismatch | identityPlanMismatch | changeOccurrenceMismatch
  | physicalInputsNonempty | fuelInvalid | evidenceShapeMismatch
  | receiptValueMismatch | receiptFuelMismatch | evaluationFault
  | unexpectedResult | subjectMismatch | observationMismatch
  | evidenceDetached | subjectChangedAfterCompile | packageChangedAfterEvidence
  | finalIdentityMismatch
deriving DecidableEq, Repr

def AuthorizationCode.tag : AuthorizationCode → UInt8
  | .manifestMismatch => 0x60 | .subjectStructure => 0x61
  | .nominalTable => 0x62 | .definitionOrderOrDuplicate => 0x63
  | .entrypointResolution => 0x64 | .entrypointAliased => 0x65
  | .entrypointSignature => 0x66 | .staticRule => 0x67
  | .physicalRequestSignature => 0x68 | .genesisWrongLineage => 0x69
  | .genesisEvidenceNotEmpty => 0x6a | .missingAnchor => 0x6b
  | .anchorBytesMismatch => 0x6c | .successorWrongLineage => 0x6d
  | .predecessorNotAccepted => 0x6e | .candidateOrSelfPredecessor => 0x6f
  | .locatorMismatch => 0x70 | .predecessorBytesMismatch => 0x71
  | .buildRequestShape => 0x72 | .detachedBuildRequest => 0x73
  | .baseMismatch => 0x74 | .coreContractMismatch => 0x75
  | .physicalProfileMismatch => 0x76 | .sourceOrderOrDuplicate => 0x77
  | .sourceArtifactMismatch => 0x78 | .identityPlanMismatch => 0x79
  | .changeOccurrenceMismatch => 0x7a | .physicalInputsNonempty => 0x7b
  | .fuelInvalid => 0x7c | .evidenceShapeMismatch => 0x7d
  | .receiptValueMismatch => 0x7e | .receiptFuelMismatch => 0x7f
  | .evaluationFault => 0x80 | .unexpectedResult => 0x81
  | .subjectMismatch => 0x82 | .observationMismatch => 0x83
  | .evidenceDetached => 0x84 | .subjectChangedAfterCompile => 0x85
  | .packageChangedAfterEvidence => 0x86 | .finalIdentityMismatch => 0x87

structure AuthorizationFailure where
  stage : AuthorizationStage
  code : AuthorizationCode
deriving DecidableEq, Repr

inductive AuthorizationVerdict where
  | authorized (exactPackageBytes : Bytes)
  | unauthorized (failure : AuthorizationFailure)
deriving DecidableEq, Repr

structure FinalPackageIdentityInput where
  packageHash : Hash32
  exactPackageBytes : Bytes
deriving DecidableEq, Repr

structure OwnerAnchorObservation where
  exactSelectedBytes : Bytes
  selectedByteLength : Nat
  selectedPackageHash : Hash32
deriving DecidableEq, Repr

/- The private constructor makes this a fixed, non-wire capability rather than
a caller-selected witness type.  Only the irreducible owner boundary that owns
this module can issue one; authorization can inspect but cannot mint it. -/
structure OwnerAnchorWitness where
  private mk ::
  observation : OwnerAnchorObservation

inductive OwnerAnchorInput where
  | missing
  | supplied (witness : OwnerAnchorWitness)

structure GenesisAuthorizationRequest where
  ownerAnchor : OwnerAnchorInput
  buildRequest : Term
  evidence : CompilerEvidence
  compileFuelLimit : Fuel
  admissionFuelLimit : Fuel
  finalIdentity : FinalPackageIdentityInput

/- Acceptance is a proof premise, not a package field, hash lookup, Boolean
registry callback, or candidate-selected rule. -/
opaque AcceptedExact : Bytes → Prop

inductive PredecessorInput where
  | absent (offeredBytes : Bytes)
  | accepted (exactBytes : Bytes) (proof : AcceptedExact exactBytes)
      (offeredBytes : Bytes)

structure SuccessorAuthorizationRequest where
  predecessor : PredecessorInput
  buildRequest : Term
  evidence : CompilerEvidence
  finalIdentity : FinalPackageIdentityInput

end ClauseCompiler
