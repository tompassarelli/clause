import * as wire from "./wasm-cartridge-port.js";
import { conj_value as $$bc$conj_value, equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key, record_value as $$bc$record_value, str as $$bc$str } from 'beagle/core.js';

const identity_bytes = 32;

const branch_command_max_bytes = (1024 * 1024);

const branch_event_max_bytes = (64 * 1024);

const branch_open_max_bytes = (4 * 1024 * 1024);

function WasmProcessBranch(handle, sequence, disposed, opened) {
  return $$bc$record_value("jump-arena-shell.branch-wasm-port/WasmProcessBranch", {_tag: "WasmProcessBranch", handle, sequence, disposed, opened});
}

function wasmprocessbranch_handle(r) { return r.handle; }

function wasmprocessbranch_sequence(r) { return r.sequence; }

function wasmprocessbranch_disposed(r) { return r.disposed; }

function wasmprocessbranch_opened(r) { return r.opened; }

function ProcessCommandEvidenceV1(occurrence, step, observation) {
  return $$bc$record_value("jump-arena-shell.branch-wasm-port/ProcessCommandEvidenceV1", {_tag: "ProcessCommandEvidenceV1", occurrence, step, observation});
}

function processcommandevidencev1_occurrence(r) { return r.occurrence; }

function processcommandevidencev1_step(r) { return r.step; }

function processcommandevidencev1_observation(r) { return r.observation; }

function identity_at(bytes, offset) {
  return wire["frozen-byte-range"](bytes, offset, (offset + identity_bytes));
}

function append_u16_bang(bytes, value) {
  bytes.push((value % 256));
  return bytes.push((Math.trunc(value / 256) % 256));
}

function append_occurrences_bang(bytes, occurrences) {
  const count = occurrences.length;
  if ((($$bc$equiv(count, 0)) || (count > 256))) {
    (() => { throw new Error("branch occurrence sequence is outside its bound"); })();
  }
  append_u16_bang(bytes, count);
  occurrences.forEach((occurrence) => {
  if ((!wire["exact-byte-array?"](occurrence, branch_command_max_bytes))) {
    (() => { throw new Error("branch occurrence must carry bounded exact bytes"); })();
  }
  wire["append-blob!"](bytes, occurrence);
});
}

function parse_command_evidence(bytes, offset, label) {
  const count = wire["little-u16"](bytes, offset);
  const start = wire["require-range"](bytes, offset, 2, label);
  if ((($$bc$equiv(count, 0)) || (count > 256))) {
    (() => { throw new Error($$bc$str(label, " count is outside its bound")); })();
  }
  return (() => { let index = 0; let cursor = start; let values = []; while (true) {
    if ((index === count)) { return {[$$bc$property_key($$bc$keyword("values"))]: Object.freeze(values), [$$bc$property_key($$bc$keyword("next"))]: cursor}; } else { const record = wire["parse-blob"](bytes, cursor, branch_command_max_bytes, label); (($$bc$equiv(record.bytes.length, 0)) ? (() => { return (() => { throw new Error($$bc$str(label, " occurrence is empty")); })(); })() : null); const identity_start = record.next; const identity_end = wire["require-range"](bytes, identity_start, (2 * identity_bytes), label); const _recur_0 = (index + 1); const _recur_1 = identity_end; const _recur_2 = $$bc$conj_value(values, ProcessCommandEvidenceV1(record.bytes, identity_at(bytes, identity_start), identity_at(bytes, (identity_start + identity_bytes)))); index = _recur_0; cursor = _recur_1; values = _recur_2; continue; }
  } })();
}

function parse_pins(bytes, offset) {
  const end = wire["require-range"](bytes, offset, 308, "branch pins");
  return {[$$bc$property_key($$bc$keyword("value"))]: {[$$bc$property_key($$bc$keyword("parentState"))]: identity_at(bytes, offset), [$$bc$property_key($$bc$keyword("programRevision"))]: identity_at(bytes, (offset + 32)), [$$bc$property_key($$bc$keyword("packageId"))]: identity_at(bytes, (offset + 64)), [$$bc$property_key($$bc$keyword("applicationSnapshot"))]: identity_at(bytes, (offset + 96)), [$$bc$property_key($$bc$keyword("applicationLocal"))]: wire["little-u32"](bytes, (offset + 128)), [$$bc$property_key($$bc$keyword("sessionId"))]: identity_at(bytes, (offset + 132)), [$$bc$property_key($$bc$keyword("runtimePolicy"))]: identity_at(bytes, (offset + 164)), [$$bc$property_key($$bc$keyword("rootPolicy"))]: identity_at(bytes, (offset + 196)), [$$bc$property_key($$bc$keyword("inputEvidence"))]: identity_at(bytes, (offset + 228)), [$$bc$property_key($$bc$keyword("physicalPlan"))]: identity_at(bytes, (offset + 260)), [$$bc$property_key($$bc$keyword("budgetUnits"))]: wire["little-safe-u64"](bytes, (offset + 292)), [$$bc$property_key($$bc$keyword("disconnectTick"))]: wire["little-safe-u64"](bytes, (offset + 300))}, [$$bc$property_key($$bc$keyword("next"))]: end};
}

function parse_ancestry(bytes, offset) {
  const end = wire["require-range"](bytes, offset, (6 * identity_bytes), "branch ancestry");
  return {[$$bc$property_key($$bc$keyword("value"))]: {[$$bc$property_key($$bc$keyword("parentState"))]: identity_at(bytes, offset), [$$bc$property_key($$bc$keyword("run"))]: identity_at(bytes, (offset + 32)), [$$bc$property_key($$bc$keyword("activation"))]: identity_at(bytes, (offset + 64)), [$$bc$property_key($$bc$keyword("disconnectStep"))]: identity_at(bytes, (offset + 96)), [$$bc$property_key($$bc$keyword("suspensionStep"))]: identity_at(bytes, (offset + 128)), [$$bc$property_key($$bc$keyword("continuation"))]: identity_at(bytes, (offset + 160))}, [$$bc$property_key($$bc$keyword("next"))]: end};
}

function parse_suspension(bytes, offset) {
  const end = wire["require-range"](bytes, offset, ((6 * identity_bytes) + 8), "branch suspension");
  return {[$$bc$property_key($$bc$keyword("value"))]: {[$$bc$property_key($$bc$keyword("step"))]: identity_at(bytes, offset), [$$bc$property_key($$bc$keyword("continuation"))]: identity_at(bytes, (offset + 32)), [$$bc$property_key($$bc$keyword("run"))]: identity_at(bytes, (offset + 64)), [$$bc$property_key($$bc$keyword("activation"))]: identity_at(bytes, (offset + 96)), [$$bc$property_key($$bc$keyword("before"))]: identity_at(bytes, (offset + 128)), [$$bc$property_key($$bc$keyword("after"))]: identity_at(bytes, (offset + 160)), [$$bc$property_key($$bc$keyword("remainingBudget"))]: wire["little-safe-u64"](bytes, (offset + 192))}, [$$bc$property_key($$bc$keyword("next"))]: end};
}

function parse_resumption(bytes, offset) {
  const end = wire["require-range"](bytes, offset, ((7 * identity_bytes) + 8), "branch resumption");
  return {[$$bc$property_key($$bc$keyword("value"))]: {[$$bc$property_key($$bc$keyword("occurrence"))]: identity_at(bytes, offset), [$$bc$property_key($$bc$keyword("step"))]: identity_at(bytes, (offset + 32)), [$$bc$property_key($$bc$keyword("continuation"))]: identity_at(bytes, (offset + 64)), [$$bc$property_key($$bc$keyword("run"))]: identity_at(bytes, (offset + 96)), [$$bc$property_key($$bc$keyword("activation"))]: identity_at(bytes, (offset + 128)), [$$bc$property_key($$bc$keyword("before"))]: identity_at(bytes, (offset + 160)), [$$bc$property_key($$bc$keyword("after"))]: identity_at(bytes, (offset + 192)), [$$bc$property_key($$bc$keyword("remainingBudget"))]: wire["little-safe-u64"](bytes, (offset + 224))}, [$$bc$property_key($$bc$keyword("next"))]: end};
}

function decode_reconnect_evidence(bytes) {
  if (wire["exact-byte-array?"](bytes, branch_event_max_bytes)) {
    if (((bytes.length < 4) || (!($$bc$equiv(wire["frozen-byte-range"](bytes, 0, 4), [67, 82, 69, 49]))))) {
      (() => { throw new Error("reconnect evidence magic is invalid"); })();
    }
    const pins = parse_pins(bytes, 4);
    const ancestry = parse_ancestry(bytes, pins.next);
    const resumption = parse_resumption(bytes, ancestry.next);
    const commands = parse_command_evidence(bytes, resumption.next, "reconnect command evidence");
    const candidate_offset = commands.next;
    const final_end = wire["require-range"](bytes, candidate_offset, (2 * identity_bytes), "reconnect candidate");
    const candidate_step = identity_at(bytes, (candidate_offset + identity_bytes));
    const command_values = commands.values;
    const command_count = command_values.length;
    const last_command = command_values[(command_count - 1)];
    if ((!($$bc$equiv(final_end, bytes.length)))) {
      (() => { throw new Error("reconnect evidence has trailing bytes"); })();
    }
    if ((!($$bc$equiv(last_command.step, candidate_step)))) {
      (() => { throw new Error("reconnect candidate Step does not match final command evidence"); })();
    }
    return {[$$bc$property_key($$bc$keyword("pins"))]: pins.value, [$$bc$property_key($$bc$keyword("ancestry"))]: ancestry.value, [$$bc$property_key($$bc$keyword("resumption"))]: resumption.value, [$$bc$property_key($$bc$keyword("commandEvidence"))]: commands.values, [$$bc$property_key($$bc$keyword("candidate"))]: identity_at(bytes, candidate_offset), [$$bc$property_key($$bc$keyword("candidateStep"))]: candidate_step, [$$bc$property_key($$bc$keyword("exactBytes"))]: bytes};
  } else {
    return (() => { throw new Error("reconnect evidence must carry bounded exact bytes"); })();
  }
}

function parse_causal_ref(bytes, offset) {
  const tag_end = wire["require-range"](bytes, offset, 1, "causal reference");
  const tag = wire["byte-at"](bytes, offset);
  if (($$bc$equiv(tag, 5))) {
    const end = wire["require-range"](bytes, tag_end, (3 * identity_bytes), "causal Step reference");
    return {[$$bc$property_key($$bc$keyword("value"))]: {[$$bc$property_key($$bc$keyword("kind"))]: tag, [$$bc$property_key($$bc$keyword("run"))]: identity_at(bytes, tag_end), [$$bc$property_key($$bc$keyword("activation"))]: identity_at(bytes, (tag_end + 32)), [$$bc$property_key($$bc$keyword("step"))]: identity_at(bytes, (tag_end + 64))}, [$$bc$property_key($$bc$keyword("next"))]: end};
  } else {
    if (((tag < 0) || (tag > 9))) {
      (() => { throw new Error("causal reference tag is invalid"); })();
    }
    const end = wire["require-range"](bytes, tag_end, identity_bytes, "causal identity reference");
    return {[$$bc$property_key($$bc$keyword("value"))]: {[$$bc$property_key($$bc$keyword("kind"))]: tag, [$$bc$property_key($$bc$keyword("identity"))]: identity_at(bytes, tag_end)}, [$$bc$property_key($$bc$keyword("next"))]: end};
  }
}

function parse_causal_predecessors(bytes, offset) {
  const count = wire["little-u16"](bytes, offset);
  const start = wire["require-range"](bytes, offset, 2, "causal predecessor count");
  return (() => { let index = 0; let cursor = start; let values = []; while (true) {
    if ((index === count)) { return {[$$bc$property_key($$bc$keyword("values"))]: Object.freeze(values), [$$bc$property_key($$bc$keyword("next"))]: cursor}; } else { const reference = parse_causal_ref(bytes, cursor); const _recur_0 = (index + 1); const _recur_1 = reference.next; const _recur_2 = $$bc$conj_value(values, reference.value); index = _recur_0; cursor = _recur_1; values = _recur_2; continue; }
  } })();
}

function parse_causal_records(bytes, offset) {
  const count = wire["little-u16"](bytes, offset);
  const start = wire["require-range"](bytes, offset, 2, "causal record count");
  if ((count > 2048)) {
    (() => { throw new Error("causal record count exceeds its bound"); })();
  }
  return (() => { let index = 0; let cursor = start; let values = []; while (true) {
    if ((index === count)) { return {[$$bc$property_key($$bc$keyword("values"))]: Object.freeze(values), [$$bc$property_key($$bc$keyword("next"))]: cursor}; } else { const occurrence = parse_causal_ref(bytes, cursor); const predecessors = parse_causal_predecessors(bytes, occurrence.next); const _recur_0 = (index + 1); const _recur_1 = predecessors.next; const _recur_2 = $$bc$conj_value(values, {[$$bc$property_key($$bc$keyword("occurrence"))]: occurrence.value, [$$bc$property_key($$bc$keyword("predecessors"))]: predecessors.values}); index = _recur_0; cursor = _recur_1; values = _recur_2; continue; }
  } })();
}

function decode_branch_explanation(bytes) {
  if (wire["exact-byte-array?"](bytes, branch_event_max_bytes)) {
    if (((bytes.length < 4) || (!($$bc$equiv(wire["frozen-byte-range"](bytes, 0, 4), [67, 66, 88, 49]))))) {
      (() => { throw new Error("branch explanation magic is invalid"); })();
    }
    const pins = parse_pins(bytes, 4);
    const ancestry = parse_ancestry(bytes, pins.next);
    const resumption = parse_resumption(bytes, ancestry.next);
    const branch_commands = parse_command_evidence(bytes, resumption.next, "explanation branch command evidence");
    const branch_commands_end = branch_commands.next;
    const authority_prefix = wire["require-range"](bytes, branch_commands_end, (4 * identity_bytes), "explanation authority prefix");
    const authoritative_commands = parse_command_evidence(bytes, authority_prefix, "explanation authoritative command evidence");
    const authoritative_commands_end = authoritative_commands.next;
    const authority_suffix = wire["require-range"](bytes, authoritative_commands_end, (5 * identity_bytes), "explanation authority suffix");
    const causal = parse_causal_records(bytes, authority_suffix);
    if ((!($$bc$equiv(causal.next, bytes.length)))) {
      (() => { throw new Error("branch explanation has trailing bytes"); })();
    }
    return {[$$bc$property_key($$bc$keyword("pins"))]: pins.value, [$$bc$property_key($$bc$keyword("ancestry"))]: ancestry.value, [$$bc$property_key($$bc$keyword("resumption"))]: resumption.value, [$$bc$property_key($$bc$keyword("branchCommandEvidence"))]: branch_commands.values, [$$bc$property_key($$bc$keyword("branchCandidate"))]: identity_at(bytes, branch_commands_end), [$$bc$property_key($$bc$keyword("authoritativeBase"))]: identity_at(bytes, (branch_commands_end + 32)), [$$bc$property_key($$bc$keyword("authoritativeRun"))]: identity_at(bytes, (branch_commands_end + 64)), [$$bc$property_key($$bc$keyword("authoritativeActivation"))]: identity_at(bytes, (branch_commands_end + 96)), [$$bc$property_key($$bc$keyword("authoritativeCommandEvidence"))]: authoritative_commands.values, [$$bc$property_key($$bc$keyword("authoritativeCandidate"))]: identity_at(bytes, authoritative_commands_end), [$$bc$property_key($$bc$keyword("authorization"))]: identity_at(bytes, (authoritative_commands_end + 32)), [$$bc$property_key($$bc$keyword("judgment"))]: identity_at(bytes, (authoritative_commands_end + 64)), [$$bc$property_key($$bc$keyword("admission"))]: identity_at(bytes, (authoritative_commands_end + 96)), [$$bc$property_key($$bc$keyword("successor"))]: identity_at(bytes, (authoritative_commands_end + 128)), [$$bc$property_key($$bc$keyword("causalRecords"))]: causal.values, [$$bc$property_key($$bc$keyword("exactBytes"))]: bytes};
  } else {
    return (() => { throw new Error("branch explanation must carry bounded exact bytes"); })();
  }
}

function branch_module_functions(module) {
  const reset = module.clause_branch_v1_io_reset;
  const push = module.clause_branch_v1_request_push;
  const open = module.clause_branch_v1_open;
  const command = module.clause_branch_v1_command;
  const event_length = module.clause_branch_v1_event_len;
  const event_byte = module.clause_branch_v1_event_byte;
  if (((reset == null) || ((push == null) || ((open == null) || ((command == null) || ((event_length == null) || (event_byte == null))))))) {
    (() => { throw new Error("Wasm module lacks the Clause branch ABI"); })();
  }
  return {[$$bc$property_key($$bc$keyword("reset"))]: reset, [$$bc$property_key($$bc$keyword("push"))]: push, [$$bc$property_key($$bc$keyword("open"))]: open, [$$bc$property_key($$bc$keyword("command"))]: command, [$$bc$property_key($$bc$keyword("eventLength"))]: event_length, [$$bc$property_key($$bc$keyword("eventByte"))]: event_byte};
}

function dispatch_branch_request(module, request, operation) {
  if (wire["exact-byte-array?"](request, (($$bc$equiv(operation, "open")) ? branch_open_max_bytes : branch_command_max_bytes))) {
    const api = branch_module_functions(module);
    (api.reset)();
    request.forEach((byte) => {
  const status = wire["process-status"]((api.push)(byte));
  if ((!($$bc$equiv(status, 0)))) {
    (() => { throw new Error($$bc$str("branch byte transfer rejected with status ", status)); })();
  }
});
    const status = wire["process-status"]((($$bc$equiv(operation, "open")) ? (api.open)() : (api.command)()));
    if ((!($$bc$equiv(status, 0)))) {
      (() => { throw new Error($$bc$str("Clause branch ", operation, " rejected with status ", status)); })();
    }
    const length = wire["process-status"]((api.eventLength)());
    if (((length < 21) || (length > branch_event_max_bytes))) {
      (() => { throw new Error("CBE1 event length is out of bounds"); })();
    }
    return (() => { let index = 0; let values = []; while (true) {
    if ((index === length)) { return Object.freeze(values); } else { const byte = wire["process-status"]((api.eventByte)(index)); if (((byte < 0) || (byte > 255))) { return (() => { throw new Error("CBE1 event byte is out of bounds"); })(); } else { const _recur_0 = (index + 1); const _recur_1 = $$bc$conj_value(values, byte); index = _recur_0; values = _recur_1; continue; } }
  } })();
  } else {
    return (() => { throw new Error("branch request must carry bounded exact bytes"); })();
  }
}

function parse_projection(bytes, offset) {
  const tag_end = wire["require-range"](bytes, offset, 1, "branch projection");
  const tag = wire["byte-at"](bytes, offset);
  return ((($$bc$equiv(tag, 0))) ? {[$$bc$property_key($$bc$keyword("value"))]: null, [$$bc$property_key($$bc$keyword("next"))]: tag_end} : (($$bc$equiv(tag, 1))) ? (() => { const observation_end = wire["require-range"](bytes, tag_end, identity_bytes, "branch projection observation"); const term = wire["parse-blob"](bytes, observation_end, branch_event_max_bytes, "branch projected Term"); return {[$$bc$property_key($$bc$keyword("value"))]: {[$$bc$property_key($$bc$keyword("observation"))]: identity_at(bytes, tag_end), [$$bc$property_key($$bc$keyword("termBytes"))]: term.bytes}, [$$bc$property_key($$bc$keyword("next"))]: term.next}; })() : (() => { throw new Error("branch projection tag is invalid"); })());
}

function decode_cbe1_event(bytes) {
  if (wire["exact-byte-array?"](bytes, branch_event_max_bytes)) {
    if (((bytes.length < 21) || (!($$bc$equiv(wire["frozen-byte-range"](bytes, 0, 4), [67, 66, 69, 49]))))) {
      (() => { throw new Error("CBE1 event magic is invalid"); })();
    }
    const slot = wire["little-u32"](bytes, 4);
    const generation = wire["little-u32"](bytes, 8);
    const sequence = wire["little-safe-u64"](bytes, 12);
    const tag = wire["byte-at"](bytes, 20);
    const common = {[$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence};
    return ((($$bc$equiv(tag, 1))) ? (() => { const pins = parse_pins(bytes, 21); const ancestry = parse_ancestry(bytes, pins.next); const suspension = parse_suspension(bytes, ancestry.next); if ((!($$bc$equiv(suspension.next, bytes.length)))) {
  (() => { throw new Error("CBE1 Opened event has trailing bytes"); })();
}
return Object.assign(common, {[$$bc$property_key($$bc$keyword("kind"))]: "opened", [$$bc$property_key($$bc$keyword("pins"))]: pins.value, [$$bc$property_key($$bc$keyword("ancestry"))]: ancestry.value, [$$bc$property_key($$bc$keyword("suspension"))]: suspension.value}); })() : (($$bc$equiv(tag, 2))) ? (() => { if ((!($$bc$equiv(bytes.length, (21 + (7 * identity_bytes)))))) {
  (() => { throw new Error("CBE1 authoritative Admission has an invalid shape"); })();
}
return Object.assign(common, {[$$bc$property_key($$bc$keyword("kind"))]: "authoritative-admission", [$$bc$property_key($$bc$keyword("candidate"))]: identity_at(bytes, 21), [$$bc$property_key($$bc$keyword("predecessor"))]: identity_at(bytes, 53), [$$bc$property_key($$bc$keyword("successor"))]: identity_at(bytes, 85), [$$bc$property_key($$bc$keyword("judgment"))]: identity_at(bytes, 117), [$$bc$property_key($$bc$keyword("admission"))]: identity_at(bytes, 149), [$$bc$property_key($$bc$keyword("run"))]: identity_at(bytes, 181), [$$bc$property_key($$bc$keyword("activation"))]: identity_at(bytes, 213)}); })() : (($$bc$equiv(tag, 3))) ? (() => { const record = wire["parse-blob"](bytes, 21, branch_event_max_bytes, "CBE1 reconnect evidence"); if ((!($$bc$equiv(record.next, bytes.length)))) {
  (() => { throw new Error("CBE1 reconnect evidence has trailing bytes"); })();
}
return Object.assign(common, {[$$bc$property_key($$bc$keyword("kind"))]: "reconnect-proposed", [$$bc$property_key($$bc$keyword("evidence"))]: decode_reconnect_evidence(record.bytes)}); })() : (($$bc$equiv(tag, 4))) ? (() => { const prefix_end = wire["require-range"](bytes, 21, (6 * identity_bytes), "CBE1 reconnect Admission"); const projection = parse_projection(bytes, prefix_end); const record = wire["parse-blob"](bytes, projection.next, branch_event_max_bytes, "CBE1 branch explanation"); if ((!($$bc$equiv(record.next, bytes.length)))) {
  (() => { throw new Error("CBE1 reconnect Admission has trailing bytes"); })();
}
return Object.assign(common, {[$$bc$property_key($$bc$keyword("kind"))]: "reconnect-admission", [$$bc$property_key($$bc$keyword("predecessor"))]: identity_at(bytes, 21), [$$bc$property_key($$bc$keyword("successor"))]: identity_at(bytes, 53), [$$bc$property_key($$bc$keyword("branchCandidate"))]: identity_at(bytes, 85), [$$bc$property_key($$bc$keyword("authoritativeCandidate"))]: identity_at(bytes, 117), [$$bc$property_key($$bc$keyword("judgment"))]: identity_at(bytes, 149), [$$bc$property_key($$bc$keyword("admission"))]: identity_at(bytes, 181), [$$bc$property_key($$bc$keyword("projection"))]: projection.value, [$$bc$property_key($$bc$keyword("explanation"))]: decode_branch_explanation(record.bytes)}); })() : (($$bc$equiv(tag, 5))) ? (() => { const record = wire["parse-blob"](bytes, 21, branch_event_max_bytes, "CBE1 retained explanation"); if ((!($$bc$equiv(record.next, bytes.length)))) {
  (() => { throw new Error("CBE1 explanation has trailing bytes"); })();
}
return Object.assign(common, {[$$bc$property_key($$bc$keyword("kind"))]: "explanation", [$$bc$property_key($$bc$keyword("explanation"))]: decode_branch_explanation(record.bytes)}); })() : (($$bc$equiv(tag, 6))) ? (() => { if ((!($$bc$equiv(bytes.length, 21)))) {
  (() => { throw new Error("CBE1 Disposed event has trailing bytes"); })();
}
return Object.assign(common, {[$$bc$property_key($$bc$keyword("kind"))]: "disposed"}); })() : (($$bc$equiv(tag, 7))) ? (() => { const reason = wire["byte-at"](bytes, 21); const expected = (($$bc$equiv(reason, 7)) ? 23 : 22); if ((!($$bc$equiv(bytes.length, expected)))) {
  (() => { throw new Error("CBE1 rejection has an invalid shape"); })();
}
return Object.assign(common, {[$$bc$property_key($$bc$keyword("kind"))]: "rejected", [$$bc$property_key($$bc$keyword("reason"))]: reason, [$$bc$property_key($$bc$keyword("pin"))]: (($$bc$equiv(reason, 7)) ? wire["byte-at"](bytes, 22) : null)}); })() : (() => { throw new Error("CBE1 event tag is invalid"); })());
  } else {
    return (() => { throw new Error("CBE1 event must carry bounded exact bytes"); })();
  }
}

function encode_open(request, disconnect_tick, disconnect_occurrence, max_commands) {
  const bytes = [67, 66, 82, 49];
  wire["append-blob!"](bytes, request.bytes);
  wire["append-u64!"](bytes, disconnect_tick);
  wire["append-blob!"](bytes, disconnect_occurrence);
  wire["append-u64!"](bytes, max_commands);
  return Object.freeze(bytes);
}

function encode_command(branch, tag, payload) {
  const bytes = [67, 66, 73, 49];
  wire["append-u32!"](bytes, branch.handle.slot);
  wire["append-u32!"](bytes, branch.handle.generation);
  wire["append-u64!"](bytes, branch.sequence.value);
  bytes.push(tag);
  if ((!(payload == null))) {
    payload.forEach((byte) => {
  bytes.push(byte);
});
  }
  return Object.freeze(bytes);
}

function require_live_branch(value) {
  return (((value == null) || ((value.handle == null) || (value.sequence == null))) ? (() => { throw new Error("Wasm process branch is invalid"); })() : (((_truthy) => _truthy !== false && _truthy != null)(value.disposed.value) ? (() => { throw new Error("Wasm process branch is disposed"); })() : value));
}

function apply_command_bang(module, branch, command) {
  const event = decode_cbe1_event(dispatch_branch_request(module, command, "command"));
  const sequence = branch.sequence.value;
  if (((!($$bc$equiv(event.slot, branch.handle.slot))) || ((!($$bc$equiv(event.generation, branch.handle.generation))) || (!($$bc$equiv(event.sequence, (sequence + 1))))))) {
    (() => { throw new Error("CBE1 event does not advance exact branch custody"); })();
  }
  (() => { const _a = branch.sequence, _v = event.sequence; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  if (($$bc$equiv(event.kind, "rejected"))) {
    (() => { throw new Error($$bc$str("Clause branch operation rejected with reason ", event.reason)); })();
  }
  return event;
}

function open_process_branch_bang(module, request, disconnect_tick, disconnect_occurrence, max_commands) {
  if (((disconnect_tick < 0) || (max_commands <= 0))) {
    (() => { throw new Error("branch tick and command budget are invalid"); })();
  }
  const event = decode_cbe1_event(dispatch_branch_request(module, encode_open(request, disconnect_tick, disconnect_occurrence, max_commands), "open"));
  if (((!($$bc$equiv(event.kind, "opened"))) || (!($$bc$equiv(event.sequence, 0))))) {
    (() => { throw new Error("Clause branch did not open exactly once"); })();
  }
  return WasmProcessBranch(Object.freeze({[$$bc$property_key($$bc$keyword("slot"))]: event.slot, [$$bc$property_key($$bc$keyword("generation"))]: event.generation}), ({value: 0, watches: {}}), ({value: false, watches: {}}), event);
}

function occurrence_command_bang(branch, tag, occurrences) {
  const payload = [];
  append_occurrences_bang(payload, occurrences);
  return encode_command(branch, tag, payload);
}

function admit_authoritative_occurrences_bang(module, incoming_branch, occurrences) {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(module, branch, occurrence_command_bang(branch, 1, occurrences));
  if ((!($$bc$equiv(event.kind, "authoritative-admission")))) {
    (() => { throw new Error("branch authority advance produced no Admission"); })();
  }
  return event;
}

function propose_branch_reconnect_bang(module, incoming_branch, occurrences) {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(module, branch, occurrence_command_bang(branch, 2, occurrences));
  if ((!($$bc$equiv(event.kind, "reconnect-proposed")))) {
    (() => { throw new Error("branch continuation produced no reconnect evidence"); })();
  }
  return event;
}

function adjudicate_branch_reconnect_bang(module, incoming_branch, proposal, authoritative_base, occurrences) {
  const branch = require_live_branch(incoming_branch);
  const evidence = proposal.evidence;
  const payload = [];
  if (((evidence == null) || (!wire["exact-byte-array?"](evidence.exactBytes, branch_event_max_bytes)))) {
    (() => { throw new Error("reconnect proposal lacks exact retained evidence"); })();
  }
  wire["append-blob!"](payload, evidence.exactBytes);
  [evidence.candidate, authoritative_base].forEach((identity) => {
  if ((!wire["exact-byte-array?"](identity, identity_bytes))) {
    (() => { throw new Error("reconnect adjudication identity is invalid"); })();
  }
  identity.forEach((byte) => {
  payload.push(byte);
});
});
  append_occurrences_bang(payload, occurrences);
  const event = apply_command_bang(module, branch, encode_command(branch, 3, payload));
  if ((!($$bc$equiv(event.kind, "reconnect-admission")))) {
    (() => { throw new Error("reconnect adjudication produced no Admission"); })();
  }
  return event;
}

function explain_process_branch_bang(module, incoming_branch) {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(module, branch, encode_command(branch, 4, null));
  if ((!($$bc$equiv(event.kind, "explanation")))) {
    (() => { throw new Error("retained branch produced no causal explanation"); })();
  }
  return event;
}

function dispose_process_branch_bang(module, incoming_branch) {
  const branch = require_live_branch(incoming_branch);
  const event = apply_command_bang(module, branch, encode_command(branch, 5, null));
  if ((!($$bc$equiv(event.kind, "disposed")))) {
    (() => { throw new Error("Clause branch did not dispose"); })();
  }
  (() => { const _a = branch.disposed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return true;
}

export { ProcessCommandEvidenceV1 as "->ProcessCommandEvidenceV1" };
export { WasmProcessBranch as "->WasmProcessBranch" };
export { ProcessCommandEvidenceV1 as "ProcessCommandEvidenceV1" };
export { WasmProcessBranch as "WasmProcessBranch" };
export { adjudicate_branch_reconnect_bang as "adjudicate-branch-reconnect!" };
export { admit_authoritative_occurrences_bang as "admit-authoritative-occurrences!" };
export { decode_branch_explanation as "decode-branch-explanation" };
export { decode_reconnect_evidence as "decode-reconnect-evidence" };
export { dispose_process_branch_bang as "dispose-process-branch!" };
export { explain_process_branch_bang as "explain-process-branch!" };
export { open_process_branch_bang as "open-process-branch!" };
export { processcommandevidencev1_observation as "processcommandevidencev1-observation" };
export { processcommandevidencev1_occurrence as "processcommandevidencev1-occurrence" };
export { processcommandevidencev1_step as "processcommandevidencev1-step" };
export { propose_branch_reconnect_bang as "propose-branch-reconnect!" };
export { wasmprocessbranch_disposed as "wasmprocessbranch-disposed" };
export { wasmprocessbranch_handle as "wasmprocessbranch-handle" };
export { wasmprocessbranch_opened as "wasmprocessbranch-opened" };
export { wasmprocessbranch_sequence as "wasmprocessbranch-sequence" };
//# sourceMappingURL=branch-wasm-port.js.map
