import * as workbench from "./workbench.js";
import * as test from "bun:test";
import { count as $$bc$count, equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key, range as $$bc$range } from 'beagle/core.js';
import { catch_dispatch as $$bd$catch_dispatch } from 'beagle/exception-dispatch.js';

function expectation_failed_bang() {
  return (() => { throw new Error("typed expectation failed"); })();
}

function expect_true_bang(condition) {
  return (condition ? null : expectation_failed_bang());
}

function expect_throws_p(action) {
  return (() => { try {
    action();
  return false;
  } catch (_catch_0) {
    switch ($$bd$catch_dispatch(_catch_0, [Error])) {
      case 0: {
        const __error = _catch_0;
        return true;
        break;
      }
    }
  } })();
}

function frozen(value) {
  return Object.freeze(value);
}

function sequence_limits(max_receipt, max_input, max_generation, max_operation, max_configuration) {
  return workbench["->WorkbenchSequenceLimits"](max_receipt, max_input, max_generation, max_operation, max_configuration);
}

function default_sequence_limits() {
  const maximum = Number.MAX_SAFE_INTEGER;
  return sequence_limits(maximum, maximum, maximum, maximum, maximum);
}

function workbench_policy(max_pending, max_sessions) {
  return workbench["->WorkbenchPolicy"](max_pending, max_sessions, 32, 128, 512, default_sequence_limits());
}

function workbench_policy_with_sequences(limits) {
  return workbench["->WorkbenchPolicy"](8, 8, 32, 128, 512, limits);
}

function envelope(value) {
  return workbench["create-workbench-envelope"](workbench_policy(8, 8), JSON.stringify([value]));
}

function envelope_tree(source) {
  return workbench["create-workbench-envelope"](workbench_policy(8, 8), JSON.stringify(source));
}

function request_at(requests, index) {
  return requests.value[index];
}

function complete_request_bang(requests, index, completion) {
  return (request_at(requests, index).complete)(completion);
}

function fake_fixture_with_policy_bang(initial_package, policy) {
  const calls = ({value: [], watches: {}});
  const package_requests = ({value: [], watches: {}});
  const session_requests = ({value: [], watches: {}});
  const candidate_requests = ({value: [], watches: {}});
  const admission_requests = ({value: [], watches: {}});
  const disposals = ({value: [], watches: {}});
  const rendered = ({value: [], watches: {}});
  const receipts = ({value: [], watches: {}});
  const package_hook = ({value: (__request) => null, watches: {}});
  const render_hook = ({value: (__frame) => null, watches: {}});
  const receipt_hook = ({value: (__receipt) => null, watches: {}});
  const session_hook = ({value: (__request) => null, watches: {}});
  const candidate_hook = ({value: (__request) => null, watches: {}});
  const admission_hook = ({value: (__request) => null, watches: {}});
  const disposal_hook = ({value: (__session) => null, watches: {}});
  const cancellation_hook = ({value: () => null, watches: {}});
  const scheduled_delay = ({value: -1, watches: {}});
  const scheduled_tick = ({value: () => null, watches: {}});
  const cancellations = ({value: 0, watches: {}});
  const port = workbench["->CartridgePort"]((package_candidate, complete) => { const request = {[$$bc$property_key($$bc$keyword("package"))]: package_candidate, [$$bc$property_key($$bc$keyword("complete"))]: complete};
calls.value.push({[$$bc$property_key($$bc$keyword("kind"))]: "acceptPackage"});
package_requests.value.push(request);
return (package_hook.value)(request); }, (accepted_package, generation, complete) => { const request = {[$$bc$property_key($$bc$keyword("package"))]: accepted_package, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("complete"))]: complete};
calls.value.push({[$$bc$property_key($$bc$keyword("kind"))]: "startSession"});
session_requests.value.push(request);
return (session_hook.value)(request); }, (session, fixed_tick, configuration, complete) => { const request = {[$$bc$property_key($$bc$keyword("session"))]: session, [$$bc$property_key($$bc$keyword("fixedTick"))]: fixed_tick, [$$bc$property_key($$bc$keyword("configuration"))]: configuration, [$$bc$property_key($$bc$keyword("complete"))]: complete};
calls.value.push({[$$bc$property_key($$bc$keyword("kind"))]: "runCandidate"});
candidate_requests.value.push(request);
return (candidate_hook.value)(request); }, (session, candidate, complete) => { const request = {[$$bc$property_key($$bc$keyword("session"))]: session, [$$bc$property_key($$bc$keyword("candidate"))]: candidate, [$$bc$property_key($$bc$keyword("complete"))]: complete};
calls.value.push({[$$bc$property_key($$bc$keyword("kind"))]: "requestAdmission"});
admission_requests.value.push(request);
return (admission_hook.value)(request); }, (session) => { calls.value.push({[$$bc$property_key($$bc$keyword("kind"))]: "disposeSession"});
disposals.value.push(session);
return (disposal_hook.value)(session); });
  const schedule = (delay, tick) => { (() => { const _a = scheduled_delay, _v = delay; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(() => { const _a = scheduled_tick, _v = tick; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return () => { (() => { const _a = cancellations; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
return (cancellation_hook.value)(); }; };
  const renderer = (frame) => { rendered.value.push(frame);
return (render_hook.value)(frame); };
  const receipt_sink = (receipt) => { receipts.value.push(receipt);
return (receipt_hook.value)(receipt); };
  const controller = workbench["create-cartridge-workbench!"](port, workbench["->FixedTick"](16), policy, schedule, renderer, receipt_sink, initial_package);
  return {[$$bc$property_key($$bc$keyword("controller"))]: controller, [$$bc$property_key($$bc$keyword("calls"))]: calls, [$$bc$property_key($$bc$keyword("packageRequests"))]: package_requests, [$$bc$property_key($$bc$keyword("sessionRequests"))]: session_requests, [$$bc$property_key($$bc$keyword("candidateRequests"))]: candidate_requests, [$$bc$property_key($$bc$keyword("admissionRequests"))]: admission_requests, [$$bc$property_key($$bc$keyword("disposals"))]: disposals, [$$bc$property_key($$bc$keyword("rendered"))]: rendered, [$$bc$property_key($$bc$keyword("receipts"))]: receipts, [$$bc$property_key($$bc$keyword("packageHook"))]: package_hook, [$$bc$property_key($$bc$keyword("renderHook"))]: render_hook, [$$bc$property_key($$bc$keyword("receiptHook"))]: receipt_hook, [$$bc$property_key($$bc$keyword("sessionHook"))]: session_hook, [$$bc$property_key($$bc$keyword("candidateHook"))]: candidate_hook, [$$bc$property_key($$bc$keyword("admissionHook"))]: admission_hook, [$$bc$property_key($$bc$keyword("disposalHook"))]: disposal_hook, [$$bc$property_key($$bc$keyword("cancellationHook"))]: cancellation_hook, [$$bc$property_key($$bc$keyword("scheduledDelay"))]: scheduled_delay, [$$bc$property_key($$bc$keyword("scheduledTick"))]: scheduled_tick, [$$bc$property_key($$bc$keyword("cancellations"))]: cancellations};
}

function fake_fixture_bang(initial_package) {
  return fake_fixture_with_policy_bang(initial_package, workbench_policy(8, 8));
}

function bootstrap_bang(fixture, accepted_package, session, revision, frame) {
  complete_request_bang(fixture.packageRequests, 0, workbench["->PackageAccepted"](accepted_package));
  return complete_request_bang(fixture.sessionRequests, 0, workbench["->SessionStarted"](session, revision, frame));
}

function tick_bang(fixture) {
  return (fixture.scheduledTick.value)();
}

function snapshot(fixture) {
  return (fixture.controller.snapshot)();
}

function receipt_events(fixture) {
  return fixture.receipts.value.map((receipt) => receipt.event);
}

function event_receipts(fixture, event) {
  return fixture.receipts.value.filter((receipt) => ($$bc$equiv(event, receipt.event)));
}

test["test"]("input changes only local configuration; candidate and rejection preserve authority", () => { const package_candidate = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-package"});
const accepted_package = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-package"});
const session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const revision = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
const initial_frame = envelope("frame-0");
const fixture = fake_fixture_bang(package_candidate);
const controller = fixture.controller;
bootstrap_bang(fixture, accepted_package, session, revision, initial_frame);
expect_true_bang(($$bc$equiv(true, Object.is(fixture.scheduledDelay.value, 16))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.rendered.value.length, 1))));
(controller.observeInput)(envelope("input-1"));
const configured = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(configured.configurationRevision, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(configured.pendingObservations, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(configured.revision, revision))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.admissionRequests.value.length, 0))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.rendered.value.length, 1))));
tick_bang(fixture);
const candidate_request = request_at(fixture.candidateRequests, 0);
const sent_configuration = candidate_request.configuration;
expect_true_bang(($$bc$equiv(true, Object.is(candidate_request.session, session))));
expect_true_bang(($$bc$equiv(true, Object.is(candidate_request.fixedTick.milliseconds, 16))));
expect_true_bang(($$bc$equiv(true, Object.is(sent_configuration.revision, 2))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(sent_configuration.observations), 1))));
const candidate = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "unadmitted-candidate", [$$bc$property_key($$bc$keyword("frame"))]: frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-frame-must-not-render"})});
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateProduced"](candidate));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "admission"))));
expect_true_bang(($$bc$equiv(true, Object.is(request_at(fixture.admissionRequests, 0).candidate, candidate))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).frame, initial_frame))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.rendered.value.length, 1))));
complete_request_bang(fixture.admissionRequests, 0, workbench["->AdmissionRejected"]("not-admitted"));
const rejected = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(rejected.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.revision, revision))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.frame, initial_frame))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.rendered.value.length, 1))));
return expect_true_bang(($$bc$equiv(true, receipt_events(fixture).includes("admission-rejected")))); });

test["test"]("fixed ticks consume input into a fresh bounded configuration identity", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}));
const controller = fixture.controller;
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(controller.observeInput)(envelope("input"));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).configurationRevision, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).pendingObservations, 1))));
tick_bang(fixture);
const consumed = request_at(fixture.candidateRequests, 0).configuration;
const current = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(consumed.revision, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(consumed.observations.length, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(current.configurationRevision, 3))));
return expect_true_bang(($$bc$equiv(true, Object.is(current.pendingObservations, 0)))); });

test["test"]("fixed tick configuration consumption fails before its declared ceiling", () => { const maximum = Number.MAX_SAFE_INTEGER;
const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}), workbench_policy_with_sequences(sequence_limits(maximum, maximum, maximum, maximum, 2)));
const controller = fixture.controller;
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(controller.observeInput)(envelope("input"));
tick_bang(fixture);
const terminal = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(terminal.phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(terminal.configurationRevision, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(terminal.pendingObservations, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.candidateRequests.value.length, 0))));
return expect_true_bang(($$bc$equiv(true, Object.is(event_receipts(fixture, "counter-exhausted")[0].detail, "configuration-revision")))); });

test["test"]("each candidate and admission cycle has one exact operation identity", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const revision_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
const frame_0 = envelope("frame-0");
const successor = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "successor-1"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_1 = envelope("frame-1");
const candidate_frame = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "not-authoritative"});
const candidate = frozen({[$$bc$property_key($$bc$keyword("frame"))]: candidate_frame});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, revision_0, frame_0);
tick_bang(fixture);
const operation_1 = snapshot(fixture).operationId;
expect_true_bang((operation_1 > 0));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "interrupting-package"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageRejected"]("interrupt-only"));
tick_bang(fixture);
const operation_2 = snapshot(fixture).operationId;
expect_true_bang((operation_2 > operation_1));
expect_true_bang(($$bc$equiv(true, Object.is(request_at(fixture.candidateRequests, 1).session, session_0))));
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "stale-candidate"})));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).operationId, operation_2))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "candidate"))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.admissionRequests.value.length, 0))));
const stale = event_receipts(fixture, "completion-stale");
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(stale), 1))));
expect_true_bang(($$bc$equiv(true, Object.is(stale[0].operationId, operation_1))));
complete_request_bang(fixture.candidateRequests, 1, workbench["->CandidateProduced"](candidate));
complete_request_bang(fixture.candidateRequests, 1, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "duplicate-candidate"})));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.admissionRequests.value.length, 1))));
complete_request_bang(fixture.admissionRequests, 0, workbench["->AdmissionAccepted"](successor, revision_1, frame_1));
complete_request_bang(fixture.admissionRequests, 0, workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "duplicate-successor"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "duplicate-revision"}), envelope("duplicate-frame")));
const advanced = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(advanced.operationId, 0))));
expect_true_bang(($$bc$equiv(true, Object.is(advanced.revision, revision_1))));
expect_true_bang(($$bc$equiv(true, Object.is(advanced.frame, frame_1))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.rendered.value.includes(candidate_frame), false))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [])));
const requested = event_receipts(fixture, "candidate-requested");
expect_true_bang(($$bc$equiv(requested.map((receipt) => receipt.operationId), [operation_1, operation_2])));
const rendered_receipts = event_receipts(fixture, "frame-rendered");
expect_true_bang(($$bc$equiv(true, Object.is(rendered_receipts[1].operationId, operation_2))));
fixture.receipts.value.forEach((receipt) => {
  expect_true_bang(($$bc$equiv(true, Object.is(receipt.schema, "clause-cartridge-workbench/v1"))));
}); });

test["test"]("failed reload keeps the old session; accepted reload is fresh and generation-fenced", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const revision_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
const frame_0 = envelope("frame-0");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, revision_0, frame_0);
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageRejected"]("package-denied"));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision_0))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).frame, frame_0))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.disposals.value.length, 0))));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-c"}));
const accepted_c = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-c"});
complete_request_bang(fixture.packageRequests, 2, workbench["->PackageAccepted"](accepted_c));
expect_true_bang(($$bc$equiv(true, Object.is(request_at(fixture.sessionRequests, 1).package, accepted_c))));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionFailed"]("runtime-start-failed"));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision_0))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).frame, frame_0))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.disposals.value.length, 0))));
tick_bang(fixture);
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "old-candidate"})));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.admissionRequests.value.length, 1))));
(controller.observeInput)(envelope("not-migrated"));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-d"}));
const accepted_d = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-d"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_1 = envelope("frame-1");
complete_request_bang(fixture.packageRequests, 3, workbench["->PackageAccepted"](accepted_d));
const fresh_request = request_at(fixture.sessionRequests, 2);
expect_true_bang(($$bc$equiv(true, Object.is(fresh_request.package, accepted_d))));
expect_true_bang(($$bc$equiv(true, Object.is(fresh_request.generation, 4))));
complete_request_bang(fixture.sessionRequests, 2, workbench["->SessionStarted"](session_1, revision_1, frame_1));
const reloaded = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(reloaded.generation, 4))));
expect_true_bang(($$bc$equiv(true, Object.is(reloaded.revision, revision_1))));
expect_true_bang(($$bc$equiv(true, Object.is(reloaded.frame, frame_1))));
expect_true_bang(($$bc$equiv(true, Object.is(reloaded.pendingObservations, 0))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
complete_request_bang(fixture.admissionRequests, 0, workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "stale-successor"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "stale-revision"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "stale-frame"})));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision_1))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
return expect_true_bang(($$bc$equiv(true, receipt_events(fixture).includes("completion-stale")))); });

test["test"]("reload rejects the currently live RuntimeSession without disposing it", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const revision = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
const frame = envelope("frame-0");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session, revision, frame);
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-reused"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "frame-reused"})));
const rejected = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(rejected.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.generation, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.revision, revision))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.frame, frame))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame])));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [])));
return expect_true_bang(($$bc$equiv(true, receipt_events(fixture).includes("session-reused")))); });

test["test"]("reload rejects every retired RuntimeSession identity", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_0 = envelope("frame-0");
const frame_1 = envelope("frame-1");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), frame_0);
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, revision_1, frame_1));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-c"}));
complete_request_bang(fixture.packageRequests, 2, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-c"})));
complete_request_bang(fixture.sessionRequests, 2, workbench["->SessionStarted"](session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "retired-revision"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "retired-frame"})));
const rejected = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(rejected.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.generation, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.revision, revision_1))));
expect_true_bang(($$bc$equiv(true, Object.is(rejected.frame, frame_1))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
return expect_true_bang(($$bc$equiv(true, receipt_events(fixture).includes("session-retired")))); });

test["test"]("a pending RuntimeSession start rejects an overlapping reload", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_0 = envelope("frame-0");
const frame_1 = envelope("frame-1");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), frame_0);
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
expect_true_bang(($$bc$equiv(true, Object.is((controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-c"})), false))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.packageRequests.value.length, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.sessionRequests.value.length, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "session-start"))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "reload-rejected")), 1))));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, revision_1, frame_1));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
const live = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(live.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(live.revision, revision_1))));
expect_true_bang(($$bc$equiv(true, Object.is(live.frame, frame_1))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
return expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0]))); });

test["test"]("session settlement receipts can serialize the next reload", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_0 = envelope("frame-0");
const frame_1 = envelope("frame-1");
const package_c = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-c"});
const package_d = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-d"});
const started_reload = ({value: false, watches: {}});
const failed_reload = ({value: false, watches: {}});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), frame_0);
(() => { const _a = fixture.receiptHook, _v = (receipt) => { if (($$bc$equiv(receipt.event, "session-started"))) {
  return (() => { const _a = started_reload, _v = (controller.reloadPackage)(package_c); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, revision_1, frame_1));
expect_true_bang(($$bc$equiv(true, Object.is(started_reload.value, true))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "package-check"))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.packageRequests.value.length, 3))));
expect_true_bang(($$bc$equiv(true, Object.is(request_at(fixture.packageRequests, 2).package, package_c))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision_1))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
(() => { const _a = fixture.receiptHook, _v = (receipt) => { if (($$bc$equiv(receipt.event, "session-failed"))) {
  return (() => { const _a = failed_reload, _v = (controller.reloadPackage)(package_d); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
complete_request_bang(fixture.packageRequests, 2, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-c"})));
complete_request_bang(fixture.sessionRequests, 2, workbench["->SessionFailed"]("start-c-failed"));
expect_true_bang(($$bc$equiv(true, Object.is(failed_reload.value, true))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "package-check"))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.packageRequests.value.length, 4))));
expect_true_bang(($$bc$equiv(true, Object.is(request_at(fixture.packageRequests, 3).package, package_d))));
complete_request_bang(fixture.packageRequests, 3, workbench["->PackageRejected"]("stop"));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "ready"))));
return expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision_1)))); });

test["test"]("a nil RuntimeSession is rejected without displacing live authority", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const revision_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
const frame_0 = envelope("frame-0");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, revision_0, frame_0);
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](null, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "nil-revision"}), envelope("nil-frame")));
const preserved = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(preserved.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(preserved.revision, revision_0))));
expect_true_bang(($$bc$equiv(true, Object.is(preserved.frame, frame_0))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0])));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [])));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "session-invalid")), 1))));
(controller.dispose)();
return expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0]))); });

test["test"]("RuntimeSession accepts only nominal reference tokens", () => { [false, 0, "session", Number.NaN].forEach((invalid_session) => {
  const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
  const controller = fixture.controller;
  const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
  const revision_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
  const frame_0 = envelope("frame-0");
  bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, revision_0, frame_0);
  (controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
  complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
  complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](invalid_session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "invalid-revision"}), envelope("invalid-frame")));
  const preserved = snapshot(fixture);
  expect_true_bang(($$bc$equiv(true, Object.is(preserved.phase, "ready"))));
  expect_true_bang(($$bc$equiv(true, Object.is(preserved.revision, revision_0))));
  expect_true_bang(($$bc$equiv(true, Object.is(preserved.frame, frame_0))));
  expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0])));
  expect_true_bang(($$bc$equiv(fixture.disposals.value, [])));
  expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "session-invalid")), 1))));
}); });

test["test"]("terminal disposal retains retired identities through queued start settlement", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), envelope("frame-0"));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"}), envelope("frame-1")));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
(() => { const _a = fixture.receiptHook, _v = (receipt) => { if (($$bc$equiv(receipt.event, "package-accepted"))) {
  return (controller.dispose)();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(() => { const _a = fixture.sessionHook, _v = (request) => (request.complete)(workbench["->SessionStarted"](session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "retired-revision"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "retired-frame"}))); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-c"}));
complete_request_bang(fixture.packageRequests, 2, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-c"})));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0, session_1])));
return expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(fixture.disposals.value.filter((session) => (session === session_0))), 1)))); });

test["test"]("session identity capacity counts live and retired RuntimeSessions", () => { const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}), workbench_policy(8, 2));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const revision_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_0 = envelope("frame-0");
const frame_1 = envelope("frame-1");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, revision_0, frame_0);
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.sessionRequests.value.length, 2))));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, revision_1, frame_1));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
expect_true_bang(($$bc$equiv(true, Object.is((controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-c"})), true))));
complete_request_bang(fixture.packageRequests, 2, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-c"})));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.sessionRequests.value.length, 2))));
expect_true_bang(($$bc$equiv(true, receipt_events(fixture).includes("session-identity-limit"))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision_1))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).frame, frame_1))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
return expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0]))); });

test["test"]("pending input is rejected visibly at its declared finite bound", () => { const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}), workbench_policy(2, 8));
const controller = fixture.controller;
const input_1 = envelope("input-1");
const input_2 = envelope("input-2");
const input_3 = envelope("input-3");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
tick_bang(fixture);
(controller.observeInput)(input_1);
(controller.observeInput)(input_2);
(controller.observeInput)(input_3);
const bounded = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(bounded.pendingObservations, 2))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "configuration-observed")), 2))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "configuration-input-rejected")), 1))));
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateFailed"]("cycle-held"));
tick_bang(fixture);
const configuration = request_at(fixture.candidateRequests, 1).configuration;
const observations = configuration.observations;
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(observations), 2))));
expect_true_bang(($$bc$equiv(true, Object.is(observations[0].value, input_1))));
return expect_true_bang(($$bc$equiv(true, Object.is(observations[1].value, input_2)))); });

test["test"]("reentrant input reserves capacity before serialized transition admission", () => { const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}), workbench_policy(2, 8));
const controller = fixture.controller;
const reentered = ({value: false, watches: {}});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
tick_bang(fixture);
(() => { const _a = fixture.receiptHook, _v = (receipt) => { if ((($$bc$equiv(receipt.event, "configuration-observed")) && (!reentered.value))) {
  (() => { const _a = reentered, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  expect_true_bang(($$bc$equiv(true, Object.is((controller.observeInput)(envelope("input-2")), true))));
  $$bc$range(8).forEach((index) => {
  expect_true_bang(($$bc$equiv(true, Object.is((controller.observeInput)(envelope(index)), false))));
});
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
expect_true_bang(($$bc$equiv(true, Object.is((controller.observeInput)(envelope("input-1")), true))));
const bounded = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(bounded.pendingObservations, 2))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "configuration-observed")), 2))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "configuration-input-rejected")), 1))));
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateFailed"]("cycle-held"));
tick_bang(fixture);
const configuration = request_at(fixture.candidateRequests, 1).configuration;
return expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(configuration.observations), 2)))); });

test["test"]("receipt-triggered disposal remains terminal after in-progress installation", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}));
const controller = fixture.controller;
const session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"});
const frame = envelope("frame");
const disposal_results = ({value: [], watches: {}});
(() => { const _a = fixture.receiptHook, _v = (receipt) => { if (($$bc$equiv(receipt.event, "session-started"))) {
  (() => { $$bc$range(32).forEach((__index) => {
  disposal_results.value.push((controller.dispose)());
}); })();
  return (() => { throw new Error("receipt-sink-failed"); })();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), frame);
const terminal = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(terminal.phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(terminal.disposed, true))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame])));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session])));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(disposal_results.value.filter((accepted) => ($$bc$equiv(accepted, true)))), 1))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.cancellations.value, 1))));
return expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "disposed")), 1)))); });

test["test"]("render-triggered reload survives the in-progress render transition", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const triggered = ({value: false, watches: {}});
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const frame_0 = envelope("frame-0");
const package_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"});
const reload_results = ({value: [], watches: {}});
(() => { const _a = fixture.renderHook, _v = (__frame) => { if ((!triggered.value)) {
  (() => { const _a = triggered, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { $$bc$range(32).forEach((__index) => {
  reload_results.value.push((controller.reloadPackage)(package_1));
}); })();
  return (() => { throw new Error("renderer-failed-after-reload"); })();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), frame_0);
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(reload_results.value.filter((accepted) => ($$bc$equiv(accepted, true)))), 1))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.packageRequests.value.length, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(request_at(fixture.packageRequests, 1).package, package_1))));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_1 = envelope("frame-1");
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, revision_1, frame_1));
const reloaded = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(reloaded.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(reloaded.revision, revision_1))));
expect_true_bang(($$bc$equiv(true, Object.is(reloaded.frame, frame_1))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
return expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0]))); });

test["test"]("receipt-triggered fixed ticks retain only one pending transition", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}));
const reentrant_results = ({value: [], watches: {}});
const reentered = ({value: false, watches: {}});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(() => { const _a = fixture.receiptHook, _v = (receipt) => { if ((($$bc$equiv(receipt.event, "candidate-requested")) && (!reentered.value))) {
  (() => { const _a = reentered, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  $$bc$range(32).forEach((__index) => {
  reentrant_results.value.push(tick_bang(fixture));
});
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
expect_true_bang(($$bc$equiv(true, Object.is(tick_bang(fixture), true))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(reentrant_results.value.filter((accepted) => ($$bc$equiv(accepted, true)))), 1))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.candidateRequests.value.length, 1))));
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateFailed"]("cycle-held"));
expect_true_bang(($$bc$equiv(true, Object.is(tick_bang(fixture), true))));
return expect_true_bang(($$bc$equiv(true, Object.is(fixture.candidateRequests.value.length, 2)))); });

test["test"]("renderer failure is visible without revoking installed authority", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const frame_0 = envelope("frame-0");
const frame_1 = envelope("frame-1");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), frame_0);
(() => { const _a = fixture.renderHook, _v = (frame) => { if ((frame === frame_1)) {
  return (() => { throw new Error("renderer-failed"); })();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
tick_bang(fixture);
const operation_id = snapshot(fixture).operationId;
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate"})));
complete_request_bang(fixture.admissionRequests, 0, workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "successor"}), revision_1, frame_1));
const installed = snapshot(fixture);
const failed_receipts = event_receipts(fixture, "frame-render-failed");
expect_true_bang(($$bc$equiv(true, Object.is(installed.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(installed.generation, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(installed.revision, revision_1))));
expect_true_bang(($$bc$equiv(true, Object.is(installed.frame, frame_1))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, frame_1])));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [])));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(failed_receipts), 1))));
return expect_true_bang(($$bc$equiv(true, Object.is(failed_receipts[0].operationId, operation_id)))); });

test["test"]("synchronous completion wins when every port throws afterward", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const revision_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"});
const revision_2 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-2"});
const frame_1 = envelope("frame-1");
const frame_2 = envelope("frame-2");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), envelope("frame-0"));
(() => { const _a = fixture.packageHook, _v = (request) => { (request.complete)(workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
return (() => { throw new Error("package-threw-after-callback"); })(); }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(() => { const _a = fixture.sessionHook, _v = (request) => { (request.complete)(workbench["->SessionStarted"](session_1, revision_1, frame_1));
return (() => { throw new Error("session-threw-after-callback"); })(); }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).revision, revision_1))));
(() => { const _a = fixture.candidateHook, _v = (request) => { (request.complete)(workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate"})));
return (() => { throw new Error("candidate-threw-after-callback"); })(); }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(() => { const _a = fixture.admissionHook, _v = (request) => { (request.complete)(workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "successor"}), revision_2, frame_2));
return (() => { throw new Error("admission-threw-after-callback"); })(); }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
tick_bang(fixture);
const settled = snapshot(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(settled.phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(settled.revision, revision_2))));
expect_true_bang(($$bc$equiv(true, Object.is(settled.frame, frame_2))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0])));
["package-boundary-failed", "session-boundary-failed", "candidate-boundary-failed", "admission-boundary-failed"].forEach((event) => {
  expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, event)), 0))));
}); });

test["test"]("port throws before completion restore authority and contain late callbacks", () => { const package_fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}));
const package_session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-session"});
const package_revision = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-revision"});
const package_frame = envelope("package-frame");
const session_fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-package"}));
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const late_session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-session"});
const session_revision = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-revision"});
const session_frame = envelope("session-frame");
const candidate_fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-package"}));
const candidate_revision = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-revision"});
const candidate_frame = envelope("candidate-frame");
const admission_fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-package"}));
const admission_revision = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-revision"});
const admission_frame = envelope("admission-frame");
bootstrap_bang(package_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-accepted"}), package_session, package_revision, package_frame);
(() => { const _a = package_fixture.packageHook, _v = (__request) => (() => { throw new Error("package-boundary"); })(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(package_fixture.controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(package_fixture).phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(package_fixture, "package-boundary-failed")), 1))));
complete_request_bang(package_fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-package"})));
expect_true_bang(($$bc$equiv(true, Object.is(package_fixture.sessionRequests.value.length, 1))));
bootstrap_bang(session_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-accepted"}), session_0, session_revision, session_frame);
(() => { const _a = session_fixture.sessionHook, _v = (__request) => (() => { throw new Error("session-boundary"); })(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(session_fixture.controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-package-b"}));
complete_request_bang(session_fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-accepted-b"})));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(session_fixture).phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(session_fixture, "session-boundary-failed")), 1))));
complete_request_bang(session_fixture.sessionRequests, 1, workbench["->SessionStarted"](late_session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-revision"}), envelope("late-frame")));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(session_fixture).revision, session_revision))));
expect_true_bang(($$bc$equiv(session_fixture.disposals.value, [late_session])));
bootstrap_bang(candidate_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-session"}), candidate_revision, candidate_frame);
(() => { const _a = candidate_fixture.candidateHook, _v = (__request) => (() => { throw new Error("candidate-boundary"); })(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
tick_bang(candidate_fixture);
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(candidate_fixture, "candidate-boundary-failed")), 1))));
complete_request_bang(candidate_fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-candidate"})));
expect_true_bang(($$bc$equiv(true, Object.is(candidate_fixture.admissionRequests.value.length, 0))));
bootstrap_bang(admission_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-session"}), admission_revision, admission_frame);
tick_bang(admission_fixture);
(() => { const _a = admission_fixture.admissionHook, _v = (__request) => (() => { throw new Error("admission-boundary"); })(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
complete_request_bang(admission_fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate"})));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(admission_fixture, "admission-boundary-failed")), 1))));
complete_request_bang(admission_fixture.admissionRequests, 0, workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-successor"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-revision"}), envelope("late-frame")));
return expect_true_bang(($$bc$equiv(true, Object.is(snapshot(admission_fixture).revision, admission_revision)))); });

test["test"]("throwing RuntimeSession starts retain bounded identity custody", () => { const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}), workbench_policy(8, 3));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
const session_2 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-2"});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), envelope("frame-0"));
(() => { const _a = fixture.sessionHook, _v = (__request) => (() => { throw new Error("session-boundary"); })(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-c"}));
complete_request_bang(fixture.packageRequests, 2, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-c"})));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-d"}));
complete_request_bang(fixture.packageRequests, 3, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-d"})));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.sessionRequests.value.length, 3))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "session-boundary-failed")), 2))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "session-identity-limit")), 1))));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-revision-1"}), envelope("late-frame-1")));
complete_request_bang(fixture.sessionRequests, 2, workbench["->SessionStarted"](session_2, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-revision-2"}), envelope("late-frame-2")));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_1, session_2])));
return expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).generation, 1)))); });

test["test"]("cancellation and session-disposal throws cannot escape terminal disposal", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}));
const controller = fixture.controller;
const session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"});
const tick_results = ({value: [], watches: {}});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(() => { const _a = fixture.cancellationHook, _v = () => { tick_results.value.push(tick_bang(fixture));
return (() => { throw new Error("cancel-failed"); })(); }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
(() => { const _a = fixture.disposalHook, _v = (__session) => { tick_results.value.push(tick_bang(fixture));
return (() => { throw new Error("dispose-failed"); })(); }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
expect_true_bang(($$bc$equiv(true, Object.is((controller.dispose)(), true))));
const terminal = snapshot(fixture);
const disposed_receipts = event_receipts(fixture, "disposed");
expect_true_bang(($$bc$equiv(true, Object.is(terminal.phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(terminal.disposed, true))));
expect_true_bang(($$bc$equiv(tick_results.value, [false, false])));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.cancellations.value, 1))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session])));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.candidateRequests.value.length, 0))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(disposed_receipts), 1))));
return expect_true_bang(($$bc$equiv(true, Object.is(disposed_receipts[0].detail, "terminal-cancel-and-disposal-uncertain")))); });

test["test"]("a throwing fixed-tick scheduler yields an inert terminal controller", () => { const package_calls = ({value: 0, watches: {}});
const retained_tick = ({value: () => false, watches: {}});
const receipts = ({value: [], watches: {}});
const port = workbench["->CartridgePort"]((__package, __complete) => (() => { const _a = package_calls; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })(), (__package, __generation, __complete) => null, (__session, __tick, __configuration, __complete) => null, (__session, __candidate, __complete) => null, (__session) => null);
const controller = workbench["create-cartridge-workbench!"](port, workbench["->FixedTick"](16), workbench_policy(8, 8), (__delay, tick) => { (() => { const _a = retained_tick, _v = tick; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
return (() => { throw new Error("scheduler-failed"); })(); }, (__frame) => null, (receipt) => receipts.value.push(receipt), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}));
const terminal = (controller.snapshot)();
expect_true_bang(($$bc$equiv(true, Object.is(terminal.phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(terminal.disposed, true))));
expect_true_bang(($$bc$equiv(true, Object.is(package_calls.value, 0))));
expect_true_bang(($$bc$equiv(true, Object.is((retained_tick.value)(), false))));
expect_true_bang(($$bc$equiv(true, Object.is((controller.dispose)(), false))));
return expect_true_bang(($$bc$equiv(true, Object.is(receipts.value[0].event, "fixed-tick-schedule-uncertain")))); });

test["test"]("frames require bounded generic deep immutability", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}));
const revision_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"});
const frame_0 = envelope("frame-0");
const source = [["envelope", ["token", "stable"]]];
const deep_frame = envelope_tree(source);
const shallow_frame = frozen({[$$bc$property_key($$bc$keyword("envelope"))]: {[$$bc$property_key($$bc$keyword("token"))]: "mutable"}});
Object.defineProperty(source[0][1], 1, {[$$bc$property_key($$bc$keyword("configurable"))]: true, [$$bc$property_key($$bc$keyword("enumerable"))]: true, [$$bc$property_key($$bc$keyword("value"))]: "source-mutated", [$$bc$property_key($$bc$keyword("writable"))]: true});
expect_true_bang(($$bc$equiv(true, Object.is(deep_frame[0][1][1], "stable"))));
expect_true_bang(($$bc$equiv(true, Object.is(Object.getPrototypeOf(deep_frame), null))));
expect_true_bang(($$bc$equiv(true, Object.is(Object.getPrototypeOf(deep_frame[0]), null))));
expect_true_bang(($$bc$equiv(true, Object.is(Object.isFrozen(deep_frame), true))));
expect_true_bang(expect_throws_p(() => Object.defineProperty(deep_frame, 0, {[$$bc$property_key($$bc$keyword("configurable"))]: true, [$$bc$property_key($$bc$keyword("enumerable"))]: true, [$$bc$property_key($$bc$keyword("value"))]: "forged", [$$bc$property_key($$bc$keyword("writable"))]: true})));
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), revision_0, frame_0);
tick_bang(fixture);
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-1"})));
complete_request_bang(fixture.admissionRequests, 0, workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "successor-1"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"}), deep_frame));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).frame, deep_frame))));
tick_bang(fixture);
complete_request_bang(fixture.candidateRequests, 1, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-2"})));
complete_request_bang(fixture.admissionRequests, 1, workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "successor-2"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-2"}), shallow_frame));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "ready"))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).frame, deep_frame))));
expect_true_bang(($$bc$equiv(fixture.rendered.value, [frame_0, deep_frame])));
expect_true_bang(($$bc$equiv(true, receipt_events(fixture).includes("admission-frame-rejected"))));
const forged = new Array(1);
const __forged_value = Object.defineProperty(forged, 0, {[$$bc$property_key($$bc$keyword("configurable"))]: true, [$$bc$property_key($$bc$keyword("enumerable"))]: true, [$$bc$property_key($$bc$keyword("value"))]: "unmeasured", [$$bc$property_key($$bc$keyword("writable"))]: true});
const __forged_prototype = Object.setPrototypeOf(forged, null);
const __forged_frozen = Object.freeze(forged);
const bounded = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "bounded-package"}), workbench["->WorkbenchPolicy"](8, 8, 1, 8, 512, default_sequence_limits()));
const bounded_properties = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "property-package"}), workbench["->WorkbenchPolicy"](8, 8, 32, 2, 512, default_sequence_limits()));
const bounded_source = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "source-package"}), workbench["->WorkbenchPolicy"](8, 8, 32, 128, 8, default_sequence_limits()));
const forged_fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "forged-package"}));
const custom_prototype_frame = Object.freeze(Object.create({[$$bc$property_key($$bc$keyword("inherited"))]: "mutable"}));
const custom_prototype = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "custom-prototype-package"}));
const holey_frame = Object.freeze(new Array(1));
const holey = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "holey-package"}));
bootstrap_bang(bounded, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "bounded-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "bounded-session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "bounded-revision"}), deep_frame);
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(bounded).phase, "idle"))));
expect_true_bang(($$bc$equiv(bounded.rendered.value, [])));
expect_true_bang(($$bc$equiv(true, receipt_events(bounded).includes("session-frame-rejected"))));
bootstrap_bang(bounded_properties, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "property-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "property-session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "property-revision"}), deep_frame);
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(bounded_properties).phase, "idle"))));
expect_true_bang(($$bc$equiv(bounded_properties.rendered.value, [])));
bootstrap_bang(bounded_source, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "source-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "source-session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "source-revision"}), deep_frame);
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(bounded_source).phase, "idle"))));
expect_true_bang(($$bc$equiv(bounded_source.rendered.value, [])));
bootstrap_bang(forged_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "forged-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "forged-session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "forged-revision"}), forged);
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(forged_fixture).phase, "idle"))));
expect_true_bang(($$bc$equiv(forged_fixture.rendered.value, [])));
expect_true_bang(expect_throws_p(() => (forged_fixture.controller.observeInput)(forged)));
bootstrap_bang(custom_prototype, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "custom-prototype-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "custom-prototype-session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "custom-prototype-revision"}), custom_prototype_frame);
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(custom_prototype).phase, "idle"))));
expect_true_bang(($$bc$equiv(custom_prototype.rendered.value, [])));
bootstrap_bang(holey, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "holey-accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "holey-session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "holey-revision"}), holey_frame);
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(holey).phase, "idle"))));
expect_true_bang(($$bc$equiv(holey.rendered.value, [])));
expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](workbench["->WorkbenchPolicy"](8, 8, 8, 4, 64, default_sequence_limits()), "[0,0,0,0,0]")));
expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](workbench["->WorkbenchPolicy"](8, 8, 1, 8, 64, default_sequence_limits()), "[[\"nested\"]]")));
Object.defineProperty(Object.prototype, "workbenchEnvelopePoison", {[$$bc$property_key($$bc$keyword("configurable"))]: true, [$$bc$property_key($$bc$keyword("value"))]: "object-poison"});
Object.defineProperty(Array.prototype, "workbenchEnvelopePoison", {[$$bc$property_key($$bc$keyword("configurable"))]: true, [$$bc$property_key($$bc$keyword("value"))]: "array-poison"});
return (() => { try {
    expect_true_bang(($$bc$equiv(typeof deep_frame.workbenchEnvelopePoison, "undefined")));
  return expect_true_bang(($$bc$equiv(typeof deep_frame[0].workbenchEnvelopePoison, "undefined")));
  } finally {
    Reflect.deleteProperty(Object.prototype, "workbenchEnvelopePoison");
    Reflect.deleteProperty(Array.prototype, "workbenchEnvelopePoison");
  } })(); });

test["test"]("serialized envelope ingress excludes live objects and undeclared primitives", () => { const policy = workbench_policy(8, 8);
const primitives = workbench["create-workbench-envelope"](policy, "[null,true,false,0,1.5,\"text\"]");
const trap_reads = ({value: 0, watches: {}});
const array_proxy = new Proxy([], {[$$bc$property_key($$bc$keyword("get"))]: (__target, __property) => { (() => { const _a = trap_reads; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
return (() => { throw new Error("proxy get invoked"); })(); }, [$$bc$property_key($$bc$keyword("getOwnPropertyDescriptor"))]: (__target, __property) => { (() => { const _a = trap_reads; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
return (() => { throw new Error("proxy descriptor invoked"); })(); }});
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(primitives), 6))));
expect_true_bang(($$bc$equiv(true, Object.is(Object.getPrototypeOf(primitives), null))));
expect_true_bang(($$bc$equiv(true, Object.is(Object.isFrozen(primitives), true))));
(() => { ["{}", "[{}]", "[undefined]", "[1e999]"].forEach((source) => {
  expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](policy, source)));
}); })();
expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](policy, array_proxy)));
return expect_true_bang(($$bc$equiv(true, Object.is(trap_reads.value, 0)))); });

test["test"]("workbench policy limits must be positive safe integers", () => { const maximum = Number.MAX_SAFE_INTEGER;
const infinity = Number.POSITIVE_INFINITY;
const not_a_number = Number.NaN;
expect_true_bang(($$bc$equiv(workbench["create-workbench-envelope"](workbench_policy_with_sequences(sequence_limits(maximum, maximum, maximum, maximum, maximum)), "[]"), [])));
(() => { [0, -1, 1.5, infinity, not_a_number, (maximum + 1)].forEach((invalid_limit) => {
  const invalid_policy = workbench["->WorkbenchPolicy"](invalid_limit, 8, 32, 128, 512, default_sequence_limits());
  expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](invalid_policy, "[]")));
}); })();
expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](workbench_policy_with_sequences(sequence_limits(1, maximum, maximum, maximum, maximum)), "[]")));
[0, -1, 1.5, infinity, not_a_number, (maximum + 1)].forEach((invalid_limit) => {
  expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](workbench_policy_with_sequences(sequence_limits(2, invalid_limit, invalid_limit, invalid_limit, invalid_limit)), "[]")));
}); });

test["test"]("receipt exhaustion is one terminal occurrence under hostile reentrancy", () => { const maximum = Number.MAX_SAFE_INTEGER;
const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}), workbench_policy_with_sequences(sequence_limits(6, maximum, maximum, maximum, maximum)));
const controller = fixture.controller;
const session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"});
const terminal_results = ({value: [], watches: {}});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(() => { const _a = fixture.receiptHook, _v = (receipt) => { if (($$bc$equiv(receipt.event, "counter-exhausted"))) {
  (() => { $$bc$range(32).forEach((__index) => {
  terminal_results.value.push(tick_bang(fixture));
  terminal_results.value.push((controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-package"})));
  terminal_results.value.push((controller.observeInput)(envelope("late-input")));
  terminal_results.value.push((controller.dispose)());
}); })();
  return (() => { throw new Error("terminal-receipt-failed"); })();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
expect_true_bang(($$bc$equiv(true, Object.is(tick_bang(fixture), true))));
const terminal = snapshot(fixture);
const exhausted = event_receipts(fixture, "counter-exhausted");
expect_true_bang(($$bc$equiv(true, Object.is(terminal.phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(terminal.disposed, true))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(exhausted), 1))));
expect_true_bang(($$bc$equiv(true, Object.is(exhausted[0].sequence, 6))));
expect_true_bang(($$bc$equiv(true, Object.is(exhausted[0].detail, "receipt-sequence"))));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(terminal_results.value.filter((result) => ($$bc$equiv(result, true)))), 0))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.cancellations.value, 1))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session])));
return expect_true_bang(($$bc$equiv(true, Object.is(fixture.candidateRequests.value.length, 0)))); });

test["test"]("receipt exhaustion retires a fresh RuntimeSession with a rejected frame", () => { const maximum = Number.MAX_SAFE_INTEGER;
const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}), workbench_policy_with_sequences(sequence_limits(8, maximum, maximum, maximum, maximum)));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), envelope("frame-0"));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "unbranded-frame"})));
const terminal = snapshot(fixture);
const exhausted = event_receipts(fixture, "counter-exhausted");
expect_true_bang(($$bc$equiv(true, Object.is(terminal.phase, "disposed"))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0, session_1])));
expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(exhausted), 1))));
expect_true_bang(($$bc$equiv(true, Object.is(exhausted[0].sequence, 8))));
expect_true_bang(($$bc$equiv(true, Object.is(exhausted[0].detail, "receipt-sequence"))));
return expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(event_receipts(fixture, "session-frame-rejected")), 0)))); });

test["test"]("generation operation input and configuration counters fail closed", () => { const maximum = Number.MAX_SAFE_INTEGER;
const generation_fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "generation-package"}), workbench_policy_with_sequences(sequence_limits(maximum, maximum, 1, maximum, maximum)));
const operation_fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "operation-package"}), workbench_policy_with_sequences(sequence_limits(maximum, maximum, maximum, 1, maximum)));
const input_fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "input-package"}), workbench_policy_with_sequences(sequence_limits(maximum, 1, maximum, maximum, maximum)));
const configuration_fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "configuration-package"}), workbench_policy_with_sequences(sequence_limits(maximum, maximum, maximum, maximum, 2)));
bootstrap_bang(generation_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(generation_fixture.controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "second-package"}));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(generation_fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(generation_fixture.packageRequests.value.length, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(event_receipts(generation_fixture, "counter-exhausted")[0].detail, "generation-sequence"))));
bootstrap_bang(operation_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
tick_bang(operation_fixture);
complete_request_bang(operation_fixture.candidateRequests, 0, workbench["->CandidateFailed"]("settled"));
tick_bang(operation_fixture);
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(operation_fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(operation_fixture.candidateRequests.value.length, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(event_receipts(operation_fixture, "counter-exhausted")[0].detail, "operation-sequence"))));
bootstrap_bang(input_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(input_fixture.controller.observeInput)(envelope("input-1"));
(input_fixture.controller.observeInput)(envelope("input-2"));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(input_fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(input_fixture).pendingObservations, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(event_receipts(input_fixture, "counter-exhausted")[0].detail, "input-sequence"))));
bootstrap_bang(configuration_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), envelope("frame"));
(configuration_fixture.controller.observeInput)(envelope("input-1"));
(configuration_fixture.controller.observeInput)(envelope("input-2"));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(configuration_fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(configuration_fixture).configurationRevision, 2))));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(configuration_fixture).pendingObservations, 1))));
return expect_true_bang(($$bc$equiv(true, Object.is(event_receipts(configuration_fixture, "counter-exhausted")[0].detail, "configuration-revision")))); });

test["test"]("counter exhaustion retires a returned session that cannot be installed", () => { const maximum = Number.MAX_SAFE_INTEGER;
const fixture = fake_fixture_with_policy_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-a"}), workbench_policy_with_sequences(sequence_limits(maximum, maximum, maximum, maximum, 1)));
const controller = fixture.controller;
const session_0 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-0"});
const session_1 = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session-1"});
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-a"}), session_0, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-0"}), envelope("frame-0"));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package-b"}));
complete_request_bang(fixture.packageRequests, 1, workbench["->PackageAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted-b"})));
(() => { const _a = fixture.disposalHook, _v = (session) => { if ((session === session_1)) {
  return (() => { throw new Error("fresh-session-disposal-failed"); })();
} }; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
complete_request_bang(fixture.sessionRequests, 1, workbench["->SessionStarted"](session_1, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision-1"}), envelope("frame-1")));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session_0, session_1])));
return expect_true_bang(($$bc$equiv(true, Object.is(event_receipts(fixture, "counter-exhausted")[0].detail, "configuration-revision:disposal-uncertain")))); });

test["test"]("terminal disposal fences outstanding candidate and Admission completions", () => { const candidate_fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-package"}));
const candidate_controller = candidate_fixture.controller;
const candidate_session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-session"});
const candidate_frame = envelope("candidate-frame");
const admission_fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-package"}));
const admission_controller = admission_fixture.controller;
const admission_session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-session"});
const admission_frame = envelope("admission-frame");
bootstrap_bang(candidate_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-accepted"}), candidate_session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate-revision"}), candidate_frame);
tick_bang(candidate_fixture);
(candidate_controller.dispose)();
complete_request_bang(candidate_fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-candidate"})));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(candidate_fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(candidate_fixture.admissionRequests.value.length, 0))));
expect_true_bang(($$bc$equiv(candidate_fixture.rendered.value, [candidate_frame])));
expect_true_bang(($$bc$equiv(candidate_fixture.disposals.value, [candidate_session])));
bootstrap_bang(admission_fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-accepted"}), admission_session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "admission-revision"}), admission_frame);
tick_bang(admission_fixture);
complete_request_bang(admission_fixture.candidateRequests, 0, workbench["->CandidateProduced"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "candidate"})));
(admission_controller.dispose)();
complete_request_bang(admission_fixture.admissionRequests, 0, workbench["->AdmissionAccepted"](frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-successor"}), frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-revision"}), envelope("late-frame")));
expect_true_bang(($$bc$equiv(true, Object.is(snapshot(admission_fixture).phase, "disposed"))));
expect_true_bang(($$bc$equiv(admission_fixture.rendered.value, [admission_frame])));
return expect_true_bang(($$bc$equiv(admission_fixture.disposals.value, [admission_session]))); });

test["test"]("disposal is terminal and idempotent", () => { const fixture = fake_fixture_bang(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "package"}));
const controller = fixture.controller;
const session = frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "session"});
const frame = envelope("frame");
bootstrap_bang(fixture, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "accepted"}), session, frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "revision"}), frame);
(controller.dispose)();
(controller.dispose)();
tick_bang(fixture);
(controller.observeInput)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-input"}));
(controller.reloadPackage)(frozen({[$$bc$property_key($$bc$keyword("opaque"))]: "late-package"}));
const terminal = snapshot(fixture);
const events = receipt_events(fixture);
expect_true_bang(($$bc$equiv(true, Object.is(terminal.phase, "disposed"))));
expect_true_bang(($$bc$equiv(true, Object.is(terminal.disposed, true))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.cancellations.value, 1))));
expect_true_bang(($$bc$equiv(fixture.disposals.value, [session])));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.packageRequests.value.length, 1))));
expect_true_bang(($$bc$equiv(true, Object.is(fixture.candidateRequests.value.length, 0))));
expect_true_bang(($$bc$equiv(true, Object.is(events.length, fixture.receipts.value.length))));
return expect_true_bang(($$bc$equiv(true, Object.is($$bc$count(events.filter((event) => ($$bc$equiv(event, "disposed")))), 1)))); });

test["test"]("opaque port values are not interpreted and foreign objects cannot forge envelopes", () => { const reads = ({value: 0, watches: {}});
const poison = new Proxy(frozen({}), {[$$bc$property_key($$bc$keyword("get"))]: (__target, __property) => { (() => { const _a = reads; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
return (() => { throw new Error("opaque value was interpreted"); })(); }});
const fixture = fake_fixture_bang(poison);
const frame = envelope("frame");
expect_true_bang(expect_throws_p(() => workbench["create-workbench-envelope"](workbench_policy(8, 8), poison)));
bootstrap_bang(fixture, poison, poison, poison, frame);
expect_true_bang(expect_throws_p(() => (fixture.controller.observeInput)(poison)));
(fixture.controller.observeInput)(envelope("input"));
tick_bang(fixture);
complete_request_bang(fixture.candidateRequests, 0, workbench["->CandidateProduced"](poison));
complete_request_bang(fixture.admissionRequests, 0, workbench["->AdmissionRejected"]("opaque-rejection"));
expect_true_bang(($$bc$equiv(true, Object.is(reads.value, 0))));
return expect_true_bang(($$bc$equiv(fixture.calls.value.map((call) => call.kind), ["acceptPackage", "startSession", "runCandidate", "requestAdmission"]))); });
//# sourceMappingURL=workbench-boundary-test.js.map
