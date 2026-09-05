import { expect, test } from "bun:test";
import { file } from "bun";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
import { projectSchedule, scheduleFingerprint, type ScheduleView } from "./scheduling-view.js";

function completed<T>(action: (done: (value: T) => unknown) => unknown): T {
  const values: T[] = [];
  action(value => values.push(value));
  expect(values).toHaveLength(1);
  return values[0];
}

test("fresh Wasm projects and changes the accepted scheduling world", async () => {
  const generated = new URL("../../../build/scheduling/wasm/clause_runtime.js", import.meta.url);
  const module = await import(generated.href);
  module.initSync({
    module: await file(new URL("../../../build/scheduling/wasm/clause_runtime_bg.wasm", import.meta.url)).arrayBuffer(),
  });

  const maximum = Number.MAX_SAFE_INTEGER;
  const policy = workbench["->WorkbenchPolicy"](
    16,
    8,
    64,
    wasm["cse1-projected-term-max-properties"],
    wasm["cse1-projected-term-json-max-source-units"],
    workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum),
  );
  const port = wasm["create-wasm-cartridge-port"](module, policy);
  const request = [...new Uint8Array(await file(new URL("../../../build/scheduling/initial.cwr1", import.meta.url)).arrayBuffer())];
  const accepted = completed<workbench.PackageCheck>(done =>
    port.acceptPackage(wasm["->ExactProcessRequest"](request), done));
  if (accepted._tag !== "PackageAccepted") throw new Error(accepted.reason);
  const started = completed<workbench.SessionCompletion>(done =>
    port.startSession(accepted.acceptedPackage, 1, done));
  if (started._tag !== "SessionStarted") throw new Error(started.reason);

  const session = started.session;
  let inputSequence = 0;
  let configurationRevision = 0;
  let current = projectSchedule(wasm.projectSession(module, session));

  const taskTitles = current.tasks.map(task => task.title);
  expect(taskTitles).toHaveLength(5);
  expect(new Set(taskTitles)).toEqual(new Set([
    "Design the prototype",
    "Build the prototype",
    "Validate the prototype",
    "Write the guide",
    "Launch",
  ]));
  expect(current.tasks.reduce((count, task) => count + task.blockerKeys.length, 0)).toBe(8);

  const change = (channel: "Resolve" | "Complete" | "Extend", referent: wasm.ProjectedReferent): ScheduleView => {
    const previous = current;
    const input = {
      kind: "referent-input",
      generation: 1,
      channel,
      value: wasm.checkedProjectedReferent(referent),
    };
    const observation = workbench["->InputObservation"](
      ++inputSequence,
      workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(input)])),
    );
    const candidate = completed<workbench.CandidateCompletion>(done =>
      port.runCandidate(
        session,
        workbench["->FixedTick"](1),
        workbench["->InputConfiguration"](++configurationRevision, [observation]),
        done,
      ));
    if (candidate._tag !== "CandidateProduced") throw new Error(candidate.reason);
    expect(scheduleFingerprint(projectSchedule(wasm.projectSession(module, session))))
      .toBe(scheduleFingerprint(previous));
    const admission = completed<workbench.AdmissionCompletion>(done =>
      port.requestAdmission(session, candidate.candidate, done));
    if (admission._tag !== "AdmissionAccepted") throw new Error(admission.reason);
    current = projectSchedule(wasm.projectSession(module, session));
    return current;
  };

  const design = current.tasks.find(task => task.title === "Design the prototype");
  const prototype = current.tasks.find(task => task.title === "Build the prototype");
  const approval = Object.values(current.rootsByKey).find(root => root.reason === "Design approval");
  if (design === undefined || prototype === undefined || approval === undefined) {
    throw new Error("initial scheduling projection is incomplete");
  }

  expect(change("Complete", design.referent).tasks.find(task => task.title === design.title)?.completed).toBe(false);
  expect(change("Resolve", approval.referent).tasks.reduce((count, task) => count + task.blockerKeys.length, 0)).toBe(3);
  const completedDesign = change("Complete", design.referent).tasks.find(task => task.title === design.title);
  expect(completedDesign?.completed).toBe(true);
  expect(completedDesign?.referent).toEqual(design.referent);
  expect(change("Extend", prototype.referent).tasks.find(task => task.title === prototype.title)?.duration).toBe(5);

  port.disposeSession(session);
});
