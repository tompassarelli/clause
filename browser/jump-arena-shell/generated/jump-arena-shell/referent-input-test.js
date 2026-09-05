import { expect, test } from "bun:test";
import { file } from "bun";
import * as wasm from "./wasm-cartridge-port.js";
import * as workbench from "./workbench.js";
function completed(action) {
    const values = [];
    action(value => values.push(value));
    expect(values.length).toBe(1);
    return values[0];
}
function object(value) {
    if (typeof value !== "object" || value === null || Array.isArray(value))
        throw new Error("expected a projected object");
    return value;
}
test("fresh Wasm transports exact projected referents through the passive browser input adapter", async () => {
    const generated = new URL("../../../target/referent-wasm/clause_runtime.js", import.meta.url);
    const module = await import(generated.href);
    module.initSync({ module: await file(new URL("../../../target/referent-wasm/clause_runtime_bg.wasm", import.meta.url)).arrayBuffer() });
    const maximum = Number.MAX_SAFE_INTEGER;
    const policy = workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
    const port = wasm["create-wasm-cartridge-port"](module, policy);
    const bytes = [...new Uint8Array(await file(new URL("../../../target/referent-input.cwr1", import.meta.url)).arrayBuffer())];
    const accepted = completed(done => port.acceptPackage(wasm["->ExactProcessRequest"](bytes), done));
    if (accepted._tag !== "PackageAccepted")
        throw new Error(accepted.reason);
    const start = (generation) => {
        const result = completed(done => port.startSession(accepted.acceptedPackage, generation, done));
        if (result._tag !== "SessionStarted")
            throw new Error(result.reason);
        return result;
    };
    const started = start(1);
    let revision = 0;
    let sequence = 0;
    const run = (session, value) => {
        const observations = value === undefined ? [] : [workbench["->InputObservation"](++sequence, workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)])))];
        return completed(done => port.runCandidate(session, workbench["->FixedTick"](100), workbench["->InputConfiguration"](++revision, observations), done));
    };
    const admit = (session, candidate) => {
        if (candidate._tag !== "CandidateProduced")
            throw new Error(candidate.reason);
        const result = completed(done => port.requestAdmission(session, candidate.candidate, done));
        if (result._tag !== "AdmissionAccepted")
            throw new Error(result.reason);
        return object(wasm["decode-projected-term-frame"](result.frame));
    };
    const initial = admit(started.session, run(started.session));
    const first = object(initial.first)["$referent"];
    const second = object(initial.second)["$referent"];
    expect(first).not.toEqual(second);
    expect(object(first).domain).toEqual(object(second).domain);
    const picked = admit(started.session, run(started.session, { kind: "referent-input", generation: 1, channel: "Pick", value: first }));
    expect(object(picked.first).selected).toBe(true);
    expect(object(picked.second).selected).toBe(false);
    expect(object(picked.first).progress).toBe(0.1);
    expect(object(picked.second).progress).toBe(0);
    const both = admit(started.session, run(started.session, { kind: "referent-input", generation: 1, channel: "Pick", value: second }));
    expect(object(both.second).progress).toBe(0.1);
    for (const value of [3, { ...object(first), domain: Number(object(first).domain) + 1 }, { kind: "referent", domain: object(first).domain, identity: { kind: "created", value: [1, 2] } }]) {
        expect(run(started.session, { kind: "referent-input", generation: 1, channel: "Pick", value })._tag).toBe("CandidateFailed");
    }
    const replacement = start(2);
    const stale = run(replacement.session, { kind: "referent-input", generation: 1, channel: "Pick", value: first });
    expect(stale._tag).toBe("CandidateFailed");
    if (stale._tag === "CandidateFailed")
        expect(stale.reason).toContain("stale generation");
    const unchanged = admit(replacement.session, run(replacement.session));
    expect(object(unchanged.first).selected).toBe(false);
    port.disposeSession(replacement.session);
});
test("fresh Wasm stores independent typed targets and aggregates eligible source contributions", async () => {
    const module = await import(new URL("../../../target/referent-wasm/clause_runtime.js", import.meta.url).href);
    module.initSync({ module: await file(new URL("../../../target/referent-wasm/clause_runtime_bg.wasm", import.meta.url)).arrayBuffer() });
    const maximum = Number.MAX_SAFE_INTEGER;
    const policy = workbench["->WorkbenchPolicy"](8, 8, 32, wasm["cse1-projected-term-max-properties"], wasm["cse1-projected-term-json-max-source-units"], workbench["->WorkbenchSequenceLimits"](maximum, maximum, maximum, maximum, maximum));
    const port = wasm["create-wasm-cartridge-port"](module, policy);
    for (const specimen of ["account-contributions", "party-contributions", "cross-subject-target"]) {
        const bytes = [...new Uint8Array(await file(new URL(`../../../target/${specimen}.cwr1`, import.meta.url)).arrayBuffer())];
        const accepted = completed(done => port.acceptPackage(wasm["->ExactProcessRequest"](bytes), done));
        if (accepted._tag !== "PackageAccepted")
            throw new Error(accepted.reason);
        const start = (generation) => {
            const result = completed(done => port.startSession(accepted.acceptedPackage, generation, done));
            if (result._tag !== "SessionStarted")
                throw new Error(result.reason);
            return result.session;
        };
        const session = start(10);
        let revision = 0;
        let sequence = 0;
        const candidate = (session, values = [], milliseconds = 100) => {
            const observations = values.map(value => workbench["->InputObservation"](++sequence, workbench["create-workbench-envelope"](policy, JSON.stringify([JSON.stringify(value)]))));
            return completed(done => port.runCandidate(session, workbench["->FixedTick"](milliseconds), workbench["->InputConfiguration"](++revision, observations), done));
        };
        const admit = (session, candidate) => {
            if (candidate._tag !== "CandidateProduced")
                throw new Error(candidate.reason);
            const result = completed(done => port.requestAdmission(session, candidate.candidate, done));
            if (result._tag !== "AdmissionAccepted")
                throw new Error(result.reason);
            return object(wasm["decode-projected-term-frame"](result.frame));
        };
        const initial = admit(session, candidate(session));
        const inputDomains = object(initial["$referent-inputs"]);
        const target = object(initial[specimen === "party-contributions" ? "cinder" : "second"])["$referent"];
        const input = { kind: "referent-input", generation: 10, channel: specimen === "account-contributions" ? "Choose" : "Target", value: target };
        const chosenCandidate = candidate(session, [input]);
        expect(chosenCandidate._tag).toBe("CandidateProduced");
        // The only decoded state remains the prior admitted frame until requestAdmission.
        expect(object(initial[specimen === "party-contributions" ? "cinder" : "second"])[specimen === "party-contributions" ? "vitality" : specimen === "account-contributions" ? "balance" : "hostile"])
            .toBe(specimen === "party-contributions" ? 100 : specimen === "account-contributions" ? 200 : true);
        const chosen = admit(session, chosenCandidate);
        expect(object(chosen[specimen === "account-contributions" ? "controller" : "player"])[specimen === "account-contributions" ? "chosen-account" : "chosen-target"]).toEqual(target);
        if (specimen !== "cross-subject-target") {
            const key = specimen === "account-contributions" ? "Apply" : "Attack";
            const keydown = { kind: "keyboard", code: key, phase: "down", repeat: false };
            const changed = admit(session, candidate(session, [keydown]));
            const subject = specimen === "account-contributions" ? "second" : "cinder";
            const field = specimen === "account-contributions" ? "balance" : "vitality";
            expect(object(changed[subject])[field]).toBe(specimen === "account-contributions" ? 218 : 80);
            const unchanged = admit(session, candidate(session, [keydown]));
            expect(object(unchanged[subject])[field]).toBe(object(changed[subject])[field]);
            if (specimen === "account-contributions") {
                const facets = object(object(initial.first)["$referents"]);
                expect(object(facets[String(inputDomains.Select)]).domain).toBe(inputDomains.Select);
                expect(object(facets[String(inputDomains.Choose)]).domain).toBe(inputDomains.Choose);
                expect(Object.values(facets)).toHaveLength(2);
                const selected = admit(session, candidate(session, [{ kind: "referent-input", generation: 10, channel: "Select", value: facets[String(inputDomains.Select)] }], 1000));
                expect(object(selected.first).selected).toBe(true);
                const third = admit(session, candidate(session, [keydown]));
                expect(object(third.second).balance).toBe(286);
            }
        }
        const replacement = start(11);
        const rejected = candidate(replacement, [input]);
        expect(rejected._tag).toBe("CandidateFailed");
        if (rejected._tag === "CandidateFailed")
            expect(rejected.reason).toContain("stale generation");
        port.disposeSession(replacement);
        port.disposeSession(session);
        // The production adapter retires in bounded later event-loop turns. This
        // multi-cartridge harness must settle that custody before another open.
        for (let turn = 0; turn < 64 && module.clause_session_v1_reclaim_retired(); turn += 1) {
            await new Promise(resolve => setTimeout(resolve, 0));
        }
        expect(module.clause_session_v1_reclaim_retired()).toBe(false);
    }
});
//# sourceMappingURL=referent-input-test.js.map