import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import { createSchedulingShell, type ScheduleAction } from "./scheduling-shell.js";
import { scheduleFingerprint, type ScheduleView } from "./scheduling-view.js";

const runtimeModulePath = "/wasm/clause_runtime.js";
const runtimeBytesPath = "/wasm/clause_runtime_bg.wasm";
const schedulePath = "/schedule/initial.cwr1";
const sourceGeneration = 1;

interface SchedulingWasmModule {
  readonly initSync: (input: { readonly module: ArrayBuffer }) => unknown;
  readonly [key: string]: unknown;
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

function projectSession(module: SchedulingWasmModule, session: unknown): wasm.ProjectedValue {
  return wasm.projectSession(module, session);
}

function inputEnvelope(policy: workbench.WorkbenchPolicy, value: unknown): workbench.WorkbenchEnvelope {
  return workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)]));
}

const mount = document.querySelector("#schedule-app");
if (!(mount instanceof HTMLElement)) throw new Error("missing scheduling application mount");

let disposeSession: (() => unknown) | null = null;
let activeView: ScheduleView | null = null;
let actionSequence = 0;
let configurationRevision = 0;
let actionInFlight = false;
let dispatch: (action: ScheduleAction) => unknown = () => undefined;
const shell = createSchedulingShell(mount, action => dispatch(action));

async function start(): Promise<void> {
  const [imported, moduleBytes, requestBytes] = await Promise.all([
    import(runtimeModulePath),
    fetchBytes(runtimeBytesPath),
    fetchBytes(schedulePath),
  ]);
  if (typeof imported.initSync !== "function") throw new Error("scheduling module has no initializer");
  const module = imported as SchedulingWasmModule;
  module.initSync({ module: moduleBytes });
  const policy = schedulePolicy();
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const accepted = completed<workbench.PackageCheck>(done =>
    port.acceptPackage(wasm["->ExactProcessRequest"]([...new Uint8Array(requestBytes)]), done));
  if (accepted._tag !== "PackageAccepted") throw new Error(accepted.reason);
  const started = completed<workbench.SessionCompletion>(done =>
    port.startSession(accepted.acceptedPackage, sourceGeneration, done));
  if (started._tag !== "SessionStarted") throw new Error(started.reason);
  const session = started.session;
  disposeSession = () => port.disposeSession(session);
  activeView = shell.renderFrame(projectSession(module, session));
  shell.renderNotice({ tone: "quiet", text: "Choose a task, resolve its blockers, or request a schedule change." });

  dispatch = (action: ScheduleAction): unknown => {
    if (actionInFlight) return undefined;
    actionInFlight = true;
    shell.setBusy(true);
    try {
      const input = Object.freeze({
        kind: "referent-input",
        generation: sourceGeneration,
        channel: action.channel,
        value: wasm.checkedProjectedReferent(action.referent),
      });
      const observation = workbench["->InputObservation"](
        ++actionSequence,
        inputEnvelope(policy, input),
      );
      const candidate = completed<workbench.CandidateCompletion>(done =>
        port.runCandidate(
          session,
          workbench["->FixedTick"](1),
          workbench["->InputConfiguration"](++configurationRevision, [observation]),
          done,
        ));
      if (candidate._tag !== "CandidateProduced") {
        console.error("Scheduling candidate rejected", candidate.reason);
        shell.renderNotice({ tone: "rejected", text: "That change was rejected. Review the task's current blockers." });
        return undefined;
      }
      const admission = completed<workbench.AdmissionCompletion>(done =>
        port.requestAdmission(session, candidate.candidate, done));
      if (admission._tag !== "AdmissionAccepted") {
        console.error("Scheduling admission rejected", admission.reason);
        shell.renderNotice({ tone: "rejected", text: "That change was not accepted. The schedule is unchanged." });
        return undefined;
      }
      const previous = activeView;
      const next = shell.renderFrame(projectSession(module, session));
      activeView = next;
      shell.renderNotice(previous !== null && scheduleFingerprint(previous) === scheduleFingerprint(next)
        ? { tone: "rejected", text: "No schedule change was accepted." }
        : { tone: "success", text: "Schedule updated." });
      return undefined;
    } catch (error) {
      console.error("Scheduling action failed", error);
      shell.renderNotice({ tone: "rejected", text: "That change could not be completed. The schedule is unchanged." });
      return undefined;
    } finally {
      actionInFlight = false;
      shell.setBusy(false);
    }
  };
}

start().catch(error => {
  console.error("Scheduling application failed to start", error);
  mount.innerHTML = `<section class="schedule-unavailable" role="alert"><p class="eyebrow">Project schedule</p><h1>Schedule unavailable</h1><p>The schedule could not be opened. Refresh when it is ready.</p></section>`;
});

window.addEventListener("beforeunload", () => {
  disposeSession?.();
  shell.dispose();
}, { once: true });
