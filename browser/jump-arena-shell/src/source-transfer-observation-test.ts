import { expect, test } from "bun:test";
import {
  beginSourceTransferObservation as begin, finishSourceTransferObservation as finish,
  enterSourceTransferPhase as enter, leaveSourceTransferPhase as leave,
  observeSourceTransferPhase as observe,
} from "./source-transfer-observation.js";
import { settleRetiredWasmSession } from "./wasm-test-lifecycle.js";

test("outer transfer observation is opt-in and preserves returns, throws and nested exclusive accounting", () => {
  expect(observe("adapter", () => 7)).toBe(7);
  expect(finish()).toBeNull();
  expect(begin()).toBe(true); expect(begin()).toBe(false);
  const failure = new Error("unchanged failure");
  expect(() => observe("adapter", () => observe("cartridge-parse", () => { throw failure; }))).toThrow(failure);
  const report = finish();
  if (!report) throw new Error("missing outer observation");
  expect(report.truncated).toBe(false);
  expect(report.phases.adapter.calls).toBe(1);
  expect(report.phases["cartridge-parse"].calls).toBe(1);
  expect(report.phases.adapter.inclusiveMs).toBeGreaterThanOrEqual(report.phases["cartridge-parse"].inclusiveMs);
  expect(Math.abs(report.phases.adapter.inclusiveMs - report.phases.adapter.exclusiveMs - report.phases["cartridge-parse"].inclusiveMs)).toBeLessThan(0.001);
  for (const phase of Object.values(report.phases)) {
    expect(Number.isFinite(phase.inclusiveMs)).toBe(true);
    expect(Number.isFinite(phase.exclusiveMs)).toBe(true);
  }
  expect(finish()).toBeNull();
});

test("outer phase nesting is bounded and unfinished observations cannot be reset", () => {
  expect(begin()).toBe(true);
  const scopes = Array.from({ length: 65 }, () => enter("adapter"));
  expect(finish()).toBeNull(); expect(begin()).toBe(false);
  for (const scope of scopes.reverse()) leave(scope);
  const report = finish();
  if (!report) throw new Error("unsettled outer observation");
  expect(report.truncated).toBe(true);
  expect(report.phases.adapter.calls).toBe(64);
});

test("test retirement settlement calls only existing bounded ABI and fails visibly on exhaustion", () => {
  let calls = 0;
  expect(settleRetiredWasmSession({ clause_session_v1_reclaim_retired: () => ++calls < 3 }, 3)).toBe(3);
  expect(calls).toBe(3);
  calls = 0;
  expect(() => settleRetiredWasmSession({ clause_session_v1_reclaim_retired: () => { ++calls; return true; } }, 2)).toThrow("retirement exceeded driver bound");
  expect(calls).toBe(2);
  expect(() => settleRetiredWasmSession({ clause_session_v1_reclaim_retired: () => false }, 0)).toThrow();
});
