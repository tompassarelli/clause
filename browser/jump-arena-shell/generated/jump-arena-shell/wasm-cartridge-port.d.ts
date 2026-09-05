import * as workbench from "./workbench.js";
declare const cse1_projected_term_max_properties: number;
declare const cse1_projected_term_json_max_source_units: number;
export type ExactBytes = readonly number[];
type CanonicalBytes = ExactBytes | string;
export interface ExactProcessRequest {
    readonly _tag: "ExactProcessRequest";
    readonly bytes: ExactBytes;
}
export interface ExactProcessObservation {
    readonly _tag: "ExactProcessObservation";
    readonly bytes: ExactBytes;
}
export interface Cwo1Observation {
    readonly _tag: "Cwo1Observation";
    readonly observationId: ExactBytes;
    readonly stateRevisionId: ExactBytes;
    readonly values: readonly (number | boolean)[];
}
interface ParsedBlob {
    readonly bytes: ExactBytes;
    readonly next: number;
}
interface Cse1EventBase {
    readonly kind: string;
    readonly slot: number;
    readonly generation: number;
    readonly sequence: number;
}
type AdmissionProjection = null | Readonly<{
    observationId: ExactBytes;
    termBytes: ExactBytes;
}>;
export type Cse1Event = (Cse1EventBase & Readonly<{
    kind: "opened";
    packageId: ExactBytes;
    sessionId: ExactBytes;
    world: ExactBytes;
    allocation: ExactBytes;
}>) | (Cse1EventBase & Readonly<{
    kind: "candidate";
    candidateId: ExactBytes;
    base: ExactBytes;
}>) | (Cse1EventBase & Readonly<{
    kind: "issued";
    authorization: ExactBytes;
    packageId: ExactBytes;
    sessionId: ExactBytes;
    base: ExactBytes;
    candidateId: ExactBytes;
}>) | (Cse1EventBase & Readonly<{
    kind: "admission";
    predecessor: ExactBytes;
    successor: ExactBytes;
    admissionId: ExactBytes;
    judgmentId: ExactBytes;
    run: ExactBytes;
    activation: ExactBytes;
    sessionId: ExactBytes;
    projection: AdmissionProjection;
}>) | (Cse1EventBase & Readonly<{
    kind: "disposed";
}>) | (Cse1EventBase & Readonly<{
    kind: "rejected";
    reason: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "candidate-rejected";
    diagnostic: string;
}>) | (Cse1EventBase & Readonly<{
    kind: "suspended";
    step: ExactBytes;
    continuation: ExactBytes;
    run: ExactBytes;
    activation: ExactBytes;
    before: ExactBytes;
    after: ExactBytes;
    remainingBudget: number;
    stateRevisionCount: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "resumed";
    occurrence: ExactBytes;
    step: ExactBytes;
    continuation: ExactBytes;
    run: ExactBytes;
    activation: ExactBytes;
    before: ExactBytes;
    after: ExactBytes;
    remainingBudget: number;
    stateRevisionCount: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "effect-intent";
    intentId: ExactBytes;
    run: ExactBytes;
    activation: ExactBytes;
    step: ExactBytes;
    contractIndex: number;
    capability: Readonly<{
        snapshot: ExactBytes;
        local: number;
    }>;
    scope: Readonly<{
        application: Readonly<{
            snapshot: ExactBytes;
            local: number;
        }>;
        mode: Readonly<{
            snapshot: ExactBytes;
            operator: number;
            local: number;
        }>;
        programRevision: ExactBytes;
        world: ExactBytes;
        sessionId: ExactBytes;
        remainingBudget: number;
    }>;
    actionBytes: ExactBytes;
    resourceBytes: ExactBytes;
    payloadBytes: ExactBytes;
    stateRevisionCount: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "effect-intent-absent";
    stateRevisionCount: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "effect-authorization";
    authorizationId: ExactBytes;
    intentId: ExactBytes;
    stateRevisionCount: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "effect-attempt";
    attemptId: ExactBytes;
    intentId: ExactBytes;
    authorizationId: ExactBytes;
    actionBytes: ExactBytes;
    resourceBytes: ExactBytes;
    payloadBytes: ExactBytes;
    stateRevisionCount: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "effect-settlement";
    intentId: ExactBytes;
    attemptId: ExactBytes;
    receiptId: ExactBytes | null;
    observationId: ExactBytes | null;
    judgmentId: ExactBytes;
    disposition: "receipt-observed" | "no-receipt";
    stateRevisionCount: number;
}>) | (Cse1EventBase & Readonly<{
    kind: "input";
}>);
export interface ProjectedObject {
    readonly [key: string]: ProjectedValue;
}
export type ProjectedReferent = Readonly<{
    kind: "referent";
    domain: number;
    identity: Readonly<{
        kind: "declared";
        value: number;
    }> | Readonly<{
        kind: "created";
        value: readonly number[];
    }>;
}>;
export type ProjectedValue = number | boolean | string | readonly ProjectedValue[] | ProjectedObject;
declare function decode_cwr1_hex(source: unknown): ExactBytes;
declare function ExactProcessRequest(bytes: ExactBytes): ExactProcessRequest;
declare function exactprocessrequest_bytes(r: ExactProcessRequest): ExactBytes;
declare function ExactProcessObservation(bytes: ExactBytes): ExactProcessObservation;
declare function exactprocessobservation_bytes(r: ExactProcessObservation): ExactBytes;
declare function Cwo1Observation(observationId: ExactBytes, stateRevisionId: ExactBytes, values: readonly (number | boolean)[]): Cwo1Observation;
declare function cwo1observation_observationId(r: Cwo1Observation): ExactBytes;
declare function cwo1observation_stateRevisionId(r: Cwo1Observation): ExactBytes;
declare function cwo1observation_values(r: Cwo1Observation): readonly (number | boolean)[];
declare function exact_byte_array_p(bytes: unknown, maximum: number): bytes is ExactBytes;
declare function process_status(status: unknown): number;
declare function byte_at(bytes: CanonicalBytes, index: number): number;
declare function little_u16(bytes: ExactBytes, offset: number): number;
declare function little_u32(bytes: ExactBytes, offset: number): number;
declare function little_safe_u64(bytes: ExactBytes, offset: number): number;
declare function append_u32_bang(bytes: number[], value: number): number;
declare function append_u64_bang(bytes: number[], value: number): number;
declare function append_blob_bang(bytes: number[], value: ExactBytes): void;
declare function require_range(bytes: CanonicalBytes, offset: number, length: number, label: string): number;
declare function frozen_byte_range(bytes: ExactBytes, start: number, end: number): ExactBytes;
declare function decode_cwo1_observation(incoming: unknown): Cwo1Observation;
declare function parse_blob(bytes: ExactBytes, offset: number, maximum: number, label: string): ParsedBlob;
declare function process_request_occurrences_bang(request: unknown): readonly ExactBytes[];
declare function decode_projected_term_frame(bytes: unknown): ProjectedValue;
declare function advance_session_occurrence_bang(module: unknown, incoming_session: unknown, ordinal: number): Extract<Cse1Event, {
    kind: "input";
}>;
declare function suspend_session_bang(module: unknown, incoming_session: unknown): Extract<Cse1Event, {
    kind: "suspended";
}>;
declare function resume_session_bang(module: unknown, incoming_session: unknown): Extract<Cse1Event, {
    kind: "resumed";
}>;
declare function query_pending_effect_intent_bang(module: unknown, incoming_session: unknown): Extract<Cse1Event, {
    kind: "effect-intent" | "effect-intent-absent";
}>;
declare function emit_effect_intent_bang(module: unknown, incoming_session: unknown): Extract<Cse1Event, {
    kind: "effect-intent";
}>;
declare function issue_effect_authorization_bang(module: unknown, incoming_session: unknown, intent_id: unknown): Extract<Cse1Event, {
    kind: "effect-authorization";
}>;
declare function begin_effect_attempt_bang(module: unknown, incoming_session: unknown, authorization_id: unknown): Extract<Cse1Event, {
    kind: "effect-attempt";
}>;
declare function settle_effect_attempt_bang(module: unknown, incoming_session: unknown, attempt_id: unknown, status: unknown, exact_receipt: unknown): Extract<Cse1Event, {
    kind: "effect-settlement";
}>;
declare function admit_session_candidate_bang(module: unknown, incoming_session: unknown, incoming_candidate: unknown): Extract<Cse1Event, {
    kind: "admission";
}>;
declare function create_wasm_cartridge_port_bang(module: unknown, policy: workbench.WorkbenchPolicy): workbench.CartridgePort;
/** Apply compiler-owned CET1 to this exact live Wasm session. No source parsing,
 * identity inference, native shadow-state import, or automatic Admission. */
export declare function editSourceSession(module: unknown, incomingSession: unknown, generation: number, request: ExactProcessRequest, witness: ExactBytes, policy: workbench.WorkbenchPolicy): workbench.SessionCompletion;
export declare function explainSession(module: unknown, incomingSession: unknown, entry: number): ProjectedValue;
export declare function sourceContinuity(module: unknown, incomingSession: unknown): ProjectedValue;
/** Read-only opaque CIQ1 request: all search and semantic evaluation occurs
 * inside the live Wasm runtime against a retained actual event. */
export declare function interveneSession(module: unknown, incomingSession: unknown, query: ExactBytes): ProjectedValue;
export interface FiniteScalarChange {
    readonly slot: number;
    readonly value: boolean | number;
}
/** Passive typed serializer for a finite question supplied by the caller.
 * CPP1 tags encode the shared normalized predicate; no local evaluation. */
export declare function finiteScalarInterventionQuery(event: string, allowed: readonly FiniteScalarChange[], maximumEvaluations: number, desired: {
    readonly slot: number;
    readonly greaterThan: number;
} | boolean): ExactBytes;
declare const create_wasm_cartridge_port: typeof create_wasm_cartridge_port_bang;
export { Cwo1Observation as "->Cwo1Observation" };
export { ExactProcessObservation as "->ExactProcessObservation" };
export { ExactProcessRequest as "->ExactProcessRequest" };
export { Cwo1Observation as "Cwo1Observation" };
export { ExactProcessObservation as "ExactProcessObservation" };
export { ExactProcessRequest as "ExactProcessRequest" };
export { admit_session_candidate_bang as "admit-session-candidate!" };
export { advance_session_occurrence_bang as "advance-session-occurrence!" };
export { append_blob_bang as "append-blob!" };
export { append_u32_bang as "append-u32!" };
export { append_u64_bang as "append-u64!" };
export { begin_effect_attempt_bang as "begin-effect-attempt!" };
export { byte_at as "byte-at" };
export { create_wasm_cartridge_port as "create-wasm-cartridge-port" };
export { cse1_projected_term_json_max_source_units as "cse1-projected-term-json-max-source-units" };
export { cse1_projected_term_max_properties as "cse1-projected-term-max-properties" };
export { cwo1observation_observationId as "cwo1observation-observationId" };
export { cwo1observation_stateRevisionId as "cwo1observation-stateRevisionId" };
export { cwo1observation_values as "cwo1observation-values" };
export { decode_cwo1_observation as "decode-cwo1-observation" };
export { decode_cwr1_hex as "decode-cwr1-hex" };
export { decode_projected_term_frame as "decode-projected-term-frame" };
export { emit_effect_intent_bang as "emit-effect-intent!" };
export { exact_byte_array_p as "exact-byte-array?" };
export { exactprocessobservation_bytes as "exactprocessobservation-bytes" };
export { exactprocessrequest_bytes as "exactprocessrequest-bytes" };
export { frozen_byte_range as "frozen-byte-range" };
export { issue_effect_authorization_bang as "issue-effect-authorization!" };
export { little_safe_u64 as "little-safe-u64" };
export { little_u16 as "little-u16" };
export { little_u32 as "little-u32" };
export { parse_blob as "parse-blob" };
export { process_request_occurrences_bang as "process-request-occurrences!" };
export { process_status as "process-status" };
export { query_pending_effect_intent_bang as "query-pending-effect-intent!" };
export { require_range as "require-range" };
export { resume_session_bang as "resume-session!" };
export { settle_effect_attempt_bang as "settle-effect-attempt!" };
export { suspend_session_bang as "suspend-session!" };
