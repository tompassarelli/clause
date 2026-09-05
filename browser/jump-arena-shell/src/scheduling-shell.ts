import type { ProjectedReferent, ProjectedValue } from "./wasm-cartridge-port.js";
import { projectSchedule, type ScheduleTaskView, type ScheduleView } from "./scheduling-view.js";

export type ScheduleActionChannel = "Resolve" | "Complete" | "Extend";

export interface ScheduleAction {
  readonly channel: ScheduleActionChannel;
  readonly referent: ProjectedReferent;
}

export interface ScheduleNotice {
  readonly tone: "quiet" | "success" | "rejected";
  readonly text: string;
}

export interface SchedulingShell {
  readonly renderFrame: (frame: ProjectedValue) => ScheduleView;
  readonly renderNotice: (notice: ScheduleNotice) => void;
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

export function renderSchedulingWorkspace(
  view: ScheduleView,
  selectedKey: string,
  notice: ScheduleNotice = { tone: "quiet", text: "Choose a task to inspect its schedule." },
  busy = false,
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
            ${busy ? "disabled" : ""}>Extend by 1 hour</button>
        </div>
        <p class="schedule-notice is-${notice.tone}" role="status" aria-live="polite">${escapeHtml(notice.text)}</p>
      </section>
    </div>
  </div>`;
}

export function createSchedulingShell(
  mount: HTMLElement,
  emitAction: (action: ScheduleAction) => unknown,
): SchedulingShell {
  let view: ScheduleView | null = null;
  let selectedKey = "";
  let notice: ScheduleNotice = { tone: "quiet", text: "Choose a task to inspect its schedule." };
  let busy = false;

  const render = (): void => {
    if (view === null) return;
    mount.innerHTML = renderSchedulingWorkspace(view, selectedKey, notice, busy);
    selectedKey = view.tasksByKey[selectedKey] === undefined ? view.tasks[0]?.key ?? "" : selectedKey;
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
    const control = target.closest<HTMLButtonElement>("[data-action][data-referent]");
    if (control === null || control.disabled || view === null) return;
    const channel = control.dataset.action;
    const key = control.dataset.referent;
    if ((channel !== "Resolve" && channel !== "Complete" && channel !== "Extend") || key === undefined) return;
    const referent = channel === "Resolve" ? view.rootsByKey[key]?.referent : view.tasksByKey[key]?.referent;
    if (referent !== undefined) emitAction(Object.freeze({ channel, referent }));
  };

  mount.addEventListener("click", click);
  return Object.freeze({
    renderFrame(frame: ProjectedValue): ScheduleView {
      view = projectSchedule(frame);
      if (view.tasksByKey[selectedKey] === undefined) selectedKey = view.tasks[0]?.key ?? "";
      render();
      return view;
    },
    renderNotice(next: ScheduleNotice): void {
      notice = next;
      render();
    },
    setBusy(next: boolean): void {
      busy = next;
      render();
    },
    dispose(): void {
      mount.removeEventListener("click", click);
      mount.replaceChildren();
      view = null;
    },
  });
}
