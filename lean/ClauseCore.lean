/-!
# Clause Core: candidate Term representation

This file implements the first constitutional boundary from
`clause:docs/foundation.md`: a finite recursive Term is either an Atom or exactly
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

/-! ## Canonical v0 package codec -/

/-- A package is either the literal constitutional root or a successor carrying
the exact predecessor package bytes and a certificate proposed as predecessor
authorization. The predecessor bytes remain opaque during physical decoding;
constitutional checking decodes and binds them separately. -/
inductive PackageLineage (index : StructuralIndex) where
  | root
  | successor
      (predecessorBytes : CanonicalBytes)
      (authorization : DerivationCertificate index)

/-- All six decoded v0 package sections after the structural index has fixed
the type of every contained Term. Auxiliary blobs are ordered and opaque. -/
structure DecodedPackageSections (index : StructuralIndex) where
  lineage : PackageLineage index
  basis : DerivationBasisCandidate index
  certificate : DerivationCertificate index
  target : JudgmentClaim index
  auxiliary : List CanonicalBytes

/-- One exact decoded package paired with the raw bytes supplied to the strict
decoder. Construction alone grants neither canonicality nor authority. -/
structure CanonicalPackageCandidate where
  canonicalBytes : CanonicalBytes
  index : StructuralIndex
  decoded : DecodedPackageSections index

/-- The dependent decoded value before raw bytes are attached. -/
structure PackageValue where
  index : StructuralIndex
  decoded : DecodedPackageSections index

namespace Codec

/-- `CLCP`, followed by the v0 wire version byte. -/
def header : List UInt8 := [67, 76, 67, 80, 1]

def indexFrameTag : UInt8 := 1
def lineageFrameTag : UInt8 := 2
def basisFrameTag : UInt8 := 3
def certificateFrameTag : UInt8 := 4
def targetFrameTag : UInt8 := 5
def auxiliaryFrameTag : UInt8 := 6

def u32Maximum : Nat := 4294967295

/-- A total, overflow-rejecting unsigned 32-bit big-endian encoder. -/
def encodeU32 (value : Nat) : Option (List UInt8) :=
  if value ≤ u32Maximum then
    some [
      UInt8.ofNat (value / 16777216),
      UInt8.ofNat ((value / 65536) % 256),
      UInt8.ofNat ((value / 256) % 256),
      UInt8.ofNat (value % 256)
    ]
  else
    none

abbrev Decoder (α : Type) :=
  List UInt8 → Option (α × List UInt8)

private def decodeByte : Decoder UInt8
  | [] => none
  | value :: remaining => some (value, remaining)

/-- Decode exactly four big-endian bytes into an unbounded Lean natural, so
host arithmetic cannot wrap a wire length or count. -/
def decodeU32 : Decoder Nat
  | first :: second :: third :: fourth :: remaining =>
      some (
        first.toNat * 16777216 +
          second.toNat * 65536 +
          third.toNat * 256 +
          fourth.toNat,
        remaining)
  | _ => none

private def takeExact (count : Nat) : Decoder (List UInt8) := fun input =>
  let taken := input.take count
  if taken.length = count then
    some (taken, input.drop count)
  else
    none

private def encodeBlob (blob : CanonicalBytes) : Option (List UInt8) := do
  let lengthBytes ← encodeU32 blob.data.length
  pure (lengthBytes ++ blob.data)

private def decodeBlob : Decoder CanonicalBytes := fun input => do
  let (length, afterLength) ← decodeU32 input
  let (data, remaining) ← takeExact length afterLength
  pure (⟨data⟩, remaining)

def encodeSequence (encoder : α → Option (List UInt8)) :
    List α → Option (List UInt8)
  | [] => some []
  | value :: remaining => do
      let encodedValue ← encoder value
      let encodedRemaining ← encodeSequence encoder remaining
      pure (encodedValue ++ encodedRemaining)

private def encodeCounted (encoder : α → Option (List UInt8))
    (values : List α) : Option (List UInt8) := do
  let countBytes ← encodeU32 values.length
  let encodedValues ← encodeSequence encoder values
  pure (countBytes ++ encodedValues)

def decodeSequence (decoder : Decoder α) : Nat → Decoder (List α)
  | 0 => fun input => some ([], input)
  | count + 1 => fun input => do
      let (value, afterValue) ← decoder input
      let (remainingValues, remaining) ←
        decodeSequence decoder count afterValue
      pure (value :: remainingValues, remaining)

private def decodeCounted (decoder : Decoder α) : Decoder (List α) :=
  fun input => do
    let (count, afterCount) ← decodeU32 input
    decodeSequence decoder count afterCount

private def encodeFrame (tag : UInt8) (payload : List UInt8) :
    Option (List UInt8) := do
  let lengthBytes ← encodeU32 payload.length
  pure (tag :: (lengthBytes ++ payload))

private def decodeFrame (expectedTag : UInt8) (payloadDecoder : Decoder α) :
    Decoder α := fun input => do
  let (actualTag, afterTag) ← decodeByte input
  if actualTag = expectedTag then
    let (payloadLength, afterLength) ← decodeU32 afterTag
    let (payload, afterFrame) ← takeExact payloadLength afterLength
    let (value, payloadRemaining) ← payloadDecoder payload
    if payloadRemaining = [] then
      pure (value, afterFrame)
    else
      none
  else
    none

def encodeTerm : Term index → Option (List UInt8)
  | .atom atom => do
      let kind ← encodeBlob atom.kind.canonical
      let payload ← encodeBlob atom.canonicalPayload
      let equalityContract ← encodeBlob atom.equalityContract.canonical
      pure (0 :: (kind ++ payload ++ equalityContract))
  | .triple first second third => do
      let encodedFirst ← encodeTerm first
      let encodedSecond ← encodeTerm second
      let encodedThird ← encodeTerm third
      pure (1 :: (encodedFirst ++ encodedSecond ++ encodedThird))

def decodeTermWithFuel (index : StructuralIndex) :
    Nat → Decoder (Term index)
  | 0 => fun _ => none
  | fuel + 1 => fun input => do
      let (tag, afterTag) ← decodeByte input
      if tag = 0 then
        let (kind, afterKind) ← decodeBlob afterTag
        let (payload, afterPayload) ← decodeBlob afterKind
        let (equalityContract, remaining) ← decodeBlob afterPayload
        pure (.atom {
          kind := ⟨kind⟩
          canonicalPayload := payload
          equalityContract := ⟨equalityContract⟩
        }, remaining)
      else if tag = 1 then
        let (first, afterFirst) ← decodeTermWithFuel index fuel afterTag
        let (second, afterSecond) ← decodeTermWithFuel index fuel afterFirst
        let (third, remaining) ← decodeTermWithFuel index fuel afterSecond
        pure (.triple first second third, remaining)
      else
        none

private def decodeTerm (index : StructuralIndex) : Decoder (Term index) :=
  fun input => decodeTermWithFuel index (input.length + 1) input

private def encodeClaim (claim : JudgmentClaim index) :
    Option (List UInt8) := do
  let term ← encodeTerm claim.term
  let typeTerm ← encodeTerm claim.typeTerm
  let mode ← encodeTerm claim.mode
  pure (term ++ typeTerm ++ mode)

private def decodeClaim (index : StructuralIndex) :
    Decoder (JudgmentClaim index) := fun input => do
  let (term, afterTerm) ← decodeTerm index input
  let (typeTerm, afterType) ← decodeTerm index afterTerm
  let (mode, remaining) ← decodeTerm index afterType
  pure ({ term := term, typeTerm := typeTerm, mode := mode }, remaining)

private def encodeRule (rule : GroundRuleCandidate index) :
    Option (List UInt8) := do
  let premises ← encodeCounted encodeClaim rule.premises
  let conclusion ← encodeClaim rule.conclusion
  pure (premises ++ conclusion)

private def decodeRule (index : StructuralIndex) :
    Decoder (GroundRuleCandidate index) := fun input => do
  let (premises, afterPremises) ← decodeCounted (decodeClaim index) input
  let (conclusion, remaining) ← decodeClaim index afterPremises
  pure ({ premises := premises, conclusion := conclusion }, remaining)

private def encodeBasisPayload (basis : DerivationBasisCandidate index) :
    Option (List UInt8) := do
  let roots ← encodeCounted encodeClaim basis.roots
  let rules ← encodeCounted encodeRule basis.rules
  pure (roots ++ rules)

private def decodeBasisPayload (index : StructuralIndex) :
    Decoder (DerivationBasisCandidate index) := fun input => do
  let (roots, afterRoots) ← decodeCounted (decodeClaim index) input
  let (rules, remaining) ← decodeCounted (decodeRule index) afterRoots
  pure ({ roots := roots, rules := rules }, remaining)

private def encodeReason : CertificateReason → Option (List UInt8)
  | .root rootRef => do
      let encodedRef ← encodeU32 rootRef
      pure (0 :: encodedRef)
  | .apply ruleRef premiseRefs => do
      let encodedRuleRef ← encodeU32 ruleRef
      let encodedPremiseRefs ← encodeCounted encodeU32 premiseRefs
      pure (1 :: (encodedRuleRef ++ encodedPremiseRefs))

private def decodeReason : Decoder CertificateReason := fun input => do
  let (tag, afterTag) ← decodeByte input
  if tag = 0 then
    let (rootRef, remaining) ← decodeU32 afterTag
    pure (.root rootRef, remaining)
  else if tag = 1 then
    let (ruleRef, afterRuleRef) ← decodeU32 afterTag
    let (premiseRefs, remaining) ← decodeCounted decodeU32 afterRuleRef
    pure (.apply ruleRef premiseRefs, remaining)
  else
    none

private def encodeCertificateNode (node : CertificateNode index) :
    Option (List UInt8) := do
  let claim ← encodeClaim node.claimed
  let reason ← encodeReason node.reason
  pure (claim ++ reason)

private def decodeCertificateNode (index : StructuralIndex) :
    Decoder (CertificateNode index) := fun input => do
  let (claimed, afterClaim) ← decodeClaim index input
  let (reason, remaining) ← decodeReason afterClaim
  pure ({ claimed := claimed, reason := reason }, remaining)

private def encodeCertificatePayload
    (certificate : DerivationCertificate index) : Option (List UInt8) :=
  encodeCounted encodeCertificateNode certificate.nodes

private def decodeCertificatePayload (index : StructuralIndex) :
    Decoder (DerivationCertificate index) := fun input => do
  let (nodes, remaining) ←
    decodeCounted (decodeCertificateNode index) input
  pure (⟨nodes⟩, remaining)

private def encodeIndexPayload (index : StructuralIndex) :
    Option (List UInt8) := do
  let universeBytes ← encodeBlob index.universeId.canonical
  let semanticsBytes ← encodeBlob index.semanticsId.canonical
  pure (universeBytes ++ semanticsBytes)

private def decodeIndexPayload : Decoder StructuralIndex := fun input => do
  let (universeBytes, afterUniverse) ← decodeBlob input
  let (semanticsBytes, remaining) ← decodeBlob afterUniverse
  pure ({
    universeId := ⟨universeBytes⟩
    semanticsId := ⟨semanticsBytes⟩
  }, remaining)

private def encodeLineagePayload : PackageLineage index → Option (List UInt8)
  | .root => some [0]
  | .successor predecessorBytes authorization => do
      let predecessor ← encodeBlob predecessorBytes
      let certificate ← encodeCertificatePayload authorization
      pure (1 :: (predecessor ++ certificate))

private def decodeLineagePayload (index : StructuralIndex) :
    Decoder (PackageLineage index) := fun input => do
  let (tag, afterTag) ← decodeByte input
  if tag = 0 then
    pure (.root, afterTag)
  else if tag = 1 then
    let (predecessorBytes, afterPredecessor) ← decodeBlob afterTag
    let (authorization, remaining) ←
      decodeCertificatePayload index afterPredecessor
    pure (.successor predecessorBytes authorization, remaining)
  else
    none

private def encodeAuxiliaryPayload (auxiliary : List CanonicalBytes) :
    Option (List UInt8) :=
  encodeCounted encodeBlob auxiliary

private def decodeAuxiliaryPayload : Decoder (List CanonicalBytes) :=
  decodeCounted decodeBlob

/-- The exact encoded BASIS frame, including its tag and U32 payload length.
This is the byte value committed by the canonical basis-admission claim. -/
def encodeBasisFrame (basis : DerivationBasisCandidate index) :
    Option CanonicalBytes := do
  let payload ← encodeBasisPayload basis
  let frame ← encodeFrame basisFrameTag payload
  pure ⟨frame⟩

/-- The exact encoded INDEX frame, including its tag and U32 payload length. -/
def encodeIndexFrame (index : StructuralIndex) : Option CanonicalBytes := do
  let payload ← encodeIndexPayload index
  let frame ← encodeFrame indexFrameTag payload
  pure ⟨frame⟩

/-- Encode all package content in the frozen `INDEX`, `LINEAGE`, `BASIS`,
`CERTIFICATE`, `TARGET`, `AUXILIARY` order. Any field or frame exceeding U32
capacity rejects rather than truncating or wrapping. -/
def encodePackageValue (index : StructuralIndex)
    (sections : DecodedPackageSections index) : Option CanonicalBytes := do
  let indexPayload ← encodeIndexPayload index
  let indexFrame ← encodeFrame indexFrameTag indexPayload
  let lineagePayload ← encodeLineagePayload sections.lineage
  let lineageFrame ← encodeFrame lineageFrameTag lineagePayload
  let basisPayload ← encodeBasisPayload sections.basis
  let basisFrame ← encodeFrame basisFrameTag basisPayload
  let certificatePayload ← encodeCertificatePayload sections.certificate
  let certificateFrame ← encodeFrame certificateFrameTag certificatePayload
  let targetPayload ← encodeClaim sections.target
  let targetFrame ← encodeFrame targetFrameTag targetPayload
  let auxiliaryPayload ← encodeAuxiliaryPayload sections.auxiliary
  let auxiliaryFrame ← encodeFrame auxiliaryFrameTag auxiliaryPayload
  pure ⟨header ++ indexFrame ++ lineageFrame ++ basisFrame ++
    certificateFrame ++ targetFrame ++ auxiliaryFrame⟩

def encodePackage (package : CanonicalPackageCandidate) :
    Option CanonicalBytes :=
  encodePackageValue package.index package.decoded

private def consumeHeader (input : List UInt8) : Option (List UInt8) :=
  match input with
  | 67 :: 76 :: 67 :: 80 :: 1 :: remaining => some remaining
  | _ => none

private def decodePackageValue (raw : CanonicalBytes) : Option PackageValue := do
  let afterHeader ← consumeHeader raw.data
  let (index, afterIndex) ←
    decodeFrame indexFrameTag decodeIndexPayload afterHeader
  let (lineage, afterLineage) ←
    decodeFrame lineageFrameTag (decodeLineagePayload index) afterIndex
  let (basis, afterBasis) ←
    decodeFrame basisFrameTag (decodeBasisPayload index) afterLineage
  let (certificate, afterCertificate) ←
    decodeFrame certificateFrameTag (decodeCertificatePayload index) afterBasis
  let (target, afterTarget) ←
    decodeFrame targetFrameTag (decodeClaim index) afterCertificate
  let (auxiliary, remaining) ←
    decodeFrame auxiliaryFrameTag decodeAuxiliaryPayload afterTarget
  if remaining = [] then
    pure ⟨index, {
      lineage := lineage
      basis := basis
      certificate := certificate
      target := target
      auxiliary := auxiliary
    }⟩
  else
    none

/-- Strict canonical decoding. It consumes every frame payload and EOF, then
re-encodes the dependent value and requires exact equality with the supplied
bytes. The returned record retains those exact raw bytes. -/
def decodePackage (raw : CanonicalBytes) : Option CanonicalPackageCandidate :=
  match decodePackageValue raw with
  | none => none
  | some value =>
      match encodePackageValue value.index value.decoded with
      | none => none
      | some encoded =>
          if encoded = raw then
            some {
              canonicalBytes := raw
              index := value.index
              decoded := value.decoded
            }
          else
            none

/-- Canonical binding is strict decoding of the attached bytes to the exact
dependent package value, not digest agreement or field projection. -/
structure CanonicalBinding (package : CanonicalPackageCandidate) : Prop where
  decoded : decodePackage package.canonicalBytes = some package

/-- A successful decoder result always preserves the input bytes and proves
that the complete dependent value re-encodes to those exact bytes. -/
theorem decodePackage_canonical_binding
    {raw : CanonicalBytes} {package : CanonicalPackageCandidate}
    (accepted : decodePackage raw = some package) :
    package.canonicalBytes = raw ∧ encodePackage package = some raw := by
  unfold decodePackage at accepted
  cases valueResult : decodePackageValue raw with
  | none => simp [valueResult] at accepted
  | some value =>
      cases encodingResult : encodePackageValue value.index value.decoded with
      | none => simp [valueResult, encodingResult] at accepted
      | some encoded =>
          by_cases exactBytes : encoded = raw
          · have packageShape : package = {
                canonicalBytes := raw
                index := value.index
                decoded := value.decoded
              } := by
                simpa [valueResult, encodingResult, exactBytes] using
                  accepted.symm
            subst package
            constructor
            · rfl
            · simpa [encodePackage, exactBytes] using encodingResult
          · simp [valueResult, encodingResult, exactBytes] at accepted

/-- Strict decoding is single-valued, including the exact retained raw bytes. -/
theorem decodePackage_unique
    {raw : CanonicalBytes} {first second : CanonicalPackageCandidate}
    (firstAccepted : decodePackage raw = some first)
    (secondAccepted : decodePackage raw = some second) :
    first = second := by
  rw [firstAccepted] at secondAccepted
  exact Option.some.inj secondAccepted

/-- Canonically bound attached bytes necessarily re-encode from every bound
field to the same exact byte list. -/
theorem binding_reencodes
    {package : CanonicalPackageCandidate}
    (bound : CanonicalBinding package) :
    encodePackage package = some package.canonicalBytes :=
  (decodePackage_canonical_binding bound.decoded).2

end Codec

/-! ## Canonical basis-admission claim -/

private def basisCommitmentAtom (index : StructuralIndex)
    (payloadBytes : CanonicalBytes) : Term index :=
  .atom {
    kind := ⟨⟨[240]⟩⟩
    canonicalPayload := payloadBytes
    equalityContract := ⟨⟨[241]⟩⟩
  }

/-- The ordinary Clause claim authorizing exactly one encoded INDEX frame and
one complete encoded BASIS frame. The term Atom payload is their concatenation;
the INDEX frame's own length makes the split injective. Type and mode remain
ordinary opaque Atoms. -/
def basisAdmissionClaimForFrames (index : StructuralIndex)
    (indexFrame basisFrame : CanonicalBytes) : JudgmentClaim index := {
  term := basisCommitmentAtom index ⟨indexFrame.data ++ basisFrame.data⟩
  typeTerm := basisCommitmentAtom index ⟨[242]⟩
  mode := basisCommitmentAtom index ⟨[243]⟩
}

/-- Construct the unique admission claim only when the next basis has a finite
canonical U32 representation. -/
def basisAdmissionClaim (index : StructuralIndex)
    (basis : DerivationBasisCandidate index) :
    Option (JudgmentClaim index) := do
  let indexFrame ← Codec.encodeIndexFrame index
  let basisFrame ← Codec.encodeBasisFrame basis
  pure (basisAdmissionClaimForFrames index indexFrame basisFrame)

/-- A runtime-scoped claim is used only to state injection across dependent
indexes; it is not a third judgment or Term constructor. -/
structure ScopedJudgmentClaim where
  index : StructuralIndex
  claim : JudgmentClaim index

private def extractedBasisAdmissionPayload
    (claim : ScopedJudgmentClaim) : Option CanonicalBytes :=
  match claim.claim.term with
  | .atom value => some value.canonicalPayload
  | _ => none

def scopedBasisAdmissionClaim (index : StructuralIndex)
    (indexFrame basisFrame : CanonicalBytes) : ScopedJudgmentClaim :=
  ⟨index, basisAdmissionClaimForFrames index indexFrame basisFrame⟩

/-- The ordinary Atom payloads expose exactly the committed index bytes and
BASIS-frame bytes, without invoking host meaning or semantic equality. -/
theorem basisAdmissionClaim_commits_exact_bytes
    (index : StructuralIndex) (indexFrame basisFrame : CanonicalBytes) :
    extractedBasisAdmissionPayload
        (scopedBasisAdmissionClaim index indexFrame basisFrame) =
      some ⟨indexFrame.data ++ basisFrame.data⟩ := by
  rfl

/-- Equality of canonical scoped admission claims forces equality of the exact
structural index and exact encoded BASIS frame. The canonical INDEX encodings
first identify the same self-delimiting prefix; left cancellation then
identifies every remaining BASIS byte. -/
theorem basisAdmissionClaim_injective
    {firstIndex secondIndex : StructuralIndex}
    {firstIndexFrame secondIndexFrame : CanonicalBytes}
    {firstBasisFrame secondBasisFrame : CanonicalBytes}
    (firstIndexExact :
      Codec.encodeIndexFrame firstIndex = some firstIndexFrame)
    (secondIndexExact :
      Codec.encodeIndexFrame secondIndex = some secondIndexFrame)
    (sameClaim :
      scopedBasisAdmissionClaim firstIndex firstIndexFrame firstBasisFrame =
        scopedBasisAdmissionClaim secondIndex secondIndexFrame
          secondBasisFrame) :
    firstIndex = secondIndex ∧ firstBasisFrame = secondBasisFrame := by
  have sameIndex : firstIndex = secondIndex :=
    congrArg ScopedJudgmentClaim.index sameClaim
  constructor
  · exact sameIndex
  · subst secondIndex
    rw [firstIndexExact] at secondIndexExact
    have sameIndexFrame : firstIndexFrame = secondIndexFrame :=
      Option.some.inj secondIndexExact
    subst secondIndexFrame
    have samePayload :=
      congrArg extractedBasisAdmissionPayload sameClaim
    have joined :
        firstIndexFrame.data ++ firstBasisFrame.data =
          firstIndexFrame.data ++ secondBasisFrame.data := by
      simpa [basisAdmissionClaim_commits_exact_bytes] using samePayload
    cases firstBasisFrame with
    | mk firstData =>
        cases secondBasisFrame with
        | mk secondData =>
            simp only [CanonicalBytes.mk.injEq]
            simpa using joined

/-! ## Literal v0 constitution -/

namespace Constitution

def v0Index : StructuralIndex := {
  universeId := ⟨⟨[16]⟩⟩
  semanticsId := ⟨⟨[17]⟩⟩
}

private def opaqueAtom (kind payload equalityContract : List UInt8) :
    Term v0Index :=
  .atom {
    kind := ⟨⟨kind⟩⟩
    canonicalPayload := ⟨payload⟩
    equalityContract := ⟨⟨equalityContract⟩⟩
  }

def bootstrapTarget : JudgmentClaim v0Index := {
  term := opaqueAtom [32] [64] [48]
  typeTerm := opaqueAtom [33] [65] [49]
  mode := opaqueAtom [34] [66] [50]
}

def successorTarget : JudgmentClaim v0Index := {
  term := opaqueAtom [32] [80] [48]
  typeTerm := opaqueAtom [33] [65] [49]
  mode := opaqueAtom [34] [66] [50]
}

def successorBasis : DerivationBasisCandidate v0Index := {
  roots := [successorTarget]
  rules := []
}

def v0IndexFrame : CanonicalBytes := ⟨[
  1, 0, 0, 0, 10,
  0, 0, 0, 1, 16,
  0, 0, 0, 1, 17
]⟩

def successorBasisFrame : CanonicalBytes := ⟨[
  3, 0, 0, 0, 56,
  0, 0, 0, 1,
  0, 0, 0, 0, 1, 32, 0, 0, 0, 1, 80, 0, 0, 0, 1, 48,
  0, 0, 0, 0, 1, 33, 0, 0, 0, 1, 65, 0, 0, 0, 1, 49,
  0, 0, 0, 0, 1, 34, 0, 0, 0, 1, 66, 0, 0, 0, 1, 50,
  0, 0, 0, 0
]⟩

def successorAdmissionClaim : JudgmentClaim v0Index :=
  basisAdmissionClaimForFrames v0Index v0IndexFrame successorBasisFrame

def bootstrapBasis : DerivationBasisCandidate v0Index := {
  roots := [bootstrapTarget, successorAdmissionClaim]
  rules := []
}

def bootstrapCertificate : DerivationCertificate v0Index :=
  ⟨[⟨bootstrapTarget, .root 0⟩]⟩

def successorAuthorization : DerivationCertificate v0Index :=
  ⟨[⟨successorAdmissionClaim, .root 1⟩]⟩

def successorCertificate : DerivationCertificate v0Index :=
  ⟨[⟨successorTarget, .root 0⟩]⟩

def bootstrapSections : DecodedPackageSections v0Index := {
    lineage := .root
    basis := bootstrapBasis
    certificate := bootstrapCertificate
    target := bootstrapTarget
    auxiliary := []
}

/- Exact 334-byte `clause:test-vectors/canonical-package/positive/bootstrap.hex`
content from the normative corpus. -/
def bootstrapBytes : CanonicalBytes := ⟨[
  0x43, 0x4c, 0x43, 0x50, 0x01, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00,
  0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x01, 0x11, 0x02, 0x00, 0x00, 0x00,
  0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0xb3, 0x00, 0x00, 0x00, 0x02, 0x00,
  0x00, 0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x40, 0x00, 0x00,
  0x00, 0x01, 0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x21, 0x00, 0x00, 0x00,
  0x01, 0x41, 0x00, 0x00, 0x00, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x01,
  0x22, 0x00, 0x00, 0x00, 0x01, 0x42, 0x00, 0x00, 0x00, 0x01, 0x32, 0x00,
  0x00, 0x00, 0x00, 0x01, 0xf0, 0x00, 0x00, 0x00, 0x4c, 0x01, 0x00, 0x00,
  0x00, 0x0a, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x01, 0x11,
  0x03, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
  0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x50, 0x00, 0x00, 0x00, 0x01,
  0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x21, 0x00, 0x00, 0x00, 0x01, 0x41,
  0x00, 0x00, 0x00, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x01, 0x22, 0x00,
  0x00, 0x00, 0x01, 0x42, 0x00, 0x00, 0x00, 0x01, 0x32, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x01, 0xf1, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf0,
  0x00, 0x00, 0x00, 0x01, 0xf2, 0x00, 0x00, 0x00, 0x01, 0xf1, 0x00, 0x00,
  0x00, 0x00, 0x01, 0xf0, 0x00, 0x00, 0x00, 0x01, 0xf3, 0x00, 0x00, 0x00,
  0x01, 0xf1, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x39, 0x00,
  0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00,
  0x01, 0x40, 0x00, 0x00, 0x00, 0x01, 0x30, 0x00, 0x00, 0x00, 0x00, 0x01,
  0x21, 0x00, 0x00, 0x00, 0x01, 0x41, 0x00, 0x00, 0x00, 0x01, 0x31, 0x00,
  0x00, 0x00, 0x00, 0x01, 0x22, 0x00, 0x00, 0x00, 0x01, 0x42, 0x00, 0x00,
  0x00, 0x01, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
  0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x40,
  0x00, 0x00, 0x00, 0x01, 0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x21, 0x00,
  0x00, 0x00, 0x01, 0x41, 0x00, 0x00, 0x00, 0x01, 0x31, 0x00, 0x00, 0x00,
  0x00, 0x01, 0x22, 0x00, 0x00, 0x00, 0x01, 0x42, 0x00, 0x00, 0x00, 0x01,
  0x32, 0x06, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00
]⟩

def successorSections : DecodedPackageSections v0Index := {
    lineage := .successor bootstrapBytes successorAuthorization
    basis := successorBasis
    certificate := successorCertificate
    target := successorTarget
    auxiliary := []
}

/- Exact 681-byte `clause:test-vectors/canonical-package/positive/successor.hex`
content, factored only at the exact embedded 334-byte predecessor blob
boundary. -/
def successorBytes : CanonicalBytes := ⟨[
  0x43, 0x4c, 0x43, 0x50, 0x01, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00,
  0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x01, 0x11, 0x02, 0x00, 0x00, 0x01,
  0xd7, 0x01, 0x00, 0x00, 0x01, 0x4e
] ++ bootstrapBytes.data ++ [
  0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf0, 0x00, 0x00,
  0x00, 0x4c, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x01, 0x10,
  0x00, 0x00, 0x00, 0x01, 0x11, 0x03, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00,
  0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01,
  0x50, 0x00, 0x00, 0x00, 0x01, 0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x21,
  0x00, 0x00, 0x00, 0x01, 0x41, 0x00, 0x00, 0x00, 0x01, 0x31, 0x00, 0x00,
  0x00, 0x00, 0x01, 0x22, 0x00, 0x00, 0x00, 0x01, 0x42, 0x00, 0x00, 0x00,
  0x01, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf1, 0x00,
  0x00, 0x00, 0x00, 0x01, 0xf0, 0x00, 0x00, 0x00, 0x01, 0xf2, 0x00, 0x00,
  0x00, 0x01, 0xf1, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf0, 0x00, 0x00, 0x00,
  0x01, 0xf3, 0x00, 0x00, 0x00, 0x01, 0xf1, 0x00, 0x00, 0x00, 0x00, 0x01,
  0x03, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
  0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x50, 0x00, 0x00, 0x00, 0x01,
  0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x21, 0x00, 0x00, 0x00, 0x01, 0x41,
  0x00, 0x00, 0x00, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x01, 0x22, 0x00,
  0x00, 0x00, 0x01, 0x42, 0x00, 0x00, 0x00, 0x01, 0x32, 0x00, 0x00, 0x00,
  0x00, 0x04, 0x00, 0x00, 0x00, 0x39, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
  0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x50, 0x00, 0x00, 0x00,
  0x01, 0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x21, 0x00, 0x00, 0x00, 0x01,
  0x41, 0x00, 0x00, 0x00, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x01, 0x22,
  0x00, 0x00, 0x00, 0x01, 0x42, 0x00, 0x00, 0x00, 0x01, 0x32, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00,
  0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x50, 0x00, 0x00, 0x00, 0x01, 0x30,
  0x00, 0x00, 0x00, 0x00, 0x01, 0x21, 0x00, 0x00, 0x00, 0x01, 0x41, 0x00,
  0x00, 0x00, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x01, 0x22, 0x00, 0x00,
  0x00, 0x01, 0x42, 0x00, 0x00, 0x00, 0x01, 0x32, 0x06, 0x00, 0x00, 0x00,
  0x04, 0x00, 0x00, 0x00, 0x00
]⟩

def bootstrapPackage : CanonicalPackageCandidate := {
  canonicalBytes := bootstrapBytes
  index := v0Index
  decoded := bootstrapSections
}

def successorPackage : CanonicalPackageCandidate := {
  canonicalBytes := successorBytes
  index := v0Index
  decoded := successorSections
}

theorem successor_basis_frame_is_exact :
    Codec.encodeBasisFrame successorBasis = some successorBasisFrame := by
  decide

theorem v0_index_frame_is_exact :
    Codec.encodeIndexFrame v0Index = some v0IndexFrame := by
  decide

theorem successor_admission_claim_is_exact :
    basisAdmissionClaim v0Index successorBasis =
      some successorAdmissionClaim := by
  rfl

set_option maxRecDepth 100000 in
theorem bootstrap_decodes_exactly :
    Codec.decodePackage bootstrapBytes = some bootstrapPackage := by
  rfl

set_option maxRecDepth 100000 in
theorem successor_decodes_exactly :
    Codec.decodePackage successorBytes = some successorPackage := by
  rfl

set_option maxRecDepth 100000 in
theorem bootstrap_is_canonically_bound :
    Codec.CanonicalBinding bootstrapPackage :=
  ⟨bootstrap_decodes_exactly⟩

set_option maxRecDepth 100000 in
theorem successor_is_canonically_bound :
    Codec.CanonicalBinding successorPackage :=
  ⟨successor_decodes_exactly⟩

theorem bootstrap_certificate_checks :
    DerivationCertificate.checkRelative bootstrapBasis bootstrapCertificate
      bootstrapTarget = true := by
  decide

theorem successor_authorization_checks_under_predecessor :
    DerivationCertificate.checkRelative bootstrapBasis successorAuthorization
      successorAdmissionClaim = true := by
  decide

theorem successor_certificate_checks :
    DerivationCertificate.checkRelative successorBasis successorCertificate
      successorTarget = true := by
  decide

end Constitution

/-! ## Predecessor-authorized package authority -/

def checkPackage (package : CanonicalPackageCandidate) : Bool :=
  DerivationCertificate.checkRelative package.decoded.basis
    package.decoded.certificate package.decoded.target

/-- Check only the lineage authorization certificate against the exact prior
basis and the canonical admission claim for the proposed next BASIS frame.
The successor basis and packaged target certificate are deliberately absent. -/
def checkLineageAuthorization
    (prior next : CanonicalPackageCandidate) : Bool :=
  match next.decoded.lineage with
  | .root => false
  | .successor _ authorization =>
      if sameIndex : prior.index = next.index then
        match basisAdmissionClaim next.index next.decoded.basis with
        | none => false
        | some requested =>
            DerivationCertificate.checkRelative
              (sameIndex ▸ prior.decoded.basis) authorization requested
      else
        false

/-- The whole v0 authority relation has exactly two introduction paths: the
one closed literal bootstrap package, and a canonically decoded successor
authorized by an already authoritative exact predecessor package. -/
inductive AuthoritativePackage : CanonicalPackageCandidate → Prop where
  | literalBootstrap : AuthoritativePackage Constitution.bootstrapPackage
  | successor
      {prior next : CanonicalPackageCandidate}
      (authorization : DerivationCertificate next.index)
      (priorAuthority : AuthoritativePackage prior)
      (predecessorBound : Codec.CanonicalBinding prior)
      (nextBound : Codec.CanonicalBinding next)
      (priorIndex : prior.index = Constitution.v0Index)
      (nextIndex : next.index = Constitution.v0Index)
      (lineageExact : next.decoded.lineage =
        .successor prior.canonicalBytes authorization)
      (authorizationAccepted :
        checkLineageAuthorization prior next = true)
      (packageAccepted : checkPackage next = true) :
      AuthoritativePackage next

/-- The literal introduction path can authorize only the one fully closed
bootstrap record. -/
theorem literal_bootstrap_unique
    {package : CanonicalPackageCandidate}
    (authority : AuthoritativePackage package)
    (rootLineage : package.decoded.lineage = .root) :
    package = Constitution.bootstrapPackage := by
  cases authority with
  | literalBootstrap => rfl
  | successor authorization _ _ _ _ _ lineageExact _ _ =>
      rw [lineageExact] at rootLineage
      cases rootLineage

/-- Authority inversion: a nonliteral authoritative package necessarily names
an authoritative, canonically decoded, exact-byte predecessor and carries the
accepted predecessor-basis authorization certificate. -/
theorem authoritative_nonbootstrap_has_predecessor
    {package : CanonicalPackageCandidate}
    (authority : AuthoritativePackage package)
    (notBootstrap : package ≠ Constitution.bootstrapPackage) :
    ∃ (prior : CanonicalPackageCandidate)
      (authorization : DerivationCertificate package.index),
      AuthoritativePackage prior ∧
      Codec.CanonicalBinding prior ∧
      Codec.CanonicalBinding package ∧
      prior.index = Constitution.v0Index ∧
      package.index = Constitution.v0Index ∧
      package.decoded.lineage =
        .successor prior.canonicalBytes authorization ∧
      checkLineageAuthorization prior package = true ∧
      checkPackage package = true := by
  cases authority with
  | literalBootstrap => exact (notBootstrap rfl).elim
  | successor authorization priorAuthority predecessorBound nextBound
      priorIndex nextIndex lineageExact authorizationAccepted packageAccepted =>
      exact ⟨_, authorization, priorAuthority, predecessorBound, nextBound,
        priorIndex, nextIndex, lineageExact, authorizationAccepted,
        packageAccepted⟩

/-- Every authoritative package is strictly bound to its complete decoded
value and exact raw bytes. -/
theorem authoritative_is_canonically_bound
    {package : CanonicalPackageCandidate}
    (authority : AuthoritativePackage package) :
    Codec.CanonicalBinding package := by
  cases authority with
  | literalBootstrap => exact Constitution.bootstrap_is_canonically_bound
  | successor _ _ _ nextBound _ _ _ _ _ => exact nextBound

/-- Every authoritative package remains at the one literal v0 structural
index; a successor cannot silently cross a universe or semantics epoch. -/
theorem authoritative_has_v0_index
    {package : CanonicalPackageCandidate}
    (authority : AuthoritativePackage package) :
    package.index = Constitution.v0Index := by
  cases authority with
  | literalBootstrap => rfl
  | successor _ _ _ _ _ nextIndex _ _ _ => exact nextIndex

/-- Authority promotes only the packaged target's ordinary derivability from
the exact packaged basis. It does not prove semantic truth or general
Admission. -/
theorem authoritative_is_relatively_derivable
    {package : CanonicalPackageCandidate}
    (authority : AuthoritativePackage package) :
    DerivableFrom package.decoded.basis package.decoded.target := by
  cases authority with
  | literalBootstrap =>
      exact DerivationCertificate.checkRelative_sound
        Constitution.bootstrapBasis Constitution.bootstrapCertificate
        Constitution.bootstrapTarget Constitution.bootstrap_certificate_checks
  | successor _ _ _ _ _ _ _ _ packageAccepted =>
      exact DerivationCertificate.checkRelative_sound
        package.decoded.basis package.decoded.certificate package.decoded.target
        packageAccepted

set_option maxRecDepth 100000 in
/-- The positive bootstrap-to-successor path uses the literal predecessor
bytes, the predecessor's preauthorized exact BASIS-frame claim, and separately
checks the successor's own packaged certificate. -/
theorem literal_successor_is_authoritative :
    AuthoritativePackage Constitution.successorPackage := by
  apply AuthoritativePackage.successor Constitution.successorAuthorization
    AuthoritativePackage.literalBootstrap
    Constitution.bootstrap_is_canonically_bound
    Constitution.successor_is_canonically_bound
  · rfl
  · rfl
  · rfl
  · decide
  · exact Constitution.successor_certificate_checks

end ClauseCore
