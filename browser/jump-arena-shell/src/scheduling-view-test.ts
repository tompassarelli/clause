import { expect, test } from "bun:test";
import type { ProjectedObject, ProjectedReferent, ProjectedValue } from "./wasm-cartridge-port.js";
import { renderSchedulingWorkspace } from "./scheduling-shell.js";
import { projectSchedule } from "./scheduling-view.js";

const ref = (domain: number, value: number): ProjectedReferent => Object.freeze({
  kind: "referent",
  domain,
  identity: Object.freeze({ kind: "declared", value }),
});

const task = {
  design: ref(1, 1),
  prototype: ref(1, 2),
  validation: ref(1, 3),
  documentation: ref(1, 4),
  launch: ref(1, 5),
};
const root = { approval: ref(2, 1), components: ref(2, 2) };

function table(rows: readonly (readonly [ProjectedReferent, readonly ProjectedValue[]])[]): ProjectedObject {
  return Object.freeze({
    kind: "relation-table",
    rows: Object.freeze(rows.map(([subject, values]) => Object.freeze({ subject, values: Object.freeze(values) }))),
  });
}

function fixture(): ProjectedValue {
  const taskRows = <T extends ProjectedValue>(values: readonly T[]) =>
    Object.values(task).map((subject, index) => [subject, [values[index]]] as const);
  return Object.freeze({
    relations: Object.freeze({
      title: table(taskRows(["Design the prototype", "Build the prototype", "Validate the prototype", "Write the guide", "Launch"])),
      duration: table(taskRows([2, 4, 2, 1, 1])),
      completed: table(taskRows([false, false, false, false, false])),
      reason: table([[root.approval, ["Design approval"]], [root.components, ["Prototype components"]]]),
      prerequisite: table([
        [task.prototype, [task.design]],
        [task.validation, [task.prototype]],
        [task.documentation, [task.design]],
        [task.launch, [task.validation, task.documentation]],
      ]),
      obstruction: table([[task.design, [root.approval]], [task.prototype, [root.components]]]),
      waiting: table([
        [task.prototype, [task.design]],
        [task.validation, [task.prototype, task.design]],
        [task.documentation, [task.design]],
        [task.launch, [task.validation, task.documentation, task.prototype, task.design]],
      ]),
      blocker: table([
        [task.design, [root.approval]],
        [task.prototype, [root.approval, root.components]],
        [task.validation, [root.approval, root.components]],
        [task.documentation, [root.approval]],
        [task.launch, [root.approval, root.components]],
      ]),
    }),
  });
}

test("joins authored titles and reasons to projected identities without inventing schedule order", () => {
  const view = projectSchedule(fixture());
  expect(view.tasks.map(value => value.title)).toEqual([
    "Design the prototype",
    "Build the prototype",
    "Validate the prototype",
    "Write the guide",
    "Launch",
  ]);
  const prototype = view.tasks[1];
  expect(prototype.duration).toBe(4);
  expect(prototype.completed).toBe(false);
  expect(prototype.prerequisiteKeys.map(key => view.tasksByKey[key].title)).toEqual(["Design the prototype"]);
  expect(prototype.waitingKeys.map(key => view.tasksByKey[key].title)).toEqual(["Design the prototype"]);
  expect(prototype.blockerKeys.map(key => view.rootsByKey[key].reason)).toEqual([
    "Design approval",
    "Prototype components",
  ]);
});

test("renders the selected task's actual relationships and typed schedule controls", () => {
  const view = projectSchedule(fixture());
  const prototype = view.tasks[1];
  const markup = renderSchedulingWorkspace(view, prototype.key);
  expect(markup).toContain("Build the prototype");
  expect(markup).toContain("4 hours");
  expect(markup).toContain("Direct prerequisites");
  expect(markup).toContain("Still waiting on");
  expect(markup).toContain("Design approval");
  expect(markup).toContain('data-action="Resolve"');
  expect(markup).toContain('data-action="Complete"');
  expect(markup).toContain('data-action="Extend"');
  expect(markup).not.toContain("eligible");
});
