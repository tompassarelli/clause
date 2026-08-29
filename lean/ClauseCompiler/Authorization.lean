import ClauseCompiler.Checker

/-! Exact external-genesis and exact-predecessor successor authorization. -/

namespace ClauseCompiler.Authorization

open ClauseCompiler

def deny (stage : AuthorizationStage) (code : AuthorizationCode) : AuthorizationVerdict :=
  .unauthorized { stage := stage, code := code }

def evidenceEqual (left right : CompilerEvidence) : Bool :=
  Encoding.evidence left = Encoding.evidence right

def nominalKey (declaration : NominalDeclaration) : Bytes :=
  declaration.domain ++ declaration.id

def refKey (reference : ABI.NominalRef) : Bytes := reference.domain ++ reference.id

def findNominal (domain id : Id32) : List NominalDeclaration → Option NominalDeclaration
  | [] => none
  | declaration :: tail =>
      if declaration.domain = domain && declaration.id = id then some declaration
      else findNominal domain id tail

def nominalExists (declarations : List NominalDeclaration) (domain id : Id32) : Bool :=
  (findNominal domain id declarations).isSome

def nominalDomain (component : String) : Id32 :=
  domainHash "clause/nominal-domain/v1" [ascii component]

def definitionDomain : Id32 := nominalDomain "definition"
def sourceUnitDomain : Id32 := nominalDomain "source-unit"
def changeOccurrenceDomain : Id32 := nominalDomain "change-occurrence"

mutual
  /- A fixed-ABI tag is authoritative wherever it occurs in opaque Term data.
  A 04 record must therefore be well shaped and resolve, even inside a
  TermLiteral or an otherwise Clause-owned request field. -/
  def termNominalReferencesValid (declarations : List NominalDeclaration) : Term → Bool
    | .atom _ _ _ => true
    | value@(.triple first second third) =>
        let childrenValid := termNominalReferencesValid declarations first &&
          termNominalReferencesValid declarations second &&
          termNominalReferencesValid declarations third
        if ABI.asTag first = some 0x04 then
          match ABI.decodeNominalRef value with
          | some reference => childrenValid &&
              nominalExists declarations reference.domain reference.id
          | none => false
        else childrenValid

  def expressionNominalReferencesValid (declarations : List NominalDeclaration) :
      KExpr → Bool
    | .bytesLiteral _ | .var _ => true
    | .termLiteral value => termNominalReferencesValid declarations value
    | .makeAtom a b c | .makeTriple a b c =>
        expressionNominalReferencesValid declarations a &&
        expressionNominalReferencesValid declarations b &&
        expressionNominalReferencesValid declarations c
    | .letValue a b => expressionNominalReferencesValid declarations a &&
        expressionNominalReferencesValid declarations b
    | .caseTerm a b c | .caseBytes a b c =>
        expressionNominalReferencesValid declarations a &&
        expressionNominalReferencesValid declarations b &&
        expressionNominalReferencesValid declarations c
    | .concatBytes parts => expressionSeqNominalReferencesValid declarations parts
    | .caseBytesEqual a b c d =>
        expressionNominalReferencesValid declarations a &&
        expressionNominalReferencesValid declarations b &&
        expressionNominalReferencesValid declarations c &&
        expressionNominalReferencesValid declarations d
    | .call _ arguments | .request _ arguments =>
        expressionSeqNominalReferencesValid declarations arguments

  def expressionSeqNominalReferencesValid (declarations : List NominalDeclaration) :
      KExprSeq → Bool
    | .nil => true
    | .cons head tail => expressionNominalReferencesValid declarations head &&
        expressionSeqNominalReferencesValid declarations tail
end

def allocationShapeValid (declarations : List NominalDeclaration) :
    NominalDeclaration → Bool
  | .seed domain id | .retainedSeed domain id _ =>
      domain.length = 32 && id.length = 32
  | .allocated domain id changeDomain changeId producerDomain producerId slot =>
      domain.length = 32 && id.length = 32 &&
      nominalExists declarations changeDomain changeId &&
      nominalExists declarations producerDomain producerId &&
      id = newNominalId domain changeDomain changeId producerDomain producerId slot

def allocationDependencies (declaration : NominalDeclaration) : List Bytes :=
  match declaration with
  | .allocated _ _ changeDomain changeId producerDomain producerId _ =>
      [changeDomain ++ changeId, producerDomain ++ producerId]
  | _ => []

def allocationAcyclic (declarations : List NominalDeclaration) : Bool :=
  let rec visit (budget : Nat) (path : List Bytes) (key : Bytes) : Bool :=
    match budget with
    | 0 => false
    | budget + 1 =>
        if contains key path then false else
        match declarations.find? (fun declaration => nominalKey declaration = key) with
        | none => false
        | some declaration =>
            all (visit budget (key :: path)) (allocationDependencies declaration)
  all (fun declaration => visit (declarations.length + 1) [] (nominalKey declaration))
    declarations

def nominalTableValid (subject : CompilerSubject) : Bool :=
  sortedUniqueBy nominalKey subject.nominalDeclarations &&
  all (allocationShapeValid subject.nominalDeclarations) subject.nominalDeclarations &&
  allocationAcyclic subject.nominalDeclarations &&
  all (fun definition =>
    nominalExists subject.nominalDeclarations definitionDomain definition.id) subject.program &&
  nominalExists subject.nominalDeclarations definitionDomain subject.interface.compile &&
  nominalExists subject.nominalDeclarations definitionDomain subject.interface.admitPropose &&
  all (fun definition => expressionNominalReferencesValid subject.nominalDeclarations
    definition.body) subject.program &&
  termNominalReferencesValid subject.nominalDeclarations subject.buildRequest

def requestSignatureConforms (program : List Definition) (environment : List KSort)
    (operation : Id32) (arguments : KExprSeq) : Bool :=
  operation = Fixed.sha256OperationId &&
  match arguments with
  | .cons argument .nil =>
      Static.infer program Fixed.coreManifest.physicalProfile environment argument = some .bytes
  | _ => false

mutual
  def requestsConform (program : List Definition) (environment : List KSort) : KExpr → Bool
    | .bytesLiteral _ | .termLiteral _ | .var _ => true
    | .makeAtom a b c | .makeTriple a b c =>
        requestsConform program environment a && requestsConform program environment b &&
        requestsConform program environment c
    | .letValue value body =>
        requestsConform program environment value &&
        match Static.infer program Fixed.coreManifest.physicalProfile environment value with
        | none => false
        | some sort => requestsConform program (sort :: environment) body
    | .caseTerm scrutinee atomBody tripleBody =>
        requestsConform program environment scrutinee &&
        requestsConform program ([.bytes, .bytes, .bytes] ++ environment) atomBody &&
        requestsConform program ([.term, .term, .term] ++ environment) tripleBody
    | .caseBytes scrutinee emptyBody consBody =>
        requestsConform program environment scrutinee &&
        requestsConform program environment emptyBody &&
        requestsConform program ([.bytes, .bytes] ++ environment) consBody
    | .concatBytes parts => requestSeqConform program environment parts
    | .caseBytesEqual a b c d =>
        requestsConform program environment a && requestsConform program environment b &&
        requestsConform program environment c && requestsConform program environment d
    | .call _ arguments => requestSeqConform program environment arguments
    | .request operation arguments =>
        requestSignatureConforms program environment operation arguments &&
        requestSeqConform program environment arguments

  def requestSeqConform (program : List Definition) (environment : List KSort) :
      KExprSeq → Bool
    | .nil => true
    | .cons head tail => requestsConform program environment head &&
        requestSeqConform program environment tail
end

def allRequestsConform (subject : CompilerSubject) : Bool :=
  all (fun definition => requestsConform subject.program definition.arguments definition.body)
    subject.program

def coreFailure (candidate : DecodedPackage) : Option AuthorizationFailure :=
  let subject := candidate.package.subject
  if Encoding.subject subject ≠ some candidate.exactSubjectPayload then
    some { stage := .coreWellFormedness, code := .subjectStructure }
  else if !nominalTableValid subject then
    some { stage := .coreWellFormedness, code := .nominalTable }
  else if !Static.definitionsSortedUnique subject.program then
    some { stage := .coreWellFormedness, code := .definitionOrderOrDuplicate }
  else
    match Static.findDefinition subject.interface.compile subject.program,
        Static.findDefinition subject.interface.admitPropose subject.program with
    | none, _ | _, none =>
        some { stage := .coreWellFormedness, code := .entrypointResolution }
    | some compile, some admit =>
        if subject.interface.compile = subject.interface.admitPropose then
          some { stage := .coreWellFormedness, code := .entrypointAliased }
        else if compile.arguments ≠ [.term] || compile.result ≠ .term ||
            admit.arguments ≠ [.term] || admit.result ≠ .term then
          some { stage := .coreWellFormedness, code := .entrypointSignature }
        else if !Static.definitionsWellFormed subject.program
            candidate.package.manifest.physicalProfile then
          some { stage := .coreWellFormedness, code := .staticRule }
        else if !allRequestsConform subject then
          some { stage := .coreWellFormedness, code := .physicalRequestSignature }
        else none

def refsSortedUnique (references : List ABI.NominalRef) : Bool :=
  sortedUniqueBy refKey references

def refListsDisjoint (left right : List ABI.NominalRef) : Bool :=
  all (fun value => !contains value right) left

def declarationReference : NominalDeclaration → ABI.NominalRef
  | .seed domain id | .retainedSeed domain id _ | .allocated domain id _ _ _ _ _ =>
      { domain := domain, id := id }

def retainedProvenanceValid (subject : CompilerSubject)
    (predecessor : Option DecodedPackage) : Bool :=
  match subject.lineage, predecessor with
  | .genesis, none => all (fun declaration => match declaration with
      | .retainedSeed .. => false
      | _ => true) subject.nominalDeclarations
  | .successor _ _, some prior =>
      let priorRevision := compilerRevisionId prior.exactSubjectPayload
      all (fun declaration => match declaration with
        | .retainedSeed domain id revision =>
            revision = priorRevision &&
            match findNominal domain id prior.package.subject.nominalDeclarations with
            | some (.seed ..) | some (.retainedSeed ..) => true
            | _ => false
        | .seed domain id =>
            (findNominal domain id prior.package.subject.nominalDeclarations).isNone
        | allocated@(.allocated domain id ..) =>
            match findNominal domain id prior.package.subject.nominalDeclarations with
            | none => true
            | some priorDeclaration => decide (allocated = priorDeclaration))
        subject.nominalDeclarations
  | _, _ => false

def planValid (subject : CompilerSubject) (plan : ABI.IdentityPlan)
    (predecessor : Option DecodedPackage) : Bool :=
  refsSortedUnique plan.retained && refsSortedUnique plan.seedInputs &&
  refListsDisjoint plan.retained plan.seedInputs &&
  all (fun reference => match findNominal reference.domain reference.id
      subject.nominalDeclarations with
    | some (.retainedSeed ..) => true
    | _ => false) plan.retained &&
  all (fun reference => match findNominal reference.domain reference.id
      subject.nominalDeclarations with
    | some (.seed ..) => true
    | _ => false) plan.seedInputs &&
  retainedProvenanceValid subject predecessor &&
  match subject.lineage with
  | .genesis => plan.retained = [] &&
      all (fun declaration => match declaration with
        | .seed .. => contains (declarationReference declaration) plan.seedInputs
        | .retainedSeed .. => false
        | .allocated .. => true) subject.nominalDeclarations
  | .successor _ _ =>
      all (fun declaration => match declaration with
        | .retainedSeed .. => contains (declarationReference declaration) plan.retained
        | .seed .. => contains (declarationReference declaration) plan.seedInputs
        | .allocated .. => true) subject.nominalDeclarations

def sourcesSortedUnique (sources : List ABI.SourceUnit) : Bool :=
  sortedUniqueBy ABI.SourceUnit.unitId sources

def sourceArtifactsValid (sources : List ABI.SourceUnit) : Bool :=
  all (fun source => source.artifactId = sourceArtifactId source.bytes) sources

def sourceUnitsDeclared (subject : CompilerSubject) (sources : List ABI.SourceUnit) : Bool :=
  all (fun source => nominalExists subject.nominalDeclarations sourceUnitDomain source.unitId)
    sources

def changeOccurrenceDeclared (subject : CompilerSubject) (id : Id32) : Bool :=
  nominalExists subject.nominalDeclarations changeOccurrenceDomain id

def buildFailure (candidate : DecodedPackage) (requestTerm : Term)
    (genesisFuels : Option (Fuel × Fuel)) (predecessor : Option DecodedPackage) :
    Option AuthorizationFailure :=
  let fail code := some { stage := AuthorizationStage.buildRequest, code := code }
  match ABI.decodeBuildRequest requestTerm with
  | none => fail .buildRequestShape
  | some request =>
      if requestTerm ≠ candidate.package.subject.buildRequest then fail .detachedBuildRequest
      else
        let routeMatches := match candidate.package.subject.lineage, request.base, predecessor with
          | .genesis, .genesis, none => true
          | .successor _ _, .accepted hash revision, some prior =>
              hash = compilerPackageHash prior.exactInput &&
                revision = compilerRevisionId prior.exactSubjectPayload
          | _, _, _ => false
        if !routeMatches then fail .baseMismatch
        else if request.coreContractId ≠ Fixed.coreContractId then fail .coreContractMismatch
        else if request.physicalProfileId ≠ Fixed.physicalProfileId then
          fail .physicalProfileMismatch
        else if !sourcesSortedUnique request.sourceUnits ||
            !sourceUnitsDeclared candidate.package.subject request.sourceUnits then
          fail .sourceOrderOrDuplicate
        else if !sourceArtifactsValid request.sourceUnits then fail .sourceArtifactMismatch
        else if !planValid candidate.package.subject request.identityPlan predecessor then
          fail .identityPlanMismatch
        else
          let changeMatches := match candidate.package.subject.lineage with
            | .genesis => changeOccurrenceDeclared candidate.package.subject
                request.changeOccurrenceId
            | .successor _ change => request.changeOccurrenceId = change &&
                changeOccurrenceDeclared candidate.package.subject change
          if !changeMatches then fail .changeOccurrenceMismatch
          else if request.declaredPhysicalInputs ≠ [] then fail .physicalInputsNonempty
          else
            match genesisFuels with
            | some (compileFuel, admissionFuel) =>
                if compileFuel = 0 || admissionFuel = 0 ||
                    request.compileFuel ≠ compileFuel ||
                    request.admissionFuel ≠ admissionFuel then fail .fuelInvalid
                else none
            | none =>
                if request.compileFuel = 0 || request.admissionFuel = 0 then
                  fail .fuelInvalid
                else none

def finalFailure (candidate : DecodedPackage) (identity : FinalPackageIdentityInput) :
    Option AuthorizationFailure :=
  if identity.exactPackageBytes ≠ candidate.exactInput ||
      identity.packageHash ≠ compilerPackageHash identity.exactPackageBytes then
    some { stage := .finalAuthorization, code := .finalIdentityMismatch }
  else none

def commonPrefix (candidate : DecodedPackage) : Option AuthorizationFailure :=
  if candidate.exactManifestPayload ≠ Fixed.exactCoreManifestBytes ||
      candidate.package.manifest ≠ Fixed.coreManifest then
    some { stage := .coreManifest, code := .manifestMismatch }
  else coreFailure candidate

private def authorizeDecodedGenesis (request : GenesisAuthorizationRequest)
    (exactInput : Bytes) (candidate : DecodedPackage)
    (_binding : Codec.strictDecode exactInput = .decoded candidate) : AuthorizationVerdict :=
  if candidate.exactInput ≠ exactInput ||
      Encoding.package candidate.package ≠ some exactInput then
    deny .coreWellFormedness .subjectStructure
  else
    match commonPrefix candidate with
    | some failure => .unauthorized failure
    | none =>
        match candidate.package.subject.lineage with
        | .successor _ _ => deny .genesisAnchor .genesisWrongLineage
        | .genesis =>
          if !evidenceEqual request.evidence candidate.package.evidence ||
              !evidenceEqual request.evidence .genesis then
            deny .genesisAnchor .genesisEvidenceNotEmpty
          else
            match request.ownerAnchor with
            | .missing => deny .genesisAnchor .missingAnchor
            | .supplied witness =>
                let observation := witness.observation
                if observation.selectedByteLength ≠ observation.exactSelectedBytes.length ||
                    observation.selectedPackageHash ≠
                      compilerPackageHash observation.exactSelectedBytes ||
                    observation.exactSelectedBytes ≠ exactInput then
                  deny .genesisAnchor .anchorBytesMismatch
                else
                  match buildFailure candidate request.buildRequest
                      (some (request.compileFuelLimit, request.admissionFuelLimit)) none with
                  | some failure => .unauthorized failure
                  | none =>
                      match finalFailure candidate request.finalIdentity with
                      | some failure => .unauthorized failure
                      | none => .authorized exactInput

structure AcceptedPredecessor where
  exactBytes : Bytes
  decoded : DecodedPackage
  binding : Codec.strictDecode exactBytes = .decoded decoded
  acceptance : AcceptedExact exactBytes

def resolvePredecessor : PredecessorInput → Option AcceptedPredecessor
  | .absent _ => none
  | .accepted exactBytes acceptance _ =>
      match h : Codec.strictDecode exactBytes with
      | .rejected _ => none
      | .decoded decoded => some {
          exactBytes := exactBytes
          decoded := decoded
          binding := h
          acceptance := acceptance
        }

def statementShapeMatches (statement : EvalStatement) (predecessor : AcceptedPredecessor)
    (entrypoint : Id32) (argument : Term) (fuel : Fuel) : Bool :=
  statement.exactAcceptedPredecessor = predecessor.exactBytes &&
  statement.coreContractId = Fixed.coreContractId &&
  statement.physicalProfileId = Fixed.physicalProfileId &&
  statement.entrypoint = entrypoint && statement.arguments = [.term argument] &&
  statement.fuelLimit = fuel

def certificateShapeValid (certificate : EvalCertificate) : Bool :=
  certificate.formatVersion = Fixed.coreManifest.certificateFormatVersion

/- This is intentionally separate from evaluator preflight.  The authorization
dispatcher must classify malformed format, statement, and graph rows before a
later evaluation fault, result mismatch, or observation mismatch. -/
def certificateGraphValid (predecessor : AcceptedPredecessor)
    (certificate : EvalCertificate) : Bool :=
  let statement := certificate.statement
  let subject := predecessor.decoded.package.subject
  let profile := predecessor.decoded.package.manifest.physicalProfile
  predecessor.decoded.exactManifestPayload = Fixed.exactCoreManifestBytes &&
  predecessor.decoded.package.manifest = Fixed.coreManifest &&
  statement.coreContractId = Fixed.coreContractId &&
  statement.physicalProfileId = Fixed.physicalProfileId &&
  Static.definitionsWellFormed subject.program profile &&
  allRequestsConform subject && Static.entrypointsWellFormed subject &&
  match Static.findDefinition statement.entrypoint subject.program with
  | none => false
  | some entrypoint =>
      all₂ (fun value expected => value.sort = expected)
        statement.arguments entrypoint.arguments &&
      Certificate.checkGraph subject.program profile statement certificate.nodes

def evaluateStatement (predecessor : AcceptedPredecessor)
    (statement : EvalStatement) : Option Evaluator.Result :=
  Evaluator.run predecessor.decoded.package.subject.program
    predecessor.decoded.package.manifest.physicalProfile
    (Certificate.requiredRoot statement).expression [] statement.fuelLimit []

def compileFailure (candidate : DecodedPackage) (request : ABI.BuildRequest)
    (predecessor : AcceptedPredecessor) (certificate : EvalCertificate) :
    Option AuthorizationFailure :=
  let fail code := some { stage := AuthorizationStage.compileEvaluation, code := code }
  if !certificateShapeValid certificate then fail .evidenceShapeMismatch
  else if !statementShapeMatches certificate.statement predecessor
      predecessor.decoded.package.subject.interface.compile candidate.package.subject.buildRequest
      request.compileFuel then fail .certificateStatementMismatch
  else if !certificateGraphValid predecessor certificate then fail .certificateRuleInvalid
  else
    match evaluateStatement predecessor certificate.statement with
    | none => fail .evaluationFault
    | some result =>
      if result.value ≠ certificate.statement.expected.value ||
          result.fuel ≠ certificate.statement.expected.remainingFuel then
        fail .certificateRuleInvalid
      else
        match result.value with
        | .term value =>
            match ABI.builtBytes value with
            | none => fail .unexpectedResult
            | some bytes =>
                if bytes ≠ candidate.exactSubjectPayload then fail .subjectMismatch
                else if ABI.observations result.observations ≠
                    certificate.statement.expected.observations then
                  fail .observationMismatch
                else none
        | _ => fail .unexpectedResult

def admissionRequestTerm (request : Term) (subjectBytes : Bytes)
    (observations : Term) : Term :=
  ABI.record 0x16 [request, ABI.bytes subjectBytes, observations]

def admissionFailure (candidate : DecodedPackage) (request : ABI.BuildRequest)
    (predecessor : AcceptedPredecessor) (compileCertificate admissionCertificate : EvalCertificate) :
    Option AuthorizationFailure :=
  let fail code := some { stage := AuthorizationStage.admissionEvaluation, code := code }
  let argument := admissionRequestTerm candidate.package.subject.buildRequest
    candidate.exactSubjectPayload compileCertificate.statement.expected.observations
  if !certificateShapeValid admissionCertificate then fail .evidenceShapeMismatch
  else if !statementShapeMatches admissionCertificate.statement predecessor
      predecessor.decoded.package.subject.interface.admitPropose argument
      request.admissionFuel then fail .certificateStatementMismatch
  else if !certificateGraphValid predecessor admissionCertificate then
    fail .certificateRuleInvalid
  else
    match evaluateStatement predecessor admissionCertificate.statement with
    | none => fail .evaluationFault
    | some result =>
      if result.value ≠ admissionCertificate.statement.expected.value ||
          result.fuel ≠ admissionCertificate.statement.expected.remainingFuel then
        fail .certificateRuleInvalid
      else
        match result.value with
        | .term value =>
            match ABI.proposedBytes value with
            | none => fail .unexpectedResult
            | some bytes =>
                if bytes ≠ candidate.exactSubjectPayload then fail .subjectMismatch
                else if ABI.observations result.observations ≠
                    admissionCertificate.statement.expected.observations then
                  fail .observationMismatch
                else none
        | _ => fail .unexpectedResult

private def authorizeDecodedSuccessor (request : SuccessorAuthorizationRequest)
    (exactInput : Bytes) (candidate : DecodedPackage)
    (_binding : Codec.strictDecode exactInput = .decoded candidate) : AuthorizationVerdict :=
  if candidate.exactInput ≠ exactInput ||
      Encoding.package candidate.package ≠ some exactInput then
    deny .coreWellFormedness .subjectStructure
  else
    match commonPrefix candidate with
    | some failure => .unauthorized failure
    | none =>
    match candidate.package.subject.lineage with
    | .genesis => deny .exactPredecessor .successorWrongLineage
    | .successor locator _ =>
      let offered := match request.predecessor with
        | .absent bytes | .accepted _ _ bytes => bytes
      let acceptedCandidate := match request.predecessor with
        | .accepted bytes _ _ => bytes = candidate.exactInput
        | .absent _ => false
      if offered = candidate.exactInput || acceptedCandidate then
        deny .exactPredecessor .candidateOrSelfPredecessor
      else
        match resolvePredecessor request.predecessor with
        | none => deny .exactPredecessor .predecessorNotAccepted
        | some predecessor =>
          if locator ≠ compilerPackageHash predecessor.exactBytes then
            deny .exactPredecessor .locatorMismatch
          else if offered ≠ predecessor.exactBytes ||
              predecessor.exactBytes ≠ predecessor.decoded.exactInput then
            deny .exactPredecessor .predecessorBytesMismatch
          else
            match buildFailure candidate request.buildRequest none (some predecessor.decoded) with
            | some failure => .unauthorized failure
            | none =>
              match ABI.decodeBuildRequest request.buildRequest with
              | none => deny .buildRequest .buildRequestShape
              | some buildRequest =>
                match request.evidence with
                | .genesis => deny .compileEvaluation .evidenceShapeMismatch
                | .successor compile admission =>
                  match compileFailure candidate buildRequest predecessor compile with
                  | some failure => .unauthorized failure
                  | none =>
                    match admissionFailure candidate buildRequest predecessor compile admission with
                    | some failure => .unauthorized failure
                    | none =>
                      if !evidenceEqual request.evidence candidate.package.evidence then
                        deny .evidenceAttachment .evidenceDetached
                      else if Encoding.subject candidate.package.subject ≠
                          some candidate.exactSubjectPayload then
                        deny .evidenceAttachment .subjectChangedAfterCompile
                      else if Encoding.package candidate.package ≠ some candidate.exactInput then
                        deny .evidenceAttachment .packageChangedAfterEvidence
                      else
                        match finalFailure candidate request.finalIdentity with
                        | some failure => .unauthorized failure
                        | none => .authorized candidate.exactInput

def authorizeBytesGenesis (request : GenesisAuthorizationRequest) (input : Bytes) :
    DecodeVerdict × Option AuthorizationVerdict :=
  match h : Codec.strictDecode input with
  | verdict@(.rejected _) => (verdict, none)
  | verdict@(.decoded candidate) =>
      (verdict, some (authorizeDecodedGenesis request input candidate h))

def authorizeBytesSuccessor (request : SuccessorAuthorizationRequest) (input : Bytes) :
    DecodeVerdict × Option AuthorizationVerdict :=
  match h : Codec.strictDecode input with
  | verdict@(.rejected _) => (verdict, none)
  | verdict@(.decoded candidate) =>
      (verdict, some (authorizeDecodedSuccessor request input candidate h))

/- Kernel-checked adversarial precedence vectors.  Each theorem leaves every
later input unconstrained, so the earlier false row must dominate any paired
later defect. -/
theorem compileFormatPrecedesLaterFailures (candidate : DecodedPackage)
    (request : ABI.BuildRequest) (predecessor : AcceptedPredecessor)
    (certificate : EvalCertificate) (shapeInvalid : certificateShapeValid certificate = false) :
    compileFailure candidate request predecessor certificate = some {
      stage := .compileEvaluation
      code := .evidenceShapeMismatch
    } := by
  simp [compileFailure, shapeInvalid]

theorem compileStatementPrecedesGraphAndEvaluation (candidate : DecodedPackage)
    (request : ABI.BuildRequest) (predecessor : AcceptedPredecessor)
    (certificate : EvalCertificate) (shapeValid : certificateShapeValid certificate = true)
    (statementInvalid : !statementShapeMatches certificate.statement predecessor
      predecessor.decoded.package.subject.interface.compile
      candidate.package.subject.buildRequest request.compileFuel) :
    compileFailure candidate request predecessor certificate = some {
      stage := .compileEvaluation
      code := .certificateStatementMismatch
    } := by
  simp [compileFailure, shapeValid, statementInvalid]

theorem compileGraphPrecedesEvaluation (candidate : DecodedPackage)
    (request : ABI.BuildRequest) (predecessor : AcceptedPredecessor)
    (certificate : EvalCertificate) (shapeValid : certificateShapeValid certificate = true)
    (statementValid : statementShapeMatches certificate.statement predecessor
      predecessor.decoded.package.subject.interface.compile
      candidate.package.subject.buildRequest request.compileFuel)
    (graphInvalid : certificateGraphValid predecessor certificate = false) :
    compileFailure candidate request predecessor certificate = some {
      stage := .compileEvaluation
      code := .certificateRuleInvalid
    } := by
  simp [compileFailure, shapeValid, statementValid, graphInvalid]

theorem unknownRequestSignatureFailsProfile (program : List Definition)
    (environment : List KSort)
    (operation : Id32) (arguments : KExprSeq)
    (unknown : operation ≠ Fixed.sha256OperationId) :
    requestSignatureConforms program environment operation arguments = false := by
  simp [requestSignatureConforms, unknown]

theorem zeroArityRequestSignatureFailsProfile (program : List Definition)
    (environment : List KSort) :
    requestSignatureConforms program environment Fixed.sha256OperationId .nil = false := by
  simp [requestSignatureConforms]

theorem wrongSortRequestSignatureFailsProfile (program : List Definition)
    (environment : List KSort) (value : Term) :
    requestSignatureConforms program environment Fixed.sha256OperationId
      (.cons (.termLiteral value) .nil) = false := by
  simp [requestSignatureConforms, Static.infer]

end ClauseCompiler.Authorization
