import * as branch from "./branch-wasm-port.js";
import * as wire from "./wasm-cartridge-port.js";
import * as test from "bun:test";
import { "clause_branch_v1_command" as clause__branch__v1__command, "clause_branch_v1_event_byte" as clause__branch__v1__event__byte, "clause_branch_v1_event_len" as clause__branch__v1__event__len, "clause_branch_v1_io_reset" as clause__branch__v1__io__reset, "clause_branch_v1_open" as clause__branch__v1__open, "clause_branch_v1_request_push" as clause__branch__v1__request__push, "initSync" as initSync } from "#clause-runtime-wasm";
import { keyword as $$bc$keyword, property_key as $$bc$property_key, str as $$bc$str } from 'beagle/core.js';

function initialize_real_branch_module(module) {
  const input = module;
  const __initialized = initSync(input);
  return Object.freeze({[$$bc$property_key($$bc$keyword("clause_branch_v1_io_reset"))]: () => clause__branch__v1__io__reset(), [$$bc$property_key($$bc$keyword("clause_branch_v1_request_push"))]: (byte) => clause__branch__v1__request__push(byte), [$$bc$property_key($$bc$keyword("clause_branch_v1_open"))]: () => clause__branch__v1__open(), [$$bc$property_key($$bc$keyword("clause_branch_v1_command"))]: () => clause__branch__v1__command(), [$$bc$property_key($$bc$keyword("clause_branch_v1_event_len"))]: () => clause__branch__v1__event__len(), [$$bc$property_key($$bc$keyword("clause_branch_v1_event_byte"))]: (index) => clause__branch__v1__event__byte(index)});
}

function json_string(value) {
  return JSON.stringify(value);
}

const runtime = require("bun:test");
const register_test = runtime.test;
register_test("real Wasm forks reconnects admits and explains one construct-blind process", () => Promise.all([Bun.file("./generated/wasm/clause_runtime_bg.wasm").arrayBuffer(), Bun.file("./fixtures/wasm-process-continuation-v1/process-continuation-v1.cwr1.hex").text()]).then((assets) => { const module = initialize_real_branch_module(assets[0]);
const request = wire["->ExactProcessRequest"](wire["decode-cwr1-hex"](assets[1]));
const occurrences = wire["process-request-occurrences!"](request);
const process_branch = branch["open-process-branch!"](module, request, 41, occurrences[0], 8);
const opened = process_branch.opened;
const parent = opened.pins.parentState;
const branch_run = opened.ancestry.run;
const authoritative = branch["admit-authoritative-occurrences!"](module, process_branch, occurrences);
const r1 = authoritative.successor;
const proposal = branch["propose-branch-reconnect!"](module, process_branch, occurrences);
const evidence = proposal.evidence;
const admitted = branch["adjudicate-branch-reconnect!"](module, process_branch, proposal, r1, occurrences);
const explanation = admitted.explanation;
const queried = branch["explain-process-branch!"](module, process_branch).explanation;
test["expect"]($$bc$str(opened.pins.disconnectTick)).toBe("41");
test["expect"](json_string(authoritative.predecessor)).toBe(json_string(parent));
test["expect"](((!(json_string(r1) === json_string(parent))) ? "true" : "false")).toBe("true");
test["expect"](((!(json_string(authoritative.run) === json_string(branch_run))) ? "true" : "false")).toBe("true");
test["expect"](json_string(evidence.pins.parentState)).toBe(json_string(parent));
test["expect"](json_string(evidence.ancestry.run)).toBe(json_string(branch_run));
test["expect"]($$bc$str(evidence.observations.length)).toBe("2");
test["expect"](json_string(admitted.predecessor)).toBe(json_string(r1));
test["expect"](((!(json_string(admitted.successor) === json_string(r1))) ? "true" : "false")).toBe("true");
test["expect"](json_string(admitted.branchCandidate)).toBe(json_string(evidence.candidate));
test["expect"](json_string(explanation.successor)).toBe(json_string(admitted.successor));
test["expect"](json_string(queried)).toBe(json_string(explanation));
test["expect"](((explanation.causalRecords.length > 0) ? "true" : "false")).toBe("true");
test["expect"]($$bc$str(branch["dispose-process-branch!"](module, process_branch))).toBe("true");
return null; }));
//# sourceMappingURL=branch-wasm-port-test.js.map
