function equivalent(left: unknown, right: unknown): boolean {
  return (
    Object.is(left, right) ||
    (Array.isArray(left) &&
      Array.isArray(right) &&
      left.length === right.length &&
      left.every((value, index) => equivalent(value, right[index])))
  );
}

function appendValue<T>(values: readonly T[], value: T): T[] {
  return [...values, value];
}

function isEmpty(values: { readonly length: number }): boolean {
  return values.length === 0;
}

function firstValue<T>(values: readonly T[]): T {
  if (values.length === 0)
    throw new Error("cannot read the first value of an empty sequence");
  return values[0];
}

function restValues<T>(values: readonly T[]): readonly T[] {
  return values.slice(1);
}

function concatenate(...values: readonly unknown[]): string {
  return values.map(String).join("");
}

function classifyError(error: unknown): 0 {
  if (error instanceof Error) return 0;
  throw error;
}

export type EnvelopePrimitive = null | string | boolean | number;
export type WorkbenchEnvelope = readonly unknown[];

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
  readonly acceptPackage: (
    candidate: unknown,
    complete: (result: PackageCheck) => unknown,
  ) => unknown;
  readonly startSession: (
    acceptedPackage: unknown,
    generation: number,
    complete: (result: SessionCompletion) => unknown,
  ) => unknown;
  readonly runCandidate: (
    session: unknown,
    fixedTick: FixedTick,
    configuration: InputConfiguration,
    complete: (result: CandidateCompletion) => unknown,
  ) => unknown;
  readonly requestAdmission: (
    session: unknown,
    candidate: unknown,
    complete: (result: AdmissionCompletion) => unknown,
  ) => unknown;
  readonly disposeSession: (session: unknown) => unknown;
}

interface ActiveRuntime {
  readonly _tag: "ActiveRuntime";
  readonly generation: number;
  readonly session: unknown;
  readonly revision: unknown;
  readonly frame: WorkbenchEnvelope;
}

interface WorkbenchState {
  readonly _tag: "WorkbenchState";
  readonly phase: string;
  readonly active: ActiveRuntime | null;
  readonly operationId: number;
  readonly configuration: InputConfiguration;
  readonly latestReload: number;
  readonly disposed: boolean;
}

type WorkbenchTransition =
  | Readonly<{ _tag: "ObserveInput"; value: WorkbenchEnvelope }>
  | Readonly<{ _tag: "InputRejected" }>
  | Readonly<{ _tag: "FixedTickElapsed" }>
  | Readonly<{
      _tag: "CandidateCompleted";
      generation: number;
      operationId: number;
      completion: CandidateCompletion;
    }>
  | Readonly<{
      _tag: "CandidateBoundaryFailed";
      generation: number;
      operationId: number;
    }>
  | Readonly<{
      _tag: "AdmissionCompleted";
      generation: number;
      operationId: number;
      completion: AdmissionCompletion;
    }>
  | Readonly<{
      _tag: "AdmissionBoundaryFailed";
      generation: number;
      operationId: number;
    }>
  | Readonly<{ _tag: "ReloadRequested"; packageCandidate: unknown }>
  | Readonly<{
      _tag: "PackageCompleted";
      generation: number;
      completion: PackageCheck;
    }>
  | Readonly<{ _tag: "PackageBoundaryFailed"; generation: number }>
  | Readonly<{ _tag: "ReloadRejected" }>
  | Readonly<{
      _tag: "SessionCompleted";
      generation: number;
      priorSession: unknown;
      completion: SessionCompletion;
    }>
  | Readonly<{
      _tag: "SessionBoundaryFailed";
      generation: number;
      priorSession: unknown;
    }>
  | Readonly<{ _tag: "DisposeRequested" }>;

interface EnvelopeMeasure {
  readonly _tag: "EnvelopeMeasure";
  readonly objectCount: number;
  readonly propertyCount: number;
  readonly sourceUnitCount: number;
}

interface EnvelopeCopy {
  readonly _tag: "EnvelopeCopy";
  readonly source: readonly unknown[];
  readonly target: unknown[];
}

type EnvelopeScan =
  | Readonly<{
      _tag: "EnvelopeScanAccepted";
      pending: readonly EnvelopeCopy[];
      targets: readonly unknown[][];
      objectCount: number;
      propertyCount: number;
    }>
  | Readonly<{ _tag: "EnvelopeScanRejected" }>;

interface Cell<T> {
  value: T;
  watches: Record<
    string,
    (key: string, cell: Cell<T>, previous: T, next: T) => void
  >;
}

export interface CartridgeWorkbench {
  readonly observeInput: (value: WorkbenchEnvelope) => boolean;
  readonly reloadPackage: (candidate: unknown) => boolean;
  readonly snapshot: () => WorkbenchSnapshot;
  readonly dispose: () => boolean;
}

function FixedTick(milliseconds: number): FixedTick {
  return Object.freeze({ _tag: "FixedTick", milliseconds });
}

function fixedtick_milliseconds(r: FixedTick): number {
  return r.milliseconds;
}

function WorkbenchSequenceLimits(
  maxReceiptSequence: number,
  maxInputSequence: number,
  maxGeneration: number,
  maxOperationId: number,
  maxConfigurationRevision: number,
): WorkbenchSequenceLimits {
  return Object.freeze({
    _tag: "WorkbenchSequenceLimits",
    maxReceiptSequence,
    maxInputSequence,
    maxGeneration,
    maxOperationId,
    maxConfigurationRevision,
  });
}

function workbenchsequencelimits_maxReceiptSequence(
  r: WorkbenchSequenceLimits,
): number {
  return r.maxReceiptSequence;
}

function workbenchsequencelimits_maxInputSequence(
  r: WorkbenchSequenceLimits,
): number {
  return r.maxInputSequence;
}

function workbenchsequencelimits_maxGeneration(
  r: WorkbenchSequenceLimits,
): number {
  return r.maxGeneration;
}

function workbenchsequencelimits_maxOperationId(
  r: WorkbenchSequenceLimits,
): number {
  return r.maxOperationId;
}

function workbenchsequencelimits_maxConfigurationRevision(
  r: WorkbenchSequenceLimits,
): number {
  return r.maxConfigurationRevision;
}

function WorkbenchPolicy(
  maxPendingObservations: number,
  maxSessionIdentities: number,
  maxImmutableObjects: number,
  maxImmutableProperties: number,
  maxEnvelopeSourceUnits: number,
  sequenceLimits: WorkbenchSequenceLimits,
): WorkbenchPolicy {
  return Object.freeze({
    _tag: "WorkbenchPolicy",
    maxPendingObservations,
    maxSessionIdentities,
    maxImmutableObjects,
    maxImmutableProperties,
    maxEnvelopeSourceUnits,
    sequenceLimits,
  });
}

function workbenchpolicy_maxPendingObservations(r: WorkbenchPolicy): number {
  return r.maxPendingObservations;
}

function workbenchpolicy_maxSessionIdentities(r: WorkbenchPolicy): number {
  return r.maxSessionIdentities;
}

function workbenchpolicy_maxImmutableObjects(r: WorkbenchPolicy): number {
  return r.maxImmutableObjects;
}

function workbenchpolicy_maxImmutableProperties(r: WorkbenchPolicy): number {
  return r.maxImmutableProperties;
}

function workbenchpolicy_maxEnvelopeSourceUnits(r: WorkbenchPolicy): number {
  return r.maxEnvelopeSourceUnits;
}

function workbenchpolicy_sequenceLimits(
  r: WorkbenchPolicy,
): WorkbenchSequenceLimits {
  return r.sequenceLimits;
}

function InputObservation(
  sequence: number,
  value: WorkbenchEnvelope,
): InputObservation {
  return Object.freeze({ _tag: "InputObservation", sequence, value });
}

function inputobservation_sequence(r: InputObservation): number {
  return r.sequence;
}

function inputobservation_value(r: InputObservation): WorkbenchEnvelope {
  return r.value;
}

function InputConfiguration(
  revision: number,
  observations: readonly InputObservation[],
): InputConfiguration {
  return Object.freeze({ _tag: "InputConfiguration", revision, observations });
}

function inputconfiguration_revision(r: InputConfiguration): number {
  return r.revision;
}

function inputconfiguration_observations(
  r: InputConfiguration,
): readonly InputObservation[] {
  return r.observations;
}

function LifecycleReceipt(
  schema: string,
  sequence: number,
  event: string,
  phase: string,
  activeGeneration: number,
  operationGeneration: number,
  operationId: number,
  configurationRevision: number,
  revision: unknown,
  detail: string,
): LifecycleReceipt {
  return Object.freeze({
    _tag: "LifecycleReceipt",
    schema,
    sequence,
    event,
    phase,
    activeGeneration,
    operationGeneration,
    operationId,
    configurationRevision,
    revision,
    detail,
  });
}

function lifecyclereceipt_schema(r: LifecycleReceipt): string {
  return r.schema;
}

function lifecyclereceipt_sequence(r: LifecycleReceipt): number {
  return r.sequence;
}

function lifecyclereceipt_event(r: LifecycleReceipt): string {
  return r.event;
}

function lifecyclereceipt_phase(r: LifecycleReceipt): string {
  return r.phase;
}

function lifecyclereceipt_activeGeneration(r: LifecycleReceipt): number {
  return r.activeGeneration;
}

function lifecyclereceipt_operationGeneration(r: LifecycleReceipt): number {
  return r.operationGeneration;
}

function lifecyclereceipt_operationId(r: LifecycleReceipt): number {
  return r.operationId;
}

function lifecyclereceipt_configurationRevision(r: LifecycleReceipt): number {
  return r.configurationRevision;
}

function lifecyclereceipt_revision(r: LifecycleReceipt): unknown {
  return r.revision;
}

function lifecyclereceipt_detail(r: LifecycleReceipt): string {
  return r.detail;
}

function WorkbenchSnapshot(
  phase: string,
  generation: number,
  operationId: number,
  configurationRevision: number,
  pendingObservations: number,
  revision: unknown,
  frame: WorkbenchEnvelope | null,
  disposed: boolean,
): WorkbenchSnapshot {
  return Object.freeze({
    _tag: "WorkbenchSnapshot",
    phase,
    generation,
    operationId,
    configurationRevision,
    pendingObservations,
    revision,
    frame,
    disposed,
  });
}

function workbenchsnapshot_phase(r: WorkbenchSnapshot): string {
  return r.phase;
}

function workbenchsnapshot_generation(r: WorkbenchSnapshot): number {
  return r.generation;
}

function workbenchsnapshot_operationId(r: WorkbenchSnapshot): number {
  return r.operationId;
}

function workbenchsnapshot_configurationRevision(r: WorkbenchSnapshot): number {
  return r.configurationRevision;
}

function workbenchsnapshot_pendingObservations(r: WorkbenchSnapshot): number {
  return r.pendingObservations;
}

function workbenchsnapshot_revision(r: WorkbenchSnapshot): unknown {
  return r.revision;
}

function workbenchsnapshot_frame(
  r: WorkbenchSnapshot,
): WorkbenchEnvelope | null {
  return r.frame;
}

function workbenchsnapshot_disposed(r: WorkbenchSnapshot): boolean {
  return r.disposed;
}

// PackageCheck = PackageAccepted | PackageRejected
function PackageAccepted(acceptedPackage: unknown): PackageAccepted {
  return Object.freeze({
    _tag: "PackageAccepted",
    acceptedPackage: acceptedPackage,
  });
}

function packageaccepted_acceptedPackage(r: PackageAccepted): unknown {
  return r.acceptedPackage;
}
function PackageRejected(reason: string): PackageRejected {
  return Object.freeze({ _tag: "PackageRejected", reason: reason });
}

function packagerejected_reason(r: PackageRejected): string {
  return r.reason;
}

// SessionCompletion = SessionStarted | SessionFailed
function SessionStarted(
  session: unknown,
  revision: unknown,
  frame: WorkbenchEnvelope,
): SessionStarted {
  return Object.freeze({
    _tag: "SessionStarted",
    session: session,
    revision: revision,
    frame: frame,
  });
}

function sessionstarted_session(r: SessionStarted): unknown {
  return r.session;
}

function sessionstarted_revision(r: SessionStarted): unknown {
  return r.revision;
}

function sessionstarted_frame(r: SessionStarted): WorkbenchEnvelope {
  return r.frame;
}
function SessionFailed(reason: string): SessionFailed {
  return Object.freeze({ _tag: "SessionFailed", reason: reason });
}

function sessionfailed_reason(r: SessionFailed): string {
  return r.reason;
}

// CandidateCompletion = CandidateProduced | CandidateFailed
function CandidateProduced(candidate: unknown): CandidateProduced {
  return Object.freeze({ _tag: "CandidateProduced", candidate: candidate });
}

function candidateproduced_candidate(r: CandidateProduced): unknown {
  return r.candidate;
}
function CandidateFailed(reason: string): CandidateFailed {
  return Object.freeze({ _tag: "CandidateFailed", reason: reason });
}

function candidatefailed_reason(r: CandidateFailed): string {
  return r.reason;
}

// AdmissionCompletion = AdmissionAccepted | AdmissionRejected
function AdmissionAccepted(
  successor: unknown,
  revision: unknown,
  frame: WorkbenchEnvelope,
): AdmissionAccepted {
  return Object.freeze({
    _tag: "AdmissionAccepted",
    successor: successor,
    revision: revision,
    frame: frame,
  });
}

function admissionaccepted_successor(r: AdmissionAccepted): unknown {
  return r.successor;
}

function admissionaccepted_revision(r: AdmissionAccepted): unknown {
  return r.revision;
}

function admissionaccepted_frame(r: AdmissionAccepted): WorkbenchEnvelope {
  return r.frame;
}
function AdmissionRejected(reason: string): AdmissionRejected {
  return Object.freeze({ _tag: "AdmissionRejected", reason: reason });
}

function admissionrejected_reason(r: AdmissionRejected): string {
  return r.reason;
}

function CartridgePort(
  acceptPackage: CartridgePort["acceptPackage"],
  startSession: CartridgePort["startSession"],
  runCandidate: CartridgePort["runCandidate"],
  requestAdmission: CartridgePort["requestAdmission"],
  disposeSession: CartridgePort["disposeSession"],
): CartridgePort {
  return Object.freeze({
    _tag: "CartridgePort",
    acceptPackage,
    startSession,
    runCandidate,
    requestAdmission,
    disposeSession,
  });
}

function cartridgeport_acceptPackage(
  r: CartridgePort,
): CartridgePort["acceptPackage"] {
  return r.acceptPackage;
}

function cartridgeport_startSession(
  r: CartridgePort,
): CartridgePort["startSession"] {
  return r.startSession;
}

function cartridgeport_runCandidate(
  r: CartridgePort,
): CartridgePort["runCandidate"] {
  return r.runCandidate;
}

function cartridgeport_requestAdmission(
  r: CartridgePort,
): CartridgePort["requestAdmission"] {
  return r.requestAdmission;
}

function cartridgeport_disposeSession(
  r: CartridgePort,
): CartridgePort["disposeSession"] {
  return r.disposeSession;
}

function ActiveRuntime(
  generation: number,
  session: unknown,
  revision: unknown,
  frame: WorkbenchEnvelope,
): ActiveRuntime {
  return Object.freeze({
    _tag: "ActiveRuntime",
    generation,
    session,
    revision,
    frame,
  });
}

function activeruntime_generation(r: ActiveRuntime): number {
  return r.generation;
}

function activeruntime_session(r: ActiveRuntime): unknown {
  return r.session;
}

function activeruntime_revision(r: ActiveRuntime): unknown {
  return r.revision;
}

function activeruntime_frame(r: ActiveRuntime): WorkbenchEnvelope {
  return r.frame;
}

function WorkbenchState(
  phase: string,
  active: ActiveRuntime | null,
  operationId: number,
  configuration: InputConfiguration,
  latestReload: number,
  disposed: boolean,
): WorkbenchState {
  return Object.freeze({
    _tag: "WorkbenchState",
    phase,
    active,
    operationId,
    configuration,
    latestReload,
    disposed,
  });
}

function workbenchstate_phase(r: WorkbenchState): string {
  return r.phase;
}

function workbenchstate_active(r: WorkbenchState): ActiveRuntime | null {
  return r.active;
}

function workbenchstate_operationId(r: WorkbenchState): number {
  return r.operationId;
}

function workbenchstate_configuration(r: WorkbenchState): InputConfiguration {
  return r.configuration;
}

function workbenchstate_latestReload(r: WorkbenchState): number {
  return r.latestReload;
}

function workbenchstate_disposed(r: WorkbenchState): boolean {
  return r.disposed;
}

// WorkbenchTransition = ObserveInput | InputRejected | FixedTickElapsed | CandidateCompleted | CandidateBoundaryFailed | AdmissionCompleted | AdmissionBoundaryFailed | ReloadRequested | PackageCompleted | PackageBoundaryFailed | ReloadRejected | SessionCompleted | SessionBoundaryFailed | DisposeRequested
function ObserveInput(
  value: WorkbenchEnvelope,
): Extract<WorkbenchTransition, { _tag: "ObserveInput" }> {
  return Object.freeze({ _tag: "ObserveInput", value: value });
}

function observeinput_value(
  r: Extract<WorkbenchTransition, { _tag: "ObserveInput" }>,
): WorkbenchEnvelope {
  return r.value;
}
function InputRejected(): Extract<
  WorkbenchTransition,
  { _tag: "InputRejected" }
> {
  return Object.freeze({ _tag: "InputRejected" });
}
function FixedTickElapsed(): Extract<
  WorkbenchTransition,
  { _tag: "FixedTickElapsed" }
> {
  return Object.freeze({ _tag: "FixedTickElapsed" });
}
function CandidateCompleted(
  generation: number,
  operationId: number,
  completion: CandidateCompletion,
): Extract<WorkbenchTransition, { _tag: "CandidateCompleted" }> {
  return Object.freeze({
    _tag: "CandidateCompleted",
    generation: generation,
    operationId: operationId,
    completion: completion,
  });
}

function candidatecompleted_generation(
  r: Extract<WorkbenchTransition, { _tag: "CandidateCompleted" }>,
): number {
  return r.generation;
}

function candidatecompleted_operationId(
  r: Extract<WorkbenchTransition, { _tag: "CandidateCompleted" }>,
): number {
  return r.operationId;
}

function candidatecompleted_completion(
  r: Extract<WorkbenchTransition, { _tag: "CandidateCompleted" }>,
): CandidateCompletion {
  return r.completion;
}
function CandidateBoundaryFailed(
  generation: number,
  operationId: number,
): Extract<WorkbenchTransition, { _tag: "CandidateBoundaryFailed" }> {
  return Object.freeze({
    _tag: "CandidateBoundaryFailed",
    generation: generation,
    operationId: operationId,
  });
}

function candidateboundaryfailed_generation(
  r: Extract<WorkbenchTransition, { _tag: "CandidateBoundaryFailed" }>,
): number {
  return r.generation;
}

function candidateboundaryfailed_operationId(
  r: Extract<WorkbenchTransition, { _tag: "CandidateBoundaryFailed" }>,
): number {
  return r.operationId;
}
function AdmissionCompleted(
  generation: number,
  operationId: number,
  completion: AdmissionCompletion,
): Extract<WorkbenchTransition, { _tag: "AdmissionCompleted" }> {
  return Object.freeze({
    _tag: "AdmissionCompleted",
    generation: generation,
    operationId: operationId,
    completion: completion,
  });
}

function admissioncompleted_generation(
  r: Extract<WorkbenchTransition, { _tag: "AdmissionCompleted" }>,
): number {
  return r.generation;
}

function admissioncompleted_operationId(
  r: Extract<WorkbenchTransition, { _tag: "AdmissionCompleted" }>,
): number {
  return r.operationId;
}

function admissioncompleted_completion(
  r: Extract<WorkbenchTransition, { _tag: "AdmissionCompleted" }>,
): AdmissionCompletion {
  return r.completion;
}
function AdmissionBoundaryFailed(
  generation: number,
  operationId: number,
): Extract<WorkbenchTransition, { _tag: "AdmissionBoundaryFailed" }> {
  return Object.freeze({
    _tag: "AdmissionBoundaryFailed",
    generation: generation,
    operationId: operationId,
  });
}

function admissionboundaryfailed_generation(
  r: Extract<WorkbenchTransition, { _tag: "AdmissionBoundaryFailed" }>,
): number {
  return r.generation;
}

function admissionboundaryfailed_operationId(
  r: Extract<WorkbenchTransition, { _tag: "AdmissionBoundaryFailed" }>,
): number {
  return r.operationId;
}
function ReloadRequested(
  packageCandidate: unknown,
): Extract<WorkbenchTransition, { _tag: "ReloadRequested" }> {
  return Object.freeze({
    _tag: "ReloadRequested",
    packageCandidate: packageCandidate,
  });
}

function reloadrequested_packageCandidate(
  r: Extract<WorkbenchTransition, { _tag: "ReloadRequested" }>,
): unknown {
  return r.packageCandidate;
}
function PackageCompleted(
  generation: number,
  completion: PackageCheck,
): Extract<WorkbenchTransition, { _tag: "PackageCompleted" }> {
  return Object.freeze({
    _tag: "PackageCompleted",
    generation: generation,
    completion: completion,
  });
}

function packagecompleted_generation(
  r: Extract<WorkbenchTransition, { _tag: "PackageCompleted" }>,
): number {
  return r.generation;
}

function packagecompleted_completion(
  r: Extract<WorkbenchTransition, { _tag: "PackageCompleted" }>,
): PackageCheck {
  return r.completion;
}
function PackageBoundaryFailed(
  generation: number,
): Extract<WorkbenchTransition, { _tag: "PackageBoundaryFailed" }> {
  return Object.freeze({
    _tag: "PackageBoundaryFailed",
    generation: generation,
  });
}

function packageboundaryfailed_generation(
  r: Extract<WorkbenchTransition, { _tag: "PackageBoundaryFailed" }>,
): number {
  return r.generation;
}
function ReloadRejected(): Extract<
  WorkbenchTransition,
  { _tag: "ReloadRejected" }
> {
  return Object.freeze({ _tag: "ReloadRejected" });
}
function SessionCompleted(
  generation: number,
  priorSession: unknown,
  completion: SessionCompletion,
): Extract<WorkbenchTransition, { _tag: "SessionCompleted" }> {
  return Object.freeze({
    _tag: "SessionCompleted",
    generation: generation,
    priorSession: priorSession,
    completion: completion,
  });
}

function sessioncompleted_generation(
  r: Extract<WorkbenchTransition, { _tag: "SessionCompleted" }>,
): number {
  return r.generation;
}

function sessioncompleted_priorSession(
  r: Extract<WorkbenchTransition, { _tag: "SessionCompleted" }>,
): unknown {
  return r.priorSession;
}

function sessioncompleted_completion(
  r: Extract<WorkbenchTransition, { _tag: "SessionCompleted" }>,
): SessionCompletion {
  return r.completion;
}
function SessionBoundaryFailed(
  generation: number,
  priorSession: unknown,
): Extract<WorkbenchTransition, { _tag: "SessionBoundaryFailed" }> {
  return Object.freeze({
    _tag: "SessionBoundaryFailed",
    generation: generation,
    priorSession: priorSession,
  });
}

function sessionboundaryfailed_generation(
  r: Extract<WorkbenchTransition, { _tag: "SessionBoundaryFailed" }>,
): number {
  return r.generation;
}

function sessionboundaryfailed_priorSession(
  r: Extract<WorkbenchTransition, { _tag: "SessionBoundaryFailed" }>,
): unknown {
  return r.priorSession;
}
function DisposeRequested(): Extract<
  WorkbenchTransition,
  { _tag: "DisposeRequested" }
> {
  return Object.freeze({ _tag: "DisposeRequested" });
}

function EnvelopeMeasure(
  objectCount: number,
  propertyCount: number,
  sourceUnitCount: number,
): EnvelopeMeasure {
  return Object.freeze({
    _tag: "EnvelopeMeasure",
    objectCount,
    propertyCount,
    sourceUnitCount,
  });
}

function envelopemeasure_objectCount(r: EnvelopeMeasure): number {
  return r.objectCount;
}

function envelopemeasure_propertyCount(r: EnvelopeMeasure): number {
  return r.propertyCount;
}

function envelopemeasure_sourceUnitCount(r: EnvelopeMeasure): number {
  return r.sourceUnitCount;
}

function EnvelopeCopy(
  source: readonly unknown[],
  target: unknown[],
): EnvelopeCopy {
  return Object.freeze({ _tag: "EnvelopeCopy", source, target });
}

function envelopecopy_source(r: EnvelopeCopy): readonly unknown[] {
  return r.source;
}

function envelopecopy_target(r: EnvelopeCopy): unknown[] {
  return r.target;
}

// EnvelopeScan = EnvelopeScanAccepted | EnvelopeScanRejected
function EnvelopeScanAccepted(
  pending: readonly EnvelopeCopy[],
  targets: readonly unknown[][],
  objectCount: number,
  propertyCount: number,
): Extract<EnvelopeScan, { _tag: "EnvelopeScanAccepted" }> {
  return Object.freeze({
    _tag: "EnvelopeScanAccepted",
    pending: pending,
    targets: targets,
    objectCount: objectCount,
    propertyCount: propertyCount,
  });
}

function envelopescanaccepted_pending(
  r: Extract<EnvelopeScan, { _tag: "EnvelopeScanAccepted" }>,
): readonly EnvelopeCopy[] {
  return r.pending;
}

function envelopescanaccepted_targets(
  r: Extract<EnvelopeScan, { _tag: "EnvelopeScanAccepted" }>,
): readonly unknown[][] {
  return r.targets;
}

function envelopescanaccepted_objectCount(
  r: Extract<EnvelopeScan, { _tag: "EnvelopeScanAccepted" }>,
): number {
  return r.objectCount;
}

function envelopescanaccepted_propertyCount(
  r: Extract<EnvelopeScan, { _tag: "EnvelopeScanAccepted" }>,
): number {
  return r.propertyCount;
}
function EnvelopeScanRejected(): Extract<
  EnvelopeScan,
  { _tag: "EnvelopeScanRejected" }
> {
  return Object.freeze({ _tag: "EnvelopeScanRejected" });
}

const lifecycle_schema = "clause-cartridge-workbench/v1";

const envelope_measures = new WeakMap<object, EnvelopeMeasure>();

function empty_configuration(revision: number): InputConfiguration {
  return InputConfiguration(revision, Object.freeze([]));
}

function active_generation(active: ActiveRuntime | null): number {
  return active == null ? 0 : active.generation;
}

function active_revision(active: ActiveRuntime | null): unknown {
  return active == null ? null : active.revision;
}

function active_frame(active: ActiveRuntime | null): WorkbenchEnvelope | null {
  return active == null ? null : active.frame;
}

function settled_phase(active: ActiveRuntime | null): string {
  return active == null ? "idle" : "ready";
}

function runtime_session_token_p(
  value: unknown,
): value is object | ((...arguments_: never[]) => unknown) {
  const kind = typeof value;
  return (
    !(value == null) &&
    (equivalent(kind, "object") || equivalent(kind, "function"))
  );
}

function positive_safe_limit_p(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value > 0;
}

function require_workbench_policy(policy: WorkbenchPolicy): WorkbenchPolicy {
  const limits = policy.sequenceLimits;
  return positive_safe_limit_p(policy.maxPendingObservations) &&
    positive_safe_limit_p(policy.maxSessionIdentities) &&
    positive_safe_limit_p(policy.maxImmutableObjects) &&
    positive_safe_limit_p(policy.maxImmutableProperties) &&
    positive_safe_limit_p(policy.maxEnvelopeSourceUnits) &&
    !(limits == null) &&
    positive_safe_limit_p(limits.maxReceiptSequence) &&
    limits.maxReceiptSequence >= 2 &&
    positive_safe_limit_p(limits.maxInputSequence) &&
    positive_safe_limit_p(limits.maxGeneration) &&
    positive_safe_limit_p(limits.maxOperationId) &&
    positive_safe_limit_p(limits.maxConfigurationRevision)
    ? policy
    : (() => {
        throw new Error(
          "workbench policy limits must be positive safe integers",
        );
      })();
}

function envelope_array_length(
  policy: WorkbenchPolicy,
  source: unknown,
): number {
  if (Array.isArray(source)) {
    const length = source.length;
    return ((_truthy) => _truthy !== false && _truthy != null)(
      ((_logical) =>
        _logical !== false && _logical != null
          ? length >= 0 && length <= policy.maxImmutableProperties
          : _logical)(Number.isSafeInteger(length)),
    )
      ? length
      : -1;
  } else {
    return -1;
  }
}

function create_envelope_target(length: number): unknown[] {
  const target = new Array(length);
  Object.setPrototypeOf(target, null);
  return target;
}

function define_envelope_index(
  target: unknown[],
  index: number,
  value: unknown,
): unknown[] {
  return Object.defineProperty(target, index, {
    configurable: true,
    enumerable: true,
    value: value,
    writable: true,
  });
}

function envelope_primitive_p(
  value: unknown,
  value_kind: string,
): value is EnvelopePrimitive {
  return (
    value == null ||
    equivalent(value_kind, "string") ||
    equivalent(value_kind, "boolean") ||
    (typeof value === "number" && Number.isFinite(value))
  );
}

function scan_envelope_copy(
  policy: WorkbenchPolicy,
  copy: EnvelopeCopy,
  copies: Map<readonly unknown[], unknown[]>,
  pending: readonly EnvelopeCopy[],
  targets: readonly unknown[][],
  object_count: number,
  property_count: number,
): EnvelopeScan {
  const source = copy.source;
  const target = copy.target;
  let next_pending = pending;
  let next_targets = targets;
  let next_object_count = object_count;
  let next_property_count = property_count;

  for (let index = 0; index < target.length; index += 1) {
    const descriptor = Object.getOwnPropertyDescriptor(source, index);
    if (descriptor == null || !Object.hasOwn(descriptor, "value"))
      return EnvelopeScanRejected();

    const child: unknown = descriptor.value;
    if (Array.isArray(child)) {
      const existing = copies.get(child);
      if (existing !== undefined) {
        define_envelope_index(target, index, existing);
        continue;
      }

      const child_length = envelope_array_length(policy, child);
      const candidate_property_count = next_property_count + child_length;
      if (
        child_length < 0 ||
        next_object_count >= policy.maxImmutableObjects ||
        candidate_property_count > policy.maxImmutableProperties
      ) {
        return EnvelopeScanRejected();
      }

      const child_target = create_envelope_target(child_length);
      copies.set(child, child_target);
      define_envelope_index(target, index, child_target);
      next_pending = appendValue(
        next_pending,
        EnvelopeCopy(child, child_target),
      );
      next_targets = appendValue(next_targets, child_target);
      next_object_count += 1;
      next_property_count = candidate_property_count;
      continue;
    }

    if (!envelope_primitive_p(child, typeof child))
      return EnvelopeScanRejected();
    define_envelope_index(target, index, child);
  }

  return EnvelopeScanAccepted(
    next_pending,
    next_targets,
    next_object_count,
    next_property_count,
  );
}

function create_workbench_envelope(
  incomingPolicy: WorkbenchPolicy,
  sourceText: unknown,
): WorkbenchEnvelope {
  const policy = require_workbench_policy(incomingPolicy);
  if (
    typeof sourceText !== "string" ||
    sourceText.length > policy.maxEnvelopeSourceUnits
  ) {
    throw new Error("workbench envelope source exceeds its policy");
  }

  const source: unknown = JSON.parse(sourceText);
  const root_length = envelope_array_length(policy, source);
  if (!Array.isArray(source) || root_length < 0) {
    throw new Error("workbench envelope source exceeds its policy");
  }

  const root = create_envelope_target(root_length);
  const copies = new Map<readonly unknown[], unknown[]>([[source, root]]);
  let pending: readonly EnvelopeCopy[] = [EnvelopeCopy(source, root)];
  let targets: readonly unknown[][] = [root];
  let object_count = 1;
  let property_count = root_length;

  while (!isEmpty(pending)) {
    const copy = pending[pending.length - 1];
    if (copy === undefined)
      throw new Error("workbench envelope source exceeds its policy");
    const scan = scan_envelope_copy(
      policy,
      copy,
      copies,
      pending.slice(0, -1),
      targets,
      object_count,
      property_count,
    );
    if (scan._tag === "EnvelopeScanRejected") {
      throw new Error("workbench envelope source exceeds its policy");
    }
    pending = scan.pending;
    targets = scan.targets;
    object_count = scan.objectCount;
    property_count = scan.propertyCount;
  }

  targets.forEach(Object.freeze);
  envelope_measures.set(
    root,
    EnvelopeMeasure(object_count, property_count, sourceText.length),
  );
  return root;
}

function immutable_envelope_p(
  policy: WorkbenchPolicy,
  value: unknown,
): value is WorkbenchEnvelope {
  if (!Array.isArray(value)) return false;
  const measure = envelope_measures.get(value);
  return (
    measure !== undefined &&
    measure.objectCount <= policy.maxImmutableObjects &&
    measure.propertyCount <= policy.maxImmutableProperties &&
    measure.sourceUnitCount <= policy.maxEnvelopeSourceUnits &&
    Object.getPrototypeOf(value) == null
  );
}

function require_immutable_input(
  policy: WorkbenchPolicy,
  value: unknown,
): WorkbenchEnvelope {
  return immutable_envelope_p(policy, value)
    ? value
    : (() => {
        throw new Error(
          "workbench input observations require a checked immutable envelope",
        );
      })();
}

function create_cartridge_workbench_bang(
  port: CartridgePort,
  fixedTick: FixedTick,
  incomingPolicy: WorkbenchPolicy,
  scheduleFixedTick: (
    milliseconds: number,
    tick: () => unknown,
  ) => () => unknown,
  renderFrame: (frame: WorkbenchEnvelope) => unknown,
  emitReceipt: (receipt: LifecycleReceipt) => unknown,
  initialPackageCandidate: unknown,
): CartridgeWorkbench {
  const policy = require_workbench_policy(incomingPolicy);
  const sequence_limits = policy.sequenceLimits;
  const state: Cell<WorkbenchState> = {
    value: WorkbenchState("idle", null, 0, empty_configuration(0), 0, false),
    watches: {},
  };
  const receipt_sequence: Cell<number> = { value: 0, watches: {} };
  const input_sequence: Cell<number> = { value: 0, watches: {} };
  const generation_sequence: Cell<number> = { value: 0, watches: {} };
  const operation_sequence: Cell<number> = { value: 0, watches: {} };
  const disposed_sessions = new Set<unknown>();
  const retirement_reservations: Cell<number> = { value: 0, watches: {} };
  const pending_input_transitions: Cell<number> = { value: 0, watches: {} };
  const input_rejection_pending: Cell<boolean> = { value: false, watches: {} };
  const tick_transition_pending: Cell<boolean> = { value: false, watches: {} };
  const reload_transition_pending: Cell<boolean> = {
    value: false,
    watches: {},
  };
  const dispose_transition_pending: Cell<boolean> = {
    value: false,
    watches: {},
  };
  const transition_queue: Cell<WorkbenchTransition[]> = {
    value: [],
    watches: {},
  };
  const dispatching: Cell<boolean> = { value: false, watches: {} };
  const ticks_enabled: Cell<boolean> = { value: false, watches: {} };
  const cancel_fixed_tick: Cell<() => unknown> = {
    value: () => null,
    watches: {},
  };
  return (() => {
    function increment_capacity_p(
      current: number,
      needed: number,
      maximum: number,
    ) {
      return needed <= maximum - current;
    }
    function normal_receipt_capacity_p(needed: number) {
      return (
        needed <=
        sequence_limits.maxReceiptSequence - receipt_sequence.value - 1
      );
    }
    function capacity_failure(
      receipts: number,
      inputs: number,
      generations: number,
      operations: number,
      configurations: number,
    ) {
      return !normal_receipt_capacity_p(receipts)
        ? "receipt-sequence"
        : !increment_capacity_p(
              input_sequence.value,
              inputs,
              sequence_limits.maxInputSequence,
            )
          ? "input-sequence"
          : !increment_capacity_p(
                generation_sequence.value,
                generations,
                sequence_limits.maxGeneration,
              )
            ? "generation-sequence"
            : !increment_capacity_p(
                  operation_sequence.value,
                  operations,
                  sequence_limits.maxOperationId,
                )
              ? "operation-sequence"
              : !increment_capacity_p(
                    state.value.configuration.revision,
                    configurations,
                    sequence_limits.maxConfigurationRevision,
                  )
                ? "configuration-revision"
                : null;
    }
    function counter_exhausted_with_session_bang(
      domain: string,
      uninstalled_session: unknown,
    ) {
      const current = state.value;
      if (!current.disposed) {
        const active = current.active;
        const generation = active_generation(active);
        const cancel = cancel_fixed_tick.value;
        (() => {
          const _a = state,
            _v = WorkbenchState(
              "disposed",
              active,
              0,
              current.configuration,
              current.latestReload,
              true,
            );
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        (() => {
          const _a = ticks_enabled,
            _v = false;
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        (() => {
          const _a = cancel_fixed_tick,
            _v = () => null;
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        const cancel_uncertain = (() => {
          try {
            cancel();
            return false;
          } catch (_catch_1) {
            switch (classifyError(_catch_1)) {
              case 0: {
                const __error = _catch_1;
                return true;
                break;
              }
            }
          }
        })();
        const active_disposal_uncertain =
          active == null ? false : dispose_session_once_bang(active.session);
        const uninstalled_disposal_uncertain =
          uninstalled_session == null
            ? false
            : dispose_session_once_bang(uninstalled_session);
        const disposal_uncertain =
          active_disposal_uncertain || uninstalled_disposal_uncertain;
        const sequence = (() => {
          const _a = receipt_sequence;
          const _old = _a.value;
          _a.value = ((_x) => _x + 1)(_old);
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value);
          return _a.value;
        })();
        const detail =
          cancel_uncertain && disposal_uncertain
            ? concatenate(domain, ":cancel-and-disposal-uncertain")
            : cancel_uncertain
              ? concatenate(domain, ":cancel-uncertain")
              : disposal_uncertain
                ? concatenate(domain, ":disposal-uncertain")
                : domain;
        const receipt = LifecycleReceipt(
          lifecycle_schema,
          sequence,
          "counter-exhausted",
          "disposed",
          generation,
          generation,
          0,
          current.configuration.revision,
          active_revision(active),
          detail,
        );
        (() => {
          try {
            return emitReceipt(receipt);
          } catch (_catch_2) {
            switch (classifyError(_catch_2)) {
              case 0: {
                const __error = _catch_2;
                return null;
                break;
              }
            }
          }
        })();
        (() => {
          const _a = state,
            _v = WorkbenchState(
              "disposed",
              null,
              0,
              current.configuration,
              current.latestReload,
              true,
            );
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        return maybe_release_retired_sessions_bang();
      }
    }
    function counter_exhausted_bang(domain: string) {
      return counter_exhausted_with_session_bang(domain, null);
    }
    function ensure_capacity_bang(
      receipts: number,
      inputs: number,
      generations: number,
      operations: number,
      configurations: number,
    ) {
      const failure = capacity_failure(
        receipts,
        inputs,
        generations,
        operations,
        configurations,
      );
      if (failure == null) {
        return true;
      } else {
        counter_exhausted_bang(failure);
        return false;
      }
    }
    function ensure_capacity_retiring_bang(
      receipts: number,
      inputs: number,
      generations: number,
      operations: number,
      configurations: number,
      uninstalled_session: unknown,
    ) {
      const failure = capacity_failure(
        receipts,
        inputs,
        generations,
        operations,
        configurations,
      );
      if (failure == null) {
        return true;
      } else {
        counter_exhausted_with_session_bang(failure, uninstalled_session);
        return false;
      }
    }
    function emit_lifecycle_for_bang(
      event: string,
      operation_generation: number,
      operation_id: number,
      detail: string,
    ) {
      if (normal_receipt_capacity_p(1)) {
        const current = state.value;
        const active = current.active;
        const sequence = (() => {
          const _a = receipt_sequence;
          const _old = _a.value;
          _a.value = ((_x) => _x + 1)(_old);
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value);
          return _a.value;
        })();
        const receipt = LifecycleReceipt(
          lifecycle_schema,
          sequence,
          event,
          current.phase,
          active_generation(active),
          operation_generation,
          operation_id,
          current.configuration.revision,
          active_revision(active),
          detail,
        );
        return (() => {
          try {
            return emitReceipt(receipt);
          } catch (_catch_3) {
            switch (classifyError(_catch_3)) {
              case 0: {
                const __error = _catch_3;
                return null;
                break;
              }
            }
          }
        })();
      } else {
        return counter_exhausted_bang("receipt-sequence");
      }
    }
    function emit_lifecycle_bang(
      event: string,
      operation_generation: number,
      detail: string,
    ) {
      return emit_lifecycle_for_bang(
        event,
        operation_generation,
        state.value.operationId,
        detail,
      );
    }
    function one_shot_transition_bang() {
      const completed: Cell<boolean> = { value: false, watches: {} };
      return (transition: WorkbenchTransition) => {
        if (completed.value) {
          return false;
        } else {
          (() => {
            const _a = completed,
              _v = true;
            const _old = _a.value;
            _a.value = _v;
            for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
            return _v;
          })();
          dispatch_bang(transition);
          return true;
        }
      };
    }
    function maybe_release_retired_sessions_bang() {
      if (
        state.value.disposed &&
        equivalent(0, retirement_reservations.value)
      ) {
        return disposed_sessions.clear();
      }
    }
    function dispose_session_once_bang(session: unknown) {
      if (
        runtime_session_token_p(session) &&
        !((_truthy) => _truthy !== false && _truthy != null)(
          disposed_sessions.has(session),
        )
      ) {
        if (disposed_sessions.size >= policy.maxSessionIdentities) {
          (() => {
            throw new Error("retired RuntimeSession identity bound exceeded");
          })();
        }
        disposed_sessions.add(session);
        return (() => {
          try {
            port.disposeSession(session);
            return false;
          } catch (_catch_4) {
            switch (classifyError(_catch_4)) {
              case 0: {
                const __error = _catch_4;
                return true;
                break;
              }
            }
          }
        })();
      } else {
        return false;
      }
    }
    function restore_settled_phase_bang() {
      const current = state.value;
      const active = current.active;
      return (() => {
        const _a = state,
          _v = WorkbenchState(
            settled_phase(active),
            active,
            0,
            current.configuration,
            current.latestReload,
            current.disposed,
          );
        const _old = _a.value;
        _a.value = _v;
        for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
        return _v;
      })();
    }
    function install_authority_bang(
      generation: number,
      session: unknown,
      revision: unknown,
      frame: WorkbenchEnvelope,
      reset_configuration: boolean,
      uninstalled_session: unknown,
      accepted_event: string,
      accepted_detail: string,
    ) {
      const configuration_increment = reset_configuration ? 1 : 0;
      if (
        ensure_capacity_retiring_bang(
          3,
          0,
          0,
          0,
          configuration_increment,
          uninstalled_session,
        )
      ) {
        const current = state.value;
        const operation_id = current.operationId;
        const previous = current.active;
        const previous_session = previous == null ? null : previous.session;
        const configuration = current.configuration;
        const next_configuration = reset_configuration
          ? empty_configuration(configuration.revision + 1)
          : configuration;
        const next_active = ActiveRuntime(generation, session, revision, frame);
        (() => {
          const _a = state,
            _v = WorkbenchState(
              "successor",
              next_active,
              operation_id,
              next_configuration,
              current.latestReload,
              false,
            );
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        emit_lifecycle_bang(accepted_event, generation, accepted_detail);
        emit_lifecycle_bang(
          "successor-installed",
          generation,
          "port-successor",
        );
        (() => {
          const _a = state,
            _v = WorkbenchState(
              "render",
              next_active,
              operation_id,
              next_configuration,
              current.latestReload,
              false,
            );
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        const render_event = (() => {
          try {
            renderFrame(frame);
            return "frame-rendered";
          } catch (_catch_5) {
            switch (classifyError(_catch_5)) {
              case 0: {
                const __error = _catch_5;
                return "frame-render-failed";
                break;
              }
            }
          }
        })();
        (() => {
          const _a = state,
            _v = WorkbenchState(
              "ready",
              next_active,
              0,
              next_configuration,
              current.latestReload,
              false,
            );
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        const disposal_uncertain =
          !(previous_session == null) && !(previous_session === session)
            ? dispose_session_once_bang(previous_session)
            : false;
        const detail = disposal_uncertain
          ? "prior-session-disposal-uncertain"
          : equivalent(render_event, "frame-rendered")
            ? "port-frame"
            : "renderer-threw";
        return emit_lifecycle_for_bang(
          render_event,
          generation,
          operation_id,
          detail,
        );
      }
    }
    function stale_completion_bang(
      operation_generation: number,
      operation_id: number,
      detail: string,
    ) {
      if (!state.value.disposed && ensure_capacity_bang(1, 0, 0, 0, 0)) {
        return emit_lifecycle_for_bang(
          "completion-stale",
          operation_generation,
          operation_id,
          detail,
        );
      }
    }
    function apply_transition_bang(transition: WorkbenchTransition) {
      return (() => {
        const _match_1 = transition;
        if (_match_1._tag === "ObserveInput") {
          const value = _match_1.value;
          return (() => {
            const __remaining_input_transitions = (() => {
              const _a = pending_input_transitions;
              const _old = _a.value;
              _a.value = ((_x) => _x - 1)(_old);
              for (const _k in _a.watches)
                _a.watches[_k](_k, _a, _old, _a.value);
              return _a.value;
            })();
            const current = state.value;
            if (!current.disposed && ensure_capacity_bang(1, 1, 0, 0, 1)) {
              const configuration = current.configuration;
              const observation_sequence = (() => {
                const _a = input_sequence;
                const _old = _a.value;
                _a.value = ((_x) => _x + 1)(_old);
                for (const _k in _a.watches)
                  _a.watches[_k](_k, _a, _old, _a.value);
                return _a.value;
              })();
              const observation = InputObservation(observation_sequence, value);
              const observations = Object.freeze(
                appendValue(configuration.observations, observation),
              );
              const next_configuration = InputConfiguration(
                configuration.revision + 1,
                observations,
              );
              (() => {
                const _a = state,
                  _v = WorkbenchState(
                    current.phase,
                    current.active,
                    current.operationId,
                    next_configuration,
                    current.latestReload,
                    false,
                  );
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
              return emit_lifecycle_bang(
                "configuration-observed",
                active_generation(current.active),
                "input",
              );
            }
          })();
        } else if (_match_1._tag === "InputRejected") {
          return (() => {
            try {
              const current = state.value;
              if (!current.disposed && ensure_capacity_bang(1, 0, 0, 0, 0)) {
                return emit_lifecycle_bang(
                  "configuration-input-rejected",
                  active_generation(current.active),
                  "pending-observation-limit",
                );
              }
            } finally {
              (() => {
                const _a = input_rejection_pending,
                  _v = false;
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
            }
          })();
        } else if (_match_1._tag === "FixedTickElapsed") {
          return (() => {
            (() => {
              const _a = tick_transition_pending,
                _v = false;
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
            const current = state.value;
            const active = current.active;
            if (
              !current.disposed &&
              equivalent(current.phase, "ready") &&
              !(active == null) &&
              ensure_capacity_bang(1, 0, 0, 1, 1)
            ) {
              const generation = active.generation;
              const operation_id = (() => {
                const _a = operation_sequence;
                const _old = _a.value;
                _a.value = ((_x) => _x + 1)(_old);
                for (const _k in _a.watches)
                  _a.watches[_k](_k, _a, _old, _a.value);
                return _a.value;
              })();
              const configuration = current.configuration;
              (() => {
                const _a = state,
                  _v = WorkbenchState(
                    "candidate",
                    active,
                    operation_id,
                    empty_configuration(configuration.revision + 1),
                    current.latestReload,
                    false,
                  );
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
              emit_lifecycle_bang(
                "candidate-requested",
                generation,
                "fixed-tick",
              );
              const settle = one_shot_transition_bang();
              return (() => {
                try {
                  return port.runCandidate(
                    active.session,
                    fixedTick,
                    configuration,
                    (completion) =>
                      settle(
                        CandidateCompleted(
                          generation,
                          operation_id,
                          completion,
                        ),
                      ),
                  );
                } catch (_catch_6) {
                  switch (classifyError(_catch_6)) {
                    case 0: {
                      const __error = _catch_6;
                      return settle(
                        CandidateBoundaryFailed(generation, operation_id),
                      );
                      break;
                    }
                  }
                }
              })();
            }
          })();
        } else if (_match_1._tag === "CandidateCompleted") {
          const generation = _match_1.generation;
          const operation_id = _match_1.operationId;
          const completion = _match_1.completion;
          return (() => {
            const current = state.value;
            const active = current.active;
            return current.disposed ||
              active == null ||
              !(generation === active.generation) ||
              !(operation_id === current.operationId) ||
              !equivalent(current.phase, "candidate")
              ? stale_completion_bang(generation, operation_id, "candidate")
              : (() => {
                  const _match_2 = completion;
                  if (_match_2._tag === "CandidateFailed") {
                    const reason = _match_2.reason;
                    return ensure_capacity_bang(1, 0, 0, 0, 0)
                      ? (() => {
                          emit_lifecycle_bang(
                            "candidate-failed",
                            generation,
                            reason,
                          );
                          return restore_settled_phase_bang();
                        })()
                      : null;
                  } else if (_match_2._tag === "CandidateProduced") {
                    const candidate = _match_2.candidate;
                    return ensure_capacity_bang(2, 0, 0, 0, 0)
                      ? (() => {
                          (() => {
                            const _a = state,
                              _v = WorkbenchState(
                                "admission",
                                active,
                                operation_id,
                                current.configuration,
                                current.latestReload,
                                false,
                              );
                            const _old = _a.value;
                            _a.value = _v;
                            for (const _k in _a.watches)
                              _a.watches[_k](_k, _a, _old, _v);
                            return _v;
                          })();
                          emit_lifecycle_bang(
                            "candidate-produced",
                            generation,
                            "candidate",
                          );
                          emit_lifecycle_bang(
                            "admission-requested",
                            generation,
                            "candidate",
                          );
                          const settle = one_shot_transition_bang();
                          return (() => {
                            try {
                              return port.requestAdmission(
                                active.session,
                                candidate,
                                (result) =>
                                  settle(
                                    AdmissionCompleted(
                                      generation,
                                      operation_id,
                                      result,
                                    ),
                                  ),
                              );
                            } catch (_catch_7) {
                              switch (classifyError(_catch_7)) {
                                case 0: {
                                  const __error = _catch_7;
                                  return settle(
                                    AdmissionBoundaryFailed(
                                      generation,
                                      operation_id,
                                    ),
                                  );
                                  break;
                                }
                              }
                            }
                          })();
                        })()
                      : null;
                  } else {
                    return null;
                  }
                })();
          })();
        } else if (_match_1._tag === "CandidateBoundaryFailed") {
          const generation = _match_1.generation;
          const operation_id = _match_1.operationId;
          return (() => {
            const current = state.value;
            const active = current.active;
            if (
              current.disposed ||
              active == null ||
              !(generation === active.generation) ||
              !(operation_id === current.operationId) ||
              !equivalent(current.phase, "candidate")
            ) {
              return stale_completion_bang(
                generation,
                operation_id,
                "candidate-boundary",
              );
            } else {
              if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
                emit_lifecycle_for_bang(
                  "candidate-boundary-failed",
                  generation,
                  operation_id,
                  "port-threw",
                );
                return restore_settled_phase_bang();
              }
            }
          })();
        } else if (_match_1._tag === "AdmissionCompleted") {
          const generation = _match_1.generation;
          const operation_id = _match_1.operationId;
          const completion = _match_1.completion;
          return (() => {
            const current = state.value;
            const active = current.active;
            return current.disposed ||
              active == null ||
              !(generation === active.generation) ||
              !(operation_id === current.operationId) ||
              !equivalent(current.phase, "admission")
              ? stale_completion_bang(generation, operation_id, "admission")
              : (() => {
                  const _match_3 = completion;
                  if (_match_3._tag === "AdmissionRejected") {
                    const reason = _match_3.reason;
                    return ensure_capacity_bang(1, 0, 0, 0, 0)
                      ? (() => {
                          emit_lifecycle_bang(
                            "admission-rejected",
                            generation,
                            reason,
                          );
                          return restore_settled_phase_bang();
                        })()
                      : null;
                  } else if (_match_3._tag === "AdmissionAccepted") {
                    const __successor = _match_3.successor;
                    const revision = _match_3.revision;
                    const frame = _match_3.frame;
                    return !immutable_envelope_p(policy, frame)
                      ? ensure_capacity_bang(1, 0, 0, 0, 0)
                        ? (() => {
                            emit_lifecycle_bang(
                              "admission-frame-rejected",
                              generation,
                              "frame-not-deeply-immutable",
                            );
                            return restore_settled_phase_bang();
                          })()
                        : null
                      : install_authority_bang(
                          generation,
                          active.session,
                          revision,
                          frame,
                          false,
                          null,
                          "admission-accepted",
                          "successor",
                        );
                  } else {
                    return null;
                  }
                })();
          })();
        } else if (_match_1._tag === "AdmissionBoundaryFailed") {
          const generation = _match_1.generation;
          const operation_id = _match_1.operationId;
          return (() => {
            const current = state.value;
            const active = current.active;
            if (
              current.disposed ||
              active == null ||
              !(generation === active.generation) ||
              !(operation_id === current.operationId) ||
              !equivalent(current.phase, "admission")
            ) {
              return stale_completion_bang(
                generation,
                operation_id,
                "admission-boundary",
              );
            } else {
              if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
                emit_lifecycle_for_bang(
                  "admission-boundary-failed",
                  generation,
                  operation_id,
                  "port-threw",
                );
                return restore_settled_phase_bang();
              }
            }
          })();
        } else if (_match_1._tag === "ReloadRequested") {
          const package_candidate = _match_1.packageCandidate;
          return (() => {
            (() => {
              const _a = reload_transition_pending,
                _v = false;
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
            const current = state.value;
            return current.disposed
              ? null
              : equivalent(current.phase, "session-start")
                ? ensure_capacity_bang(1, 0, 0, 0, 0)
                  ? (() => {
                      return emit_lifecycle_bang(
                        "reload-rejected",
                        current.latestReload,
                        "runtime-session-start-pending",
                      );
                    })()
                  : null
                : ensure_capacity_bang(1, 0, 1, 0, 0)
                  ? (() => {
                      const generation = (() => {
                        const _a = generation_sequence;
                        const _old = _a.value;
                        _a.value = ((_x) => _x + 1)(_old);
                        for (const _k in _a.watches)
                          _a.watches[_k](_k, _a, _old, _a.value);
                        return _a.value;
                      })();
                      (() => {
                        const _a = state,
                          _v = WorkbenchState(
                            "package-check",
                            current.active,
                            0,
                            current.configuration,
                            generation,
                            false,
                          );
                        const _old = _a.value;
                        _a.value = _v;
                        for (const _k in _a.watches)
                          _a.watches[_k](_k, _a, _old, _v);
                        return _v;
                      })();
                      emit_lifecycle_bang(
                        "reload-requested",
                        generation,
                        "package-candidate",
                      );
                      const settle = one_shot_transition_bang();
                      return (() => {
                        try {
                          return port.acceptPackage(
                            package_candidate,
                            (completion) =>
                              settle(PackageCompleted(generation, completion)),
                          );
                        } catch (_catch_8) {
                          switch (classifyError(_catch_8)) {
                            case 0: {
                              const __error = _catch_8;
                              return settle(PackageBoundaryFailed(generation));
                              break;
                            }
                          }
                        }
                      })();
                    })()
                  : null;
          })();
        } else if (_match_1._tag === "PackageCompleted") {
          const generation = _match_1.generation;
          const completion = _match_1.completion;
          return (() => {
            const current = state.value;
            return current.disposed ||
              !(generation === current.latestReload) ||
              !equivalent(current.phase, "package-check")
              ? stale_completion_bang(generation, 0, "package")
              : (() => {
                  const _match_4 = completion;
                  if (_match_4._tag === "PackageRejected") {
                    const reason = _match_4.reason;
                    return ensure_capacity_bang(1, 0, 0, 0, 0)
                      ? (() => {
                          restore_settled_phase_bang();
                          return emit_lifecycle_bang(
                            "package-rejected",
                            generation,
                            reason,
                          );
                        })()
                      : null;
                  } else if (_match_4._tag === "PackageAccepted") {
                    const accepted_package = _match_4.acceptedPackage;
                    return (() => {
                      const active = current.active;
                      const active_count = active == null ? 0 : 1;
                      const retired_count = disposed_sessions.size;
                      const used_identities =
                        retired_count +
                        active_count +
                        retirement_reservations.value;
                      if (used_identities >= policy.maxSessionIdentities) {
                        if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
                          emit_lifecycle_bang(
                            "session-identity-limit",
                            generation,
                            "reload-rejected",
                          );
                          return restore_settled_phase_bang();
                        }
                      } else {
                        if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
                          const prior_session =
                            active == null ? null : active.session;
                          (() => {
                            const _a = state,
                              _v = WorkbenchState(
                                "session-start",
                                active,
                                0,
                                current.configuration,
                                generation,
                                false,
                              );
                            const _old = _a.value;
                            _a.value = _v;
                            for (const _k in _a.watches)
                              _a.watches[_k](_k, _a, _old, _v);
                            return _v;
                          })();
                          emit_lifecycle_bang(
                            "package-accepted",
                            generation,
                            "accepted-package",
                          );
                          (() => {
                            const _a = retirement_reservations;
                            const _old = _a.value;
                            _a.value = ((_x) => _x + 1)(_old);
                            for (const _k in _a.watches)
                              _a.watches[_k](_k, _a, _old, _a.value);
                            return _a.value;
                          })();
                          const callback_completed: Cell<boolean> = {
                            value: false,
                            watches: {},
                          };
                          const boundary_reported: Cell<boolean> = {
                            value: false,
                            watches: {},
                          };
                          const complete_session_bang = (
                            result: SessionCompletion,
                          ) => {
                            if (callback_completed.value) {
                              return false;
                            } else {
                              (() => {
                                const _a = callback_completed,
                                  _v = true;
                                const _old = _a.value;
                                _a.value = _v;
                                for (const _k in _a.watches)
                                  _a.watches[_k](_k, _a, _old, _v);
                                return _v;
                              })();
                              dispatch_bang(
                                SessionCompleted(
                                  generation,
                                  prior_session,
                                  result,
                                ),
                              );
                              return true;
                            }
                          };
                          return (() => {
                            try {
                              return port.startSession(
                                accepted_package,
                                generation,
                                complete_session_bang,
                              );
                            } catch (_catch_9) {
                              switch (classifyError(_catch_9)) {
                                case 0: {
                                  const __error = _catch_9;
                                  if (
                                    !callback_completed.value &&
                                    !boundary_reported.value
                                  ) {
                                    (() => {
                                      const _a = boundary_reported,
                                        _v = true;
                                      const _old = _a.value;
                                      _a.value = _v;
                                      for (const _k in _a.watches)
                                        _a.watches[_k](_k, _a, _old, _v);
                                      return _v;
                                    })();
                                    return dispatch_bang(
                                      SessionBoundaryFailed(
                                        generation,
                                        prior_session,
                                      ),
                                    );
                                  }
                                  break;
                                }
                              }
                            }
                          })();
                        }
                      }
                    })();
                  } else {
                    return null;
                  }
                })();
          })();
        } else if (_match_1._tag === "PackageBoundaryFailed") {
          const generation = _match_1.generation;
          return (() => {
            const current = state.value;
            if (
              current.disposed ||
              !(generation === current.latestReload) ||
              !equivalent(current.phase, "package-check")
            ) {
              return stale_completion_bang(generation, 0, "package-boundary");
            } else {
              if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
                restore_settled_phase_bang();
                return emit_lifecycle_for_bang(
                  "package-boundary-failed",
                  generation,
                  0,
                  "port-threw",
                );
              }
            }
          })();
        } else if (_match_1._tag === "ReloadRejected") {
          return (() => {
            try {
              const current = state.value;
              if (!current.disposed && ensure_capacity_bang(1, 0, 0, 0, 0)) {
                return emit_lifecycle_bang(
                  "reload-rejected",
                  current.latestReload,
                  "runtime-session-start-pending",
                );
              }
            } finally {
              (() => {
                const _a = reload_transition_pending,
                  _v = false;
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
            }
          })();
        } else if (_match_1._tag === "SessionCompleted") {
          const generation = _match_1.generation;
          const prior_session = _match_1.priorSession;
          const completion = _match_1.completion;
          return (() => {
            const __remaining_reservations = (() => {
              const _a = retirement_reservations;
              const _old = _a.value;
              _a.value = ((_x) => _x - 1)(_old);
              for (const _k in _a.watches)
                _a.watches[_k](_k, _a, _old, _a.value);
              return _a.value;
            })();
            const current = state.value;
            const current_completion =
              !current.disposed &&
              generation === current.latestReload &&
              equivalent(current.phase, "session-start");
            if (!current_completion) {
              const can_emit = ensure_capacity_bang(1, 0, 0, 0, 0);
              const disposal_uncertain = (() => {
                const _match_5 = completion;
                if (_match_5._tag === "SessionStarted") {
                  const session = _match_5.session;
                  const __revision = _match_5.revision;
                  const __frame = _match_5.frame;
                  return (() => {
                    const active = current.active;
                    const current_session =
                      active == null ? null : active.session;
                    return runtime_session_token_p(session) &&
                      !(session === prior_session) &&
                      !(session === current_session) &&
                      !((_truthy) => _truthy !== false && _truthy != null)(
                        disposed_sessions.has(session),
                      )
                      ? dispose_session_once_bang(session)
                      : false;
                  })();
                } else if (_match_5._tag === "SessionFailed") {
                  const __reason = _match_5.reason;
                  return false;
                } else {
                  return null;
                }
              })();
              if (can_emit) {
                emit_lifecycle_for_bang(
                  "completion-stale",
                  generation,
                  0,
                  disposal_uncertain ? "session-disposal-uncertain" : "session",
                );
              }
              return maybe_release_retired_sessions_bang();
            } else {
              return (() => {
                const _match_6 = completion;
                if (_match_6._tag === "SessionFailed") {
                  const reason = _match_6.reason;
                  return ensure_capacity_bang(1, 0, 0, 0, 0)
                    ? (() => {
                        restore_settled_phase_bang();
                        return emit_lifecycle_bang(
                          "session-failed",
                          generation,
                          reason,
                        );
                      })()
                    : null;
                } else if (_match_6._tag === "SessionStarted") {
                  const session = _match_6.session;
                  const revision = _match_6.revision;
                  const frame = _match_6.frame;
                  return (() => {
                    const previous = current.active;
                    const previous_session =
                      previous == null ? null : previous.session;
                    return !runtime_session_token_p(session)
                      ? ensure_capacity_bang(1, 0, 0, 0, 0)
                        ? (() => {
                            restore_settled_phase_bang();
                            return emit_lifecycle_bang(
                              "session-invalid",
                              generation,
                              "runtime-session-reference-required",
                            );
                          })()
                        : null
                      : ((_truthy) => _truthy !== false && _truthy != null)(
                            disposed_sessions.has(session),
                          )
                        ? ensure_capacity_bang(1, 0, 0, 0, 0)
                          ? (() => {
                              restore_settled_phase_bang();
                              return emit_lifecycle_bang(
                                "session-retired",
                                generation,
                                "retired-runtime-session-rejected",
                              );
                            })()
                          : null
                        : !(previous_session == null) &&
                            previous_session === session
                          ? ensure_capacity_bang(1, 0, 0, 0, 0)
                            ? (() => {
                                restore_settled_phase_bang();
                                return emit_lifecycle_bang(
                                  "session-reused",
                                  generation,
                                  "fresh-runtime-session-required",
                                );
                              })()
                            : null
                          : !immutable_envelope_p(policy, frame)
                            ? ensure_capacity_retiring_bang(
                                1,
                                0,
                                0,
                                0,
                                0,
                                session,
                              )
                              ? (() => {
                                  const disposal_uncertain =
                                    dispose_session_once_bang(session);
                                  restore_settled_phase_bang();
                                  return emit_lifecycle_bang(
                                    "session-frame-rejected",
                                    generation,
                                    disposal_uncertain
                                      ? "session-disposal-uncertain"
                                      : "frame-not-deeply-immutable",
                                  );
                                })()
                              : null
                            : install_authority_bang(
                                generation,
                                session,
                                revision,
                                frame,
                                true,
                                session,
                                "session-started",
                                "fresh-runtime-session",
                              );
                  })();
                } else {
                  return null;
                }
              })();
            }
          })();
        } else if (_match_1._tag === "SessionBoundaryFailed") {
          const generation = _match_1.generation;
          const __prior_session = _match_1.priorSession;
          return (() => {
            const current = state.value;
            const current_completion =
              !current.disposed &&
              generation === current.latestReload &&
              equivalent(current.phase, "session-start");
            if (current_completion) {
              if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
                restore_settled_phase_bang();
                emit_lifecycle_for_bang(
                  "session-boundary-failed",
                  generation,
                  0,
                  "port-threw",
                );
              }
            } else {
              stale_completion_bang(generation, 0, "session-boundary");
            }
            return maybe_release_retired_sessions_bang();
          })();
        } else if (_match_1._tag === "DisposeRequested") {
          return (() => {
            (() => {
              const _a = dispose_transition_pending,
                _v = false;
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
            const current = state.value;
            if (!current.disposed && ensure_capacity_bang(1, 0, 0, 0, 0)) {
              const active = current.active;
              const generation = active_generation(active);
              const cancel = cancel_fixed_tick.value;
              (() => {
                const _a = state,
                  _v = WorkbenchState(
                    "disposed",
                    active,
                    0,
                    current.configuration,
                    current.latestReload,
                    true,
                  );
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
              (() => {
                const _a = ticks_enabled,
                  _v = false;
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
              (() => {
                const _a = cancel_fixed_tick,
                  _v = () => null;
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
              const cancel_uncertain = (() => {
                try {
                  cancel();
                  return false;
                } catch (_catch_10) {
                  switch (classifyError(_catch_10)) {
                    case 0: {
                      const __error = _catch_10;
                      return true;
                      break;
                    }
                  }
                }
              })();
              const disposal_uncertain =
                active == null
                  ? false
                  : dispose_session_once_bang(active.session);
              const detail =
                cancel_uncertain && disposal_uncertain
                  ? "terminal-cancel-and-disposal-uncertain"
                  : cancel_uncertain
                    ? "terminal-cancel-uncertain"
                    : disposal_uncertain
                      ? "terminal-disposal-uncertain"
                      : "terminal";
              emit_lifecycle_bang("disposed", generation, detail);
              (() => {
                const _a = state,
                  _v = WorkbenchState(
                    "disposed",
                    null,
                    0,
                    current.configuration,
                    current.latestReload,
                    true,
                  );
                const _old = _a.value;
                _a.value = _v;
                for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                return _v;
              })();
              return maybe_release_retired_sessions_bang();
            }
          })();
        } else {
          return null;
        }
      })();
    }
    function dispatch_bang(transition: WorkbenchTransition) {
      transition_queue.value.push(transition);
      if (!dispatching.value) {
        (() => {
          const _a = dispatching,
            _v = true;
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        return (() => {
          try {
            return (() => {
              while (true) {
                if (transition_queue.value.length > 0) {
                  const next_transition = firstValue(transition_queue.value);
                  (() => {
                    const _a = transition_queue,
                      _v = Array.from(restValues(transition_queue.value));
                    const _old = _a.value;
                    _a.value = _v;
                    for (const _k in _a.watches)
                      _a.watches[_k](_k, _a, _old, _v);
                    return _v;
                  })();
                  apply_transition_bang(next_transition);
                  continue;
                } else {
                  return null;
                }
              }
            })();
          } finally {
            (() => {
              const _a = dispatching,
                _v = false;
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
          }
        })();
      }
    }
    function observe_input_bang(value: WorkbenchEnvelope) {
      const current = state.value;
      const configuration = current.configuration;
      const retained_inputs =
        configuration.observations.length + pending_input_transitions.value;
      return current.disposed
        ? false
        : retained_inputs >= policy.maxPendingObservations
          ? (() => {
              if (!input_rejection_pending.value) {
                (() => {
                  const _a = input_rejection_pending,
                    _v = true;
                  const _old = _a.value;
                  _a.value = _v;
                  for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
                  return _v;
                })();
                dispatch_bang(InputRejected());
              }
              return false;
            })()
          : (() => {
              const accepted_value = require_immutable_input(policy, value);
              (() => {
                const _a = pending_input_transitions;
                const _old = _a.value;
                _a.value = ((_x) => _x + 1)(_old);
                for (const _k in _a.watches)
                  _a.watches[_k](_k, _a, _old, _a.value);
                return _a.value;
              })();
              dispatch_bang(ObserveInput(accepted_value));
              return true;
            })();
    }
    function reload_package_bang(package_candidate: unknown) {
      const current = state.value;
      return current.disposed
        ? false
        : reload_transition_pending.value
          ? false
          : equivalent(current.phase, "package-check")
            ? false
            : equivalent(current.phase, "session-start")
              ? (() => {
                  (() => {
                    const _a = reload_transition_pending,
                      _v = true;
                    const _old = _a.value;
                    _a.value = _v;
                    for (const _k in _a.watches)
                      _a.watches[_k](_k, _a, _old, _v);
                    return _v;
                  })();
                  dispatch_bang(ReloadRejected());
                  return false;
                })()
              : (() => {
                  (() => {
                    const _a = reload_transition_pending,
                      _v = true;
                    const _old = _a.value;
                    _a.value = _v;
                    for (const _k in _a.watches)
                      _a.watches[_k](_k, _a, _old, _v);
                    return _v;
                  })();
                  dispatch_bang(ReloadRequested(package_candidate));
                  return true;
                })();
    }
    function fixed_tick_elapsed_bang() {
      const current = state.value;
      if (
        !ticks_enabled.value ||
        current.disposed ||
        tick_transition_pending.value
      ) {
        return false;
      } else {
        (() => {
          const _a = tick_transition_pending,
            _v = true;
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        dispatch_bang(FixedTickElapsed());
        return true;
      }
    }
    function dispose_workbench_bang() {
      const current = state.value;
      if (current.disposed || dispose_transition_pending.value) {
        return false;
      } else {
        (() => {
          const _a = dispose_transition_pending,
            _v = true;
          const _old = _a.value;
          _a.value = _v;
          for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
          return _v;
        })();
        dispatch_bang(DisposeRequested());
        return true;
      }
    }
    function snapshot_bang() {
      const current = state.value;
      const active = current.active;
      const configuration = current.configuration;
      return WorkbenchSnapshot(
        current.phase,
        active_generation(active),
        current.operationId,
        configuration.revision,
        configuration.observations.length,
        active_revision(active),
        active_frame(active),
        current.disposed,
      );
    }
    const scheduled = (() => {
      try {
        const cancel = scheduleFixedTick(
          fixedTick.milliseconds,
          fixed_tick_elapsed_bang,
        );
        if (equivalent(typeof cancel, "function")) {
          (() => {
            const _a = cancel_fixed_tick,
              _v = cancel;
            const _old = _a.value;
            _a.value = _v;
            for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
            return _v;
          })();
          (() => {
            const _a = ticks_enabled,
              _v = true;
            const _old = _a.value;
            _a.value = _v;
            for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
            return _v;
          })();
          return true;
        } else {
          return (() => {
            throw new Error(
              "fixed-tick scheduler must return a cancellation function",
            );
          })();
        }
      } catch (_catch_11) {
        switch (classifyError(_catch_11)) {
          case 0: {
            const __error = _catch_11;
            (() => {
              const _a = ticks_enabled,
                _v = false;
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
            (() => {
              const _a = state,
                _v = WorkbenchState(
                  "disposed",
                  null,
                  0,
                  empty_configuration(0),
                  0,
                  true,
                );
              const _old = _a.value;
              _a.value = _v;
              for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v);
              return _v;
            })();
            emit_lifecycle_for_bang(
              "fixed-tick-schedule-uncertain",
              0,
              0,
              "scheduler-threw",
            );
            return false;
            break;
          }
        }
      }
    })();
    if (scheduled) {
      dispatch_bang(ReloadRequested(initialPackageCandidate));
    }
    return Object.freeze({
      observeInput: observe_input_bang,
      reloadPackage: reload_package_bang,
      snapshot: snapshot_bang,
      dispose: dispose_workbench_bang,
    });
  })();
}

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
