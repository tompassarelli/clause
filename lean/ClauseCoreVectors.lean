import ClauseCore

/-!
# Clause Core v0 executable vectors

These kernel-reduced cases consume the exact positive byte corpus frozen by
`clause:test-vectors/canonical-package/` and reproduce its named mutations.
They keep physical decoding, exact binding, relative certificate checking, and
constitutional authority as separate verdicts.
-/

namespace ClauseCore.Vectors

set_option maxRecDepth 200000

private def bytes (data : List UInt8) : CanonicalBytes := ⟨data⟩

private def replaceByte (position : Nat) (replacement : UInt8)
    (data : List UInt8) : List UInt8 :=
  data.set position replacement

private def slice (start count : Nat) (data : List UInt8) : List UInt8 :=
  (data.drop start).take count

private def decodeAccepts (candidate : CanonicalBytes) : Bool :=
  match Codec.decodePackage candidate with
  | some _ => true
  | none => false

private def lineageIsRoot (package : CanonicalPackageCandidate) : Bool :=
  match package.decoded.lineage with
  | .root => true
  | .successor _ _ => false

private def indexAt (universeByte semanticsByte : UInt8) : StructuralIndex := {
  universeId := ⟨bytes [universeByte]⟩
  semanticsId := ⟨bytes [semanticsByte]⟩
}

private def atomAt (index : StructuralIndex)
    (kind payload equalityContract : UInt8) : Term index :=
  .atom {
    kind := ⟨bytes [kind]⟩
    canonicalPayload := bytes [payload]
    equalityContract := ⟨bytes [equalityContract]⟩
  }

private def targetAt (index : StructuralIndex) (payload : UInt8) :
    JudgmentClaim index := {
  term := atomAt index 0x20 payload 0x30
  typeTerm := atomAt index 0x21 0x41 0x31
  mode := atomAt index 0x22 0x42 0x32
}

private def admissionAt (index : StructuralIndex) : JudgmentClaim index :=
  basisAdmissionClaimForFrames index Constitution.v0IndexFrame
    Constitution.successorBasisFrame

private def successorBasisAt (index : StructuralIndex) (payload : UInt8) :
    DerivationBasisCandidate index := {
  roots := [targetAt index payload]
  rules := []
}

private def successorCertificateAt (index : StructuralIndex)
    (payload : UInt8) (rootRef : Nat := 0) : DerivationCertificate index :=
  ⟨[⟨targetAt index payload, .root rootRef⟩]⟩

private def authorizationAt (index : StructuralIndex) :
    DerivationCertificate index :=
  ⟨[⟨admissionAt index, .root 1⟩]⟩

/-! ## Exact positive corpus -/

theorem bootstrap_byte_count : Constitution.bootstrapBytes.data.length = 334 := by
  decide

theorem successor_byte_count : Constitution.successorBytes.data.length = 681 := by
  decide

theorem bootstrap_positive_decode :
    Codec.decodePackage Constitution.bootstrapBytes =
      some Constitution.bootstrapPackage :=
  Constitution.bootstrap_decodes_exactly

theorem successor_positive_decode :
    Codec.decodePackage Constitution.successorBytes =
      some Constitution.successorPackage :=
  Constitution.successor_decodes_exactly

theorem bootstrap_positive_authority :
    AuthoritativePackage Constitution.bootstrapPackage :=
  AuthoritativePackage.literalBootstrap

theorem successor_positive_authority :
    AuthoritativePackage Constitution.successorPackage :=
  literal_successor_is_authoritative

theorem successor_authorization_uses_predecessor_only :
    DerivationCertificate.checkRelative Constitution.bootstrapBasis
        Constitution.successorAuthorization
        Constitution.successorAdmissionClaim = true ∧
      DerivationCertificate.checkRelative Constitution.successorBasis
        Constitution.successorAuthorization
        Constitution.successorAdmissionClaim = false := by
  decide

/-! ## Strict decoder mutations from the normative corpus -/

def badMagic : CanonicalBytes :=
  bytes (replaceByte 0 0x44 Constitution.successorBytes.data)

def badVersion : CanonicalBytes :=
  bytes (replaceByte 4 0x02 Constitution.successorBytes.data)

def badFrameOrder : CanonicalBytes := bytes (
  Constitution.successorBytes.data.take 5 ++
    slice 20 476 Constitution.successorBytes.data ++
    slice 5 15 Constitution.successorBytes.data ++
    Constitution.successorBytes.data.drop 496)

def unknownTermTag : CanonicalBytes :=
  bytes (replaceByte 624 0x02 Constitution.successorBytes.data)

def badLength : CanonicalBytes := bytes (
  replaceByte 9 0xff
    (replaceByte 8 0xff
      (replaceByte 7 0xff
        (replaceByte 6 0xff Constitution.successorBytes.data))))

def truncated : CanonicalBytes :=
  bytes (Constitution.successorBytes.data.take 680)

def trailingBytes : CanonicalBytes :=
  bytes (Constitution.successorBytes.data ++ [0])

theorem malformed_vectors_reject :
    decodeAccepts badMagic = false ∧
      decodeAccepts badVersion = false ∧
      decodeAccepts badFrameOrder = false ∧
      decodeAccepts unknownTermTag = false ∧
      decodeAccepts badLength = false ∧
      decodeAccepts truncated = false ∧
      decodeAccepts trailingBytes = false := by
  decide

theorem u32_encoder_rejects_overflow :
    Codec.encodeU32 4294967296 = none := by
  decide

/-! ## Canonically decodable bound-field mutations -/

def universeTamperBytes : CanonicalBytes :=
  bytes (replaceByte 14 0x12 Constitution.successorBytes.data)

private def universeTamperIndex : StructuralIndex := indexAt 0x12 0x11

def universeTamperPackage : CanonicalPackageCandidate := {
  canonicalBytes := universeTamperBytes
  index := universeTamperIndex
  decoded := {
    lineage := .successor Constitution.bootstrapBytes
      (authorizationAt universeTamperIndex)
    basis := successorBasisAt universeTamperIndex 0x50
    certificate := successorCertificateAt universeTamperIndex 0x50
    target := targetAt universeTamperIndex 0x50
    auxiliary := []
  }
}

def epochTamperBytes : CanonicalBytes :=
  bytes (replaceByte 19 0x12 Constitution.successorBytes.data)

private def epochTamperIndex : StructuralIndex := indexAt 0x10 0x12

def epochTamperPackage : CanonicalPackageCandidate := {
  canonicalBytes := epochTamperBytes
  index := epochTamperIndex
  decoded := {
    lineage := .successor Constitution.bootstrapBytes
      (authorizationAt epochTamperIndex)
    basis := successorBasisAt epochTamperIndex 0x50
    certificate := successorCertificateAt epochTamperIndex 0x50
    target := targetAt epochTamperIndex 0x50
    auxiliary := []
  }
}

def basisTamperBytes : CanonicalBytes :=
  bytes (replaceByte 515 0x53 Constitution.successorBytes.data)

def basisTamperPackage : CanonicalPackageCandidate := {
  canonicalBytes := basisTamperBytes
  index := Constitution.v0Index
  decoded := {
    lineage := .successor Constitution.bootstrapBytes
      Constitution.successorAuthorization
    basis := successorBasisAt Constitution.v0Index 0x53
    certificate := Constitution.successorCertificate
    target := Constitution.successorTarget
    auxiliary := []
  }
}

def certificateTamperBytes : CanonicalBytes :=
  bytes (replaceByte 618 1 Constitution.successorBytes.data)

def certificateTamperPackage : CanonicalPackageCandidate := {
  canonicalBytes := certificateTamperBytes
  index := Constitution.v0Index
  decoded := {
    lineage := .successor Constitution.bootstrapBytes
      Constitution.successorAuthorization
    basis := Constitution.successorBasis
    certificate := successorCertificateAt Constitution.v0Index 0x50 1
    target := Constitution.successorTarget
    auxiliary := []
  }
}

def targetTamperBytes : CanonicalBytes :=
  bytes (replaceByte 634 0x53 Constitution.successorBytes.data)

def targetTamperPackage : CanonicalPackageCandidate := {
  canonicalBytes := targetTamperBytes
  index := Constitution.v0Index
  decoded := {
    lineage := .successor Constitution.bootstrapBytes
      Constitution.successorAuthorization
    basis := Constitution.successorBasis
    certificate := Constitution.successorCertificate
    target := targetAt Constitution.v0Index 0x53
    auxiliary := []
  }
}

def auxiliaryTamperBytes : CanonicalBytes := bytes (
  Constitution.successorBytes.data.take 672 ++
    [6, 0, 0, 0, 9, 0, 0, 0, 1, 0, 0, 0, 1, 0x60])

def auxiliaryTamperSections : DecodedPackageSections Constitution.v0Index := {
  lineage := .successor Constitution.bootstrapBytes
    Constitution.successorAuthorization
  basis := Constitution.successorBasis
  certificate := Constitution.successorCertificate
  target := Constitution.successorTarget
  auxiliary := [bytes [0x60]]
}

def auxiliaryTamperPackage : CanonicalPackageCandidate := {
  canonicalBytes := auxiliaryTamperBytes
  index := Constitution.v0Index
  decoded := auxiliaryTamperSections
}

def nonliteralRootSections : DecodedPackageSections Constitution.v0Index := {
  lineage := .root
  basis := Constitution.successorBasis
  certificate := Constitution.successorCertificate
  target := Constitution.successorTarget
  auxiliary := []
}

def nonliteralRootBytes : CanonicalBytes :=
  (Codec.encodePackageValue Constitution.v0Index nonliteralRootSections).getD
    (bytes [])

def nonliteralRootPackage : CanonicalPackageCandidate := {
  canonicalBytes := nonliteralRootBytes
  index := Constitution.v0Index
  decoded := nonliteralRootSections
}

private def admissionShapedClaim (index : StructuralIndex)
    (payload : UInt8) : JudgmentClaim index := {
  term := atomAt index 0xf0 payload 0xf1
  typeTerm := atomAt index 0xf0 0xf2 0xf1
  mode := atomAt index 0xf0 0xf3 0xf1
}

def selfDeclaredClaim : JudgmentClaim Constitution.v0Index :=
  admissionShapedClaim Constitution.v0Index 0xee

def selfAuthorizationBasis : DerivationBasisCandidate Constitution.v0Index := {
  roots := [Constitution.successorTarget, selfDeclaredClaim]
  rules := []
}

def selfAuthorizationCertificate : DerivationCertificate Constitution.v0Index :=
  ⟨[⟨selfDeclaredClaim, .root 1⟩]⟩

def selfAuthorizationSections : DecodedPackageSections Constitution.v0Index := {
  lineage := .successor Constitution.bootstrapBytes
    selfAuthorizationCertificate
  basis := selfAuthorizationBasis
  certificate := Constitution.successorCertificate
  target := Constitution.successorTarget
  auxiliary := []
}

def selfAuthorizationBytes : CanonicalBytes :=
  (Codec.encodePackageValue Constitution.v0Index selfAuthorizationSections).getD
    (bytes [])

def selfAuthorizationPackage : CanonicalPackageCandidate := {
  canonicalBytes := selfAuthorizationBytes
  index := Constitution.v0Index
  decoded := selfAuthorizationSections
}

def alteredBootstrapSections : DecodedPackageSections Constitution.v0Index := {
  lineage := .root
  basis := Constitution.bootstrapBasis
  certificate := Constitution.bootstrapCertificate
  target := Constitution.bootstrapTarget
  auxiliary := [bytes [0x60]]
}

def alteredBootstrapBytes : CanonicalBytes :=
  (Codec.encodePackageValue Constitution.v0Index alteredBootstrapSections).getD
    (bytes [])

def alteredBootstrapPackage : CanonicalPackageCandidate := {
  canonicalBytes := alteredBootstrapBytes
  index := Constitution.v0Index
  decoded := alteredBootstrapSections
}

def wrongPredecessorSections : DecodedPackageSections Constitution.v0Index := {
  lineage := .successor alteredBootstrapBytes
    Constitution.successorAuthorization
  basis := Constitution.successorBasis
  certificate := Constitution.successorCertificate
  target := Constitution.successorTarget
  auxiliary := []
}

def wrongPredecessorBytes : CanonicalBytes :=
  (Codec.encodePackageValue Constitution.v0Index wrongPredecessorSections).getD
    (bytes [])

def wrongPredecessorPackage : CanonicalPackageCandidate := {
  canonicalBytes := wrongPredecessorBytes
  index := Constitution.v0Index
  decoded := wrongPredecessorSections
}

def transplantedAuthorizationSections :
    DecodedPackageSections Constitution.v0Index := {
  lineage := .successor Constitution.bootstrapBytes
    Constitution.bootstrapCertificate
  basis := Constitution.successorBasis
  certificate := Constitution.successorCertificate
  target := Constitution.successorTarget
  auxiliary := []
}

def transplantedAuthorizationBytes : CanonicalBytes :=
  (Codec.encodePackageValue Constitution.v0Index
    transplantedAuthorizationSections).getD (bytes [])

def transplantedAuthorizationPackage : CanonicalPackageCandidate := {
  canonicalBytes := transplantedAuthorizationBytes
  index := Constitution.v0Index
  decoded := transplantedAuthorizationSections
}

theorem accepted_mutations_decode :
    decodeAccepts universeTamperBytes = true ∧
      decodeAccepts epochTamperBytes = true ∧
      decodeAccepts basisTamperBytes = true ∧
      decodeAccepts certificateTamperBytes = true ∧
      decodeAccepts targetTamperBytes = true ∧
      decodeAccepts auxiliaryTamperBytes = true ∧
      decodeAccepts nonliteralRootBytes = true ∧
      decodeAccepts selfAuthorizationBytes = true ∧
      decodeAccepts wrongPredecessorBytes = true ∧
      decodeAccepts transplantedAuthorizationBytes = true := by
  decide

theorem generated_corpus_mutation_lengths :
    auxiliaryTamperBytes.data.length = 686 ∧
      nonliteralRootBytes.data.length = 211 ∧
      selfAuthorizationBytes.data.length = 654 ∧
      wrongPredecessorBytes.data.length = 686 := by
  decide

theorem bound_field_mutations_break_exact_positive_bytes :
    universeTamperBytes ≠ Constitution.successorBytes ∧
      epochTamperBytes ≠ Constitution.successorBytes ∧
      basisTamperBytes ≠ Constitution.successorBytes ∧
      certificateTamperBytes ≠ Constitution.successorBytes ∧
      targetTamperBytes ≠ Constitution.successorBytes ∧
      auxiliaryTamperBytes ≠ Constitution.successorBytes ∧
      nonliteralRootBytes ≠ Constitution.successorBytes ∧
      selfAuthorizationBytes ≠ Constitution.successorBytes ∧
      wrongPredecessorBytes ≠ Constitution.successorBytes ∧
      transplantedAuthorizationBytes ≠ Constitution.successorBytes := by
  decide

theorem universe_tamper_decodes_exactly :
    Codec.decodePackage universeTamperBytes = some universeTamperPackage := by
  rfl

theorem epoch_tamper_decodes_exactly :
    Codec.decodePackage epochTamperBytes = some epochTamperPackage := by
  rfl

theorem auxiliary_tamper_decodes_exactly :
    Codec.decodePackage auxiliaryTamperBytes = some auxiliaryTamperPackage := by
  rfl

theorem nonliteral_root_decodes_exactly :
    Codec.decodePackage nonliteralRootBytes = some nonliteralRootPackage := by
  rfl

theorem self_authorization_decodes_exactly :
    Codec.decodePackage selfAuthorizationBytes =
      some selfAuthorizationPackage := by
  rfl

theorem altered_bootstrap_decodes_exactly :
    Codec.decodePackage alteredBootstrapBytes = some alteredBootstrapPackage := by
  rfl

theorem wrong_predecessor_decodes_exactly :
    Codec.decodePackage wrongPredecessorBytes = some wrongPredecessorPackage := by
  rfl

theorem transplanted_authorization_decodes_exactly :
    Codec.decodePackage transplantedAuthorizationBytes =
      some transplantedAuthorizationPackage := by
  rfl

/-! ## Authority adversaries -/

private theorem successorWithLiteralBytesRequiresLineageCheck
    {package : CanonicalPackageCandidate}
    {authorization : DerivationCertificate package.index}
    (lineage : package.decoded.lineage =
      .successor Constitution.bootstrapBytes authorization)
    (authority : AuthoritativePackage package) :
    checkLineageAuthorization Constitution.bootstrapPackage package = true := by
  have notBootstrap : package ≠ Constitution.bootstrapPackage := by
    intro samePackage
    subst package
    change PackageLineage.root = PackageLineage.successor _ _ at lineage
    cases lineage
  obtain ⟨prior, admitted, priorAuthority, priorBound, _, _, _, exactLineage,
      authorizationAccepted, _⟩ :=
    authoritative_nonbootstrap_has_predecessor authority notBootstrap
  have sameLineage :
      PackageLineage.successor Constitution.bootstrapBytes authorization =
        PackageLineage.successor prior.canonicalBytes admitted := by
    exact lineage.symm.trans exactLineage
  have samePredecessorBytes :
      Constitution.bootstrapBytes = prior.canonicalBytes :=
    (PackageLineage.successor.inj sameLineage).1
  have priorDecoded :
      Codec.decodePackage Constitution.bootstrapBytes = some prior := by
    rw [samePredecessorBytes]
    exact priorBound.decoded
  have priorIsBootstrap : prior = Constitution.bootstrapPackage :=
    Codec.decodePackage_unique priorDecoded
      Constitution.bootstrap_decodes_exactly
  subst prior
  exact authorizationAccepted

private theorem checkedSuccessorFailurePreventsAuthority
    {package : CanonicalPackageCandidate}
    (notBootstrap : package ≠ Constitution.bootstrapPackage)
    (checkFails : checkPackage package = false) :
    ¬ AuthoritativePackage package := by
  intro authority
  obtain ⟨_, _, _, _, _, _, _, _, _, checked⟩ :=
    authoritative_nonbootstrap_has_predecessor authority notBootstrap
  rw [checkFails] at checked
  cases checked

private theorem rootPackageDifferentFromLiteralIsNotAuthoritative
    {package : CanonicalPackageCandidate}
    (root : package.decoded.lineage = .root)
    (differentBytes :
      package.canonicalBytes ≠ Constitution.bootstrapBytes) :
    ¬ AuthoritativePackage package := by
  intro authority
  have samePackage := literal_bootstrap_unique authority root
  have sameBytes := congrArg CanonicalPackageCandidate.canonicalBytes samePackage
  exact differentBytes (by
    simpa [Constitution.bootstrapPackage] using sameBytes)

private theorem differentBytesPreventBootstrapEquality
    {package : CanonicalPackageCandidate}
    (differentBytes :
      package.canonicalBytes ≠ Constitution.bootstrapBytes) :
    package ≠ Constitution.bootstrapPackage := by
  intro samePackage
  have sameBytes := congrArg CanonicalPackageCandidate.canonicalBytes samePackage
  exact differentBytes (by
    simpa [Constitution.bootstrapPackage] using sameBytes)

theorem cross_universe_cannot_authorize :
    ¬ AuthoritativePackage universeTamperPackage := by
  intro authority
  have exactIndex := authoritative_has_v0_index authority
  have differentIndex : universeTamperPackage.index ≠ Constitution.v0Index := by
    decide
  exact differentIndex exactIndex

theorem cross_epoch_cannot_authorize :
    ¬ AuthoritativePackage epochTamperPackage := by
  intro authority
  have exactIndex := authoritative_has_v0_index authority
  have differentIndex : epochTamperPackage.index ≠ Constitution.v0Index := by
    decide
  exact differentIndex exactIndex

theorem basis_tamper_rejects_canonical_lineage_target :
    checkLineageAuthorization Constitution.bootstrapPackage basisTamperPackage =
      false := by
  decide

theorem basis_tamper_cannot_authorize :
    ¬ AuthoritativePackage basisTamperPackage := by
  intro authority
  have accepted := successorWithLiteralBytesRequiresLineageCheck rfl authority
  rw [basis_tamper_rejects_canonical_lineage_target] at accepted
  cases accepted

theorem certificate_tamper_cannot_authorize :
    ¬ AuthoritativePackage certificateTamperPackage := by
  apply checkedSuccessorFailurePreventsAuthority
  · apply differentBytesPreventBootstrapEquality
    decide
  · decide

theorem target_tamper_cannot_authorize :
    ¬ AuthoritativePackage targetTamperPackage := by
  apply checkedSuccessorFailurePreventsAuthority
  · apply differentBytesPreventBootstrapEquality
    decide
  · decide

theorem auxiliary_is_bound_but_not_semantic_authority :
    auxiliaryTamperBytes ≠ Constitution.successorBytes ∧
      AuthoritativePackage auxiliaryTamperPackage := by
  constructor
  · decide
  · apply AuthoritativePackage.successor Constitution.successorAuthorization
      AuthoritativePackage.literalBootstrap
      Constitution.bootstrap_is_canonically_bound
      ⟨auxiliary_tamper_decodes_exactly⟩
    · rfl
    · rfl
    · rfl
    · decide
    · decide

theorem nonliteral_root_cannot_authorize :
    ¬ AuthoritativePackage nonliteralRootPackage := by
  apply rootPackageDifferentFromLiteralIsNotAuthoritative rfl
  decide

theorem self_declared_basis_check_is_not_predecessor_authority :
    DerivationCertificate.checkRelative selfAuthorizationBasis
        selfAuthorizationCertificate selfDeclaredClaim = true ∧
      checkLineageAuthorization Constitution.bootstrapPackage
        selfAuthorizationPackage = false ∧
      ¬ AuthoritativePackage selfAuthorizationPackage := by
  constructor
  · decide
  constructor
  · decide
  · intro authority
    have accepted := successorWithLiteralBytesRequiresLineageCheck rfl authority
    have rejected : checkLineageAuthorization Constitution.bootstrapPackage
        selfAuthorizationPackage = false := by decide
    rw [rejected] at accepted
    cases accepted

theorem transplanted_authorization_cannot_authorize :
    ¬ AuthoritativePackage transplantedAuthorizationPackage := by
  intro authority
  have accepted := successorWithLiteralBytesRequiresLineageCheck rfl authority
  have rejected : checkLineageAuthorization Constitution.bootstrapPackage
      transplantedAuthorizationPackage = false := by decide
  rw [rejected] at accepted
  cases accepted

theorem wrong_exact_predecessor_cannot_authorize :
    ¬ AuthoritativePackage wrongPredecessorPackage := by
  intro authority
  have notBootstrap : wrongPredecessorPackage ≠
      Constitution.bootstrapPackage := by
    apply differentBytesPreventBootstrapEquality
    decide
  obtain ⟨prior, admitted, priorAuthority, priorBound, _, _, _, exactLineage,
      _, _⟩ := authoritative_nonbootstrap_has_predecessor authority notBootstrap
  have sameLineage :
      PackageLineage.successor alteredBootstrapBytes
          Constitution.successorAuthorization =
        PackageLineage.successor prior.canonicalBytes admitted := by
    simpa [wrongPredecessorPackage, wrongPredecessorSections] using exactLineage
  have samePredecessorBytes :
      alteredBootstrapBytes = prior.canonicalBytes :=
    (PackageLineage.successor.inj sameLineage).1
  have priorDecoded :
      Codec.decodePackage alteredBootstrapBytes = some prior := by
    rw [samePredecessorBytes]
    exact priorBound.decoded
  have priorIsAltered : prior = alteredBootstrapPackage :=
    Codec.decodePackage_unique priorDecoded altered_bootstrap_decodes_exactly
  subst prior
  have rootAuthorityLiteral :=
    literal_bootstrap_unique priorAuthority rfl
  have sameBytes := congrArg CanonicalPackageCandidate.canonicalBytes
    rootAuthorityLiteral
  have differentBytes : alteredBootstrapBytes ≠ Constitution.bootstrapBytes := by
    decide
  exact differentBytes (by
    simpa [alteredBootstrapPackage, Constitution.bootstrapPackage] using
      sameBytes)

/-! ## Ordinary package and non-authority boundaries -/

private def ordinaryRoot : JudgmentClaim Constitution.v0Index :=
  targetAt Constitution.v0Index 0x60

private def ordinaryDerived : JudgmentClaim Constitution.v0Index := {
  term := .triple
    (atomAt Constitution.v0Index 0x70 0x01 0x30)
    (atomAt Constitution.v0Index 0x71 0x02 0x30)
    (atomAt Constitution.v0Index 0x72 0x03 0x30)
  typeTerm := atomAt Constitution.v0Index 0x21 0x41 0x31
  mode := atomAt Constitution.v0Index 0x22 0x42 0x32
}

private def ordinaryRule : GroundRuleCandidate Constitution.v0Index := {
  premises := [ordinaryRoot]
  conclusion := ordinaryDerived
}

def ordinarySections : DecodedPackageSections Constitution.v0Index := {
  lineage := .root
  basis := { roots := [ordinaryRoot], rules := [ordinaryRule] }
  certificate := ⟨[
    ⟨ordinaryRoot, .root 0⟩,
    ⟨ordinaryDerived, .apply 0 [0]⟩
  ]⟩
  target := ordinaryDerived
  auxiliary := [bytes [], bytes [0xaa, 0xbb]]
}

def ordinaryBytes : CanonicalBytes :=
  (Codec.encodePackageValue Constitution.v0Index ordinarySections).getD (bytes [])

def ordinaryPackage : CanonicalPackageCandidate := {
  canonicalBytes := ordinaryBytes
  index := Constitution.v0Index
  decoded := ordinarySections
}

theorem ordinary_package_roundtrip_and_relative_check :
    Codec.decodePackage ordinaryBytes = some ordinaryPackage ∧
      checkPackage ordinaryPackage = true := by
  constructor
  · rfl
  · decide

theorem ordinary_decode_and_check_do_not_create_authority :
    ¬ AuthoritativePackage ordinaryPackage := by
  apply rootPackageDifferentFromLiteralIsNotAuthoritative rfl
  decide

def nullaryRule : GroundRuleCandidate Constitution.v0Index := {
  premises := []
  conclusion := selfDeclaredClaim
}

def nullarySections : DecodedPackageSections Constitution.v0Index := {
  lineage := .root
  basis := { roots := [], rules := [nullaryRule] }
  certificate := ⟨[⟨selfDeclaredClaim, .apply 0 []⟩]⟩
  target := selfDeclaredClaim
  auxiliary := []
}

def nullaryBytes : CanonicalBytes :=
  (Codec.encodePackageValue Constitution.v0Index nullarySections).getD (bytes [])

def nullaryPackage : CanonicalPackageCandidate := {
  canonicalBytes := nullaryBytes
  index := Constitution.v0Index
  decoded := nullarySections
}

def candidateContext : ContextCandidate Constitution.v0Index :=
  ⟨[selfDeclaredClaim]⟩

theorem context_root_rule_and_bare_derivation_remain_non_authoritative :
    ContextCandidate.containsRepresentation candidateContext selfDeclaredClaim =
        true ∧
      checkPackage nullaryPackage = true ∧
      DerivableFrom nullaryPackage.decoded.basis selfDeclaredClaim ∧
      ¬ AuthoritativePackage nullaryPackage := by
  constructor
  · decide
  constructor
  · decide
  constructor
  · exact DerivationCertificate.checkRelative_sound
      nullaryPackage.decoded.basis nullaryPackage.decoded.certificate
      selfDeclaredClaim (by decide)
  · apply rootPackageDifferentFromLiteralIsNotAuthoritative rfl
    decide

def bytesValueMismatch : CanonicalPackageCandidate := {
  canonicalBytes := Constitution.bootstrapBytes
  index := Constitution.v0Index
  decoded := Constitution.successorSections
}

theorem bytes_value_mismatch_is_not_canonically_bound :
    ¬ Codec.CanonicalBinding bytesValueMismatch := by
  intro bound
  have reencoded := Codec.binding_reencodes bound
  have expected : Codec.encodePackage bytesValueMismatch =
      some Constitution.successorBytes := by decide
  rw [expected] at reencoded
  have different : Constitution.successorBytes ≠ Constitution.bootstrapBytes := by
    decide
  exact different (Option.some.inj reencoded)

def selfCycleBytes : CanonicalBytes := bytes [67, 76, 67, 80, 1]

def selfCyclePackage : CanonicalPackageCandidate := {
  canonicalBytes := selfCycleBytes
  index := Constitution.v0Index
  decoded := {
    lineage := .successor selfCycleBytes Constitution.successorAuthorization
    basis := Constitution.successorBasis
    certificate := Constitution.successorCertificate
    target := Constitution.successorTarget
    auxiliary := []
  }
}

theorem self_or_cycle_attempt_cannot_bind_or_authorize :
    ¬ Codec.CanonicalBinding selfCyclePackage ∧
      ¬ AuthoritativePackage selfCyclePackage := by
  have notBound : ¬ Codec.CanonicalBinding selfCyclePackage := by
    intro bound
    have reencoded := Codec.binding_reencodes bound
    have impossible : Codec.encodePackage selfCyclePackage ≠
        some selfCycleBytes := by decide
    exact impossible reencoded
  exact ⟨notBound, fun authority =>
    notBound (authoritative_is_canonically_bound authority)⟩

end ClauseCore.Vectors
