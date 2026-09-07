export type EnvelopePrimitive = null | string | boolean | number;
export interface WorkbenchByteEnvelope {
    readonly _tag: "WorkbenchByteEnvelope";
    readonly length: number;
    readonly [index: number]: unknown;
    readonly toJSON: () => readonly number[];
}
export type WorkbenchEnvelope = readonly unknown[] | WorkbenchByteEnvelope;
export interface FixedTick {
    readonly _tag: "FixedTick";
    readonly milliseconds: number;
}
export interface WorkbenchSequenceLimits {
    readonly _tag: "WorkbenchSequenceLimits";
    readonly maxReceiptSequence: number;
    readonly maxInputSequence: number;
    readonly maxGeneration: number;
    readonly maxOperationId: number;
    readonly maxConfigurationRevision: number;
}
export interface WorkbenchPolicy {
    readonly _tag: "WorkbenchPolicy";
    readonly maxPendingObservations: number;
    readonly maxSessionIdentities: number;
    readonly maxImmutableObjects: number;
    readonly maxImmutableProperties: number;
    readonly maxEnvelopeSourceUnits: number;
    readonly sequenceLimits: WorkbenchSequenceLimits;
}
export interface InputObservation {
    readonly _tag: "InputObservation";
    readonly sequence: number;
    readonly value: WorkbenchEnvelope;
}
export interface InputConfiguration {
    readonly _tag: "InputConfiguration";
    readonly revision: number;
    readonly observations: readonly InputObservation[];
}
export interface LifecycleReceipt {
    readonly _tag: "LifecycleReceipt";
    readonly schema: string;
    readonly sequence: number;
    readonly event: string;
    readonly phase: string;
    readonly activeGeneration: number;
    readonly operationGeneration: number;
    readonly operationId: number;
    readonly configurationRevision: number;
    readonly revision: unknown;
    readonly detail: string;
}
export interface WorkbenchSnapshot {
    readonly _tag: "WorkbenchSnapshot";
    readonly phase: string;
    readonly generation: number;
    readonly operationId: number;
    readonly configurationRevision: number;
    readonly pendingObservations: number;
    readonly revision: unknown;
    readonly frame: WorkbenchEnvelope | null;
    readonly disposed: boolean;
}
export interface PackageAccepted {
    readonly _tag: "PackageAccepted";
    readonly acceptedPackage: unknown;
}
export interface PackageRejected {
    readonly _tag: "PackageRejected";
    readonly reason: string;
}
export type PackageCheck = PackageAccepted | PackageRejected;
export interface SessionStarted {
    readonly _tag: "SessionStarted";
    readonly session: unknown;
    readonly revision: unknown;
    readonly frame: WorkbenchEnvelope;
}
export interface SessionFailed {
    readonly _tag: "SessionFailed";
    readonly reason: string;
}
export type SessionCompletion = SessionStarted | SessionFailed;
export interface CandidateProduced {
    readonly _tag: "CandidateProduced";
    readonly candidate: unknown;
}
export interface CandidateFailed {
    readonly _tag: "CandidateFailed";
    readonly reason: string;
}
export type CandidateCompletion = CandidateProduced | CandidateFailed;
export interface AdmissionAccepted {
    readonly _tag: "AdmissionAccepted";
    readonly successor: unknown;
    readonly revision: unknown;
    readonly frame: WorkbenchEnvelope;
}
export interface AdmissionRejected {
    readonly _tag: "AdmissionRejected";
    readonly reason: string;
}
export type AdmissionCompletion = AdmissionAccepted | AdmissionRejected;
export interface CartridgePort {
    readonly _tag: "CartridgePort";
    readonly acceptPackage: (candidate: unknown, complete: (result: PackageCheck) => unknown) => unknown;
    readonly startSession: (acceptedPackage: unknown, generation: number, complete: (result: SessionCompletion) => unknown) => unknown;
    readonly runCandidate: (session: unknown, fixedTick: FixedTick, configuration: InputConfiguration, complete: (result: CandidateCompletion) => unknown) => unknown;
    readonly requestAdmission: (session: unknown, candidate: unknown, complete: (result: AdmissionCompletion) => unknown) => unknown;
    readonly disposeSession: (session: unknown) => unknown;
}
export interface CartridgeWorkbench {
    readonly observeInput: (value: WorkbenchEnvelope) => boolean;
    readonly reloadPackage: (candidate: unknown) => boolean;
    readonly snapshot: () => WorkbenchSnapshot;
    readonly dispose: () => boolean;
}
declare function FixedTick(milliseconds: number): FixedTick;
declare function fixedtick_milliseconds(r: FixedTick): number;
declare function WorkbenchSequenceLimits(maxReceiptSequence: number, maxInputSequence: number, maxGeneration: number, maxOperationId: number, maxConfigurationRevision: number): WorkbenchSequenceLimits;
declare function workbenchsequencelimits_maxReceiptSequence(r: WorkbenchSequenceLimits): number;
declare function workbenchsequencelimits_maxInputSequence(r: WorkbenchSequenceLimits): number;
declare function workbenchsequencelimits_maxGeneration(r: WorkbenchSequenceLimits): number;
declare function workbenchsequencelimits_maxOperationId(r: WorkbenchSequenceLimits): number;
declare function workbenchsequencelimits_maxConfigurationRevision(r: WorkbenchSequenceLimits): number;
declare function WorkbenchPolicy(maxPendingObservations: number, maxSessionIdentities: number, maxImmutableObjects: number, maxImmutableProperties: number, maxEnvelopeSourceUnits: number, sequenceLimits: WorkbenchSequenceLimits): WorkbenchPolicy;
declare function workbenchpolicy_maxPendingObservations(r: WorkbenchPolicy): number;
declare function workbenchpolicy_maxSessionIdentities(r: WorkbenchPolicy): number;
declare function workbenchpolicy_maxImmutableObjects(r: WorkbenchPolicy): number;
declare function workbenchpolicy_maxImmutableProperties(r: WorkbenchPolicy): number;
declare function workbenchpolicy_maxEnvelopeSourceUnits(r: WorkbenchPolicy): number;
declare function workbenchpolicy_sequenceLimits(r: WorkbenchPolicy): WorkbenchSequenceLimits;
declare function InputObservation(sequence: number, value: WorkbenchEnvelope): InputObservation;
declare function inputobservation_sequence(r: InputObservation): number;
declare function inputobservation_value(r: InputObservation): WorkbenchEnvelope;
declare function InputConfiguration(revision: number, observations: readonly InputObservation[]): InputConfiguration;
declare function inputconfiguration_revision(r: InputConfiguration): number;
declare function inputconfiguration_observations(r: InputConfiguration): readonly InputObservation[];
declare function LifecycleReceipt(schema: string, sequence: number, event: string, phase: string, activeGeneration: number, operationGeneration: number, operationId: number, configurationRevision: number, revision: unknown, detail: string): LifecycleReceipt;
declare function lifecyclereceipt_schema(r: LifecycleReceipt): string;
declare function lifecyclereceipt_sequence(r: LifecycleReceipt): number;
declare function lifecyclereceipt_event(r: LifecycleReceipt): string;
declare function lifecyclereceipt_phase(r: LifecycleReceipt): string;
declare function lifecyclereceipt_activeGeneration(r: LifecycleReceipt): number;
declare function lifecyclereceipt_operationGeneration(r: LifecycleReceipt): number;
declare function lifecyclereceipt_operationId(r: LifecycleReceipt): number;
declare function lifecyclereceipt_configurationRevision(r: LifecycleReceipt): number;
declare function lifecyclereceipt_revision(r: LifecycleReceipt): unknown;
declare function lifecyclereceipt_detail(r: LifecycleReceipt): string;
declare function WorkbenchSnapshot(phase: string, generation: number, operationId: number, configurationRevision: number, pendingObservations: number, revision: unknown, frame: WorkbenchEnvelope | null, disposed: boolean): WorkbenchSnapshot;
declare function workbenchsnapshot_phase(r: WorkbenchSnapshot): string;
declare function workbenchsnapshot_generation(r: WorkbenchSnapshot): number;
declare function workbenchsnapshot_operationId(r: WorkbenchSnapshot): number;
declare function workbenchsnapshot_configurationRevision(r: WorkbenchSnapshot): number;
declare function workbenchsnapshot_pendingObservations(r: WorkbenchSnapshot): number;
declare function workbenchsnapshot_revision(r: WorkbenchSnapshot): unknown;
declare function workbenchsnapshot_frame(r: WorkbenchSnapshot): WorkbenchEnvelope | null;
declare function workbenchsnapshot_disposed(r: WorkbenchSnapshot): boolean;
declare function PackageAccepted(acceptedPackage: unknown): PackageAccepted;
declare function packageaccepted_acceptedPackage(r: PackageAccepted): unknown;
declare function PackageRejected(reason: string): PackageRejected;
declare function packagerejected_reason(r: PackageRejected): string;
declare function SessionStarted(session: unknown, revision: unknown, frame: WorkbenchEnvelope): SessionStarted;
declare function sessionstarted_session(r: SessionStarted): unknown;
declare function sessionstarted_revision(r: SessionStarted): unknown;
declare function sessionstarted_frame(r: SessionStarted): WorkbenchEnvelope;
declare function SessionFailed(reason: string): SessionFailed;
declare function sessionfailed_reason(r: SessionFailed): string;
declare function CandidateProduced(candidate: unknown): CandidateProduced;
declare function candidateproduced_candidate(r: CandidateProduced): unknown;
declare function CandidateFailed(reason: string): CandidateFailed;
declare function candidatefailed_reason(r: CandidateFailed): string;
declare function AdmissionAccepted(successor: unknown, revision: unknown, frame: WorkbenchEnvelope): AdmissionAccepted;
declare function admissionaccepted_successor(r: AdmissionAccepted): unknown;
declare function admissionaccepted_revision(r: AdmissionAccepted): unknown;
declare function admissionaccepted_frame(r: AdmissionAccepted): WorkbenchEnvelope;
declare function AdmissionRejected(reason: string): AdmissionRejected;
declare function admissionrejected_reason(r: AdmissionRejected): string;
declare function CartridgePort(acceptPackage: CartridgePort["acceptPackage"], startSession: CartridgePort["startSession"], runCandidate: CartridgePort["runCandidate"], requestAdmission: CartridgePort["requestAdmission"], disposeSession: CartridgePort["disposeSession"]): CartridgePort;
declare function cartridgeport_acceptPackage(r: CartridgePort): CartridgePort["acceptPackage"];
declare function cartridgeport_startSession(r: CartridgePort): CartridgePort["startSession"];
declare function cartridgeport_runCandidate(r: CartridgePort): CartridgePort["runCandidate"];
declare function cartridgeport_requestAdmission(r: CartridgePort): CartridgePort["requestAdmission"];
declare function cartridgeport_disposeSession(r: CartridgePort): CartridgePort["disposeSession"];
declare function create_workbench_envelope(incomingPolicy: WorkbenchPolicy, sourceText: unknown): WorkbenchEnvelope;
declare function create_workbench_byte_envelope(incomingPolicy: WorkbenchPolicy, sourceText: unknown): WorkbenchByteEnvelope;
declare function workbench_byte_envelope_source(value: unknown): string | null;
declare function create_cartridge_workbench_bang(port: CartridgePort, fixedTick: FixedTick, incomingPolicy: WorkbenchPolicy, scheduleFixedTick: (milliseconds: number, tick: () => unknown) => () => unknown, renderFrame: (frame: WorkbenchEnvelope) => unknown, emitReceipt: (receipt: LifecycleReceipt) => unknown, initialPackageCandidate: unknown): CartridgeWorkbench;
export { AdmissionAccepted as "->AdmissionAccepted" };
export { AdmissionRejected as "->AdmissionRejected" };
export { CandidateFailed as "->CandidateFailed" };
export { CandidateProduced as "->CandidateProduced" };
export { CartridgePort as "->CartridgePort" };
export { FixedTick as "->FixedTick" };
export { InputConfiguration as "->InputConfiguration" };
export { InputObservation as "->InputObservation" };
export { LifecycleReceipt as "->LifecycleReceipt" };
export { PackageAccepted as "->PackageAccepted" };
export { PackageRejected as "->PackageRejected" };
export { SessionFailed as "->SessionFailed" };
export { SessionStarted as "->SessionStarted" };
export { WorkbenchPolicy as "->WorkbenchPolicy" };
export { WorkbenchSequenceLimits as "->WorkbenchSequenceLimits" };
export { WorkbenchSnapshot as "->WorkbenchSnapshot" };
export { AdmissionAccepted as "AdmissionAccepted" };
export { AdmissionRejected as "AdmissionRejected" };
export { CandidateFailed as "CandidateFailed" };
export { CandidateProduced as "CandidateProduced" };
export { CartridgePort as "CartridgePort" };
export { FixedTick as "FixedTick" };
export { InputConfiguration as "InputConfiguration" };
export { InputObservation as "InputObservation" };
export { LifecycleReceipt as "LifecycleReceipt" };
export { PackageAccepted as "PackageAccepted" };
export { PackageRejected as "PackageRejected" };
export { SessionFailed as "SessionFailed" };
export { SessionStarted as "SessionStarted" };
export { WorkbenchPolicy as "WorkbenchPolicy" };
export { WorkbenchSequenceLimits as "WorkbenchSequenceLimits" };
export { WorkbenchSnapshot as "WorkbenchSnapshot" };
export { admissionaccepted_frame as "admissionaccepted-frame" };
export { admissionaccepted_revision as "admissionaccepted-revision" };
export { admissionaccepted_successor as "admissionaccepted-successor" };
export { admissionrejected_reason as "admissionrejected-reason" };
export { candidatefailed_reason as "candidatefailed-reason" };
export { candidateproduced_candidate as "candidateproduced-candidate" };
export { cartridgeport_acceptPackage as "cartridgeport-acceptPackage" };
export { cartridgeport_disposeSession as "cartridgeport-disposeSession" };
export { cartridgeport_requestAdmission as "cartridgeport-requestAdmission" };
export { cartridgeport_runCandidate as "cartridgeport-runCandidate" };
export { cartridgeport_startSession as "cartridgeport-startSession" };
export { create_cartridge_workbench_bang as "create-cartridge-workbench!" };
export { create_workbench_byte_envelope as "create-workbench-byte-envelope" };
export { create_workbench_envelope as "create-workbench-envelope" };
export { fixedtick_milliseconds as "fixedtick-milliseconds" };
export { inputconfiguration_observations as "inputconfiguration-observations" };
export { inputconfiguration_revision as "inputconfiguration-revision" };
export { inputobservation_sequence as "inputobservation-sequence" };
export { inputobservation_value as "inputobservation-value" };
export { lifecyclereceipt_activeGeneration as "lifecyclereceipt-activeGeneration" };
export { lifecyclereceipt_configurationRevision as "lifecyclereceipt-configurationRevision" };
export { lifecyclereceipt_detail as "lifecyclereceipt-detail" };
export { lifecyclereceipt_event as "lifecyclereceipt-event" };
export { lifecyclereceipt_operationGeneration as "lifecyclereceipt-operationGeneration" };
export { lifecyclereceipt_operationId as "lifecyclereceipt-operationId" };
export { lifecyclereceipt_phase as "lifecyclereceipt-phase" };
export { lifecyclereceipt_revision as "lifecyclereceipt-revision" };
export { lifecyclereceipt_schema as "lifecyclereceipt-schema" };
export { lifecyclereceipt_sequence as "lifecyclereceipt-sequence" };
export { packageaccepted_acceptedPackage as "packageaccepted-acceptedPackage" };
export { packagerejected_reason as "packagerejected-reason" };
export { sessionfailed_reason as "sessionfailed-reason" };
export { sessionstarted_frame as "sessionstarted-frame" };
export { sessionstarted_revision as "sessionstarted-revision" };
export { sessionstarted_session as "sessionstarted-session" };
export { workbenchpolicy_maxEnvelopeSourceUnits as "workbenchpolicy-maxEnvelopeSourceUnits" };
export { workbenchpolicy_maxImmutableObjects as "workbenchpolicy-maxImmutableObjects" };
export { workbenchpolicy_maxImmutableProperties as "workbenchpolicy-maxImmutableProperties" };
export { workbenchpolicy_maxPendingObservations as "workbenchpolicy-maxPendingObservations" };
export { workbenchpolicy_maxSessionIdentities as "workbenchpolicy-maxSessionIdentities" };
export { workbenchpolicy_sequenceLimits as "workbenchpolicy-sequenceLimits" };
export { workbenchsequencelimits_maxConfigurationRevision as "workbenchsequencelimits-maxConfigurationRevision" };
export { workbenchsequencelimits_maxGeneration as "workbenchsequencelimits-maxGeneration" };
export { workbenchsequencelimits_maxInputSequence as "workbenchsequencelimits-maxInputSequence" };
export { workbenchsequencelimits_maxOperationId as "workbenchsequencelimits-maxOperationId" };
export { workbenchsequencelimits_maxReceiptSequence as "workbenchsequencelimits-maxReceiptSequence" };
export { workbenchsnapshot_configurationRevision as "workbenchsnapshot-configurationRevision" };
export { workbenchsnapshot_disposed as "workbenchsnapshot-disposed" };
export { workbenchsnapshot_frame as "workbenchsnapshot-frame" };
export { workbenchsnapshot_generation as "workbenchsnapshot-generation" };
export { workbenchsnapshot_operationId as "workbenchsnapshot-operationId" };
export { workbenchsnapshot_pendingObservations as "workbenchsnapshot-pendingObservations" };
export { workbenchsnapshot_phase as "workbenchsnapshot-phase" };
export { workbenchsnapshot_revision as "workbenchsnapshot-revision" };
export { workbench_byte_envelope_source as "workbench-byte-envelope-source" };
