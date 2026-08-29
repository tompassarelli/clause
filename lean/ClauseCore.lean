/-!
# Clause Core: candidate Term representation

This file implements the first constitutional boundary from
`docs/foundation.md`: a finite recursive Term is either an Atom or exactly
three Terms. It also checks finite ground certificates only relative to an
explicitly supplied candidate basis. It does not define accepted Clause
judgments, Runs, admission, source syntax, persistence, or language-feature
constructors.

Universe and semantics epoch index candidate representation comparison. Atom
comparison never invokes a host callback. Semantic structural equality and
nominal identity remain unavailable until later Clause judgments validate the
Atom contract and grant an ordinary Atom Term an identity role.
-/

namespace ClauseCore

/-- Bytes selected as the canonical representation of one candidate Clause
value. Whether that claim is admissible belongs to a later checker. -/
structure CanonicalBytes where
  data : List UInt8
deriving DecidableEq

/-- The abstraction boundary at which Atom payloads are opaque. -/
structure UniverseId where
  canonical : CanonicalBytes
deriving DecidableEq

/-- The exact Clause equality and interpretation epoch. -/
structure ClauseSemanticsId where
  canonical : CanonicalBytes
deriving DecidableEq

/-- Candidate representation comparison is meaningful only inside one universe
and one Clause semantics epoch. -/
structure StructuralIndex where
  universeId : UniverseId
  semanticsId : ClauseSemanticsId
deriving DecidableEq

/-- A host-neutral reference to an Atom kind declared in Clause data. -/
structure AtomKindId where
  canonical : CanonicalBytes
deriving DecidableEq

/-- A host-neutral reference to a Clause-authored equality contract. It is
never an executable Lean callback. -/
structure EqualityContractId where
  canonical : CanonicalBytes
deriving DecidableEq

/-- An opaque value at one structural index. Canonicalization and contract
admissibility are checked separately from representation comparison. -/
structure Atom (index : StructuralIndex) where
  kind : AtomKindId
  canonicalPayload : CanonicalBytes
  equalityContract : EqualityContractId

/- The only recursive holdable structure in Clause Core.

The Triple positions are structurally neutral. A contextual Clause judgment may
later interpret them relationally; construction grants no meaning or authority.
The Triple constructor intentionally contains exactly three Terms and no
nominal identity field. -/
inductive Term (index : StructuralIndex) where
  | atom (value : Atom index)
  | triple (first second third : Term index)

namespace Atom

/-- Executable comparison of candidate Atom representations. -/
def sameRepresentation (left right : Atom index) : Bool :=
  decide (left.kind = right.kind) &&
    decide (left.canonicalPayload = right.canonicalPayload) &&
    decide (left.equalityContract = right.equalityContract)

/-- Representation sameness is not semantic structural equality until Atom
contract admission establishes that these bytes are canonical. -/
def SameRepresentation (left right : Atom index) : Prop :=
  sameRepresentation left right = true

theorem sameRepresentation_iff_eq (left right : Atom index) :
    sameRepresentation left right = true ↔ left = right := by
  cases left
  cases right
  simp [sameRepresentation, and_assoc]

theorem sameRepresentation_self (value : Atom index) :
    sameRepresentation value value = true := by
  exact (sameRepresentation_iff_eq value value).2 rfl

end Atom

namespace Term

/-- Executable recursive comparison of candidate Term representations. -/
def sameRepresentation : Term index → Term index → Bool
  | .atom left, .atom right => Atom.sameRepresentation left right
  | .triple first₁ second₁ third₁, .triple first₂ second₂ third₂ =>
      sameRepresentation first₁ first₂ &&
        sameRepresentation second₁ second₂ &&
        sameRepresentation third₁ third₂
  | _, _ => false

/-- Representation sameness for recursive candidate Terms. -/
def SameRepresentation (left right : Term index) : Prop :=
  sameRepresentation left right = true

theorem sameRepresentation_iff_eq : ∀ left right : Term index,
    sameRepresentation left right = true ↔ left = right
  | .atom left, .atom right => by
      simp [sameRepresentation, Atom.sameRepresentation_iff_eq]
  | .atom _, .triple _ _ _ => by
      simp [sameRepresentation]
  | .triple _ _ _, .atom _ => by
      simp [sameRepresentation]
  | .triple first₁ second₁ third₁, .triple first₂ second₂ third₂ => by
      simp [sameRepresentation, sameRepresentation_iff_eq first₁ first₂,
        sameRepresentation_iff_eq second₁ second₂,
        sameRepresentation_iff_eq third₁ third₂, and_assoc]

theorem sameRepresentation_self (value : Term index) :
    sameRepresentation value value = true :=
  (sameRepresentation_iff_eq value value).2 rfl

end Term

/-- A runtime-discovered candidate Term paired with its exact representation
index. This package is not a third Term form. -/
structure ScopedTerm where
  index : StructuralIndex
  term : Term index

namespace ScopedTerm

/-- Representation comparison includes the index; no migration or
reinterpretation is implicit. -/
def sameRepresentation (left right : ScopedTerm) : Bool :=
  if sameIndex : left.index = right.index then
    Term.sameRepresentation (sameIndex ▸ left.term) right.term
  else
    false

/-- Representation sameness for runtime-discovered candidate Terms. -/
def SameRepresentation (left right : ScopedTerm) : Prop :=
  sameRepresentation left right = true

theorem sameRepresentation_self (value : ScopedTerm) :
    sameRepresentation value value = true := by
  simp [sameRepresentation, Term.sameRepresentation_self]

theorem sameRepresentation_false_of_index_ne (left right : ScopedTerm)
    (differentIndex : left.index ≠ right.index) :
    sameRepresentation left right = false := by
  simp [sameRepresentation, differentIndex]

end ScopedTerm

/-! ## Candidate context and judgment structure -/

/-- The conclusion proposed by the right-hand side of
`Γ ⊢ term clause : type @ mode`.

All three fields remain ordinary Terms. Constructing this value records only a
candidate claim; it does not establish a Clause judgment. -/
structure JudgmentClaim (index : StructuralIndex) where
  term : Term index
  typeTerm : Term index
  mode : Term index

namespace JudgmentClaim

/-- Exact candidate-representation comparison for a judgment claim. This is
not contextual validity or semantic equality. -/
def sameRepresentation (left right : JudgmentClaim index) : Bool :=
  Term.sameRepresentation left.term right.term &&
    Term.sameRepresentation left.typeTerm right.typeTerm &&
    Term.sameRepresentation left.mode right.mode

/-- Representation sameness for candidate judgment claims. -/
def SameRepresentation (left right : JudgmentClaim index) : Prop :=
  sameRepresentation left right = true

theorem sameRepresentation_iff_eq (left right : JudgmentClaim index) :
    sameRepresentation left right = true ↔ left = right := by
  cases left
  cases right
  simp [sameRepresentation, Term.sameRepresentation_iff_eq, and_assoc]

theorem sameRepresentation_self (claim : JudgmentClaim index) :
    sameRepresentation claim claim = true :=
  (sameRepresentation_iff_eq claim claim).2 rfl

end JudgmentClaim

/-- An immutable candidate enumeration of judgment premises at one exact
structural index.

The list is a transport representation only. Position, multiplicity, and raw
membership grant no truth, validity, authority, or source order. -/
structure ContextCandidate (index : StructuralIndex) where
  premises : List (JudgmentClaim index)

namespace ContextCandidate

/-- Representation-only premise lookup. It may support external addressing,
but the relative certificate checker deliberately does not consume candidate
Context membership, and a successful lookup is not a derivation or admission. -/
def containsRepresentation (context : ContextCandidate index)
    (claim : JudgmentClaim index) : Bool :=
  context.premises.any (fun premise =>
    JudgmentClaim.sameRepresentation premise claim)

/-- Candidate representation membership, not judgment validity. -/
def ContainsRepresentation (context : ContextCandidate index)
    (claim : JudgmentClaim index) : Prop :=
  containsRepresentation context claim = true

theorem empty_contains_no_representation (claim : JudgmentClaim index) :
    containsRepresentation ⟨[]⟩ claim = false := by
  rfl

theorem head_has_matching_representation (claim : JudgmentClaim index)
    (remaining : List (JudgmentClaim index)) :
    containsRepresentation ⟨claim :: remaining⟩ claim = true := by
  simp [containsRepresentation, JudgmentClaim.sameRepresentation_self]

end ContextCandidate

/-- Raw data for one proposed contextual Clause judgment. The future generic
checker, not this constructor, decides whether the context entails the claim. -/
structure ClauseJudgmentCandidate (index : StructuralIndex) where
  context : ContextCandidate index
  claim : JudgmentClaim index

/-! ## Finite relative derivation certificates -/

/-- One already-ground rule candidate. Premises and conclusion remain generic
judgment claims; this structure contains no executable Lean callback, schema
matcher, or language-feature tag. -/
structure GroundRuleCandidate (index : StructuralIndex) where
  premises : List (JudgmentClaim index)
  conclusion : JudgmentClaim index

/-- The exact roots and ground rules against which a certificate is checked.

This is candidate data supplied across an explicit authority boundary. The
checker establishes derivability relative to this basis; constructing a basis
does not establish that the basis is accepted, valid, or authoritative. A
candidate Context is deliberately absent, so raw Context membership cannot be
used as a proof step. -/
structure DerivationBasisCandidate (index : StructuralIndex) where
  roots : List (JudgmentClaim index)
  rules : List (GroundRuleCandidate index)

/-- The two generic operations in a finite ground derivation certificate.
Natural-number references are package-local addresses, not semantic identity. -/
inductive CertificateReason where
  | root (rootRef : Nat)
  | apply (ruleRef : Nat) (premiseRefs : List Nat)

/-- One proposed conclusion and the generic reason offered for it. -/
structure CertificateNode (index : StructuralIndex) where
  claimed : JudgmentClaim index
  reason : CertificateReason

/-- A topologically ordered finite certificate trace. The checker rejects an
empty trace and permits each application to reference only the already checked
prefix. -/
structure DerivationCertificate (index : StructuralIndex) where
  nodes : List (CertificateNode index)

/-- The independent propositional meaning of derivability relative to a
supplied basis. It never consults a candidate Context and makes no claim that
the basis itself is accepted. -/
inductive DerivableFrom (basis : DerivationBasisCandidate index) :
    JudgmentClaim index → Prop where
  | root {claim : JudgmentClaim index}
      (available : claim ∈ basis.roots) : DerivableFrom basis claim
  | apply {rule : GroundRuleCandidate index}
      (available : rule ∈ basis.rules)
      (premises : ∀ claim, claim ∈ rule.premises → DerivableFrom basis claim) :
      DerivableFrom basis rule.conclusion

/-- A constructive, order-preserving collection of relative derivations. -/
inductive AllDerivable (basis : DerivationBasisCandidate index) :
    List (JudgmentClaim index) → Prop where
  | nil : AllDerivable basis []
  | cons {claim : JudgmentClaim index} {remaining : List (JudgmentClaim index)}
      (head : DerivableFrom basis claim)
      (tail : AllDerivable basis remaining) :
      AllDerivable basis (claim :: remaining)

namespace DerivationCertificate

private theorem allDerivable_elim
    (evidence : AllDerivable basis claims) :
    ∀ claim, claim ∈ claims → DerivableFrom basis claim := by
  induction evidence with
  | nil =>
      intro claim member
      cases member
  | cons headEvidence _ inductionHypothesis =>
      intro claim member
      cases member with
      | head => exact headEvidence
      | tail _ inTail => exact inductionHypothesis claim inTail

private theorem allDerivable_append
    (leftEvidence : AllDerivable basis left)
    (rightEvidence : AllDerivable basis right) :
    AllDerivable basis (left ++ right) := by
  induction leftEvidence with
  | nil => exact rightEvidence
  | cons headEvidence _ inductionHypothesis =>
      exact AllDerivable.cons headEvidence
        inductionHypothesis

private theorem claimed_mem_of_node_mem
    {node : CertificateNode index} {nodes : List (CertificateNode index)}
    (member : node ∈ nodes) :
    node.claimed ∈ nodes.map CertificateNode.claimed := by
  induction member with
  | head => exact List.Mem.head _
  | tail prior _ inductionHypothesis =>
      exact List.Mem.tail prior.claimed inductionHypothesis

/-- Exact representation matching for a premise-reference list. List order is
the rule's declared ground-premise sequence, not source or Context order. Every
reference resolves only inside the already checked prefix. -/
def referencesMatch (priorClaims : List (JudgmentClaim index))
    : List Nat → List (JudgmentClaim index) → Bool
  | [], [] => true
  | premiseRef :: laterRefs, premise :: laterPremises =>
      match priorClaims[premiseRef]? with
      | some checked =>
          !(laterRefs.any fun laterRef => decide (premiseRef = laterRef)) &&
            (JudgmentClaim.sameRepresentation checked premise &&
              referencesMatch priorClaims laterRefs laterPremises)
      | none => false
  | _, _ => false

/-- Check one certificate node against the supplied basis and the already
checked prefix. A self-reference, forward reference, or back-edge is out of
range in that prefix and therefore rejected. -/
def checkNode (basis : DerivationBasisCandidate index)
    (priorClaims : List (JudgmentClaim index))
    (node : CertificateNode index) : Bool :=
  match node.reason with
  | .root rootRef =>
      match basis.roots[rootRef]? with
      | some root => JudgmentClaim.sameRepresentation node.claimed root
      | none => false
  | .apply ruleRef premiseRefs =>
      match basis.rules[ruleRef]? with
      | some rule =>
          referencesMatch priorClaims premiseRefs rule.premises &&
            JudgmentClaim.sameRepresentation node.claimed rule.conclusion
      | none => false

private def extendCheckedPrefix (basis : DerivationBasisCandidate index)
    (state : Option (List (JudgmentClaim index)))
    (node : CertificateNode index) : Option (List (JudgmentClaim index)) :=
  match state with
  | none => none
  | some priorClaims =>
      if checkNode basis priorClaims node = true then
        some (priorClaims ++ [node.claimed])
      else
        none

/-- Execute the one-pass finite checker. `some` contains the exact checked
conclusion prefix; `none` means that at least one obligation failed. -/
def checkTrace (basis : DerivationBasisCandidate index)
    (certificate : DerivationCertificate index) :
    Option (List (JudgmentClaim index)) :=
  certificate.nodes.foldl (extendCheckedPrefix basis) (some [])

/-- Check a nonempty certificate and bind its final conclusion to the requested
claim. Acceptance remains relative to `basis`; it is not admission. -/
def checkRelative (basis : DerivationBasisCandidate index)
    (certificate : DerivationCertificate index)
    (requested : JudgmentClaim index) : Bool :=
  match checkTrace basis certificate, certificate.nodes.getLast? with
  | some _, some last =>
      JudgmentClaim.sameRepresentation last.claimed requested
  | _, _ => false

private theorem mem_of_get?_eq_some {values : List α} {position : Nat}
    {value : α} (found : values[position]? = some value) : value ∈ values := by
  induction values generalizing position with
  | nil => simp at found
  | cons head tail inductionHypothesis =>
      cases position with
      | zero =>
          simp at found
          subst value
          simp
      | succ previous =>
          simp at found
          exact List.mem_cons_of_mem head (inductionHypothesis found)

private theorem referencesMatch_sound
    {priorClaims : List (JudgmentClaim index)}
    {premiseRefs : List Nat} {premises : List (JudgmentClaim index)}
    (matched : referencesMatch priorClaims premiseRefs premises = true)
    (priorSound : ∀ claim, claim ∈ priorClaims → DerivableFrom basis claim) :
    ∀ claim, claim ∈ premises → DerivableFrom basis claim := by
  induction premiseRefs generalizing premises with
  | nil =>
      cases premises with
      | nil =>
          intro claim member
          cases member
      | cons _ _ => simp [referencesMatch] at matched
  | cons premiseRef remainingRefs inductionHypothesis =>
      cases premises with
      | nil => simp [referencesMatch] at matched
      | cons premise remainingPremises =>
          cases found : priorClaims[premiseRef]? with
          | none => simp [referencesMatch, found] at matched
          | some checked =>
              have unfolded :
                  (!(remainingRefs.any fun laterRef =>
                      decide (premiseRef = laterRef)) &&
                    (JudgmentClaim.sameRepresentation checked premise &&
                      referencesMatch priorClaims remainingRefs
                        remainingPremises)) = true := by
                simpa only [referencesMatch, found] using matched
              have matchParts :
                  JudgmentClaim.sameRepresentation checked premise = true ∧
                      referencesMatch priorClaims remainingRefs
                      remainingPremises = true :=
                Bool.and_eq_true_iff.mp (Bool.and_eq_true_iff.mp unfolded).2
              have checkedInPrior : checked ∈ priorClaims :=
                mem_of_get?_eq_some found
              have checkedDerivable : DerivableFrom basis checked :=
                priorSound checked checkedInPrior
              have headMatches :
                  JudgmentClaim.sameRepresentation checked premise = true := by
                exact matchParts.1
              have premiseDerivable : DerivableFrom basis premise := by
                have sameClaim : checked = premise :=
                  (JudgmentClaim.sameRepresentation_iff_eq checked premise).1
                    headMatches
                simpa [sameClaim] using checkedDerivable
              have tailMatched :
                  referencesMatch priorClaims remainingRefs remainingPremises = true := by
                exact matchParts.2
              have tailDerivable :=
                inductionHypothesis tailMatched
              intro claim member
              cases member with
              | head => exact premiseDerivable
              | tail _ isTail => exact tailDerivable claim isTail

private theorem checkNode_sound
    {priorClaims : List (JudgmentClaim index)} {node : CertificateNode index}
    (priorSound : ∀ claim, claim ∈ priorClaims → DerivableFrom basis claim)
    (accepted : checkNode basis priorClaims node = true) :
    DerivableFrom basis node.claimed := by
  cases reason : node.reason with
  | root rootRef =>
      cases found : basis.roots[rootRef]? with
      | none => simp [checkNode, reason, found] at accepted
      | some root =>
          have rootAvailable : root ∈ basis.roots :=
            mem_of_get?_eq_some found
          have sameClaim : node.claimed = root :=
            (JudgmentClaim.sameRepresentation_iff_eq node.claimed root).1 (by
              simpa [checkNode, reason, found] using accepted)
          simpa [sameClaim] using DerivableFrom.root rootAvailable
  | apply ruleRef premiseRefs =>
      cases found : basis.rules[ruleRef]? with
      | none => simp [checkNode, reason, found] at accepted
      | some rule =>
          have acceptedParts :
              referencesMatch priorClaims premiseRefs rule.premises = true ∧
                JudgmentClaim.sameRepresentation node.claimed
                  rule.conclusion = true := by
            simpa [checkNode, reason, found] using accepted
          have ruleAvailable : rule ∈ basis.rules :=
            mem_of_get?_eq_some found
          have premisesMatch :
              referencesMatch priorClaims premiseRefs rule.premises = true := by
            exact acceptedParts.1
          have conclusionMatches :
              JudgmentClaim.sameRepresentation node.claimed
                rule.conclusion = true := by
            exact acceptedParts.2
          have premisesDerivable :
              ∀ claim, claim ∈ rule.premises → DerivableFrom basis claim :=
            referencesMatch_sound premisesMatch priorSound
          have ruleDerivation : DerivableFrom basis rule.conclusion :=
            DerivableFrom.apply ruleAvailable premisesDerivable
          have sameConclusion : node.claimed = rule.conclusion :=
            (JudgmentClaim.sameRepresentation_iff_eq node.claimed
              rule.conclusion).1 conclusionMatches
          simpa [sameConclusion] using ruleDerivation

private theorem failedStateRemainsFailed
    (nodes : List (CertificateNode index)) :
    nodes.foldl (extendCheckedPrefix basis) none = none := by
  induction nodes with
  | nil => rfl
  | cons node remaining inductionHypothesis =>
      simpa [List.foldl, extendCheckedPrefix] using inductionHypothesis

private theorem checkFold_sound
    {nodes : List (CertificateNode index)}
    {priorClaims result : List (JudgmentClaim index)}
    (priorSound : AllDerivable basis priorClaims)
    (accepted :
      nodes.foldl (extendCheckedPrefix basis) (some priorClaims) = some result) :
    AllDerivable basis
      (priorClaims ++ nodes.map CertificateNode.claimed) := by
  induction nodes generalizing priorClaims result with
  | nil =>
      simpa only [List.map, List.append_nil] using priorSound
  | cons node remaining inductionHypothesis =>
      by_cases nodeAccepted : checkNode basis priorClaims node = true
      · have nodeDerivable : DerivableFrom basis node.claimed :=
          checkNode_sound (allDerivable_elim priorSound) nodeAccepted
        have extendedSound : AllDerivable basis
            (priorClaims ++ [node.claimed]) :=
          allDerivable_append priorSound
            (AllDerivable.cons nodeDerivable AllDerivable.nil)
        have remainingAccepted :
            remaining.foldl (extendCheckedPrefix basis)
              (some (priorClaims ++ [node.claimed])) = some result := by
          simpa [List.foldl, extendCheckedPrefix, nodeAccepted] using accepted
        have remainingSound :=
          inductionHypothesis extendedSound remainingAccepted
        change AllDerivable basis
          (priorClaims ++ ([node.claimed] ++
            remaining.map CertificateNode.claimed))
        simpa only [List.append_assoc] using remainingSound
      · have remainsFailed :
            remaining.foldl (extendCheckedPrefix basis) none = none :=
          failedStateRemainsFailed remaining
        simp [List.foldl, extendCheckedPrefix, nodeAccepted,
          remainsFailed] at accepted

/-- Successful execution establishes the independent propositional relation,
but only relative to the exact supplied basis. -/
theorem checkRelative_sound
    (basis : DerivationBasisCandidate index)
    (certificate : DerivationCertificate index)
    (requested : JudgmentClaim index)
    (accepted : checkRelative basis certificate requested = true) :
    DerivableFrom basis requested := by
  cases traceResult : checkTrace basis certificate with
  | none => simp [checkRelative, traceResult] at accepted
  | some checked =>
      cases lastResult : certificate.nodes.getLast? with
      | none => simp [checkRelative, traceResult, lastResult] at accepted
      | some last =>
          have checkedSound :
              AllDerivable basis
                (certificate.nodes.map CertificateNode.claimed) := by
            have foldSound := checkFold_sound (basis := basis)
              (priorClaims := []) (result := checked)
              (nodes := certificate.nodes)
              AllDerivable.nil (by simpa [checkTrace] using traceResult)
            exact foldSound
          have lastInNodes : last ∈ certificate.nodes :=
            List.mem_of_getLast? lastResult
          have lastDerivable : DerivableFrom basis last.claimed :=
            allDerivable_elim checkedSound last.claimed
              (claimed_mem_of_node_mem lastInNodes)
          have requestedMatches :
              JudgmentClaim.sameRepresentation last.claimed requested = true := by
            simpa [checkRelative, traceResult, lastResult] using accepted
          have sameClaim : last.claimed = requested :=
            (JudgmentClaim.sameRepresentation_iff_eq last.claimed requested).1
              requestedMatches
          simpa [sameClaim] using lastDerivable

end DerivationCertificate

/-! ## Exact package and constitutional-anchor binding -/

/-- The decoded content of one candidate Clause Core package.

The basis, certificate, requested target, and every other decoded section are
held together so that authority cannot bind only a convenient projection. The
auxiliary section carrier remains opaque canonical data until Clause defines
the corresponding typed package sections. It carries no authority or admission
evidence. -/
structure DecodedPackageSections (index : StructuralIndex) where
  basis : DerivationBasisCandidate index
  certificate : DerivationCertificate index
  target : JudgmentClaim index
  auxiliary : List CanonicalBytes

/-- One exact candidate package record. `index.semanticsId` is its semantics
epoch. Grouping bytes, index, and all decoded sections does not make any of
them canonical or authoritative. -/
structure CanonicalPackageCandidate where
  canonicalBytes : CanonicalBytes
  index : StructuralIndex
  decoded : DecodedPackageSections index

/-- Exact whole-record binding between a candidate package and an externally
selected package. A digest or separately reconstructed projection is not an
acceptable substitute. -/
def ExactPackageBinding (candidate selected : CanonicalPackageCandidate) : Prop :=
  candidate = selected

/-- The closed constitutional authority boundary for exact package records.

There is deliberately no constructor: no constitutional bootstrap package has
yet been selected. A future tranche may add only a literal, independently
reviewed anchor. Candidate bytes, decoded fields, Context membership, and
relative derivations cannot inhabit this predicate. -/
inductive ConstitutionalPackageAnchor : CanonicalPackageCandidate → Prop

/-- The narrow conclusion justified by exact package authority plus relative
certificate checking. This is neither semantic truth nor general Admission. -/
def PackageBoundDerivable (package : CanonicalPackageCandidate) : Prop :=
  ConstitutionalPackageAnchor package ∧
    DerivableFrom package.decoded.basis package.decoded.target

/-- Checking the certificate from the exact anchored package promotes only its
requested target to package-bound relative derivability. The unchanged package
record keeps bytes, epoch, decoded sections, basis, certificate, and target in
one authority index. -/
theorem checkExactPackage_sound
    (candidate selected : CanonicalPackageCandidate)
    (bound : ExactPackageBinding candidate selected)
    (anchored : ConstitutionalPackageAnchor selected)
    (accepted :
      DerivationCertificate.checkRelative candidate.decoded.basis
        candidate.decoded.certificate candidate.decoded.target = true) :
    PackageBoundDerivable candidate := by
  have candidateAnchored : ConstitutionalPackageAnchor candidate := by
    rw [bound]
    exact anchored
  exact ⟨candidateAnchored,
    DerivationCertificate.checkRelative_sound candidate.decoded.basis
      candidate.decoded.certificate candidate.decoded.target accepted⟩

/-- Until a literal constitutional anchor is selected, no candidate package
can manufacture authority. -/
theorem noConstitutionalPackageAnchor (package : CanonicalPackageCandidate) :
    ¬ ConstitutionalPackageAnchor package := by
  intro anchored
  cases anchored

/-! ## Kernel-checked constitutional examples -/

namespace Examples

private def bytes (data : List UInt8) : CanonicalBytes := ⟨data⟩
private def universeId (tag : UInt8) : UniverseId := ⟨bytes [tag]⟩
private def semanticsId (tag : UInt8) : ClauseSemanticsId := ⟨bytes [tag]⟩
private def atomKind (tag : UInt8) : AtomKindId := ⟨bytes [tag]⟩
private def equalityContract (tag : UInt8) : EqualityContractId := ⟨bytes [tag]⟩

private def indexA : StructuralIndex := ⟨universeId 1, semanticsId 1⟩
private def indexB : StructuralIndex := ⟨universeId 1, semanticsId 2⟩
private def indexC : StructuralIndex := ⟨universeId 2, semanticsId 1⟩

private def atomAt (index : StructuralIndex) (kind contract payload : UInt8) :
    Term index :=
  .atom {
    kind := atomKind kind
    canonicalPayload := bytes [payload]
    equalityContract := equalityContract contract
  }

private def alice : Term indexA := atomAt indexA 1 10 1
private def transfers : Term indexA := atomAt indexA 2 10 2
private def tenDollars : Term indexA := atomAt indexA 3 10 10

private def transferContent : Term indexA :=
  .triple alice transfers tenDollars

theorem separately_constructed_triples_have_the_same_representation :
    Term.sameRepresentation transferContent
      (.triple alice transfers tenDollars) = true := by
  decide

theorem constructor_shape_is_observable :
    Term.sameRepresentation transferContent alice = false := by
  decide

theorem atom_kind_is_observable :
    Term.sameRepresentation (atomAt indexA 1 10 1)
      (atomAt indexA 2 10 1) = false := by
  decide

theorem atom_payload_is_observable :
    Term.sameRepresentation (atomAt indexA 1 10 1)
      (atomAt indexA 1 10 2) = false := by
  decide

theorem atom_equality_contract_is_observable :
    Term.sameRepresentation (atomAt indexA 1 10 1)
      (atomAt indexA 1 11 1) = false := by
  decide

-- Byte-identical Atom candidates at different indexes have distinct representations.
private def epochATerm : ScopedTerm :=
  ⟨indexA, atomAt indexA 1 10 1⟩
private def epochBTerm : ScopedTerm :=
  ⟨indexB, atomAt indexB 1 10 1⟩
private def universeBTerm : ScopedTerm :=
  ⟨indexC, atomAt indexC 1 10 1⟩

theorem same_index_terms_have_the_same_representation :
    ScopedTerm.sameRepresentation epochATerm epochATerm = true := by
  decide

theorem cross_epoch_terms_do_not_have_the_same_representation :
    ScopedTerm.sameRepresentation epochATerm epochBTerm = false := by
  decide

theorem cross_universe_terms_do_not_have_the_same_representation :
    ScopedTerm.sameRepresentation epochATerm universeBTerm = false := by
  decide

private def propositionType : Term indexA := atomAt indexA 4 10 20
private def quotedType : Term indexA := atomAt indexA 5 10 21
private def pureMode : Term indexA := atomAt indexA 6 10 22
private def quotedMode : Term indexA := atomAt indexA 7 10 23

private def propositionClaim : JudgmentClaim indexA := {
  term := transferContent
  typeTerm := propositionType
  mode := pureMode
}

private def quotedClaim : JudgmentClaim indexA := {
  term := transferContent
  typeTerm := quotedType
  mode := quotedMode
}

private def emptyContext : ContextCandidate indexA := ⟨[]⟩
private def contextWithProposition : ContextCandidate indexA :=
  ⟨[propositionClaim]⟩

private def proposedJudgment : ClauseJudgmentCandidate indexA := {
  context := contextWithProposition
  claim := propositionClaim
}

theorem one_term_can_have_distinct_candidate_judgments :
    Term.sameRepresentation propositionClaim.term quotedClaim.term = true ∧
      JudgmentClaim.sameRepresentation propositionClaim quotedClaim = false := by
  decide

theorem term_construction_grants_no_context_membership :
    ContextCandidate.containsRepresentation emptyContext propositionClaim = false := by
  decide

theorem candidate_premise_membership_is_explicit :
    ContextCandidate.containsRepresentation contextWithProposition
      propositionClaim = true := by
  decide

theorem proposed_judgment_keeps_context_outside_the_term :
    proposedJudgment.claim.term = transferContent := by
  rfl

/-! ### Relative certificate checking -/

private def claimA : JudgmentClaim indexA := propositionClaim

private def claimB : JudgmentClaim indexA := {
  term := atomAt indexA 20 10 1
  typeTerm := propositionType
  mode := pureMode
}

private def claimC : JudgmentClaim indexA := {
  term := atomAt indexA 20 10 2
  typeTerm := propositionType
  mode := pureMode
}

private def claimD : JudgmentClaim indexA := {
  term := atomAt indexA 20 10 3
  typeTerm := propositionType
  mode := pureMode
}

private def ruleAB : GroundRuleCandidate indexA := {
  premises := [claimA]
  conclusion := claimB
}

private def ruleAC : GroundRuleCandidate indexA := {
  premises := [claimA]
  conclusion := claimC
}

private def ruleBCD : GroundRuleCandidate indexA := {
  premises := [claimB, claimC]
  conclusion := claimD
}

private def finiteBasis : DerivationBasisCandidate indexA := {
  roots := [claimA]
  rules := [ruleAB, ruleAC, ruleBCD]
}

private def sharedDagCertificate : DerivationCertificate indexA := ⟨[
  ⟨claimA, .root 0⟩,
  ⟨claimB, .apply 0 [0]⟩,
  ⟨claimC, .apply 1 [0]⟩,
  ⟨claimD, .apply 2 [1, 2]⟩
]⟩

theorem finite_shared_dag_certificate_is_accepted :
    DerivationCertificate.checkRelative finiteBasis sharedDagCertificate
      claimD = true := by
  decide

theorem accepted_certificate_is_relatively_derivable :
    DerivableFrom finiteBasis claimD :=
  DerivationCertificate.checkRelative_sound finiteBasis sharedDagCertificate
    claimD (by decide)

theorem empty_certificate_is_rejected :
    DerivationCertificate.checkRelative finiteBasis ⟨[]⟩ claimA = false := by
  decide

theorem missing_root_is_rejected :
    DerivationCertificate.checkRelative finiteBasis
      ⟨[⟨claimA, .root 1⟩]⟩ claimA = false := by
  decide

theorem missing_rule_is_rejected :
    DerivationCertificate.checkRelative finiteBasis
      ⟨[⟨claimB, .apply 9 []⟩]⟩ claimB = false := by
  decide

theorem missing_premise_is_rejected :
    DerivationCertificate.checkRelative finiteBasis
      ⟨[⟨claimA, .root 0⟩, ⟨claimB, .apply 0 []⟩]⟩ claimB = false := by
  decide

theorem mismatched_premise_is_rejected :
    DerivationCertificate.checkRelative finiteBasis
      ⟨[⟨claimA, .root 0⟩, ⟨claimB, .apply 0 [0]⟩,
        ⟨claimB, .apply 0 [1]⟩]⟩ claimB = false := by
  decide

theorem mismatched_rule_conclusion_is_rejected :
    DerivationCertificate.checkRelative finiteBasis
      ⟨[⟨claimA, .root 0⟩, ⟨claimC, .apply 0 [0]⟩]⟩ claimB = false := by
  decide

theorem altered_target_is_rejected :
    DerivationCertificate.checkRelative finiteBasis sharedDagCertificate
      claimC = false := by
  decide

private def selfRule : GroundRuleCandidate indexA := {
  premises := [claimA]
  conclusion := claimA
}

private def selfRuleBasis : DerivationBasisCandidate indexA := {
  roots := []
  rules := [selfRule]
}

theorem self_reference_is_rejected :
    DerivationCertificate.checkRelative selfRuleBasis
      ⟨[⟨claimA, .apply 0 [0]⟩]⟩ claimA = false := by
  decide

theorem forward_reference_is_rejected :
    DerivationCertificate.checkRelative finiteBasis
      ⟨[⟨claimB, .apply 0 [1]⟩, ⟨claimA, .root 0⟩]⟩ claimB = false := by
  decide

private def mutualCycleBasis : DerivationBasisCandidate indexA := {
  roots := []
  rules := [
    ⟨[claimA], claimB⟩,
    ⟨[claimB], claimA⟩
  ]
}

theorem mutual_cycle_is_rejected :
    DerivationCertificate.checkRelative mutualCycleBasis
      ⟨[⟨claimA, .apply 1 [1]⟩, ⟨claimB, .apply 0 [0]⟩]⟩
      claimB = false := by
  decide

private def duplicatePremiseBasis : DerivationBasisCandidate indexA := {
  roots := [claimA]
  rules := [⟨[claimA, claimA], claimB⟩]
}

theorem one_node_address_cannot_fill_two_premise_positions :
    DerivationCertificate.checkRelative duplicatePremiseBasis
      ⟨[⟨claimA, .root 0⟩, ⟨claimB, .apply 0 [0, 0]⟩]⟩ claimB = false := by
  decide

-- Candidate declarations for a contract that claims all payloads equal.
-- They remain raw Context data and are intentionally not roots or rules.
private def allEqualContractClaim : JudgmentClaim indexA := {
  term := atomAt indexA 30 99 1
  typeTerm := propositionType
  mode := pureMode
}

private def allEqualTotalityClaim : JudgmentClaim indexA := {
  term := atomAt indexA 31 99 1
  typeTerm := propositionType
  mode := pureMode
}

private def allEqualDeterminismClaim : JudgmentClaim indexA := {
  term := atomAt indexA 32 99 1
  typeTerm := propositionType
  mode := pureMode
}

private def allEqualCanonicalityClaim : JudgmentClaim indexA := {
  term := atomAt indexA 33 99 1
  typeTerm := propositionType
  mode := pureMode
}

private def selfAuthorizingContext : ContextCandidate indexA := ⟨[
  allEqualContractClaim,
  allEqualTotalityClaim,
  allEqualDeterminismClaim,
  allEqualCanonicalityClaim
]⟩

private def noIndependentAuthority : DerivationBasisCandidate indexA := {
  roots := []
  rules := []
}

private def selfAuthorizingCertificate : DerivationCertificate indexA :=
  ⟨[⟨allEqualContractClaim, .root 0⟩]⟩

private def proposedAllEqualRule : GroundRuleCandidate indexA := {
  premises := []
  conclusion := allEqualContractClaim
}

private def candidateRuleCertificate : DerivationCertificate indexA :=
  ⟨[⟨proposedAllEqualRule.conclusion, .apply 0 []⟩]⟩

theorem raw_all_equal_context_is_present :
    ContextCandidate.containsRepresentation selfAuthorizingContext
      allEqualContractClaim = true := by
  decide

theorem raw_all_equal_context_cannot_authorize_itself :
    DerivationCertificate.checkRelative noIndependentAuthority
      selfAuthorizingCertificate allEqualContractClaim = false := by
  decide

theorem candidate_all_equal_rule_absent_from_basis_is_rejected :
    DerivationCertificate.checkRelative noIndependentAuthority
      candidateRuleCertificate allEqualContractClaim = false := by
  decide

theorem rejected_all_equal_data_does_not_collapse_payload_representations :
    Term.sameRepresentation (atomAt indexA 30 99 1)
      (atomAt indexA 30 99 2) = false := by
  decide

/-! ### Exact package binding -/

private def packageAt (index : StructuralIndex)
    (packageBytes : CanonicalBytes)
    (sections : DecodedPackageSections index) : CanonicalPackageCandidate := {
  canonicalBytes := packageBytes
  index := index
  decoded := sections
}

private def checkedPackage : CanonicalPackageCandidate :=
  packageAt indexA (bytes [100, 1]) {
    basis := finiteBasis
    certificate := sharedDagCertificate
    target := claimD
    auxiliary := [bytes [110, 1], bytes [110, 2]]
  }

private def packageCheck (package : CanonicalPackageCandidate) : Bool :=
  DerivationCertificate.checkRelative package.decoded.basis
    package.decoded.certificate package.decoded.target

private def packageAuxiliary
    (package : CanonicalPackageCandidate) : List CanonicalBytes :=
  package.decoded.auxiliary

private def packageSemanticsBytes
    (package : CanonicalPackageCandidate) : CanonicalBytes :=
  package.index.semanticsId.canonical

private theorem differentCheckBreaksExactBinding
    (candidate selected : CanonicalPackageCandidate)
    (different : packageCheck candidate ≠ packageCheck selected) :
    ¬ ExactPackageBinding candidate selected := by
  intro bound
  change candidate = selected at bound
  exact different (congrArg packageCheck bound)

theorem checked_package_is_exactly_self_bound :
    ExactPackageBinding checkedPackage checkedPackage := by
  rfl

theorem checked_package_certificate_is_relatively_accepted :
    packageCheck checkedPackage = true := by
  decide

private def bytesTamperedPackage : CanonicalPackageCandidate :=
  packageAt indexA (bytes [100, 2]) checkedPackage.decoded

theorem changed_package_bytes_break_exact_binding :
    ¬ ExactPackageBinding bytesTamperedPackage checkedPackage := by
  intro bound
  change bytesTamperedPackage = checkedPackage at bound
  have bytesEqual :=
    congrArg CanonicalPackageCandidate.canonicalBytes bound
  have bytesDifferent :
      bytesTamperedPackage.canonicalBytes ≠ checkedPackage.canonicalBytes := by
    decide
  exact bytesDifferent bytesEqual

private def epochClaim (index : StructuralIndex) : JudgmentClaim index := {
  term := atomAt index 90 90 90
  typeTerm := atomAt index 91 90 91
  mode := atomAt index 92 90 92
}

private def epochPackage (index : StructuralIndex) : CanonicalPackageCandidate :=
  packageAt index (bytes [101, 1]) {
    basis := { roots := [], rules := [] }
    certificate := ⟨[]⟩
    target := epochClaim index
    auxiliary := [bytes [111, 1]]
  }

theorem changed_semantics_epoch_breaks_exact_binding :
    ¬ ExactPackageBinding (epochPackage indexB) (epochPackage indexA) := by
  intro bound
  change epochPackage indexB = epochPackage indexA at bound
  have epochEqual := congrArg packageSemanticsBytes bound
  have epochsDifferent :
      packageSemanticsBytes (epochPackage indexB) ≠
        packageSemanticsBytes (epochPackage indexA) := by
    decide
  exact epochsDifferent epochEqual

private def auxiliaryTamperedPackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := finiteBasis
    certificate := sharedDagCertificate
    target := claimD
    auxiliary := [bytes [110, 1], bytes [110, 3]]
  }

theorem changed_decoded_auxiliary_content_breaks_exact_binding :
    ¬ ExactPackageBinding auxiliaryTamperedPackage checkedPackage := by
  intro bound
  change auxiliaryTamperedPackage = checkedPackage at bound
  have auxiliaryEqual := congrArg packageAuxiliary bound
  have auxiliaryDifferent :
      packageAuxiliary auxiliaryTamperedPackage ≠
        packageAuxiliary checkedPackage := by
    decide
  exact auxiliaryDifferent auxiliaryEqual

private def rootTamperedBasis : DerivationBasisCandidate indexA := {
  roots := [claimB]
  rules := finiteBasis.rules
}

private def rootTamperedPackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := rootTamperedBasis
    certificate := sharedDagCertificate
    target := claimD
    auxiliary := checkedPackage.decoded.auxiliary
  }

theorem changed_root_content_breaks_exact_binding :
    ¬ ExactPackageBinding rootTamperedPackage checkedPackage := by
  apply differentCheckBreaksExactBinding
  decide

private def reorderedRuleBasis : DerivationBasisCandidate indexA := {
  roots := finiteBasis.roots
  rules := [ruleBCD, ruleAB, ruleAC]
}

private def reorderedRulePackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := reorderedRuleBasis
    certificate := sharedDagCertificate
    target := claimD
    auxiliary := checkedPackage.decoded.auxiliary
  }

theorem changed_rule_order_breaks_exact_binding :
    ¬ ExactPackageBinding reorderedRulePackage checkedPackage := by
  apply differentCheckBreaksExactBinding
  decide

private def removedRuleBasis : DerivationBasisCandidate indexA := {
  roots := finiteBasis.roots
  rules := [ruleAB, ruleBCD]
}

private def removedRulePackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := removedRuleBasis
    certificate := sharedDagCertificate
    target := claimD
    auxiliary := checkedPackage.decoded.auxiliary
  }

theorem changed_rule_content_breaks_exact_binding :
    ¬ ExactPackageBinding removedRulePackage checkedPackage := by
  apply differentCheckBreaksExactBinding
  decide

private def changedPremiseRule : GroundRuleCandidate indexA := {
  premises := []
  conclusion := claimB
}

private def changedPremiseBasis : DerivationBasisCandidate indexA := {
  roots := finiteBasis.roots
  rules := [changedPremiseRule, ruleAC, ruleBCD]
}

private def changedPremisePackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := changedPremiseBasis
    certificate := sharedDagCertificate
    target := claimD
    auxiliary := checkedPackage.decoded.auxiliary
  }

theorem changed_rule_premises_break_exact_binding :
    ¬ ExactPackageBinding changedPremisePackage checkedPackage := by
  apply differentCheckBreaksExactBinding
  decide

private def changedConclusionRule : GroundRuleCandidate indexA := {
  premises := [claimA]
  conclusion := claimC
}

private def changedConclusionBasis : DerivationBasisCandidate indexA := {
  roots := finiteBasis.roots
  rules := [changedConclusionRule, ruleAC, ruleBCD]
}

private def changedConclusionPackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := changedConclusionBasis
    certificate := sharedDagCertificate
    target := claimD
    auxiliary := checkedPackage.decoded.auxiliary
  }

theorem changed_rule_conclusion_breaks_exact_binding :
    ¬ ExactPackageBinding changedConclusionPackage checkedPackage := by
  apply differentCheckBreaksExactBinding
  decide

private def certificateTamperedPackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := finiteBasis
    certificate := ⟨[
      ⟨claimA, .root 1⟩,
      ⟨claimB, .apply 0 [0]⟩,
      ⟨claimC, .apply 1 [0]⟩,
      ⟨claimD, .apply 2 [1, 2]⟩
    ]⟩
    target := claimD
    auxiliary := checkedPackage.decoded.auxiliary
  }

theorem changed_certificate_breaks_exact_binding :
    ¬ ExactPackageBinding certificateTamperedPackage checkedPackage := by
  apply differentCheckBreaksExactBinding
  decide

private def targetTamperedPackage : CanonicalPackageCandidate :=
  packageAt indexA checkedPackage.canonicalBytes {
    basis := finiteBasis
    certificate := sharedDagCertificate
    target := claimC
    auxiliary := checkedPackage.decoded.auxiliary
  }

theorem changed_target_breaks_exact_binding :
    ¬ ExactPackageBinding targetTamperedPackage checkedPackage := by
  apply differentCheckBreaksExactBinding
  decide

private def transplantedCertificatePackage : CanonicalPackageCandidate :=
  packageAt indexA (bytes [102, 1]) checkedPackage.decoded

theorem cross_package_certificate_transplant_breaks_exact_binding :
    packageCheck transplantedCertificatePackage = true ∧
      ¬ ExactPackageBinding transplantedCertificatePackage checkedPackage := by
  constructor
  · decide
  · intro bound
    change transplantedCertificatePackage = checkedPackage at bound
    have bytesEqual :=
      congrArg CanonicalPackageCandidate.canonicalBytes bound
    have bytesDifferent :
        transplantedCertificatePackage.canonicalBytes ≠
          checkedPackage.canonicalBytes := by
      decide
    exact bytesDifferent bytesEqual

private def selfDeclaredRootBasis : DerivationBasisCandidate indexA := {
  roots := [allEqualContractClaim]
  rules := []
}

private def selfDeclaredRootCertificate : DerivationCertificate indexA :=
  ⟨[⟨allEqualContractClaim, .root 0⟩]⟩

private def selfDeclaredRootPackage : CanonicalPackageCandidate :=
  packageAt indexA (bytes [120, 1]) {
    basis := selfDeclaredRootBasis
    certificate := selfDeclaredRootCertificate
    target := allEqualContractClaim
    auxiliary := []
  }

theorem self_declared_root_is_relative_but_cannot_create_authority :
    packageCheck selfDeclaredRootPackage = true ∧
      ¬ ConstitutionalPackageAnchor selfDeclaredRootPackage := by
  exact ⟨by decide, noConstitutionalPackageAnchor selfDeclaredRootPackage⟩

private def nullarySelfRule : GroundRuleCandidate indexA := {
  premises := []
  conclusion := allEqualContractClaim
}

private def nullarySelfRuleBasis : DerivationBasisCandidate indexA := {
  roots := []
  rules := [nullarySelfRule]
}

private def nullarySelfRuleCertificate : DerivationCertificate indexA :=
  ⟨[⟨allEqualContractClaim, .apply 0 []⟩]⟩

private def nullarySelfRulePackage : CanonicalPackageCandidate :=
  packageAt indexA (bytes [120, 2]) {
    basis := nullarySelfRuleBasis
    certificate := nullarySelfRuleCertificate
    target := allEqualContractClaim
    auxiliary := []
  }

theorem nullary_self_rule_is_relative_but_cannot_create_authority :
    packageCheck nullarySelfRulePackage = true ∧
      ¬ ConstitutionalPackageAnchor nullarySelfRulePackage := by
  exact ⟨by decide, noConstitutionalPackageAnchor nullarySelfRulePackage⟩

theorem all_equal_context_membership_cannot_create_package_authority :
    ContextCandidate.ContainsRepresentation selfAuthorizingContext
        allEqualContractClaim ∧
      ¬ ConstitutionalPackageAnchor nullarySelfRulePackage := by
  exact ⟨raw_all_equal_context_is_present,
    noConstitutionalPackageAnchor nullarySelfRulePackage⟩

theorem bare_relative_derivability_cannot_create_package_authority :
    DerivableFrom selfDeclaredRootBasis allEqualContractClaim ∧
      ¬ ConstitutionalPackageAnchor selfDeclaredRootPackage := by
  exact ⟨DerivationCertificate.checkRelative_sound selfDeclaredRootBasis
      selfDeclaredRootCertificate allEqualContractClaim (by decide),
    noConstitutionalPackageAnchor selfDeclaredRootPackage⟩

end Examples

end ClauseCore
