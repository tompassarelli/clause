// Opt-in timing only. No observation participates in byte custody, validation,
// identity, session selection, source semantics or Admission.
const phases = [
  "adapter", "witness-validation", "cartridge-parse", "request-custody",
  "byte-validation", "frozen-byte-range", "cws1-assembly",
  "typed-array-construction", "bulk-call", "event-bulk",
  "event-array-construction", "cse1-decode", "session-construction",
] as const;
export type SourceTransferPhase = typeof phases[number];
interface Measurement { calls: number; inclusiveMs: number; exclusiveMs: number }
interface Frame { phase: SourceTransferPhase; started: number; children: number }
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
export interface SourceTransferScope { readonly owner: Active; readonly depth: number }
let active: Active | undefined;

export function beginSourceTransferObservation(): boolean {
  if (active) return false;
  active = { started: performance.now(), truncated: false, frames: [],
    phases: new Map(phases.map(phase => [phase, { calls: 0, inclusiveMs: 0, exclusiveMs: 0 }])) };
  return true;
}

export function finishSourceTransferObservation(): SourceTransferObservation | null {
  if (!active || active.frames.length) return null;
  const observation = active; active = undefined;
  return { clock: "monotonic-wall-ms", wallMs: Math.max(0, performance.now() - observation.started),
    truncated: observation.truncated, phases: Object.fromEntries(observation.phases) };
}

// Disabled hooks read no clock and allocate no scope records. The extra calls,
// branches and measured-callback closures still have a cost in both modes.
export function enterSourceTransferPhase(phase: SourceTransferPhase): SourceTransferScope | undefined {
  if (!active) return undefined;
  if (active.frames.length >= 64) { active.truncated = true; return undefined; }
  const depth = active.frames.length;
  active.frames.push({ phase, started: performance.now(), children: 0 });
  return { owner: active, depth };
}

export function leaveSourceTransferPhase(scope: SourceTransferScope | undefined): void {
  if (!scope) return;
  const owner = scope.owner;
  if (owner !== active || owner.frames.length !== scope.depth + 1) { owner.truncated = true; return; }
  const frame = owner.frames.pop();
  if (!frame) { owner.truncated = true; return; }
  const elapsed = Math.max(0, performance.now() - frame.started);
  const measurement = owner.phases.get(frame.phase);
  if (!measurement) { owner.truncated = true; return; }
  ++measurement.calls;
  measurement.inclusiveMs += elapsed;
  measurement.exclusiveMs += Math.max(0, elapsed - frame.children);
  const parent = owner.frames.at(-1);
  if (parent) parent.children += elapsed;
}

export function observeSourceTransferPhase<T>(phase: SourceTransferPhase, action: () => T): T {
  const scope = enterSourceTransferPhase(phase);
  try { return action(); } finally { leaveSourceTransferPhase(scope); }
}
