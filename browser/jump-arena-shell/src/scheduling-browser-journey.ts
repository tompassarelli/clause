import { expect } from "bun:test";

interface CdpResponse {
  readonly id?: number;
  readonly result?: { readonly result?: { readonly value?: unknown }; readonly [key: string]: unknown };
  readonly error?: unknown;
}

const pages = await fetch("http://127.0.0.1:9228/json").then(response => response.json()) as readonly {
  readonly url: string;
  readonly webSocketDebuggerUrl: string;
}[];
const page = pages.find(candidate => candidate.url === "http://127.0.0.1:4184/");
if (page === undefined) throw new Error("scheduling browser page is absent");

const socket = new WebSocket(page.webSocketDebuggerUrl);
await new Promise<void>((resolve, reject) => {
  socket.addEventListener("open", () => resolve(), { once: true });
  socket.addEventListener("error", () => reject(new Error("scheduling browser debugger failed to open")), { once: true });
});

let sequence = 0;
const pending = new Map<number, { resolve: (value: CdpResponse) => void; reject: (error: Error) => void }>();
socket.addEventListener("message", event => {
  const response = JSON.parse(String(event.data)) as CdpResponse;
  if (response.id === undefined) return;
  const completion = pending.get(response.id);
  if (completion === undefined) return;
  pending.delete(response.id);
  if (response.error === undefined) completion.resolve(response);
  else completion.reject(new Error(JSON.stringify(response.error)));
});

function send(method: string, params: Record<string, unknown> = {}): Promise<CdpResponse> {
  const id = ++sequence;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate<T>(expression: string): Promise<T> {
  const response = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  return response.result?.result?.value as T;
}

async function waitFor<T>(expression: string): Promise<T> {
  const started = performance.now();
  for (;;) {
    const result = await evaluate<T | null>(expression);
    if (result !== null) return result;
    const operationFailure = await evaluate<string | null>("document.body.dataset.operationFailure ?? null");
    if (operationFailure !== null) throw new Error(`browser operation failed: ${operationFailure}`);
    if (performance.now() - started > 30_000) throw new Error(`browser condition timed out: ${expression}`);
    await Bun.sleep(25);
  }
}

if (Bun.argv.includes("--inspect")) {
  console.log(JSON.stringify(await evaluate("({body: document.body.innerText, runtimeFailure: document.body.dataset.runtimeFailure, operationFailure: document.body.dataset.operationFailure, events: window.__SCHEDULING_WORKBENCH_EVENTS__})"), null, 2));
  socket.close();
  process.exit(0);
}

const eventCount = async (): Promise<number> => evaluate<number>("window.__SCHEDULING_WORKBENCH_EVENTS__.length");
const waitForPhase = (phase: string, after: number) => waitFor<Record<string, unknown>>(`(() => {
  const values = window.__SCHEDULING_WORKBENCH_EVENTS__.slice(${after});
  return values.find(value => value.phase === ${JSON.stringify(phase)}) ?? null;
})()`);
const clickTask = (title: string) => evaluate<void>(`(() => {
  const button = [...document.querySelectorAll('[data-select-task]')]
    .find(candidate => candidate.querySelector('strong')?.textContent === ${JSON.stringify(title)});
  if (!(button instanceof HTMLButtonElement)) throw new Error('task button not found');
  button.click();
})()`);
const selectedTitle = () => evaluate<string>("document.querySelector('#selected-task-title')?.textContent ?? ''");

await send("Page.reload", { ignoreCache: true });
await Bun.sleep(500);
const opened = await evaluate<boolean>("window.__SCHEDULING_WORKBENCH_EVENTS__ !== undefined");
if (!opened) {
  const detail = await evaluate("({body: document.body.innerText, failure: document.body.dataset.runtimeFailure, resources: performance.getEntriesByType('resource').map(value => ({name: value.name, duration: value.duration, size: value.transferSize}))})");
  throw new Error(`scheduling application did not evaluate: ${JSON.stringify(detail)}`);
}
await Bun.sleep(1_000);
if (await eventCount() === 0) {
  const detail = await evaluate("({body: document.body.innerText, failure: document.body.dataset.runtimeFailure, resources: performance.getEntriesByType('resource').map(value => ({name: value.name, duration: value.duration, size: value.transferSize}))})");
  throw new Error(`scheduling application did not open: ${JSON.stringify(detail)}`);
}
await waitFor<number>("window.__SCHEDULING_WORKBENCH_EVENTS__?.length || null");
expect(await evaluate<number>("document.querySelectorAll('[data-select-task]').length")).toBe(5);

await clickTask("Write the guide");
let count = await eventCount();
await evaluate<void>(`(() => {
  const button = [...document.querySelectorAll('.blocker-list li')]
    .find(candidate => candidate.textContent?.includes('Design approval'))?.querySelector('button');
  if (!(button instanceof HTMLButtonElement)) throw new Error('approval control not found');
  button.click();
})()`);
const resolved = await waitForPhase("schedule-action", count);
expect(resolved.channel).toBe("Resolve");
expect(await selectedTitle()).toBe("Write the guide");

count = await eventCount();
await evaluate<void>("document.querySelector('[data-command=\"explain\"]')?.click()");
const explained = await waitForPhase("explained", count);
expect(explained.before).toBe(explained.after);
expect(explained.task).toBe("Write the guide");
expect(await evaluate<string>("document.querySelector('[data-explanation-step]')?.getAttribute('data-explanation-step') ?? ''"))
  .toMatch(/^[0-9a-f]{64}$/);
expect(await evaluate<string>("document.querySelector('[data-explanation-step] strong')?.textContent ?? ''"))
  .toBe("Completion request remained blocked.");

count = await eventCount();
await evaluate<void>("document.querySelector('[data-command=\"hypothesis\"]')?.click()");
const hypothesis = await waitForPhase("hypothetical", count);
expect(hypothesis.before).toBe(hypothesis.after);
expect(hypothesis.found).toBe(true);
expect(await evaluate<string>("document.querySelector('[data-hypothesis-found=\"true\"] strong')?.textContent ?? ''"))
  .toContain("would succeed");
expect(await evaluate<string>("document.querySelector('[data-hypothesis-found=\"true\"]')?.textContent ?? ''"))
  .toContain("complete Design the prototype");

await clickTask("Design the prototype");
count = await eventCount();
await evaluate<void>("document.querySelector('[data-action=\"Complete\"]')?.click()");
const completed = await waitForPhase("schedule-action", count);
expect(completed.channel).toBe("Complete");
await clickTask("Write the guide");
expect(await evaluate<string>("document.querySelector('.relation-grid section:nth-child(2)')?.textContent ?? ''"))
  .toContain("No unfinished prerequisite");

count = await eventCount();
await evaluate<void>(`(() => {
  const input = document.querySelector('#schedule-rule-expression');
  if (!(input instanceof HTMLInputElement)) throw new Error('source expression input absent');
  input.value = 'true';
  input.form?.requestSubmit();
})()`);
const rejected = await waitForPhase("source-edit-rejected", count);
expect(rejected.before).toBe(rejected.after);
expect(await evaluate<string>("document.querySelector('[data-source-edit-status]')?.textContent ?? ''"))
  .toContain("rejected");

const editSamples: Record<string, unknown>[] = [];
for (const expression of ["?prior + 2.0", "?prior + 1.0", "?prior + 2.0"]) {
  count = await eventCount();
  await evaluate<void>(`(() => {
    const input = document.querySelector('#schedule-rule-expression');
    if (!(input instanceof HTMLInputElement)) throw new Error('source expression input absent');
    input.value = ${JSON.stringify(expression)};
    input.form?.requestSubmit();
  })()`);
  const edited = await waitForPhase("source-edit-visible", count);
  expect(edited.beforeProgress).toBe(edited.afterProgress);
  expect(edited.oldTaskKeys).not.toEqual(edited.newTaskKeys);
  expect(edited.elapsedMillis).toBeGreaterThan(0);
  expect(await selectedTitle()).toBe("Write the guide");
  expect(await evaluate<string>("document.querySelector('[data-source-edit-status]')?.textContent ?? ''"))
    .toContain("Checked change visible");
  editSamples.push(edited);
}

await clickTask("Build the prototype");
count = await eventCount();
await evaluate<void>("document.querySelector('[data-action=\"Extend\"]')?.click()");
const extended = await waitForPhase("schedule-action", count);
expect(extended.channel).toBe("Extend");
expect(await evaluate<string>("document.querySelector('.duration-card dd')?.textContent ?? ''")).toBe("6 hours");

const result = await evaluate(`(() => ({
  selected: document.querySelector('#selected-task-title')?.textContent,
  duration: document.querySelector('.duration-card dd')?.textContent,
}))()`);
console.log(JSON.stringify({
  ...result as object,
  explained: { unchanged: explained.before === explained.after, selectedRuleCount: explained.selectedRuleCount },
  hypothetical: {
    unchanged: hypothesis.before === hypothesis.after,
    found: hypothesis.found,
    evaluations: hypothesis.evaluations,
    cost: hypothesis.cost,
  },
  rejectedEditMillis: rejected.elapsedMillis,
  checkedEditSamples: editSamples.map(sample => ({
    generation: sample.generation,
    elapsedMillis: sample.elapsedMillis,
    compilerMillis: sample.compilerMillis,
    progressRetained: sample.beforeProgress === sample.afterProgress,
    identitiesRemapped: JSON.stringify(sample.oldTaskKeys) !== JSON.stringify(sample.newTaskKeys),
  })),
}, null, 2));
socket.close();
