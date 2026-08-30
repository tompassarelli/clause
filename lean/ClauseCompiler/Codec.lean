import ClauseCompiler.Encoding

/-!
# Strict CLCP-v3 codec

The decoder retains one absolute cursor into the complete input and one bounded
end for the current frame.  It reports a separate deterministic decode algebra;
no malformed byte string can enter constitutional authorization.
-/

namespace ClauseCompiler.Codec

open ClauseCompiler

structure Cursor where
  input : Bytes
  remaining : Bytes
  position : Nat
  limit : Nat

abbrev Decoder (α : Type) := Cursor → Except DecodeFailure (α × Cursor)

def failure (code : DecodeCode) (offset : Nat) : Decoder α :=
  fun _ => .error { code := code, offset := offset }

def readByte : Decoder UInt8 := fun cursor =>
  if cursor.position ≥ cursor.limit then
    if cursor.limit < cursor.input.length then
      .error { code := .boundedValueOverConsumed, offset := cursor.position }
    else
      .error { code := .truncated, offset := cursor.input.length }
  else
    match cursor.remaining with
    | [] => .error { code := .truncated, offset := cursor.input.length }
    | value :: remaining => .ok (value, {
        cursor with
        remaining := remaining
        position := cursor.position + 1
      })

def readBytes (count : Nat) : Decoder Bytes := fun cursor =>
  let rec loop : Nat → Cursor → Bytes → Except DecodeFailure (Bytes × Cursor)
    | 0, current, reversed => pure (reversed.reverse, current)
    | remaining + 1, current, reversed => do
        let (head, afterHead) ← readByte current
        loop remaining afterHead (head :: reversed)
  loop count cursor []

def expectByte (expected : UInt8) (code : DecodeCode) : Decoder Unit :=
  fun cursor => do
    let offset := cursor.position
    let (actual, after) ← readByte cursor
    if actual = expected then pure ((), after)
    else .error { code := code, offset := offset }

def u32 : Decoder Nat := fun cursor => do
  let (bytes, after) ← readBytes 4 cursor
  match bytes with
  | [a, b, c, d] => pure (a.toNat * 16777216 + b.toNat * 65536 +
      c.toNat * 256 + d.toNat, after)
  | _ => .error { code := .truncated, offset := cursor.input.length }

def u64 : Decoder Nat := fun cursor => do
  let (bytes, after) ← readBytes 8 cursor
  match bytes with
  | [a, b, c, d, e, f, g, h] =>
      pure (a.toNat * 72057594037927936 + b.toNat * 281474976710656 +
        c.toNat * 1099511627776 + d.toNat * 4294967296 +
        e.toNat * 16777216 + f.toNat * 65536 + g.toNat * 256 + h.toNat,
        after)
  | _ => .error { code := .truncated, offset := cursor.input.length }

def fixed32 : Decoder Bytes := readBytes 32

def blob : Decoder Bytes := fun cursor => do
  let (length, afterLength) ← u32 cursor
  readBytes length afterLength

def counted (decoder : Decoder α) : Decoder (List α) := fun cursor => do
  let countOffset := cursor.position
  let (count, afterCount) ← u32 cursor
  if count > afterCount.limit - afterCount.position then
    .error { code := .lengthOrCountOverflow, offset := countOffset }
  else
    let rec loop : Nat → Cursor → List α →
        Except DecodeFailure (List α × Cursor)
      | 0, current, reversed => pure (reversed.reverse, current)
      | remaining + 1, current, reversed => do
          let (head, afterHead) ← decoder current
          loop remaining afterHead (head :: reversed)
    loop count afterCount []

def byteSeq : Decoder (List UInt8) := counted readByte

def decodeTermFuel : Nat → Decoder Term
  | 0 => failure .lengthOrCountOverflow 0
  | fuel + 1 => fun cursor => do
      let tagOffset := cursor.position
      let (tag, afterTag) ← readByte cursor
      match tag with
      | 0x00 =>
          let (kind, afterKind) ← blob afterTag
          let (payload, afterPayload) ← blob afterKind
          let (equality, afterEquality) ← blob afterPayload
          pure (.atom kind payload equality, afterEquality)
      | 0x01 =>
          let (first, afterFirst) ← decodeTermFuel fuel afterTag
          let (second, afterSecond) ← decodeTermFuel fuel afterFirst
          let (third, afterThird) ← decodeTermFuel fuel afterSecond
          pure (.triple first second third, afterThird)
      | _ => .error { code := .unknownSumTag, offset := tagOffset }

def term : Decoder Term := fun cursor =>
  decodeTermFuel (cursor.limit - cursor.position + 1) cursor

def sort : Decoder KSort := fun cursor => do
  let offset := cursor.position
  let (tag, after) ← readByte cursor
  match tag with
  | 0x00 => pure (.bytes, after)
  | 0x01 => pure (.term, after)
  | _ => .error { code := .unknownSumTag, offset := offset }

def decodeExprFuel : Nat → Decoder KExpr
  | 0 => failure .lengthOrCountOverflow 0
  | fuel + 1 => fun cursor => do
      let tagOffset := cursor.position
      let (tag, afterTag) ← readByte cursor
      let nested := decodeExprFuel fuel
      match tag with
      | 0x00 => let (v, c) ← blob afterTag; pure (.bytesLiteral v, c)
      | 0x01 => let (v, c) ← decodeTermFuel fuel afterTag; pure (.termLiteral v, c)
      | 0x02 => let (v, c) ← u32 afterTag; pure (.var v, c)
      | 0x03 =>
          let (a, c1) ← nested afterTag
          let (b, c2) ← nested c1
          let (c, c3) ← nested c2
          pure (.makeAtom a b c, c3)
      | 0x04 =>
          let (a, c1) ← nested afterTag
          let (b, c2) ← nested c1
          let (c, c3) ← nested c2
          pure (.makeTriple a b c, c3)
      | 0x05 =>
          let (a, c1) ← nested afterTag
          let (b, c2) ← nested c1
          pure (.letValue a b, c2)
      | 0x06 =>
          let (a, c1) ← nested afterTag
          let (b, c2) ← nested c1
          let (c, c3) ← nested c2
          pure (.caseTerm a b c, c3)
      | 0x07 =>
          let (a, c1) ← nested afterTag
          let (b, c2) ← nested c1
          let (c, c3) ← nested c2
          pure (.caseBytes a b c, c3)
      | 0x08 =>
          let (parts, c) ← counted nested afterTag
          pure (.concatBytes (KExprSeq.ofList parts), c)
      | 0x09 =>
          let (a, c1) ← nested afterTag
          let (b, c2) ← nested c1
          let (c, c3) ← nested c2
          let (d, c4) ← nested c3
          pure (.caseBytesEqual a b c d, c4)
      | 0x0a =>
          let (id, c1) ← fixed32 afterTag
          let (arguments, c2) ← counted nested c1
          pure (.call id (KExprSeq.ofList arguments), c2)
      | 0x0b =>
          let (id, c1) ← fixed32 afterTag
          let (arguments, c2) ← counted nested c1
          pure (.request id (KExprSeq.ofList arguments), c2)
      | _ => .error { code := .unknownSumTag, offset := tagOffset }

def expr : Decoder KExpr := fun cursor =>
  decodeExprFuel (cursor.limit - cursor.position + 1) cursor

def namedSignature : Decoder NamedSignature := fun cursor => do
  let (tag, c1) ← readByte cursor
  let (signature, c2) ← blob c1
  pure ({ tag := tag, signature := signature }, c2)

def ruleSignature : Decoder RuleSignature := fun cursor => do
  let (tag, c1) ← readByte cursor
  let (premise, c2) ← readByte c1
  let (clause, c3) ← blob c2
  pure ({ tag := tag, premisePolicy := premise, clause := clause }, c3)

def physicalOperation : Decoder PhysicalOperation := fun cursor => do
  let (operationId, c1) ← fixed32 cursor
  let (arguments, c2) ← counted sort c1
  let (result, c3) ← sort c2
  pure ({
    operationId := operationId
    arguments := arguments
    result := result
  }, c3)

def physicalProfile : Decoder PhysicalProfile := fun cursor => do
  let (version, c1) ← readByte cursor
  let (policy, c2) ← readByte c1
  let (operations, c3) ← counted physicalOperation c2
  pure ({
    profileVersion := version
    observationPolicy := policy
    operations := operations
  }, c3)

def manifest : Decoder CoreManifest := fun cursor => do
  let (manifestVersion, c1) ← readByte cursor
  let (frameTags, c2) ← byteSeq c1
  let (termTags, c3) ← byteSeq c2
  let (sortTags, c4) ← byteSeq c3
  let (expressionForms, c5) ← counted namedSignature c4
  let (abiForms, c6) ← counted namedSignature c5
  let (premisePolicyTags, c7) ← byteSeq c6
  let (lineageTags, c8) ← byteSeq c7
  let (nominalDeclarationTags, c9) ← byteSeq c8
  let (compilerEvidenceTags, c10) ← byteSeq c9
  let (valueTags, c11) ← byteSeq c10
  let (decodeVerdictTags, c12) ← byteSeq c11
  let (decodeCodeTags, c13) ← byteSeq c12
  let (authorizationStageTags, c14) ← byteSeq c13
  let (authorizationCodeTags, c15) ← byteSeq c14
  let (staticRules, c16) ← counted ruleSignature c15
  let (evaluationRules, c17) ← counted ruleSignature c16
  let (receiptFormatVersion, c18) ← readByte c17
  let (receiptSignature, c19) ← blob c18
  let (contractClauses, c20) ← counted blob c19
  let (profile, c21) ← physicalProfile c20
  pure ({
    manifestVersion := manifestVersion
    frameTags := frameTags
    termTags := termTags
    sortTags := sortTags
    expressionForms := expressionForms
    abiForms := abiForms
    premisePolicyTags := premisePolicyTags
    lineageTags := lineageTags
    nominalDeclarationTags := nominalDeclarationTags
    compilerEvidenceTags := compilerEvidenceTags
    valueTags := valueTags
    decodeVerdictTags := decodeVerdictTags
    decodeCodeTags := decodeCodeTags
    authorizationStageTags := authorizationStageTags
    authorizationCodeTags := authorizationCodeTags
    staticRules := staticRules
    evaluationRules := evaluationRules
    receiptFormatVersion := receiptFormatVersion
    receiptSignature := receiptSignature
    contractClauses := contractClauses
    physicalProfile := profile
  }, c21)

def lineage : Decoder CompilerLineage := fun cursor => do
  let offset := cursor.position
  let (tag, c1) ← readByte cursor
  match tag with
  | 0x00 => pure (.genesis, c1)
  | 0x01 =>
      let (locator, c2) ← fixed32 c1
      let (change, c3) ← fixed32 c2
      pure (.successor locator change, c3)
  | _ => .error { code := .unknownSumTag, offset := offset }

def nominalDeclaration : Decoder NominalDeclaration := fun cursor => do
  let offset := cursor.position
  let (tag, c1) ← readByte cursor
  match tag with
  | 0x00 =>
      let (domain, c2) ← fixed32 c1
      let (id, c3) ← fixed32 c2
      pure (.seed domain id, c3)
  | 0x01 =>
      let (domain, c2) ← fixed32 c1
      let (id, c3) ← fixed32 c2
      let (revision, c4) ← fixed32 c3
      pure (.retainedSeed domain id revision, c4)
  | 0x02 =>
      let (domain, c2) ← fixed32 c1
      let (id, c3) ← fixed32 c2
      let (changeDomain, c4) ← fixed32 c3
      let (changeId, c5) ← fixed32 c4
      let (producerDomain, c6) ← fixed32 c5
      let (producerId, c7) ← fixed32 c6
      let (slot, c8) ← u64 c7
      pure (.allocated domain id changeDomain changeId producerDomain producerId slot, c8)
  | _ => .error { code := .unknownSumTag, offset := offset }

def interface : Decoder CompilerInterface := fun cursor => do
  let (compile, c1) ← fixed32 cursor
  let (admit, c2) ← fixed32 c1
  pure ({ compile := compile, admitPropose := admit }, c2)

def definition : Decoder Definition := fun cursor => do
  let (id, c1) ← fixed32 cursor
  let (arguments, c2) ← counted sort c1
  let (result, c3) ← sort c2
  let (body, c4) ← expr c3
  pure ({
    id := id
    arguments := arguments
    result := result
    body := body
  }, c4)

def subject : Decoder CompilerSubject := fun cursor => do
  let (lineageValue, c1) ← lineage cursor
  let (nominals, c2) ← counted nominalDeclaration c1
  let (interfaceValue, c3) ← interface c2
  let (program, c4) ← counted definition c3
  let (request, c5) ← term c4
  pure ({
    lineage := lineageValue
    nominalDeclarations := nominals
    interface := interfaceValue
    program := program
    buildRequest := request
  }, c5)

def kvalue : Decoder KValue := fun cursor => do
  let offset := cursor.position
  let (tag, c1) ← readByte cursor
  match tag with
  | 0x00 => let (v, c2) ← blob c1; pure (.bytes v, c2)
  | 0x01 => let (v, c2) ← term c1; pure (.term v, c2)
  | _ => .error { code := .unknownSumTag, offset := offset }

def evalReceipt : Decoder EvalReceipt := fun cursor => do
  let formatOffset := cursor.position
  let (formatVersion, c1) ← readByte cursor
  if formatVersion ≠ 0x00 then
    .error { code := .unknownSumTag, offset := formatOffset }
  else
    let (expectedValueHash, c2) ← fixed32 c1
    let (expectedRemainingFuel, c3) ← u64 c2
    let (expectedObservationsHash, c4) ← fixed32 c3
    pure ({
      formatVersion := formatVersion
      expectedValueHash := expectedValueHash
      expectedRemainingFuel := expectedRemainingFuel
      expectedObservationsHash := expectedObservationsHash
    }, c4)

def evidence : Decoder CompilerEvidence := fun cursor => do
  let offset := cursor.position
  let (tag, c1) ← readByte cursor
  match tag with
  | 0x00 => pure (.genesis, c1)
  | 0x01 =>
      let (compile, c2) ← evalReceipt c1
      let (admission, c3) ← evalReceipt c2
      pure (.successor compile admission, c3)
  | _ => .error { code := .unknownSumTag, offset := offset }

def boundedFrame (expectedTag : UInt8) (decoder : Decoder α) :
    Decoder (α × Bytes) := fun cursor => do
  let tagOffset := cursor.position
  if cursor.position ≥ cursor.input.length then
    .error { code := .frameTagOrderOrCount, offset := cursor.position }
  else
    let (tag, c1) ← readByte cursor
    if tag ≠ expectedTag then
      .error { code := .frameTagOrderOrCount, offset := tagOffset }
    else
      let lengthOffset := c1.position
      let (length, c2) ← u32 c1
      if length > Encoding.u32Maximum then
        .error { code := .lengthOrCountOverflow, offset := lengthOffset }
      else
        let payloadStart := c2.position
        let payloadEnd := payloadStart + length
        let bounded := { c2 with limit := payloadEnd }
        let (value, afterValue) ← decoder bounded
        if afterValue.position < payloadEnd then
          .error { code := .boundedValueUnderConsumed, offset := payloadEnd }
        else if afterValue.position > payloadEnd then
          .error { code := .boundedValueOverConsumed, offset := payloadEnd }
        else
          let payload := c2.remaining.take length
          pure ((value, payload), {
            c2 with
            remaining := afterValue.remaining
            position := payloadEnd
            limit := cursor.limit
          })

def consumeMagic : Decoder Unit := fun cursor => do
  let expected : Bytes := [0x43, 0x4c, 0x43, 0x50]
  let rec loop : Bytes → Cursor → Except DecodeFailure (Unit × Cursor)
    | [], current => pure ((), current)
    | byte :: remaining, current => do
        let ((), after) ← expectByte byte .wrongMagic current
        loop remaining after
  loop expected cursor

def strictDecode (input : Bytes) : DecodeVerdict :=
  let start : Cursor := {
    input := input
    remaining := input
    position := 0
    limit := input.length
  }
  match (do
    let ((), c1) ← consumeMagic start
    let ((), c2) ← expectByte 0x03 .unknownVersion c1
    let ((manifestValue, manifestBytes), c3) ← boundedFrame 0x01 manifest c2
    let ((subjectValue, subjectBytes), c4) ← boundedFrame 0x02 subject c3
    let ((evidenceValue, evidenceBytes), c5) ← boundedFrame 0x03 evidence c4
    if c5.position = input.length then
      let decoded : DecodedPackage := {
        exactInput := input
        exactManifestPayload := manifestBytes
        exactSubjectPayload := subjectBytes
        exactEvidencePayload := evidenceBytes
        package := {
          manifest := manifestValue
          subject := subjectValue
          evidence := evidenceValue
        }
      }
      pure decoded
    else
      let next := c5.remaining.head?.getD 0xff
      if next = 0x01 || next = 0x02 || next = 0x03 then
        .error { code := .frameTagOrderOrCount, offset := c5.position }
      else
        .error { code := .trailingBytes, offset := c5.position }) with
  | .ok decoded => .decoded decoded
  | .error error => .rejected error

def reencodesExactly (decoded : DecodedPackage) : Bool :=
  Encoding.package decoded.package = some decoded.exactInput

def rejection? : DecodeVerdict → Option DecodeFailure
  | .decoded _ => none
  | .rejected failure => some failure

theorem wrong_magic_vector : rejection? (strictDecode [0x00]) =
    some { code := .wrongMagic, offset := 0 } := by
  decide

theorem unknown_version_vector :
    rejection? (strictDecode [0x43, 0x4c, 0x43, 0x50, 0x02]) =
      some { code := .unknownVersion, offset := 4 } := by
  decide

theorem missing_frames_vector :
    rejection? (strictDecode [0x43, 0x4c, 0x43, 0x50, 0x03]) =
      some { code := .frameTagOrderOrCount, offset := 5 } := by
  decide

namespace Regression

def compactReceipt : EvalReceipt := Encoding.Regression.compactReceipt

def compactReceiptBytes : Bytes :=
  (Encoding.evalReceipt compactReceipt).getD []

def decodeReceiptExactly (input : Bytes) : Except DecodeFailure EvalReceipt := do
  let (receipt, cursor) ← evalReceipt {
    input := input
    remaining := input
    position := 0
    limit := input.length
  }
  if cursor.position = input.length then
    pure receipt
  else
    .error { code := .trailingBytes, offset := cursor.position }

def decodedReceipt? (input : Bytes) : Option EvalReceipt :=
  match decodeReceiptExactly input with
  | .ok receipt => some receipt
  | .error _ => none

def receiptFailure? (input : Bytes) : Option DecodeFailure :=
  match decodeReceiptExactly input with
  | .ok _ => none
  | .error failure => some failure

theorem compactReceiptRoundTrips :
    compactReceiptBytes.length = 73 ∧
      decodedReceipt? compactReceiptBytes = some compactReceipt := by
  set_option maxRecDepth 100000 in
    decide

theorem compactReceiptUnknownFormatRejects :
    receiptFailure? (0x01 :: compactReceiptBytes.drop 1) =
      some { code := .unknownSumTag, offset := 0 } := by
  decide

theorem compactReceiptTruncatedValueHashRejects :
    receiptFailure? (compactReceiptBytes.take 32) =
      some { code := .truncated, offset := 32 } := by
  decide

theorem compactReceiptTruncatedFuelRejects :
    receiptFailure? (compactReceiptBytes.take 40) =
      some { code := .truncated, offset := 40 } := by
  decide

end Regression

end ClauseCompiler.Codec
