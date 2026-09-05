import {
  checkedProjectedReferent,
  type ProjectedObject,
  type ProjectedReferent,
  type ProjectedValue,
} from "./wasm-cartridge-port.js";

export interface ScheduleTaskView {
  readonly key: string;
  readonly referent: ProjectedReferent;
  readonly title: string;
  readonly duration: number;
  readonly completed: boolean;
  readonly prerequisiteKeys: readonly string[];
  readonly waitingKeys: readonly string[];
  readonly blockerKeys: readonly string[];
}

export interface ScheduleRootView {
  readonly key: string;
  readonly referent: ProjectedReferent;
  readonly reason: string;
}

export interface ScheduleView {
  readonly tasks: readonly ScheduleTaskView[];
  readonly tasksByKey: Readonly<Record<string, ScheduleTaskView>>;
  readonly rootsByKey: Readonly<Record<string, ScheduleRootView>>;
}

interface ProjectedRelationRow {
  readonly subject: ProjectedReferent;
  readonly values: readonly ProjectedValue[];
}

function projectedObject(value: ProjectedValue, label: string): ProjectedObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} is not a projected object`);
  }
  return value as ProjectedObject;
}

function relationRows(frame: ProjectedValue, name: string): readonly ProjectedRelationRow[] {
  const relations = projectedObject(projectedObject(frame, "schedule").relations, "schedule relations");
  const table = projectedObject(relations[name], `${name} relation`);
  if (table.kind !== "relation-table" || !Array.isArray(table.rows)) {
    throw new Error(`${name} is not a projected relation`);
  }
  return table.rows.map((value, index) => {
    const row = projectedObject(value, `${name} row ${index + 1}`);
    if (!Array.isArray(row.values)) {
      throw new Error(`${name} row ${index + 1} has no projected values`);
    }
    return Object.freeze({
      subject: checkedProjectedReferent(row.subject),
      values: Object.freeze([...row.values]),
    });
  });
}

export function projectedReferentKey(referent: ProjectedReferent): string {
  const checked = checkedProjectedReferent(referent);
  const identity = checked.identity.kind === "declared"
    ? `declared-${checked.identity.value}`
    : `created-${checked.identity.value.map(byte => byte.toString(16).padStart(2, "0")).join("")}`;
  return `${checked.domain}-${identity}`;
}

function oneValue(row: ProjectedRelationRow, relation: string): ProjectedValue {
  if (row.values.length !== 1) {
    throw new Error(`${relation} must project exactly one value per subject`);
  }
  return row.values[0];
}

function scalarValues<T extends string | number | boolean>(
  frame: ProjectedValue,
  relation: string,
  kind: "string" | "number" | "boolean",
): ReadonlyMap<string, T> {
  const values = new Map<string, T>();
  for (const row of relationRows(frame, relation)) {
    const value = oneValue(row, relation);
    if (typeof value !== kind || (kind === "number" && !Number.isFinite(value))) {
      throw new Error(`${relation} projected an invalid value`);
    }
    values.set(projectedReferentKey(row.subject), value as T);
  }
  return values;
}

function referentValues(frame: ProjectedValue, relation: string): ReadonlyMap<string, readonly string[]> {
  const values = new Map<string, readonly string[]>();
  for (const row of relationRows(frame, relation)) {
    values.set(
      projectedReferentKey(row.subject),
      Object.freeze(row.values.map(value => projectedReferentKey(checkedProjectedReferent(value)))),
    );
  }
  return values;
}

function recordOf<T extends { readonly key: string }>(values: readonly T[]): Readonly<Record<string, T>> {
  return Object.freeze(Object.fromEntries(values.map(value => [value.key, value])));
}

export function projectSchedule(frame: ProjectedValue): ScheduleView {
  const titles = scalarValues<string>(frame, "title", "string");
  const durations = scalarValues<number>(frame, "duration", "number");
  const completed = scalarValues<boolean>(frame, "completed", "boolean");
  const prerequisites = referentValues(frame, "prerequisite");
  const waiting = referentValues(frame, "waiting");
  const blockers = referentValues(frame, "blocker");

  // The title relation supplies the projected task sequence. The browser keeps
  // that order; it does not infer another scheduling order.
  const tasks = relationRows(frame, "title").map(row => {
    const key = projectedReferentKey(row.subject);
    const title = titles.get(key);
    const duration = durations.get(key);
    const isCompleted = completed.get(key);
    if (title === undefined || duration === undefined || isCompleted === undefined) {
      throw new Error("a scheduled task is missing an authored field");
    }
    return Object.freeze({
      key,
      referent: row.subject,
      title,
      duration,
      completed: isCompleted,
      prerequisiteKeys: prerequisites.get(key) ?? Object.freeze([]),
      waitingKeys: waiting.get(key) ?? Object.freeze([]),
      blockerKeys: blockers.get(key) ?? Object.freeze([]),
    });
  });

  const roots = relationRows(frame, "reason").map(row => {
    const reason = oneValue(row, "reason");
    if (typeof reason !== "string") throw new Error("an obstruction has no reason");
    return Object.freeze({ key: projectedReferentKey(row.subject), referent: row.subject, reason });
  });

  return Object.freeze({
    tasks: Object.freeze(tasks),
    tasksByKey: recordOf(tasks),
    rootsByKey: recordOf(roots),
  });
}

export function scheduleFingerprint(view: ScheduleView): string {
  return JSON.stringify(view.tasks.map(task => [
    task.key,
    task.duration,
    task.completed,
    task.prerequisiteKeys,
    task.waitingKeys,
    task.blockerKeys,
  ]));
}
