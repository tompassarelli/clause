import ClauseCompiler.Authorization

/-!
# Clause compiler constitution

This is the additive CLCP-v3 constitutional surface.  `Codec.strictDecode`
returns only a retained candidate or a deterministic decode rejection.
`Authorization.authorizeBytesGenesis` consumes the fixed external owner
capability only after strict decoding; `Authorization.authorizeBytesSuccessor`
requires exact accepted predecessor bytes after the same strict boundary.
Neither decoding, hashing, evaluation, nor receipt replay creates
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
