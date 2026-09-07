import type { ProjectedReferent, ProjectedValue } from "../../runtime/src/wasm-cartridge-port.js";
import {
  continuedReferentKey,
  projectSchedule,
  type ScheduleTaskView,
  type ScheduleView,
} from "./scheduling-view.js";

export type ScheduleActionChannel = "Resolve" | "Complete" | "Extend";

export type ScheduleCommand =
  | Readonly<{ kind: "schedule-action"; channel: ScheduleActionChannel; referent: ProjectedReferent }>
  | Readonly<{ kind: "explain"; referent: ProjectedReferent }>
  | Readonly<{ kind: "hypothesis"; referent: ProjectedReferent }>
  | Readonly<{
      kind: "source-edit";
      capturedGeneration: number;
      catalogIndex: number;
      expression: string;
    }>;

export interface ScheduleNotice {
  readonly tone: "quiet" | "success" | "rejected";
  readonly text: string;
}

export interface ScheduleExplanation {
  readonly taskKey: string;
  readonly step: string;
  readonly completed: boolean;
}

export interface ScheduleHypothesis {
  readonly taskKey: string;
  readonly found: boolean;
  readonly completed: boolean;
  readonly exhausted: boolean;
  readonly evaluations: number;
  readonly cost: number | null;
  readonly changedTaskKeys: readonly string[];
}

export interface ScheduleRuleEditor {
  readonly generation: number;
  readonly catalogIndex: number;
  readonly expression: string;
  readonly start: number;
  readonly end: number;
  readonly status: ScheduleNotice;
}

export interface SchedulingShell {
  readonly renderFrame: (frame: ProjectedValue, continuity?: ProjectedValue) => ScheduleView;
  readonly renderNotice: (notice: ScheduleNotice) => void;
  readonly renderExplanation: (explanation: ScheduleExplanation | null) => void;
  readonly renderHypothesis: (hypothesis: ScheduleHypothesis | null) => void;
  readonly renderRuleEditor: (editor: ScheduleRuleEditor | null) => void;
  readonly setBusy: (busy: boolean) => void;
  readonly dispose: () => void;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function taskTitle(view: ScheduleView, key: string): string {
  const task = view.tasksByKey[key];
  if (task === undefined) throw new Error("a projected task relationship has no title");
  return task.title;
}

function durationLabel(value: number): string {
  return `${value.toLocaleString("en-US", { maximumFractionDigits: 2 })} ${value === 1 ? "hour" : "hours"}`;
}

function relationshipList(values: readonly string[], empty: string): string {
  return values.length === 0
    ? `<p class="empty-relation">${escapeHtml(empty)}</p>`
    : `<ul class="relationship-list">${values.map(value => `<li>${escapeHtml(value)}</li>`).join("")}</ul>`;
}

function taskListItem(view: ScheduleView, task: ScheduleTaskView, selectedKey: string, busy: boolean): string {
  const prerequisites = task.prerequisiteKeys.map(key => taskTitle(view, key));
  const relationship = prerequisites.length === 0
    ? "Starts independently"
    : `After ${prerequisites.join(", ")}`;
  return `<li class="task-node${task.completed ? " is-complete" : ""}">
    <button type="button" class="task-select" data-select-task="${escapeHtml(task.key)}"
      aria-current="${task.key === selectedKey ? "true" : "false"}" ${busy ? "disabled" : ""}>
      <span class="task-state" aria-hidden="true">${task.completed ? "✓" : "○"}</span>
      <span class="task-copy"><strong>${escapeHtml(task.title)}</strong><small>${escapeHtml(relationship)}</small></span>
      <span class="task-duration">${escapeHtml(durationLabel(task.duration))}</span>
    </button>
  </li>`;
}

function blockerControls(view: ScheduleView, task: ScheduleTaskView, busy: boolean): string {
  if (task.blockerKeys.length === 0) {
    return `<p class="empty-relation">No blocker reasons are listed.</p>`;
  }
  return `<ul class="blocker-list">${task.blockerKeys.map(key => {
    const root = view.rootsByKey[key];
    if (root === undefined) throw new Error("a projected blocker has no reason");
    return `<li><span>${escapeHtml(root.reason)}</span><button type="button" class="subtle-action"
      data-action="Resolve" data-referent="${escapeHtml(root.key)}" ${busy ? "disabled" : ""}>
      Resolve
    </button></li>`;
  }).join("")}</ul>`;
}

function explanationPanel(
  view: ScheduleView,
  selected: ScheduleTaskView,
  explanation: ScheduleExplanation | null,
  busy: boolean,
): string {
  const active = explanation?.taskKey === selected.key ? explanation : null;
  const waiting = selected.waitingKeys.map(key => taskTitle(view, key));
  const blockers = selected.blockerKeys.map(key => view.rootsByKey[key]?.reason ?? "Unknown blocker");
  const result = active === null
    ? `<p class="empty-relation">Check this completion request to record why it can or cannot proceed.</p>`
    : `<div class="diagnostic-result" data-explanation-step="${escapeHtml(active.step)}">
        <p><strong>${active.completed ? "Completion request succeeded." : "Completion request remained blocked."}</strong></p>
        <p>${waiting.length === 0 ? "No unfinished prerequisite was read." : `Waiting on ${escapeHtml(waiting.join(", "))}.`}</p>
        <p>${blockers.length === 0 ? "No blocker reason was derived." : `Derived blockers: ${escapeHtml(blockers.join(", "))}.`}</p>
        <small>Recorded check ${escapeHtml(active.step.slice(0, 12))}</small>
      </div>`;
  return `<section class="workbench-card explanation-card">
    <div class="workbench-title"><div><p class="eyebrow">Same-rule explanation</p><h3>Why this request?</h3></div>
      <button type="button" class="secondary-action" data-command="explain" data-referent="${escapeHtml(selected.key)}"
        ${busy || selected.completed || (selected.waitingKeys.length === 0 && selected.blockerKeys.length === 0) ? "disabled" : ""}>Check and explain</button></div>
    ${result}
  </section>`;
}

function hypothesisPanel(
  view: ScheduleView,
  selected: ScheduleTaskView,
  hypothesis: ScheduleHypothesis | null,
  busy: boolean,
): string {
  const active = hypothesis?.taskKey === selected.key ? hypothesis : null;
  const result = active === null
    ? `<p class="empty-relation">After recording an explanation, test whether completing current prerequisites would make this request succeed. The schedule will not change.</p>`
    : active.found
      ? `<div class="diagnostic-result is-found" data-hypothesis-found="true">
          <p><strong>Yes. The completion request would succeed.</strong></p>
          <p>Smallest change: complete ${escapeHtml(active.changedTaskKeys.map(key => taskTitle(view, key)).join(", "))}.</p>
          <small>${active.evaluations} checked ${active.evaluations === 1 ? "case" : "cases"}; cost ${active.cost ?? 0}; authoritative schedule unchanged</small>
        </div>`
      : `<div class="diagnostic-result" data-hypothesis-found="false">
          <p><strong>No satisfying case was found within this finite question.</strong></p>
          <small>${active.evaluations} checked ${active.evaluations === 1 ? "case" : "cases"}; ${active.exhausted ? "bound exhausted" : active.completed ? "all cases checked" : "incomplete"}; authoritative schedule unchanged</small>
        </div>`;
  return `<section class="workbench-card hypothesis-card">
    <div class="workbench-title"><div><p class="eyebrow">Finite hypothetical</p><h3>What if waiting tasks were complete?</h3></div>
      <button type="button" class="secondary-action" data-command="hypothesis" data-referent="${escapeHtml(selected.key)}"
        ${busy || selected.waitingKeys.length === 0 ? "disabled" : ""}>Run hypothetical</button></div>
    ${result}
  </section>`;
}

function ruleEditorPanel(editor: ScheduleRuleEditor | null, busy: boolean): string {
  if (editor === null) return "";
  return `<section class="workbench-card source-card">
    <div class="workbench-title"><div><p class="eyebrow">Live schedule rule</p><h3>Extension amount</h3></div>
      <span class="source-generation">version ${editor.generation}</span></div>
    <p class="source-help">Change the existing extension expression. A valid change keeps this open schedule and its progress; an invalid change is rejected.</p>
    <form class="source-edit-form" data-source-edit>
      <label for="schedule-rule-expression">Rule expression</label>
      <div class="source-edit-row"><input id="schedule-rule-expression" name="expression" value="${escapeHtml(editor.expression)}"
        autocomplete="off" spellcheck="false" ${busy ? "disabled" : ""}>
        <button type="submit" class="primary-action" ${busy ? "disabled" : ""}>Check and apply</button></div>
      <input type="hidden" name="generation" value="${editor.generation}">
      <input type="hidden" name="catalogIndex" value="${editor.catalogIndex}">
      <small>Exact offered source span ${editor.start}–${editor.end}</small>
    </form>
    <p class="schedule-notice is-${editor.status.tone}" data-source-edit-status role="status" aria-live="polite">${escapeHtml(editor.status.text)}</p>
  </section>`;
}

export function renderSchedulingWorkspace(
  view: ScheduleView,
  selectedKey: string,
  notice: ScheduleNotice = { tone: "quiet", text: "Choose a task to inspect its schedule." },
  busy = false,
  explanation: ScheduleExplanation | null = null,
  hypothesis: ScheduleHypothesis | null = null,
  editor: ScheduleRuleEditor | null = null,
): string {
  const selected = view.tasksByKey[selectedKey] ?? view.tasks[0];
  if (selected === undefined) throw new Error("the schedule contains no tasks");
  const prerequisites = selected.prerequisiteKeys.map(key => taskTitle(view, key));
  const waiting = selected.waitingKeys.map(key => taskTitle(view, key));
  const completedCount = view.tasks.filter(task => task.completed).length;

  return `<div class="schedule-workspace">
    <header class="schedule-heading">
      <div><p class="eyebrow">Project schedule</p><h1>Prototype launch plan</h1></div>
      <p class="schedule-progress"><strong>${completedCount}</strong><span>of ${view.tasks.length} complete</span></p>
    </header>
    <div class="schedule-layout">
      <section class="task-graph" aria-labelledby="task-list-title">
        <div class="section-heading"><h2 id="task-list-title">Tasks</h2><span>${view.tasks.length} tasks</span></div>
        <ol>${view.tasks.map(task => taskListItem(view, task, selected.key, busy)).join("")}</ol>
      </section>
      <section class="task-detail" aria-labelledby="selected-task-title">
        <div class="detail-title">
          <div><p class="eyebrow">Selected task</p><h2 id="selected-task-title">${escapeHtml(selected.title)}</h2></div>
          <span class="completion-value ${selected.completed ? "is-complete" : ""}">${selected.completed ? "Completed" : "Not completed"}</span>
        </div>
        <dl class="duration-card"><div><dt>Actual duration</dt><dd>${escapeHtml(durationLabel(selected.duration))}</dd></div></dl>
        <div class="relation-grid">
          <section><h3>Direct prerequisites</h3>${relationshipList(prerequisites, "No direct prerequisites.")}</section>
          <section><h3>Still waiting on</h3>${relationshipList(waiting, "No unfinished prerequisite is listed.")}</section>
        </div>
        <section class="blockers"><h3>Blocker reasons</h3>${blockerControls(view, selected, busy)}</section>
        <div class="task-actions">
          <button type="button" class="primary-action" data-action="Complete" data-referent="${escapeHtml(selected.key)}"
            ${busy || selected.completed ? "disabled" : ""}>Request completion</button>
          <button type="button" class="secondary-action" data-action="Extend" data-referent="${escapeHtml(selected.key)}"
            ${busy ? "disabled" : ""}>Extend from rule</button>
        </div>
        <p class="schedule-notice is-${notice.tone}" role="status" aria-live="polite">${escapeHtml(notice.text)}</p>
        <div class="workbench-stack">
          ${explanationPanel(view, selected, explanation, busy)}
          ${hypothesisPanel(view, selected, hypothesis, busy)}
          ${ruleEditorPanel(editor, busy)}
        </div>
      </section>
    </div>
  </div>`;
}

export function createSchedulingShell(
  mount: HTMLElement,
  emitCommand: (command: ScheduleCommand) => unknown,
): SchedulingShell {
  let view: ScheduleView | null = null;
  let selectedKey = "";
  let notice: ScheduleNotice = { tone: "quiet", text: "Choose a task to inspect its schedule." };
  let explanation: ScheduleExplanation | null = null;
  let hypothesis: ScheduleHypothesis | null = null;
  let editor: ScheduleRuleEditor | null = null;
  let busy = false;

  const render = (): void => {
    if (view === null) return;
    if (view.tasksByKey[selectedKey] === undefined) selectedKey = view.tasks[0]?.key ?? "";
    mount.innerHTML = renderSchedulingWorkspace(view, selectedKey, notice, busy, explanation, hypothesis, editor);
  };

  const click = (event: Event): void => {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const selection = target.closest<HTMLButtonElement>("[data-select-task]");
    if (selection !== null && selection.dataset.selectTask !== undefined) {
      selectedKey = selection.dataset.selectTask;
      render();
      return;
    }
    const diagnostic = target.closest<HTMLButtonElement>("[data-command][data-referent]");
    if (diagnostic !== null && !diagnostic.disabled && view !== null) {
      const task = diagnostic.dataset.referent === undefined
        ? undefined
        : view.tasksByKey[diagnostic.dataset.referent];
      if (task !== undefined && (diagnostic.dataset.command === "explain" || diagnostic.dataset.command === "hypothesis")) {
        emitCommand(Object.freeze({ kind: diagnostic.dataset.command, referent: task.referent }));
      }
      return;
    }
    const control = target.closest<HTMLButtonElement>("[data-action][data-referent]");
    if (control === null || control.disabled || view === null) return;
    const channel = control.dataset.action;
    const key = control.dataset.referent;
    if ((channel !== "Resolve" && channel !== "Complete" && channel !== "Extend") || key === undefined) return;
    const referent = channel === "Resolve" ? view.rootsByKey[key]?.referent : view.tasksByKey[key]?.referent;
    if (referent !== undefined) emitCommand(Object.freeze({ kind: "schedule-action", channel, referent }));
  };

  const submit = (event: SubmitEvent): void => {
    const form = event.target;
    if (!(form instanceof HTMLFormElement) || !form.matches("[data-source-edit]")) return;
    event.preventDefault();
    const data = new FormData(form);
    const expression = data.get("expression");
    const generation = Number.parseInt(String(data.get("generation")), 10);
    const catalogIndex = Number.parseInt(String(data.get("catalogIndex")), 10);
    if (typeof expression !== "string" || !Number.isSafeInteger(generation) || !Number.isSafeInteger(catalogIndex)) return;
    emitCommand(Object.freeze({ kind: "source-edit", capturedGeneration: generation, catalogIndex, expression }));
  };

  mount.addEventListener("click", click);
  mount.addEventListener("submit", submit);
  return Object.freeze({
    renderFrame(frame: ProjectedValue, continuity?: ProjectedValue): ScheduleView {
      if (continuity !== undefined && view !== null && view.tasksByKey[selectedKey] !== undefined) {
        selectedKey = continuedReferentKey(view.tasksByKey[selectedKey]!.referent, continuity);
      }
      view = projectSchedule(frame);
      render();
      return view;
    },
    renderNotice(next: ScheduleNotice): void {
      notice = next;
      render();
    },
    renderExplanation(next: ScheduleExplanation | null): void {
      explanation = next;
      hypothesis = null;
      render();
    },
    renderHypothesis(next: ScheduleHypothesis | null): void {
      hypothesis = next;
      render();
    },
    renderRuleEditor(next: ScheduleRuleEditor | null): void {
      editor = next;
      render();
    },
    setBusy(next: boolean): void {
      busy = next;
      render();
    },
    dispose(): void {
      mount.removeEventListener("click", click);
      mount.removeEventListener("submit", submit);
      mount.replaceChildren();
      view = null;
    },
  });
}
