export interface Difference {
    readonly path: readonly string[];
    readonly expected: unknown;
    readonly actual: unknown;
}
/** Preserve field values, array order, missing fields, and exact numbers. */
export declare function firstDifference(expected: unknown, actual: unknown, path?: readonly string[]): Difference | undefined;
export interface ReplayMismatch {
    readonly inputIndex: number | null;
    readonly command: number | null;
    readonly phase: string;
    readonly input: string;
    readonly source: string;
    readonly byteOffset: number;
    readonly difference: Difference;
    readonly entity?: unknown;
    readonly sourceState?: unknown;
}
export declare function compareEvents(expected: Uint8Array, actual: Uint8Array): {
    byteOffset: number;
    difference: Difference;
    entity?: unknown;
} | undefined;
export declare function replay(directory: string, modulePath: string): Promise<{
    inputs: number;
    events: number;
    admissions: number;
    mismatch?: ReplayMismatch;
}>;
