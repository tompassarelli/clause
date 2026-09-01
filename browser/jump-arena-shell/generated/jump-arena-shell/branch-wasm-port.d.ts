type ExactBytes = readonly number[];
interface Cell<T> {
    value: T;
    watches: Record<string, (key: string, cell: Cell<T>, previous: T, next: T) => void>;
}
interface BranchHandle {
    readonly slot: number;
    readonly generation: number;
}
interface BranchPins {
    readonly parentState: ExactBytes;
    readonly programRevision: ExactBytes;
    readonly packageId: ExactBytes;
    readonly applicationSnapshot: ExactBytes;
    readonly applicationLocal: number;
    readonly sessionId: ExactBytes;
    readonly runtimePolicy: ExactBytes;
    readonly rootPolicy: ExactBytes;
    readonly inputEvidence: ExactBytes;
    readonly physicalPlan: ExactBytes;
    readonly budgetUnits: number;
    readonly disconnectTick: number;
}
interface BranchAncestry {
    readonly parentState: ExactBytes;
    readonly run: ExactBytes;
    readonly activation: ExactBytes;
    readonly disconnectStep: ExactBytes;
    readonly suspensionStep: ExactBytes;
    readonly continuation: ExactBytes;
}
interface BranchSuspension {
    readonly step: ExactBytes;
    readonly continuation: ExactBytes;
    readonly run: ExactBytes;
    readonly activation: ExactBytes;
    readonly before: ExactBytes;
    readonly after: ExactBytes;
    readonly remainingBudget: number;
}
interface BranchResumption extends BranchSuspension {
    readonly occurrence: ExactBytes;
}
export interface ProcessCommandEvidenceV1 {
    readonly _tag: "ProcessCommandEvidenceV1";
    readonly occurrence: ExactBytes;
    readonly step: ExactBytes;
    readonly observation: ExactBytes;
}
export interface ReconnectEvidence {
    readonly pins: BranchPins;
    readonly ancestry: BranchAncestry;
    readonly resumption: BranchResumption;
    readonly commandEvidence: readonly ProcessCommandEvidenceV1[];
    readonly candidate: ExactBytes;
    readonly candidateStep: ExactBytes;
    readonly exactBytes: ExactBytes;
}
type CausalIdentityKind = 0 | 1 | 2 | 3 | 4 | 6 | 7 | 8 | 9;
type CausalReference = Readonly<{
    kind: CausalIdentityKind;
    identity: ExactBytes;
}> | Readonly<{
    kind: 5;
    run: ExactBytes;
    activation: ExactBytes;
    step: ExactBytes;
}>;
interface BranchExplanation {
    readonly pins: BranchPins;
    readonly ancestry: BranchAncestry;
    readonly resumption: BranchResumption;
    readonly branchCommandEvidence: readonly ProcessCommandEvidenceV1[];
    readonly branchCandidate: ExactBytes;
    readonly authoritativeBase: ExactBytes;
    readonly authoritativeRun: ExactBytes;
    readonly authoritativeActivation: ExactBytes;
    readonly authoritativeCommandEvidence: readonly ProcessCommandEvidenceV1[];
    readonly authoritativeCandidate: ExactBytes;
    readonly authorization: ExactBytes;
    readonly judgment: ExactBytes;
    readonly admission: ExactBytes;
    readonly successor: ExactBytes;
    readonly causalRecords: readonly Readonly<{
        occurrence: CausalReference;
        predecessors: readonly CausalReference[];
    }>[];
    readonly exactBytes: ExactBytes;
}
interface BranchEventBase {
    readonly slot: number;
    readonly generation: number;
    readonly sequence: number;
}
type BranchEvent = (BranchEventBase & Readonly<{
    kind: "opened";
    pins: BranchPins;
    ancestry: BranchAncestry;
    suspension: BranchSuspension;
}>) | (BranchEventBase & Readonly<{
    kind: "authoritative-admission";
    candidate: ExactBytes;
    predecessor: ExactBytes;
    successor: ExactBytes;
    judgment: ExactBytes;
    admission: ExactBytes;
    run: ExactBytes;
    activation: ExactBytes;
}>) | (BranchEventBase & Readonly<{
    kind: "reconnect-proposed";
    evidence: ReconnectEvidence;
}>) | (BranchEventBase & Readonly<{
    kind: "reconnect-admission";
    predecessor: ExactBytes;
    successor: ExactBytes;
    branchCandidate: ExactBytes;
    authoritativeCandidate: ExactBytes;
    judgment: ExactBytes;
    admission: ExactBytes;
    projection: null | Readonly<{
        observation: ExactBytes;
        termBytes: ExactBytes;
    }>;
    explanation: BranchExplanation;
}>) | (BranchEventBase & Readonly<{
    kind: "explanation";
    explanation: BranchExplanation;
}>) | (BranchEventBase & Readonly<{
    kind: "disposed";
}>) | (BranchEventBase & Readonly<{
    kind: "rejected";
    reason: number;
    pin: number | null;
}>);
export interface WasmProcessBranch {
    readonly _tag: "WasmProcessBranch";
    readonly handle: BranchHandle;
    readonly sequence: Cell<number>;
    readonly disposed: Cell<boolean>;
    readonly opened: Extract<BranchEvent, {
        kind: "opened";
    }>;
}
declare function WasmProcessBranch(handle: BranchHandle, sequence: Cell<number>, disposed: Cell<boolean>, opened: Extract<BranchEvent, {
    kind: "opened";
}>): WasmProcessBranch;
declare function wasmprocessbranch_handle(r: WasmProcessBranch): BranchHandle;
declare function wasmprocessbranch_sequence(r: WasmProcessBranch): Cell<number>;
declare function wasmprocessbranch_disposed(r: WasmProcessBranch): Cell<boolean>;
declare function wasmprocessbranch_opened(r: WasmProcessBranch): Extract<BranchEvent, {
    kind: "opened";
}>;
declare function ProcessCommandEvidenceV1(occurrence: ExactBytes, step: ExactBytes, observation: ExactBytes): ProcessCommandEvidenceV1;
declare function processcommandevidencev1_occurrence(r: ProcessCommandEvidenceV1): ExactBytes;
declare function processcommandevidencev1_step(r: ProcessCommandEvidenceV1): ExactBytes;
declare function processcommandevidencev1_observation(r: ProcessCommandEvidenceV1): ExactBytes;
declare function decode_reconnect_evidence(bytes: unknown): ReconnectEvidence;
declare function decode_branch_explanation(bytes: unknown): BranchExplanation;
declare function open_process_branch_bang(module: unknown, request: {
    readonly bytes: ExactBytes;
}, disconnect_tick: number, disconnect_occurrence: ExactBytes, max_commands: number): WasmProcessBranch;
declare function admit_authoritative_occurrences_bang(module: unknown, incoming_branch: unknown, occurrences: readonly ExactBytes[]): Extract<BranchEvent, {
    kind: "authoritative-admission";
}>;
declare function propose_branch_reconnect_bang(module: unknown, incoming_branch: unknown, occurrences: readonly ExactBytes[]): Extract<BranchEvent, {
    kind: "reconnect-proposed";
}>;
declare function adjudicate_branch_reconnect_bang(module: unknown, incoming_branch: unknown, proposal: Extract<BranchEvent, {
    kind: "reconnect-proposed";
}>, authoritative_base: ExactBytes, occurrences: readonly ExactBytes[]): Extract<BranchEvent, {
    kind: "reconnect-admission";
}>;
declare function explain_process_branch_bang(module: unknown, incoming_branch: unknown): Extract<BranchEvent, {
    kind: "explanation";
}>;
declare function dispose_process_branch_bang(module: unknown, incoming_branch: unknown): boolean;
export { ProcessCommandEvidenceV1 as "->ProcessCommandEvidenceV1" };
export { WasmProcessBranch as "->WasmProcessBranch" };
export { ProcessCommandEvidenceV1 as "ProcessCommandEvidenceV1" };
export { WasmProcessBranch as "WasmProcessBranch" };
export { adjudicate_branch_reconnect_bang as "adjudicate-branch-reconnect!" };
export { admit_authoritative_occurrences_bang as "admit-authoritative-occurrences!" };
export { decode_branch_explanation as "decode-branch-explanation" };
export { decode_reconnect_evidence as "decode-reconnect-evidence" };
export { dispose_process_branch_bang as "dispose-process-branch!" };
export { explain_process_branch_bang as "explain-process-branch!" };
export { open_process_branch_bang as "open-process-branch!" };
export { processcommandevidencev1_observation as "processcommandevidencev1-observation" };
export { processcommandevidencev1_occurrence as "processcommandevidencev1-occurrence" };
export { processcommandevidencev1_step as "processcommandevidencev1-step" };
export { propose_branch_reconnect_bang as "propose-branch-reconnect!" };
export { wasmprocessbranch_disposed as "wasmprocessbranch-disposed" };
export { wasmprocessbranch_handle as "wasmprocessbranch-handle" };
export { wasmprocessbranch_opened as "wasmprocessbranch-opened" };
export { wasmprocessbranch_sequence as "wasmprocessbranch-sequence" };
