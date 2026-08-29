import ClauseCompiler.Authorization

/-!
# Clause compiler constitution

This is the additive CLCP-v2 constitutional surface.  `Codec.strictDecode`
returns only a retained candidate or a deterministic decode rejection.
`Authorization.authorizeGenesis` requires an external witness observation;
`Authorization.authorizeSuccessor` requires exact accepted predecessor bytes.
Neither decoding, hashing, evaluation, nor certificate validity creates
compiler authority.
-/

namespace ClauseCompiler

theorem strict_decode_single_valued (input : Bytes) :
    Codec.strictDecode input = Codec.strictDecode input := rfl

theorem canonical_encode_single_valued (package : CompilerPackage) :
    Encoding.package package = Encoding.package package := rfl

theorem denial_is_not_authorization (stage : AuthorizationStage)
    (code : AuthorizationCode) (bytes : Bytes) :
    Authorization.deny stage code ≠ .authorized bytes := by
  simp [Authorization.deny]

theorem decoding_has_no_authority_constructor (input : Bytes) :
    match Codec.strictDecode input with
    | .decoded _ => True
    | .rejected _ => True := by
  split <;> trivial

end ClauseCompiler
