import { conj_value as $$bc$conj_value, empty_p as $$bc$empty_p, equivV as $$bc$equiv, first as $$bc$first, keyword as $$bc$keyword, property_key as $$bc$property_key, record_value as $$bc$record_value, rest as $$bc$rest, str as $$bc$str } from 'beagle/core.js';
import { catch_dispatch as $$bd$catch_dispatch } from 'beagle/exception-dispatch.js';

function FixedTick(milliseconds) {
  return $$bc$record_value("jump-arena-shell.workbench/FixedTick", {_tag: "FixedTick", milliseconds});
}

function fixedtick_milliseconds(r) { return r.milliseconds; }

function WorkbenchSequenceLimits(maxReceiptSequence, maxInputSequence, maxGeneration, maxOperationId, maxConfigurationRevision) {
  return $$bc$record_value("jump-arena-shell.workbench/WorkbenchSequenceLimits", {_tag: "WorkbenchSequenceLimits", maxReceiptSequence, maxInputSequence, maxGeneration, maxOperationId, maxConfigurationRevision});
}

function workbenchsequencelimits_maxReceiptSequence(r) { return r.maxReceiptSequence; }

function workbenchsequencelimits_maxInputSequence(r) { return r.maxInputSequence; }

function workbenchsequencelimits_maxGeneration(r) { return r.maxGeneration; }

function workbenchsequencelimits_maxOperationId(r) { return r.maxOperationId; }

function workbenchsequencelimits_maxConfigurationRevision(r) { return r.maxConfigurationRevision; }

function WorkbenchPolicy(maxPendingObservations, maxSessionIdentities, maxImmutableObjects, maxImmutableProperties, maxEnvelopeSourceUnits, sequenceLimits) {
  return $$bc$record_value("jump-arena-shell.workbench/WorkbenchPolicy", {_tag: "WorkbenchPolicy", maxPendingObservations, maxSessionIdentities, maxImmutableObjects, maxImmutableProperties, maxEnvelopeSourceUnits, sequenceLimits});
}

function workbenchpolicy_maxPendingObservations(r) { return r.maxPendingObservations; }

function workbenchpolicy_maxSessionIdentities(r) { return r.maxSessionIdentities; }

function workbenchpolicy_maxImmutableObjects(r) { return r.maxImmutableObjects; }

function workbenchpolicy_maxImmutableProperties(r) { return r.maxImmutableProperties; }

function workbenchpolicy_maxEnvelopeSourceUnits(r) { return r.maxEnvelopeSourceUnits; }

function workbenchpolicy_sequenceLimits(r) { return r.sequenceLimits; }

function InputObservation(sequence, value) {
  return $$bc$record_value("jump-arena-shell.workbench/InputObservation", {_tag: "InputObservation", sequence, value});
}

function inputobservation_sequence(r) { return r.sequence; }

function inputobservation_value(r) { return r.value; }

function InputConfiguration(revision, observations) {
  return $$bc$record_value("jump-arena-shell.workbench/InputConfiguration", {_tag: "InputConfiguration", revision, observations});
}

function inputconfiguration_revision(r) { return r.revision; }

function inputconfiguration_observations(r) { return r.observations; }

function LifecycleReceipt(schema, sequence, event, phase, activeGeneration, operationGeneration, operationId, configurationRevision, revision, detail) {
  return $$bc$record_value("jump-arena-shell.workbench/LifecycleReceipt", {_tag: "LifecycleReceipt", schema, sequence, event, phase, activeGeneration, operationGeneration, operationId, configurationRevision, revision, detail});
}

function lifecyclereceipt_schema(r) { return r.schema; }

function lifecyclereceipt_sequence(r) { return r.sequence; }

function lifecyclereceipt_event(r) { return r.event; }

function lifecyclereceipt_phase(r) { return r.phase; }

function lifecyclereceipt_activeGeneration(r) { return r.activeGeneration; }

function lifecyclereceipt_operationGeneration(r) { return r.operationGeneration; }

function lifecyclereceipt_operationId(r) { return r.operationId; }

function lifecyclereceipt_configurationRevision(r) { return r.configurationRevision; }

function lifecyclereceipt_revision(r) { return r.revision; }

function lifecyclereceipt_detail(r) { return r.detail; }

function WorkbenchSnapshot(phase, generation, operationId, configurationRevision, pendingObservations, revision, frame, disposed) {
  return $$bc$record_value("jump-arena-shell.workbench/WorkbenchSnapshot", {_tag: "WorkbenchSnapshot", phase, generation, operationId, configurationRevision, pendingObservations, revision, frame, disposed});
}

function workbenchsnapshot_phase(r) { return r.phase; }

function workbenchsnapshot_generation(r) { return r.generation; }

function workbenchsnapshot_operationId(r) { return r.operationId; }

function workbenchsnapshot_configurationRevision(r) { return r.configurationRevision; }

function workbenchsnapshot_pendingObservations(r) { return r.pendingObservations; }

function workbenchsnapshot_revision(r) { return r.revision; }

function workbenchsnapshot_frame(r) { return r.frame; }

function workbenchsnapshot_disposed(r) { return r.disposed; }

// PackageCheck = PackageAccepted | PackageRejected
function PackageAccepted(acceptedPackage) { return $$bc$record_value("jump-arena-shell.workbench/PackageAccepted", { _tag: "PackageAccepted", acceptedPackage: acceptedPackage }); }

function packageaccepted_acceptedPackage(r) { return r.acceptedPackage; }
function PackageRejected(reason) { return $$bc$record_value("jump-arena-shell.workbench/PackageRejected", { _tag: "PackageRejected", reason: reason }); }

function packagerejected_reason(r) { return r.reason; }

// SessionCompletion = SessionStarted | SessionFailed
function SessionStarted(session, revision, frame) { return $$bc$record_value("jump-arena-shell.workbench/SessionStarted", { _tag: "SessionStarted", session: session, revision: revision, frame: frame }); }

function sessionstarted_session(r) { return r.session; }

function sessionstarted_revision(r) { return r.revision; }

function sessionstarted_frame(r) { return r.frame; }
function SessionFailed(reason) { return $$bc$record_value("jump-arena-shell.workbench/SessionFailed", { _tag: "SessionFailed", reason: reason }); }

function sessionfailed_reason(r) { return r.reason; }

// CandidateCompletion = CandidateProduced | CandidateFailed
function CandidateProduced(candidate) { return $$bc$record_value("jump-arena-shell.workbench/CandidateProduced", { _tag: "CandidateProduced", candidate: candidate }); }

function candidateproduced_candidate(r) { return r.candidate; }
function CandidateFailed(reason) { return $$bc$record_value("jump-arena-shell.workbench/CandidateFailed", { _tag: "CandidateFailed", reason: reason }); }

function candidatefailed_reason(r) { return r.reason; }

// AdmissionCompletion = AdmissionAccepted | AdmissionRejected
function AdmissionAccepted(successor, revision, frame) { return $$bc$record_value("jump-arena-shell.workbench/AdmissionAccepted", { _tag: "AdmissionAccepted", successor: successor, revision: revision, frame: frame }); }

function admissionaccepted_successor(r) { return r.successor; }

function admissionaccepted_revision(r) { return r.revision; }

function admissionaccepted_frame(r) { return r.frame; }
function AdmissionRejected(reason) { return $$bc$record_value("jump-arena-shell.workbench/AdmissionRejected", { _tag: "AdmissionRejected", reason: reason }); }

function admissionrejected_reason(r) { return r.reason; }

function CartridgePort(acceptPackage, startSession, runCandidate, requestAdmission, disposeSession) {
  return $$bc$record_value("jump-arena-shell.workbench/CartridgePort", {_tag: "CartridgePort", acceptPackage, startSession, runCandidate, requestAdmission, disposeSession});
}

function cartridgeport_acceptPackage(r) { return r.acceptPackage; }

function cartridgeport_startSession(r) { return r.startSession; }

function cartridgeport_runCandidate(r) { return r.runCandidate; }

function cartridgeport_requestAdmission(r) { return r.requestAdmission; }

function cartridgeport_disposeSession(r) { return r.disposeSession; }

function ActiveRuntime(generation, session, revision, frame) {
  return $$bc$record_value("jump-arena-shell.workbench/ActiveRuntime", {_tag: "ActiveRuntime", generation, session, revision, frame});
}

function activeruntime_generation(r) { return r.generation; }

function activeruntime_session(r) { return r.session; }

function activeruntime_revision(r) { return r.revision; }

function activeruntime_frame(r) { return r.frame; }

function WorkbenchState(phase, active, operationId, configuration, latestReload, disposed) {
  return $$bc$record_value("jump-arena-shell.workbench/WorkbenchState", {_tag: "WorkbenchState", phase, active, operationId, configuration, latestReload, disposed});
}

function workbenchstate_phase(r) { return r.phase; }

function workbenchstate_active(r) { return r.active; }

function workbenchstate_operationId(r) { return r.operationId; }

function workbenchstate_configuration(r) { return r.configuration; }

function workbenchstate_latestReload(r) { return r.latestReload; }

function workbenchstate_disposed(r) { return r.disposed; }

// WorkbenchTransition = ObserveInput | InputRejected | FixedTickElapsed | CandidateCompleted | CandidateBoundaryFailed | AdmissionCompleted | AdmissionBoundaryFailed | ReloadRequested | PackageCompleted | PackageBoundaryFailed | ReloadRejected | SessionCompleted | SessionBoundaryFailed | DisposeRequested
function ObserveInput(value) { return $$bc$record_value("jump-arena-shell.workbench/ObserveInput", { _tag: "ObserveInput", value: value }); }

function observeinput_value(r) { return r.value; }
function InputRejected() { return $$bc$record_value("jump-arena-shell.workbench/InputRejected", { _tag: "InputRejected" }); }
function FixedTickElapsed() { return $$bc$record_value("jump-arena-shell.workbench/FixedTickElapsed", { _tag: "FixedTickElapsed" }); }
function CandidateCompleted(generation, operationId, completion) { return $$bc$record_value("jump-arena-shell.workbench/CandidateCompleted", { _tag: "CandidateCompleted", generation: generation, operationId: operationId, completion: completion }); }

function candidatecompleted_generation(r) { return r.generation; }

function candidatecompleted_operationId(r) { return r.operationId; }

function candidatecompleted_completion(r) { return r.completion; }
function CandidateBoundaryFailed(generation, operationId) { return $$bc$record_value("jump-arena-shell.workbench/CandidateBoundaryFailed", { _tag: "CandidateBoundaryFailed", generation: generation, operationId: operationId }); }

function candidateboundaryfailed_generation(r) { return r.generation; }

function candidateboundaryfailed_operationId(r) { return r.operationId; }
function AdmissionCompleted(generation, operationId, completion) { return $$bc$record_value("jump-arena-shell.workbench/AdmissionCompleted", { _tag: "AdmissionCompleted", generation: generation, operationId: operationId, completion: completion }); }

function admissioncompleted_generation(r) { return r.generation; }

function admissioncompleted_operationId(r) { return r.operationId; }

function admissioncompleted_completion(r) { return r.completion; }
function AdmissionBoundaryFailed(generation, operationId) { return $$bc$record_value("jump-arena-shell.workbench/AdmissionBoundaryFailed", { _tag: "AdmissionBoundaryFailed", generation: generation, operationId: operationId }); }

function admissionboundaryfailed_generation(r) { return r.generation; }

function admissionboundaryfailed_operationId(r) { return r.operationId; }
function ReloadRequested(packageCandidate) { return $$bc$record_value("jump-arena-shell.workbench/ReloadRequested", { _tag: "ReloadRequested", packageCandidate: packageCandidate }); }

function reloadrequested_packageCandidate(r) { return r.packageCandidate; }
function PackageCompleted(generation, completion) { return $$bc$record_value("jump-arena-shell.workbench/PackageCompleted", { _tag: "PackageCompleted", generation: generation, completion: completion }); }

function packagecompleted_generation(r) { return r.generation; }

function packagecompleted_completion(r) { return r.completion; }
function PackageBoundaryFailed(generation) { return $$bc$record_value("jump-arena-shell.workbench/PackageBoundaryFailed", { _tag: "PackageBoundaryFailed", generation: generation }); }

function packageboundaryfailed_generation(r) { return r.generation; }
function ReloadRejected() { return $$bc$record_value("jump-arena-shell.workbench/ReloadRejected", { _tag: "ReloadRejected" }); }
function SessionCompleted(generation, priorSession, completion) { return $$bc$record_value("jump-arena-shell.workbench/SessionCompleted", { _tag: "SessionCompleted", generation: generation, priorSession: priorSession, completion: completion }); }

function sessioncompleted_generation(r) { return r.generation; }

function sessioncompleted_priorSession(r) { return r.priorSession; }

function sessioncompleted_completion(r) { return r.completion; }
function SessionBoundaryFailed(generation, priorSession) { return $$bc$record_value("jump-arena-shell.workbench/SessionBoundaryFailed", { _tag: "SessionBoundaryFailed", generation: generation, priorSession: priorSession }); }

function sessionboundaryfailed_generation(r) { return r.generation; }

function sessionboundaryfailed_priorSession(r) { return r.priorSession; }
function DisposeRequested() { return $$bc$record_value("jump-arena-shell.workbench/DisposeRequested", { _tag: "DisposeRequested" }); }

function EnvelopeMeasure(objectCount, propertyCount, sourceUnitCount) {
  return $$bc$record_value("jump-arena-shell.workbench/EnvelopeMeasure", {_tag: "EnvelopeMeasure", objectCount, propertyCount, sourceUnitCount});
}

function envelopemeasure_objectCount(r) { return r.objectCount; }

function envelopemeasure_propertyCount(r) { return r.propertyCount; }

function envelopemeasure_sourceUnitCount(r) { return r.sourceUnitCount; }

function EnvelopeCopy(source, target) {
  return $$bc$record_value("jump-arena-shell.workbench/EnvelopeCopy", {_tag: "EnvelopeCopy", source, target});
}

function envelopecopy_source(r) { return r.source; }

function envelopecopy_target(r) { return r.target; }

// EnvelopeScan = EnvelopeScanAccepted | EnvelopeScanRejected
function EnvelopeScanAccepted(pending, targets, objectCount, propertyCount) { return $$bc$record_value("jump-arena-shell.workbench/EnvelopeScanAccepted", { _tag: "EnvelopeScanAccepted", pending: pending, targets: targets, objectCount: objectCount, propertyCount: propertyCount }); }

function envelopescanaccepted_pending(r) { return r.pending; }

function envelopescanaccepted_targets(r) { return r.targets; }

function envelopescanaccepted_objectCount(r) { return r.objectCount; }

function envelopescanaccepted_propertyCount(r) { return r.propertyCount; }
function EnvelopeScanRejected() { return $$bc$record_value("jump-arena-shell.workbench/EnvelopeScanRejected", { _tag: "EnvelopeScanRejected" }); }

const lifecycle_schema = "clause-cartridge-workbench/v1";

const envelope_measures = new WeakMap();

function empty_configuration(revision) {
  return InputConfiguration(revision, Object.freeze([]));
}

function active_generation(active) {
  return ((active == null) ? 0 : active.generation);
}

function active_revision(active) {
  return ((active == null) ? null : active.revision);
}

function active_frame(active) {
  return ((active == null) ? null : active.frame);
}

function settled_phase(active) {
  return ((active == null) ? "idle" : "ready");
}

function runtime_session_token_p(value) {
  const kind = typeof value;
  return ((!(value == null)) && (($$bc$equiv(kind, "object")) || ($$bc$equiv(kind, "function"))));
}

function positive_safe_limit_p(value) {
  return ((_logical) => (_logical !== false && _logical != null ? (value > 0) : _logical))(Number.isSafeInteger(value));
}

function require_workbench_policy(policy) {
  const limits = policy.sequenceLimits;
  return ((positive_safe_limit_p(policy.maxPendingObservations) && (positive_safe_limit_p(policy.maxSessionIdentities) && (positive_safe_limit_p(policy.maxImmutableObjects) && (positive_safe_limit_p(policy.maxImmutableProperties) && (positive_safe_limit_p(policy.maxEnvelopeSourceUnits) && ((!(limits == null)) && (positive_safe_limit_p(limits.maxReceiptSequence) && ((limits.maxReceiptSequence >= 2) && (positive_safe_limit_p(limits.maxInputSequence) && (positive_safe_limit_p(limits.maxGeneration) && (positive_safe_limit_p(limits.maxOperationId) && positive_safe_limit_p(limits.maxConfigurationRevision)))))))))))) ? policy : (() => { throw new Error("workbench policy limits must be positive safe integers"); })());
}

function envelope_array_length(policy, source) {
  if (((_truthy) => _truthy !== false && _truthy != null)(Array.isArray(source))) {
    const length = source.length;
    return (((_truthy) => _truthy !== false && _truthy != null)(((_logical) => (_logical !== false && _logical != null ? ((length >= 0) && (length <= policy.maxImmutableProperties)) : _logical))(Number.isSafeInteger(length))) ? length : -1);
  } else {
    return -1;
  }
}

function create_envelope_target(length) {
  const target = new Array(length);
  Object.setPrototypeOf(target, null);
  return target;
}

function define_envelope_index(target, index, value) {
  return Object.defineProperty(target, index, {[$$bc$property_key($$bc$keyword("configurable"))]: true, [$$bc$property_key($$bc$keyword("enumerable"))]: true, [$$bc$property_key($$bc$keyword("value"))]: value, [$$bc$property_key($$bc$keyword("writable"))]: true});
}

function envelope_primitive_p(value, value_kind) {
  return ((value == null) || (($$bc$equiv(value_kind, "string")) || (($$bc$equiv(value_kind, "boolean")) || (($$bc$equiv(value_kind, "number")) && Number.isFinite(value)))));
}

function scan_envelope_copy(policy, copy, copies, pending, targets, object_count, property_count) {
  const source = copy.source;
  const target = copy.target;
  const length = target.length;
  return (() => { let index = 0; let next_pending = pending; let next_targets = targets; let next_object_count = object_count; let next_property_count = property_count; while (true) {
    if ((index === length)) { return EnvelopeScanAccepted(next_pending, next_targets, next_object_count, next_property_count); } else { const descriptor = Object.getOwnPropertyDescriptor(source, index); if (((descriptor == null) || (!((_truthy) => _truthy !== false && _truthy != null)(Object.hasOwn(descriptor, "value"))))) { return EnvelopeScanRejected(); } else { const child = descriptor.value; const child_kind = typeof child; if (($$bc$equiv(child_kind, "function"))) { return EnvelopeScanRejected(); } else if ((($$bc$equiv(child_kind, "object")) && (!(child == null)))) { if (((_truthy) => _truthy !== false && _truthy != null)(Array.isArray(child))) { if (((_truthy) => _truthy !== false && _truthy != null)(copies.has(child))) { define_envelope_index(target, index, copies.get(child)); const _recur_0 = (index + 1); const _recur_1 = next_pending; const _recur_2 = next_targets; const _recur_3 = next_object_count; const _recur_4 = next_property_count; index = _recur_0; next_pending = _recur_1; next_targets = _recur_2; next_object_count = _recur_3; next_property_count = _recur_4; continue; } else { const child_length = envelope_array_length(policy, child); const candidate_property_count = (next_property_count + child_length); if (((child_length < 0) || ((next_object_count >= policy.maxImmutableObjects) || (candidate_property_count > policy.maxImmutableProperties)))) { return EnvelopeScanRejected(); } else { const child_target = create_envelope_target(child_length); copies.set(child, child_target); define_envelope_index(target, index, child_target); const _recur_0 = (index + 1); const _recur_1 = $$bc$conj_value(next_pending, EnvelopeCopy(child, child_target)); const _recur_2 = $$bc$conj_value(next_targets, child_target); const _recur_3 = (next_object_count + 1); const _recur_4 = candidate_property_count; index = _recur_0; next_pending = _recur_1; next_targets = _recur_2; next_object_count = _recur_3; next_property_count = _recur_4; continue; } } } else { return EnvelopeScanRejected(); } } else if (envelope_primitive_p(child, child_kind)) { define_envelope_index(target, index, child); const _recur_0 = (index + 1); const _recur_1 = next_pending; const _recur_2 = next_targets; const _recur_3 = next_object_count; const _recur_4 = next_property_count; index = _recur_0; next_pending = _recur_1; next_targets = _recur_2; next_object_count = _recur_3; next_property_count = _recur_4; continue; } else { return EnvelopeScanRejected(); } } }
  } })();
}

function create_workbench_envelope(incomingPolicy, sourceText) {
  const policy = require_workbench_policy(incomingPolicy);
  const source_kind = typeof sourceText;
  const source_units = (($$bc$equiv(source_kind, "string")) ? sourceText.length : -1);
  if (((source_units < 0) || (source_units > policy.maxEnvelopeSourceUnits))) {
    return (() => { throw new Error("workbench envelope source exceeds its policy"); })();
  } else {
    const source = JSON.parse(sourceText);
    const root_length = envelope_array_length(policy, source);
    if ((root_length < 0)) {
      return (() => { throw new Error("workbench envelope source exceeds its policy"); })();
    } else {
      const root = create_envelope_target(root_length);
      const copies = new Map();
      copies.set(source, root);
      return (() => { let pending = [EnvelopeCopy(source, root)]; let targets = [root]; let object_count = 1; let property_count = root_length; while (true) {
    if ($$bc$empty_p(pending)) { return (() => { (() => { targets.forEach((target) => {
  Object.freeze(target);
}); })();
envelope_measures.set(root, EnvelopeMeasure(object_count, property_count, source_units));
return root; })(); } else { const copy = (() => { const _x = pending; return _x[_x.length - 1]; })(); const remaining = pending.slice(0, -1); { const _match_0 = scan_envelope_copy(policy, copy, copies, remaining, targets, object_count, property_count); if (_match_0._tag === "EnvelopeScanRejected") { return (() => { throw new Error("workbench envelope source exceeds its policy"); })(); } else if (_match_0._tag === "EnvelopeScanAccepted") { const next_pending = _match_0.pending; const next_targets = _match_0.targets; const next_object_count = _match_0.objectCount; const next_property_count = _match_0.propertyCount; const _recur_0 = next_pending; const _recur_1 = next_targets; const _recur_2 = next_object_count; const _recur_3 = next_property_count; pending = _recur_0; targets = _recur_1; object_count = _recur_2; property_count = _recur_3; continue; } else { return null; } } }
  } })();
    }
  }
}

function immutable_envelope_p(policy, value) {
  return (() => { try {
    const measure = envelope_measures.get(value);
  return ((!(measure == null)) && ((measure.objectCount <= policy.maxImmutableObjects) && ((measure.propertyCount <= policy.maxImmutableProperties) && ((measure.sourceUnitCount <= policy.maxEnvelopeSourceUnits) && (Object.getPrototypeOf(value) == null)))));
  } catch (_catch_0) {
    switch ($$bd$catch_dispatch(_catch_0, [Error])) {
      case 0: {
        const __error = _catch_0;
        return false;
        break;
      }
    }
  } })();
}

function require_immutable_input(policy, value) {
  return (immutable_envelope_p(policy, value) ? value : (() => { throw new Error("workbench input observations require a checked immutable envelope"); })());
}

function create_cartridge_workbench_bang(port, fixedTick, incomingPolicy, scheduleFixedTick, renderFrame, emitReceipt, initialPackageCandidate) {
  const policy = require_workbench_policy(incomingPolicy);
  const sequence_limits = policy.sequenceLimits;
  const state = ({value: WorkbenchState("idle", null, 0, empty_configuration(0), 0, false), watches: {}});
  const receipt_sequence = ({value: 0, watches: {}});
  const input_sequence = ({value: 0, watches: {}});
  const generation_sequence = ({value: 0, watches: {}});
  const operation_sequence = ({value: 0, watches: {}});
  const disposed_sessions = new Set();
  const retirement_reservations = ({value: 0, watches: {}});
  const pending_input_transitions = ({value: 0, watches: {}});
  const input_rejection_pending = ({value: false, watches: {}});
  const tick_transition_pending = ({value: false, watches: {}});
  const reload_transition_pending = ({value: false, watches: {}});
  const dispose_transition_pending = ({value: false, watches: {}});
  const transition_queue = ({value: [], watches: {}});
  const dispatching = ({value: false, watches: {}});
  const ticks_enabled = ({value: false, watches: {}});
  const cancel_fixed_tick = ({value: () => null, watches: {}});
  return (() => { function increment_capacity_p(current, needed, maximum) { return (needed <= (maximum - current)); } function normal_receipt_capacity_p(needed) { return (needed <= ((sequence_limits.maxReceiptSequence - receipt_sequence.value) - 1)); } function capacity_failure(receipts, inputs, generations, operations, configurations) { return (((!normal_receipt_capacity_p(receipts))) ? "receipt-sequence" : ((!increment_capacity_p(input_sequence.value, inputs, sequence_limits.maxInputSequence))) ? "input-sequence" : ((!increment_capacity_p(generation_sequence.value, generations, sequence_limits.maxGeneration))) ? "generation-sequence" : ((!increment_capacity_p(operation_sequence.value, operations, sequence_limits.maxOperationId))) ? "operation-sequence" : ((!increment_capacity_p(state.value.configuration.revision, configurations, sequence_limits.maxConfigurationRevision))) ? "configuration-revision" : null); } function counter_exhausted_with_session_bang(domain, uninstalled_session) { const current = state.value;
if ((!current.disposed)) {
  const active = current.active;
  const generation = active_generation(active);
  const cancel = cancel_fixed_tick.value;
  (() => { const _a = state, _v = WorkbenchState("disposed", active, 0, current.configuration, current.latestReload, true); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = ticks_enabled, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = cancel_fixed_tick, _v = () => null; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  const cancel_uncertain = (() => { try {
    cancel();
  return false;
  } catch (_catch_1) {
    switch ($$bd$catch_dispatch(_catch_1, [Error])) {
      case 0: {
        const __error = _catch_1;
        return true;
        break;
      }
    }
  } })();
  const active_disposal_uncertain = ((active == null) ? false : dispose_session_once_bang(active.session));
  const uninstalled_disposal_uncertain = ((uninstalled_session == null) ? false : dispose_session_once_bang(uninstalled_session));
  const disposal_uncertain = (active_disposal_uncertain || uninstalled_disposal_uncertain);
  const sequence = (() => { const _a = receipt_sequence; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
  const detail = (((cancel_uncertain && disposal_uncertain)) ? $$bc$str(domain, ":cancel-and-disposal-uncertain") : (cancel_uncertain) ? $$bc$str(domain, ":cancel-uncertain") : (disposal_uncertain) ? $$bc$str(domain, ":disposal-uncertain") : domain);
  const receipt = LifecycleReceipt(lifecycle_schema, sequence, "counter-exhausted", "disposed", generation, generation, 0, current.configuration.revision, active_revision(active), detail);
  (() => { try {
    return emitReceipt(receipt);
  } catch (_catch_2) {
    switch ($$bd$catch_dispatch(_catch_2, [Error])) {
      case 0: {
        const __error = _catch_2;
        return null;
        break;
      }
    }
  } })();
  (() => { const _a = state, _v = WorkbenchState("disposed", null, 0, current.configuration, current.latestReload, true); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return maybe_release_retired_sessions_bang();
} } function counter_exhausted_bang(domain) { return counter_exhausted_with_session_bang(domain, null); } function ensure_capacity_bang(receipts, inputs, generations, operations, configurations) { const failure = capacity_failure(receipts, inputs, generations, operations, configurations);
if ((failure == null)) {
  return true;
} else {
  counter_exhausted_bang(failure);
  return false;
} } function ensure_capacity_retiring_bang(receipts, inputs, generations, operations, configurations, uninstalled_session) { const failure = capacity_failure(receipts, inputs, generations, operations, configurations);
if ((failure == null)) {
  return true;
} else {
  counter_exhausted_with_session_bang(failure, uninstalled_session);
  return false;
} } function emit_lifecycle_for_bang(event, operation_generation, operation_id, detail) { if (normal_receipt_capacity_p(1)) {
  const current = state.value;
  const active = current.active;
  const sequence = (() => { const _a = receipt_sequence; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
  const receipt = LifecycleReceipt(lifecycle_schema, sequence, event, current.phase, active_generation(active), operation_generation, operation_id, current.configuration.revision, active_revision(active), detail);
  return (() => { try {
    return emitReceipt(receipt);
  } catch (_catch_3) {
    switch ($$bd$catch_dispatch(_catch_3, [Error])) {
      case 0: {
        const __error = _catch_3;
        return null;
        break;
      }
    }
  } })();
} else {
  return counter_exhausted_bang("receipt-sequence");
} } function emit_lifecycle_bang(event, operation_generation, detail) { return emit_lifecycle_for_bang(event, operation_generation, state.value.operationId, detail); } function one_shot_transition_bang() { const completed = ({value: false, watches: {}});
return (transition) => { if (completed.value) {
  return false;
} else {
  (() => { const _a = completed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  dispatch_bang(transition);
  return true;
} }; } function maybe_release_retired_sessions_bang() { if ((state.value.disposed && ($$bc$equiv(0, retirement_reservations.value)))) {
  return disposed_sessions.clear();
} } function dispose_session_once_bang(session) { if ((runtime_session_token_p(session) && (!((_truthy) => _truthy !== false && _truthy != null)(disposed_sessions.has(session))))) {
  if ((disposed_sessions.size >= policy.maxSessionIdentities)) {
    (() => { throw new Error("retired RuntimeSession identity bound exceeded"); })();
  }
  disposed_sessions.add(session);
  return (() => { try {
    (port.disposeSession)(session);
  return false;
  } catch (_catch_4) {
    switch ($$bd$catch_dispatch(_catch_4, [Error])) {
      case 0: {
        const __error = _catch_4;
        return true;
        break;
      }
    }
  } })();
} else {
  return false;
} } function restore_settled_phase_bang() { const current = state.value;
const active = current.active;
return (() => { const _a = state, _v = WorkbenchState(settled_phase(active), active, 0, current.configuration, current.latestReload, current.disposed); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })(); } function install_authority_bang(generation, session, revision, frame, reset_configuration, uninstalled_session, accepted_event, accepted_detail) { const configuration_increment = (reset_configuration ? 1 : 0);
if (ensure_capacity_retiring_bang(3, 0, 0, 0, configuration_increment, uninstalled_session)) {
  const current = state.value;
  const operation_id = current.operationId;
  const previous = current.active;
  const previous_session = ((previous == null) ? null : previous.session);
  const configuration = current.configuration;
  const next_configuration = (reset_configuration ? empty_configuration((configuration.revision + 1)) : configuration);
  const next_active = ActiveRuntime(generation, session, revision, frame);
  (() => { const _a = state, _v = WorkbenchState("successor", next_active, operation_id, next_configuration, current.latestReload, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  emit_lifecycle_bang(accepted_event, generation, accepted_detail);
  emit_lifecycle_bang("successor-installed", generation, "port-successor");
  (() => { const _a = state, _v = WorkbenchState("render", next_active, operation_id, next_configuration, current.latestReload, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  const render_event = (() => { try {
    renderFrame(frame);
  return "frame-rendered";
  } catch (_catch_5) {
    switch ($$bd$catch_dispatch(_catch_5, [Error])) {
      case 0: {
        const __error = _catch_5;
        return "frame-render-failed";
        break;
      }
    }
  } })();
  (() => { const _a = state, _v = WorkbenchState("ready", next_active, 0, next_configuration, current.latestReload, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  const disposal_uncertain = (((!(previous_session == null)) && (!(previous_session === session))) ? dispose_session_once_bang(previous_session) : false);
  const detail = ((disposal_uncertain) ? "prior-session-disposal-uncertain" : (($$bc$equiv(render_event, "frame-rendered"))) ? "port-frame" : "renderer-threw");
  return emit_lifecycle_for_bang(render_event, generation, operation_id, detail);
} } function stale_completion_bang(operation_generation, operation_id, detail) { if (((!state.value.disposed) && ensure_capacity_bang(1, 0, 0, 0, 0))) {
  return emit_lifecycle_for_bang("completion-stale", operation_generation, operation_id, detail);
} } function apply_transition_bang(transition) { return (() => { const _match_1 = transition; if (_match_1._tag === "ObserveInput") { const value = _match_1.value; return (() => { const __remaining_input_transitions = (() => { const _a = pending_input_transitions; const _old = _a.value; _a.value = (((_x) => (_x - 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })(); const current = state.value; if (((!current.disposed) && ensure_capacity_bang(1, 1, 0, 0, 1))) {
  const configuration = current.configuration;
  const observation_sequence = (() => { const _a = input_sequence; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
  const observation = InputObservation(observation_sequence, value);
  const observations = Object.freeze($$bc$conj_value(configuration.observations, observation));
  const next_configuration = InputConfiguration((configuration.revision + 1), observations);
  (() => { const _a = state, _v = WorkbenchState(current.phase, current.active, current.operationId, next_configuration, current.latestReload, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return emit_lifecycle_bang("configuration-observed", active_generation(current.active), "input");
} })(); } else if (_match_1._tag === "InputRejected") { return (() => { try {
    const current = state.value;
  if (((!current.disposed) && ensure_capacity_bang(1, 0, 0, 0, 0))) {
    return emit_lifecycle_bang("configuration-input-rejected", active_generation(current.active), "pending-observation-limit");
  }
  } finally {
    (() => { const _a = input_rejection_pending, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  } })(); } else if (_match_1._tag === "FixedTickElapsed") { return (() => { (() => { const _a = tick_transition_pending, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
const current = state.value;
const active = current.active;
if (((!current.disposed) && (($$bc$equiv(current.phase, "ready")) && ((!(active == null)) && ensure_capacity_bang(1, 0, 0, 1, 1))))) {
  const generation = active.generation;
  const operation_id = (() => { const _a = operation_sequence; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
  const configuration = current.configuration;
  (() => { const _a = state, _v = WorkbenchState("candidate", active, operation_id, empty_configuration((configuration.revision + 1)), current.latestReload, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  emit_lifecycle_bang("candidate-requested", generation, "fixed-tick");
  const settle = one_shot_transition_bang();
  return (() => { try {
    return (port.runCandidate)(active.session, fixedTick, configuration, (completion) => settle(CandidateCompleted(generation, operation_id, completion)));
  } catch (_catch_6) {
    switch ($$bd$catch_dispatch(_catch_6, [Error])) {
      case 0: {
        const __error = _catch_6;
        return settle(CandidateBoundaryFailed(generation, operation_id));
        break;
      }
    }
  } })();
} })(); } else if (_match_1._tag === "CandidateCompleted") { const generation = _match_1.generation; const operation_id = _match_1.operationId; const completion = _match_1.completion; return (() => { const current = state.value; const active = current.active; return ((current.disposed || ((active == null) || ((!(generation === active.generation)) || ((!(operation_id === current.operationId)) || (!($$bc$equiv(current.phase, "candidate"))))))) ? stale_completion_bang(generation, operation_id, "candidate") : (() => { const _match_2 = completion; if (_match_2._tag === "CandidateFailed") { const reason = _match_2.reason; return (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { emit_lifecycle_bang("candidate-failed", generation, reason);
return restore_settled_phase_bang(); })() : null); } else if (_match_2._tag === "CandidateProduced") { const candidate = _match_2.candidate; return (ensure_capacity_bang(2, 0, 0, 0, 0) ? (() => { (() => { const _a = state, _v = WorkbenchState("admission", active, operation_id, current.configuration, current.latestReload, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
emit_lifecycle_bang("candidate-produced", generation, "candidate");
emit_lifecycle_bang("admission-requested", generation, "candidate");
const settle = one_shot_transition_bang();
return (() => { try {
    return (port.requestAdmission)(active.session, candidate, (result) => settle(AdmissionCompleted(generation, operation_id, result)));
  } catch (_catch_7) {
    switch ($$bd$catch_dispatch(_catch_7, [Error])) {
      case 0: {
        const __error = _catch_7;
        return settle(AdmissionBoundaryFailed(generation, operation_id));
        break;
      }
    }
  } })(); })() : null); } else { return null; } })()); })(); } else if (_match_1._tag === "CandidateBoundaryFailed") { const generation = _match_1.generation; const operation_id = _match_1.operationId; return (() => { const current = state.value; const active = current.active; if ((current.disposed || ((active == null) || ((!(generation === active.generation)) || ((!(operation_id === current.operationId)) || (!($$bc$equiv(current.phase, "candidate")))))))) {
  return stale_completion_bang(generation, operation_id, "candidate-boundary");
} else {
  if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
    emit_lifecycle_for_bang("candidate-boundary-failed", generation, operation_id, "port-threw");
    return restore_settled_phase_bang();
  }
} })(); } else if (_match_1._tag === "AdmissionCompleted") { const generation = _match_1.generation; const operation_id = _match_1.operationId; const completion = _match_1.completion; return (() => { const current = state.value; const active = current.active; return ((current.disposed || ((active == null) || ((!(generation === active.generation)) || ((!(operation_id === current.operationId)) || (!($$bc$equiv(current.phase, "admission"))))))) ? stale_completion_bang(generation, operation_id, "admission") : (() => { const _match_3 = completion; if (_match_3._tag === "AdmissionRejected") { const reason = _match_3.reason; return (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { emit_lifecycle_bang("admission-rejected", generation, reason);
return restore_settled_phase_bang(); })() : null); } else if (_match_3._tag === "AdmissionAccepted") { const __successor = _match_3.successor; const revision = _match_3.revision; const frame = _match_3.frame; return ((!immutable_envelope_p(policy, frame)) ? (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { emit_lifecycle_bang("admission-frame-rejected", generation, "frame-not-deeply-immutable");
return restore_settled_phase_bang(); })() : null) : install_authority_bang(generation, active.session, revision, frame, false, null, "admission-accepted", "successor")); } else { return null; } })()); })(); } else if (_match_1._tag === "AdmissionBoundaryFailed") { const generation = _match_1.generation; const operation_id = _match_1.operationId; return (() => { const current = state.value; const active = current.active; if ((current.disposed || ((active == null) || ((!(generation === active.generation)) || ((!(operation_id === current.operationId)) || (!($$bc$equiv(current.phase, "admission")))))))) {
  return stale_completion_bang(generation, operation_id, "admission-boundary");
} else {
  if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
    emit_lifecycle_for_bang("admission-boundary-failed", generation, operation_id, "port-threw");
    return restore_settled_phase_bang();
  }
} })(); } else if (_match_1._tag === "ReloadRequested") { const package_candidate = _match_1.packageCandidate; return (() => { (() => { const _a = reload_transition_pending, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
const current = state.value;
return ((current.disposed) ? null : (($$bc$equiv(current.phase, "session-start"))) ? (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { return emit_lifecycle_bang("reload-rejected", current.latestReload, "runtime-session-start-pending"); })() : null) : (ensure_capacity_bang(1, 0, 1, 0, 0) ? (() => { const generation = (() => { const _a = generation_sequence; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
(() => { const _a = state, _v = WorkbenchState("package-check", current.active, 0, current.configuration, generation, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
emit_lifecycle_bang("reload-requested", generation, "package-candidate");
const settle = one_shot_transition_bang();
return (() => { try {
    return (port.acceptPackage)(package_candidate, (completion) => settle(PackageCompleted(generation, completion)));
  } catch (_catch_8) {
    switch ($$bd$catch_dispatch(_catch_8, [Error])) {
      case 0: {
        const __error = _catch_8;
        return settle(PackageBoundaryFailed(generation));
        break;
      }
    }
  } })(); })() : null)); })(); } else if (_match_1._tag === "PackageCompleted") { const generation = _match_1.generation; const completion = _match_1.completion; return (() => { const current = state.value; return ((current.disposed || ((!(generation === current.latestReload)) || (!($$bc$equiv(current.phase, "package-check"))))) ? stale_completion_bang(generation, 0, "package") : (() => { const _match_4 = completion; if (_match_4._tag === "PackageRejected") { const reason = _match_4.reason; return (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { restore_settled_phase_bang();
return emit_lifecycle_bang("package-rejected", generation, reason); })() : null); } else if (_match_4._tag === "PackageAccepted") { const accepted_package = _match_4.acceptedPackage; return (() => { const active = current.active; const active_count = ((active == null) ? 0 : 1); const retired_count = disposed_sessions.size; const used_identities = (retired_count + active_count + retirement_reservations.value); if ((used_identities >= policy.maxSessionIdentities)) {
  if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
    emit_lifecycle_bang("session-identity-limit", generation, "reload-rejected");
    return restore_settled_phase_bang();
  }
} else {
  if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
    const prior_session = ((active == null) ? null : active.session);
    (() => { const _a = state, _v = WorkbenchState("session-start", active, 0, current.configuration, generation, false); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
    emit_lifecycle_bang("package-accepted", generation, "accepted-package");
    (() => { const _a = retirement_reservations; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
    const callback_completed = ({value: false, watches: {}});
    const boundary_reported = ({value: false, watches: {}});
    const complete_session_bang = (result) => { if (callback_completed.value) {
  return false;
} else {
  (() => { const _a = callback_completed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  dispatch_bang(SessionCompleted(generation, prior_session, result));
  return true;
} };
    return (() => { try {
    return (port.startSession)(accepted_package, generation, complete_session_bang);
  } catch (_catch_9) {
    switch ($$bd$catch_dispatch(_catch_9, [Error])) {
      case 0: {
        const __error = _catch_9;
        if (((!callback_completed.value) && (!boundary_reported.value))) {
          (() => { const _a = boundary_reported, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
          return dispatch_bang(SessionBoundaryFailed(generation, prior_session));
        }
        break;
      }
    }
  } })();
  }
} })(); } else { return null; } })()); })(); } else if (_match_1._tag === "PackageBoundaryFailed") { const generation = _match_1.generation; return (() => { const current = state.value; if ((current.disposed || ((!(generation === current.latestReload)) || (!($$bc$equiv(current.phase, "package-check")))))) {
  return stale_completion_bang(generation, 0, "package-boundary");
} else {
  if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
    restore_settled_phase_bang();
    return emit_lifecycle_for_bang("package-boundary-failed", generation, 0, "port-threw");
  }
} })(); } else if (_match_1._tag === "ReloadRejected") { return (() => { try {
    const current = state.value;
  if (((!current.disposed) && ensure_capacity_bang(1, 0, 0, 0, 0))) {
    return emit_lifecycle_bang("reload-rejected", current.latestReload, "runtime-session-start-pending");
  }
  } finally {
    (() => { const _a = reload_transition_pending, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  } })(); } else if (_match_1._tag === "SessionCompleted") { const generation = _match_1.generation; const prior_session = _match_1.priorSession; const completion = _match_1.completion; return (() => { const __remaining_reservations = (() => { const _a = retirement_reservations; const _old = _a.value; _a.value = (((_x) => (_x - 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })(); const current = state.value; const current_completion = ((!current.disposed) && ((generation === current.latestReload) && ($$bc$equiv(current.phase, "session-start")))); if ((!current_completion)) {
  const can_emit = ensure_capacity_bang(1, 0, 0, 0, 0);
  const disposal_uncertain = (() => { const _match_5 = completion; if (_match_5._tag === "SessionStarted") { const session = _match_5.session; const __revision = _match_5.revision; const __frame = _match_5.frame; return (() => { const active = current.active; const current_session = ((active == null) ? null : active.session); return ((runtime_session_token_p(session) && ((!(session === prior_session)) && ((!(session === current_session)) && (!((_truthy) => _truthy !== false && _truthy != null)(disposed_sessions.has(session)))))) ? dispose_session_once_bang(session) : false); })(); } else if (_match_5._tag === "SessionFailed") { const __reason = _match_5.reason; return false; } else { return null; } })();
  if (can_emit) {
    emit_lifecycle_for_bang("completion-stale", generation, 0, (disposal_uncertain ? "session-disposal-uncertain" : "session"));
  }
  return maybe_release_retired_sessions_bang();
} else {
  return (() => { const _match_6 = completion; if (_match_6._tag === "SessionFailed") { const reason = _match_6.reason; return (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { restore_settled_phase_bang();
return emit_lifecycle_bang("session-failed", generation, reason); })() : null); } else if (_match_6._tag === "SessionStarted") { const session = _match_6.session; const revision = _match_6.revision; const frame = _match_6.frame; return (() => { const previous = current.active; const previous_session = ((previous == null) ? null : previous.session); return (((!runtime_session_token_p(session))) ? (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { restore_settled_phase_bang();
return emit_lifecycle_bang("session-invalid", generation, "runtime-session-reference-required"); })() : null) : (((_truthy) => _truthy !== false && _truthy != null)(disposed_sessions.has(session))) ? (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { restore_settled_phase_bang();
return emit_lifecycle_bang("session-retired", generation, "retired-runtime-session-rejected"); })() : null) : (((!(previous_session == null)) && (previous_session === session))) ? (ensure_capacity_bang(1, 0, 0, 0, 0) ? (() => { restore_settled_phase_bang();
return emit_lifecycle_bang("session-reused", generation, "fresh-runtime-session-required"); })() : null) : ((!immutable_envelope_p(policy, frame))) ? (ensure_capacity_retiring_bang(1, 0, 0, 0, 0, session) ? (() => { const disposal_uncertain = dispose_session_once_bang(session);
restore_settled_phase_bang();
return emit_lifecycle_bang("session-frame-rejected", generation, (disposal_uncertain ? "session-disposal-uncertain" : "frame-not-deeply-immutable")); })() : null) : install_authority_bang(generation, session, revision, frame, true, session, "session-started", "fresh-runtime-session")); })(); } else { return null; } })();
} })(); } else if (_match_1._tag === "SessionBoundaryFailed") { const generation = _match_1.generation; const __prior_session = _match_1.priorSession; return (() => { const current = state.value; const current_completion = ((!current.disposed) && ((generation === current.latestReload) && ($$bc$equiv(current.phase, "session-start")))); if (current_completion) {
  if (ensure_capacity_bang(1, 0, 0, 0, 0)) {
    restore_settled_phase_bang();
    emit_lifecycle_for_bang("session-boundary-failed", generation, 0, "port-threw");
  }
} else {
  stale_completion_bang(generation, 0, "session-boundary");
}
return maybe_release_retired_sessions_bang(); })(); } else if (_match_1._tag === "DisposeRequested") { return (() => { (() => { const _a = dispose_transition_pending, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
const current = state.value;
if (((!current.disposed) && ensure_capacity_bang(1, 0, 0, 0, 0))) {
  const active = current.active;
  const generation = active_generation(active);
  const cancel = cancel_fixed_tick.value;
  (() => { const _a = state, _v = WorkbenchState("disposed", active, 0, current.configuration, current.latestReload, true); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = ticks_enabled, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = cancel_fixed_tick, _v = () => null; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  const cancel_uncertain = (() => { try {
    cancel();
  return false;
  } catch (_catch_10) {
    switch ($$bd$catch_dispatch(_catch_10, [Error])) {
      case 0: {
        const __error = _catch_10;
        return true;
        break;
      }
    }
  } })();
  const disposal_uncertain = ((active == null) ? false : dispose_session_once_bang(active.session));
  const detail = (((cancel_uncertain && disposal_uncertain)) ? "terminal-cancel-and-disposal-uncertain" : (cancel_uncertain) ? "terminal-cancel-uncertain" : (disposal_uncertain) ? "terminal-disposal-uncertain" : "terminal");
  emit_lifecycle_bang("disposed", generation, detail);
  (() => { const _a = state, _v = WorkbenchState("disposed", null, 0, current.configuration, current.latestReload, true); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return maybe_release_retired_sessions_bang();
} })(); } else { return null; } })(); } function dispatch_bang(transition) { transition_queue.value.push(transition);
if ((!dispatching.value)) {
  (() => { const _a = dispatching, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return (() => { try {
    return (() => {  while (true) {
    if ((transition_queue.value.length > 0)) { const next_transition = $$bc$first(transition_queue.value); (() => { const _a = transition_queue, _v = Array.from($$bc$rest(transition_queue.value)); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })(); apply_transition_bang(next_transition);  continue; } else { return null; }
  } })();
  } finally {
    (() => { const _a = dispatching, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  } })();
} } function observe_input_bang(value) { const current = state.value;
const configuration = current.configuration;
const retained_inputs = (configuration.observations.length + pending_input_transitions.value);
return ((current.disposed) ? false : ((retained_inputs >= policy.maxPendingObservations)) ? (() => { if ((!input_rejection_pending.value)) {
  (() => { const _a = input_rejection_pending, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  dispatch_bang(InputRejected());
}
return false; })() : (() => { const accepted_value = require_immutable_input(policy, value); (() => { const _a = pending_input_transitions; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
dispatch_bang(ObserveInput(accepted_value));
return true; })()); } function reload_package_bang(package_candidate) { const current = state.value;
return ((current.disposed) ? false : (reload_transition_pending.value) ? false : (($$bc$equiv(current.phase, "package-check"))) ? false : (($$bc$equiv(current.phase, "session-start"))) ? (() => { (() => { const _a = reload_transition_pending, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
dispatch_bang(ReloadRejected());
return false; })() : (() => { (() => { const _a = reload_transition_pending, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
dispatch_bang(ReloadRequested(package_candidate));
return true; })()); } function fixed_tick_elapsed_bang() { const current = state.value;
if (((!ticks_enabled.value) || (current.disposed || tick_transition_pending.value))) {
  return false;
} else {
  (() => { const _a = tick_transition_pending, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  dispatch_bang(FixedTickElapsed());
  return true;
} } function dispose_workbench_bang() { const current = state.value;
if ((current.disposed || dispose_transition_pending.value)) {
  return false;
} else {
  (() => { const _a = dispose_transition_pending, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  dispatch_bang(DisposeRequested());
  return true;
} } function snapshot_bang() { const current = state.value;
const active = current.active;
const configuration = current.configuration;
return WorkbenchSnapshot(current.phase, active_generation(active), current.operationId, configuration.revision, configuration.observations.length, active_revision(active), active_frame(active), current.disposed); } const scheduled = (() => { try {
    const cancel = scheduleFixedTick(fixedTick.milliseconds, fixed_tick_elapsed_bang);
  if (($$bc$equiv(typeof cancel, "function"))) {
    (() => { const _a = cancel_fixed_tick, _v = cancel; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
    (() => { const _a = ticks_enabled, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
    return true;
  } else {
    return (() => { throw new Error("fixed-tick scheduler must return a cancellation function"); })();
  }
  } catch (_catch_11) {
    switch ($$bd$catch_dispatch(_catch_11, [Error])) {
      case 0: {
        const __error = _catch_11;
        (() => { const _a = ticks_enabled, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
        (() => { const _a = state, _v = WorkbenchState("disposed", null, 0, empty_configuration(0), 0, true); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
        emit_lifecycle_for_bang("fixed-tick-schedule-uncertain", 0, 0, "scheduler-threw");
        return false;
        break;
      }
    }
  } })();
if (scheduled) {
  dispatch_bang(ReloadRequested(initialPackageCandidate));
}
return Object.freeze({[$$bc$property_key($$bc$keyword("observeInput"))]: observe_input_bang, [$$bc$property_key($$bc$keyword("reloadPackage"))]: reload_package_bang, [$$bc$property_key($$bc$keyword("snapshot"))]: snapshot_bang, [$$bc$property_key($$bc$keyword("dispose"))]: dispose_workbench_bang}); })();
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
//# sourceMappingURL=workbench.js.map
