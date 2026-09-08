import { expect, test } from "bun:test";
import { cp, mkdtemp, rm } from "node:fs/promises";
import { resolve } from "node:path";
import { decodeSessionEvent, "decode-projected-term-frame" as decodeProjection } from "./wasm-cartridge-port.js";

const root = resolve(import.meta.dir, "../../..");
const record = resolve(root, "target/game-replay");
const modulePath = resolve(root, "target/game-replay-wasm/clause_runtime.js");
const run = async (directory: string) => {
  const child = Bun.spawn([process.execPath, resolve(import.meta.dir, "game-replay.ts"), directory, modulePath], { stdout: "pipe", stderr: "pipe" });
  const [code, output, errors] = await Promise.all([child.exited, new Response(child.stdout).text(), new Response(child.stderr).text()]);
  return { code, output, errors };
};

test("actual encounter replay agrees and altered expected vitality reports the first admission and entity", async () => {
  const agreed = await run(record);
  expect(agreed.errors).toBe("");
  expect(agreed.code).toBe(0);
  expect(JSON.parse(agreed.output)).toEqual({ inputs: 14, events: 29, admissions: 7 });

  const directory = await mkdtemp(resolve(root, "target/game-replay-mismatch-"));
  try {
    await cp(record, directory, { recursive: true });
    // Alter only the expected canonical numeric observation, after the real
    // Attack admission. The executable input and both runtimes stay intact.
    const command = 13;
    const path = resolve(directory, `${command}.cse1`);
    const bytes = new Uint8Array(await Bun.file(path).arrayBuffer());
    const event = decodeSessionEvent([...bytes]);
    if (event.kind !== "admission" || !event.projection) throw new Error("expected attack admission");
    const frame = decodeProjection(event.projection.termBytes);
    expect(JSON.stringify(frame)).toContain('"vitality":9');
    const needle = new Uint8Array(8), replacement = new Uint8Array(8);
    new DataView(needle.buffer).setFloat64(0, 9, true);
    new DataView(replacement.buffer).setFloat64(0, 10, true);
    const term = typeof event.projection.termBytes === "string"
      ? Uint8Array.from(event.projection.termBytes, (character) => character.charCodeAt(0))
      : Uint8Array.from(event.projection.termBytes);
    const offset = term.findIndex((_, index) => needle.every((byte, position) => term[index + position] === byte));
    expect(offset).toBeGreaterThanOrEqual(0);
    bytes.set(replacement, bytes.length - term.length + offset);
    await Bun.write(path, bytes);
    const divergent = await run(directory);
    expect(divergent.errors).toBe("");
    expect(divergent.code).toBe(1);
    const report = JSON.parse(divergent.output);
    expect(report.events).toBe(14);
    expect(report.mismatch.inputIndex).toBe(7);
    expect(report.mismatch.command).toBe(command);
    expect(report.mismatch.phase).toBe("admit");
    expect(report.mismatch.difference).toEqual({ path: ["projection", "cinder-1", "vitality"], expected: 10, actual: 9 });
    expect(report.mismatch.entity).toBeDefined();
    expect(report.mismatch.sourceState.relation).toBe("vitality");
    expect(report.mismatch.source).toBe(resolve(directory, "source.clause"));
    console.log(divergent.output);
  } finally {
    await rm(directory, { recursive: true });
  }
}, 60_000);
