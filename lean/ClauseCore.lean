/-!
# Clause Core: candidate Term representation

This file implements the first constitutional boundary from
`docs/foundation.md`: a finite recursive Term is either an Atom or exactly
three Terms. It does not define Clause judgments, Runs, admission, source
syntax, persistence, or language-feature constructors.

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

/-- Representation-only premise lookup. A later generic checker may use this
for exact certificate addressing, but a successful lookup is not a derivation
or admission. -/
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

end Examples

end ClauseCore
