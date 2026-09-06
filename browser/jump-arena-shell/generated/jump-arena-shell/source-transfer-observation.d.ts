declare const phases: readonly ["adapter", "witness-validation", "cartridge-parse", "request-custody", "byte-validation", "frozen-byte-range", "cws1-assembly", "typed-array-construction", "bulk-call", "event-bulk", "event-array-construction", "cse1-decode", "session-construction"];
export type SourceTransferPhase = typeof phases[number];
interface Measurement {
    calls: number;
    inclusiveMs: number;
    exclusiveMs: number;
}
interface Frame {
    phase: SourceTransferPhase;
    started: number;
    children: number;
}
interface Active {
    started: number;
    truncated: boolean;
    frames: Frame[];
    phases: Map<SourceTransferPhase, Measurement>;
}
export interface SourceTransferObservation {
    clock: "monotonic-wall-ms";
    wallMs: number;
    truncated: boolean;
    phases: Readonly<Record<string, Readonly<Measurement>>>;
}
export interface SourceTransferScope {
    readonly owner: Active;
    readonly depth: number;
}
export declare function beginSourceTransferObservation(): boolean;
export declare function finishSourceTransferObservation(): SourceTransferObservation | null;
export declare function enterSourceTransferPhase(phase: SourceTransferPhase): SourceTransferScope | undefined;
export declare function leaveSourceTransferPhase(scope: SourceTransferScope | undefined): void;
export declare function observeSourceTransferPhase<T>(phase: SourceTransferPhase, action: () => T): T;
export {};
