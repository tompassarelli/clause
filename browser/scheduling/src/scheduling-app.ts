import * as wasm from "../../runtime/src/wasm-cartridge-port.js";
import * as workbench from "../../runtime/src/workbench.js";
import {
  createSchedulingShell,
  type ScheduleCommand,
  type ScheduleRuleEditor,
} from "./scheduling-shell.js";
import {
  projectSchedule,
  projectedReferentKey,
  scheduleFingerprint,
  type ScheduleTaskView,
  type ScheduleView,
} from "./scheduling-view.js";

const runtimeModulePath = "/wasm/clause_runtime.js";
const runtimeBytesPath = "/wasm/clause_runtime_bg.wasm";

interface SchedulingWasmModule {
  readonly initSync: (input: { readonly module: ArrayBuffer }) => unknown;
  readonly [key: string]: unknown;
}

interface ScalarEffectPayload {
  readonly index: number;
  readonly entry: number;
  readonly start: number;
  readonly end: number;
  readonly artifact: string;
  readonly expression: string;
  readonly designation: string;
}

interface GenerationPayload {
  readonly generation: number;
  readonly compilerMicros: number;
  readonly cwr1: string;
  readonly cet1: string | null;
  readonly scalarEffects: readonly ScalarEffectPayload[];
  readonly completeEntry: number;
}

interface RecordedCompletion {
  readonly taskKey: string;
  readonly step: string;
  readonly rows: readonly wasm.ExplainedRelationRow[];
}

type DiagnosticEvent = Readonly<Record<string, unknown>>;
const diagnosticEvents: DiagnosticEvent[] = [];
(window as unknown as { __SCHEDULING_WORKBENCH_EVENTS__: DiagnosticEvent[] })
  .__SCHEDULING_WORKBENCH_EVENTS__ = diagnosticEvents;

function recordDiagnostic(event: DiagnosticEvent): void {
  diagnosticEvents.push(Object.freeze(event));
}

function completed<T>(action: (done: (result: T) => unknown) => unknown): T {
  let present = false;
  let value: T | undefined;
  action(result => {
    if (present) throw new Error("browser boundary completed an operation twice");
    present = true;
    value = result;
  });
  if (!present) throw new Error("browser boundary did not complete synchronously");
  return value as T;
}

function schedulePolicy(): workbench.WorkbenchPolicy {
  const maximum = Number.MAX_SAFE_INTEGER;
  return workbench["->WorkbenchPolicy"](
    16,
    8,
    64,
    wasm["cse1-projected-term-max-properties"],
    wasm["cse1-projected-term-json-max-source-units"],
    workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum),
  );
}

async function fetchBytes(path: string): Promise<ArrayBuffer> {
  const response = await fetch(path);
  if (!response.ok) throw new Error(`failed to fetch ${path}: ${response.status}`);
  return response.arrayBuffer();
}

function object(value: unknown, context: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${context} is not an object`);
  }
  return value as Record<string, unknown>;
}

function parseGeneration(value: unknown): GenerationPayload {
  const payload = object(value, "schedule generation");
  if (!Number.isSafeInteger(payload.generation)
    || typeof payload.compilerMicros !== "number"
    || typeof payload.cwr1 !== "string"
    || !(payload.cet1 === null || typeof payload.cet1 === "string")
    || !Array.isArray(payload.scalarEffects)
    || !Number.isSafeInteger(payload.completeEntry)) {
    throw new Error("schedule generation is malformed");
  }
  const scalarEffects = payload.scalarEffects.map((value, position) => {
    const effect = object(value, `schedule edit ${position + 1}`);
    if (!Number.isSafeInteger(effect.index) || !Number.isSafeInteger(effect.entry)
      || !Number.isSafeInteger(effect.start) || !Number.isSafeInteger(effect.end)
      || typeof effect.artifact !== "string" || typeof effect.expression !== "string"
      || typeof effect.designation !== "string") {
      throw new Error("schedule edit catalog is malformed");
    }
    return effect as unknown as ScalarEffectPayload;
  });
  return Object.freeze({
    generation: payload.generation as number,
    compilerMicros: payload.compilerMicros,
    cwr1: payload.cwr1,
    cet1: payload.cet1 as string | null,
    scalarEffects: Object.freeze(scalarEffects),
    completeEntry: payload.completeEntry as number,
  });
}

async function fetchGeneration(): Promise<GenerationPayload> {
  const response = await fetch("/resident-generation", { cache: "no-store" });
  const value: unknown = await response.json();
  if (!response.ok) throw new Error("The checked schedule source is unavailable.");
  return parseGeneration(value);
}

function projectSession(module: SchedulingWasmModule, session: unknown): wasm.ProjectedValue {
  return wasm.projectSession(module, session);
}

function inputEnvelope(policy: workbench.WorkbenchPolicy, value: unknown): workbench.WorkbenchEnvelope {
  return workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)]));
}

function progressFingerprint(view: ScheduleView): string {
  return JSON.stringify(view.tasks
    .map(task => [
      task.title,
      task.duration,
      task.completed,
      task.prerequisiteKeys.map(key => view.tasksByKey[key]?.title).sort(),
      task.waitingKeys.map(key => view.tasksByKey[key]?.title).sort(),
      task.blockerKeys.map(key => view.rootsByKey[key]?.reason).sort(),
    ])
    .sort(([left], [right]) => String(left).localeCompare(String(right))));
}

function selectedTask(view: ScheduleView, referent: wasm.ProjectedReferent): ScheduleTaskView {
  const task = view.tasksByKey[projectedReferentKey(referent)];
  if (task === undefined) throw new Error("the selected task is no longer in this schedule");
  return task;
}

function editorFor(payload: GenerationPayload, status: ScheduleRuleEditor["status"]): ScheduleRuleEditor {
  const effect = payload.scalarEffects.find(candidate => candidate.designation === "extend");
  if (effect === undefined) throw new Error("the schedule has no offered extension rule expression");
  return Object.freeze({
    generation: payload.generation,
    catalogIndex: effect.index,
    expression: effect.expression,
    start: effect.start,
    end: effect.end,
    status,
  });
}

const mount = document.querySelector("#schedule-app");
if (!(mount instanceof HTMLElement)) throw new Error("missing scheduling application mount");

let disposeSession: (() => unknown) | null = null;
let activeView: ScheduleView | null = null;
let activeGeneration: GenerationPayload | null = null;
let activeCompletion: RecordedCompletion | null = null;
let actionSequence = 0;
let configurationRevision = 0;
let actionInFlight = false;
let dispatch: (command: ScheduleCommand) => unknown = () => undefined;
const shell = createSchedulingShell(mount, command => dispatch(command));

async function start(): Promise<void> {
  const [imported, moduleBytes, generation] = await Promise.all([
    import(runtimeModulePath),
    fetchBytes(runtimeBytesPath),
    fetchGeneration(),
  ]);
  if (typeof imported.initSync !== "function") throw new Error("scheduling module has no initializer");
  const module = imported as SchedulingWasmModule;
  module.initSync({ module: moduleBytes });
  const policy = schedulePolicy();
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const accepted = completed<workbench.PackageCheck>(done =>
    port.acceptPackage(wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](generation.cwr1)), done));
  if (accepted._tag !== "PackageAccepted") throw new Error(accepted.reason);
  const started = completed<workbench.SessionCompletion>(done =>
    port.startSession(accepted.acceptedPackage, generation.generation, done));
  if (started._tag !== "SessionStarted") throw new Error(started.reason);
  let session = started.session;
  activeGeneration = generation;
  disposeSession = () => port.disposeSession(session);
  activeView = shell.renderFrame(projectSession(module, session));
  shell.renderRuleEditor(editorFor(generation, {
    tone: "quiet",
    text: "The current checked expression is ready to edit.",
  }));
  shell.renderNotice({ tone: "quiet", text: "Choose a task, resolve its blockers, or inspect a checked completion request." });
  recordDiagnostic({
    phase: "opened",
    generation: generation.generation,
    fingerprint: scheduleFingerprint(activeView),
    taskKeys: activeView.tasks.map(task => task.key),
  });

  const candidateFor = (input: unknown | null): workbench.CandidateCompletion => {
    const observations = input === null ? [] : [workbench["->InputObservation"](
      ++actionSequence,
      inputEnvelope(policy, input),
    )];
    return completed<workbench.CandidateCompletion>(done => port.runCandidate(
      session,
      workbench["->FixedTick"](1),
      workbench["->InputConfiguration"](++configurationRevision, observations),
      done,
    ));
  };

  const admit = (candidate: workbench.CandidateCompletion): ScheduleView => {
    if (candidate._tag !== "CandidateProduced") throw new Error(candidate.reason);
    const admission = completed<workbench.AdmissionCompletion>(done =>
      port.requestAdmission(session, candidate.candidate, done));
    if (admission._tag !== "AdmissionAccepted") throw new Error(admission.reason);
    return shell.renderFrame(projectSession(module, session));
  };

  const referentInput = (channel: "Resolve" | "Complete" | "Extend", referent: wasm.ProjectedReferent): unknown => {
    if (activeGeneration === null) throw new Error("the checked schedule source is unavailable");
    return Object.freeze({
      kind: "referent-input",
      generation: activeGeneration.generation,
      channel,
      value: wasm.checkedProjectedReferent(referent),
    });
  };

  const runAction = (channel: "Resolve" | "Complete" | "Extend", referent: wasm.ProjectedReferent): ScheduleView => {
    const previous = activeView;
    if (previous === null) throw new Error("the schedule has not opened");
    const before = scheduleFingerprint(previous);
    const next = admit(candidateFor(referentInput(channel, referent)));
    activeView = next;
    activeCompletion = null;
    shell.renderExplanation(null);
    shell.renderNotice(before === scheduleFingerprint(next)
      ? { tone: "rejected", text: "No schedule change was accepted." }
      : { tone: "success", text: "Schedule updated." });
    recordDiagnostic({ phase: "schedule-action", channel, before, after: scheduleFingerprint(next) });
    return next;
  };

  const explain = (referent: wasm.ProjectedReferent): void => {
    if (activeView === null || activeGeneration === null) throw new Error("the schedule has not opened");
    const task = selectedTask(activeView, referent);
    if (task.completed || (task.waitingKeys.length === 0 && task.blockerKeys.length === 0)) {
      throw new Error("this completion request is already ready and cannot be checked without changing the schedule");
    }
    const before = scheduleFingerprint(activeView);
    const checkedView = admit(candidateFor(referentInput("Complete", referent)));
    activeView = checkedView;
    const after = scheduleFingerprint(checkedView);
    if (after !== before) throw new Error("the explanation request unexpectedly changed the schedule");
    const explanation = wasm.explainSession(module, session, activeGeneration.completeEntry);
    const detail = object(explanation, "completion explanation");
    const step = detail.step;
    const rules = detail.rules;
    if (typeof step !== "string") throw new Error("the completion explanation omitted its recorded check");
    const selectedRuleCount = Object.values(object(rules, "completion rules"))
      .map((rule, index) => object(rule, `completion rule ${index + 1}`))
      .filter(rule => rule.selected === true).length;
    const rows = wasm.explanationRelationRows(explanation);
    const target = rows
      .filter(row => row.source.relation === "completed")
      .find(row => projectedReferentKey(row.subject) === task.key);
    if (typeof target?.after !== "boolean") {
      throw new Error("the completion explanation omitted the selected task outcome");
    }
    activeCompletion = Object.freeze({
      taskKey: task.key,
      step,
      rows,
    });
    shell.renderExplanation({ taskKey: task.key, step, completed: target.after });
    shell.renderNotice({ tone: "quiet", text: "Completion was checked and explained; the schedule is unchanged." });
    recordDiagnostic({ phase: "explained", task: task.title, step, selectedRuleCount, before, after });
  };

  const hypothesize = (referent: wasm.ProjectedReferent): void => {
    if (activeView === null || activeCompletion === null) {
      throw new Error("record an explanation for this task before asking the hypothetical");
    }
    const task = selectedTask(activeView, referent);
    if (activeCompletion.taskKey !== task.key) {
      throw new Error("record an explanation for this task before asking the hypothetical");
    }
    const waiting = new Set(task.waitingKeys);
    const completedRows = activeCompletion.rows.filter(row => row.source.relation === "completed");
    const target = completedRows.find(row => projectedReferentKey(row.subject) === task.key);
    const allowedRows = completedRows
      .filter(row => waiting.has(projectedReferentKey(row.subject)) && row.before === false);
    const allowed = allowedRows.map(row => ({ slot: row.slot, subject: row.subject, value: true }));
    if (target === undefined || allowed.length === 0 || allowed.length > 12) {
      throw new Error("this recorded completion has no bounded prerequisite question");
    }
    const before = scheduleFingerprint(activeView);
    const result = wasm.interveneSession(module, session, wasm.finiteScalarInterventionQuery(
      activeCompletion.step,
      allowed,
      2 ** allowed.length,
      { slot: target.slot, subject: target.subject, equals: true },
    ));
    const answer = object(result, "schedule hypothetical");
    if (typeof answer.found !== "boolean" || typeof answer.completed !== "boolean"
      || typeof answer.exhausted !== "boolean" || typeof answer.evaluations !== "number") {
      throw new Error("the schedule hypothetical answer is malformed");
    }
    const after = scheduleFingerprint(projectSchedule(projectSession(module, session)));
    if (before !== scheduleFingerprint(activeView) || before !== after) {
      throw new Error("the isolated hypothetical changed the schedule");
    }
    const cost = typeof answer.cost === "number" ? answer.cost : null;
    if (answer.found === true && answer.predicted === undefined) {
      throw new Error("the schedule hypothetical omitted its prediction");
    }
    const predicted = answer.found === true ? answer.predicted as wasm.ProjectedValue : null;
    const changedTaskKeys = predicted === null ? [] : allowedRows
      .filter(row => {
        const table = wasm.projectedDiagnosticIndexValue(predicted, row.slot);
        if (table === undefined) {
          throw new Error(`the schedule hypothetical omitted predicted relation ${row.slot}`);
        }
        return wasm.projectedRelationRowValue(table, row.subject) === true;
      })
      .map(row => projectedReferentKey(row.subject));
    if (answer.found === true && changedTaskKeys.length !== cost) {
      throw new Error("the schedule hypothetical answer did not match its predicted change count");
    }
    shell.renderHypothesis({
      taskKey: task.key,
      found: answer.found,
      completed: answer.completed,
      exhausted: answer.exhausted,
      evaluations: answer.evaluations,
      cost,
      changedTaskKeys,
    });
    shell.renderNotice({ tone: "quiet", text: "Hypothetical checked against the recorded schedule; no actual task changed." });
    recordDiagnostic({
      phase: "hypothetical",
      task: task.title,
      before,
      after,
      found: answer.found,
      completed: answer.completed,
      exhausted: answer.exhausted,
      evaluations: answer.evaluations,
      cost,
    });
  };

  const editSource = async (command: Extract<ScheduleCommand, { kind: "source-edit" }>): Promise<void> => {
    if (activeView === null || activeGeneration === null) throw new Error("the schedule has not opened");
    if (command.capturedGeneration !== activeGeneration.generation) {
      throw new Error("this rule changed before your edit; review the current expression");
    }
    const beforeView = activeView;
    const before = scheduleFingerprint(beforeView);
    const beforeProgress = progressFingerprint(beforeView);
    const startedAt = performance.now();
    const response = await fetch("/resident-edit", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(command),
    });
    const body: unknown = await response.json();
    if (!response.ok) {
      const message = object(body, "schedule edit rejection").error;
      const after = scheduleFingerprint(activeView);
      if (after !== before) throw new Error("a rejected source edit changed the schedule");
      shell.renderRuleEditor(editorFor(activeGeneration, {
        tone: "rejected",
        text: "That expression was rejected. The open schedule and its progress are unchanged.",
      }));
      recordDiagnostic({
        phase: "source-edit-rejected",
        generation: activeGeneration.generation,
        before,
        after,
        elapsedMillis: performance.now() - startedAt,
        reason: typeof message === "string" ? message : "rejected",
      });
      return;
    }
    const nextGeneration = parseGeneration(body);
    if (nextGeneration.generation === activeGeneration.generation && nextGeneration.cet1 === null) {
      shell.renderRuleEditor(editorFor(nextGeneration, {
        tone: "quiet",
        text: "The expression is unchanged; the open schedule was retained.",
      }));
      return;
    }
    if (nextGeneration.generation <= activeGeneration.generation || nextGeneration.cet1 === null) {
      throw new Error("the checked source edit lost its generation witness");
    }
    const result = wasm.editSourceSession(
      module,
      session,
      nextGeneration.generation,
      wasm["->ExactProcessRequest"](wasm["decode-cwr1-hex"](nextGeneration.cwr1)),
      wasm["decode-cwr1-hex"](nextGeneration.cet1),
      policy,
    );
    if (result._tag !== "SessionStarted") throw new Error(result.reason);
    session = result.session;
    activeGeneration = nextGeneration;
    actionSequence = 0;
    configurationRevision = 0;
    const continuity = wasm.sourceContinuity(module, session);
    const carried = candidateFor(null);
    if (carried._tag !== "CandidateProduced") throw new Error(carried.reason);
    const admission = completed<workbench.AdmissionCompletion>(done =>
      port.requestAdmission(session, carried.candidate, done));
    if (admission._tag !== "AdmissionAccepted") throw new Error(admission.reason);
    const nextView = shell.renderFrame(projectSession(module, session), continuity);
    activeView = nextView;
    activeCompletion = null;
    shell.renderExplanation(null);
    const afterProgress = progressFingerprint(nextView);
    if (beforeProgress !== afterProgress) {
      throw new Error("the checked rule edit did not retain the open schedule progress");
    }
    const elapsedMillis = performance.now() - startedAt;
    shell.renderRuleEditor(editorFor(nextGeneration, {
      tone: "success",
      text: `Checked change visible in ${elapsedMillis.toFixed(1)} ms; open schedule and progress retained.`,
    }));
    shell.renderNotice({ tone: "success", text: "Schedule rule updated. Current task state and progress were retained." });
    recordDiagnostic({
      phase: "source-edit-visible",
      generation: nextGeneration.generation,
      elapsedMillis,
      compilerMillis: nextGeneration.compilerMicros / 1_000,
      beforeProgress,
      afterProgress,
      oldTaskKeys: beforeView.tasks.map(task => task.key),
      newTaskKeys: nextView.tasks.map(task => task.key),
      continuity,
    });
  };

  dispatch = (command: ScheduleCommand): unknown => {
    if (actionInFlight) return undefined;
    actionInFlight = true;
    shell.setBusy(true);
    const run = async (): Promise<void> => {
      if (command.kind === "schedule-action") runAction(command.channel, command.referent);
      else if (command.kind === "explain") explain(command.referent);
      else if (command.kind === "hypothesis") hypothesize(command.referent);
      else await editSource(command);
    };
    return run().catch(error => {
      console.error("Scheduling workbench operation failed", error);
      document.body.dataset.operationFailure = error instanceof Error ? error.message : String(error);
      shell.renderNotice({ tone: "rejected", text: "That request could not be completed. The schedule remains available." });
    }).finally(() => {
      actionInFlight = false;
      shell.setBusy(false);
    });
  };
}

start().catch(error => {
  console.error("Scheduling application failed to start", error);
  document.body.dataset.runtimeFailure = error instanceof Error ? error.message : String(error);
  mount.innerHTML = `<section class="schedule-unavailable" role="alert"><p class="eyebrow">Project schedule</p><h1>Schedule unavailable</h1><p>The schedule could not be opened. Refresh when it is ready.</p></section>`;
});

window.addEventListener("beforeunload", () => {
  disposeSession?.();
  shell.dispose();
}, { once: true });
