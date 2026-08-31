import * as workbench from "./workbench.js";
import { conj_value as $$bc$conj_value, count as $$bc$count, equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key, record_value as $$bc$record_value, str as $$bc$str } from 'beagle/core.js';
import { catch_dispatch as $$bd$catch_dispatch } from 'beagle/exception-dispatch.js';

const cwr1_max_bytes = (4 * 1024 * 1024);

const cwr1_hex_max_source_units = (3 * cwr1_max_bytes);

const cwo1_max_bytes = (64 * 1024);

const cwo1_prefix_bytes = (4 + 32 + 32);

const cwo1_identity_bytes = 32;

const cwo1_max_values = 256;

const cse1_max_bytes = (64 * 1024);

const cse1_projected_term_max_properties = cse1_max_bytes;

const cse1_projected_term_json_max_source_units = ((4 * cse1_max_bytes) + 1);

const session_command_max_bytes = (1024 * 1024);

const session_command_limit = 4096;

const identity_bytes = 32;

const allocation_epoch_bytes = 304;

function hex_whitespace_code_p(code) {
  return (($$bc$equiv(code, 9)) || (($$bc$equiv(code, 10)) || (($$bc$equiv(code, 13)) || ($$bc$equiv(code, 32)))));
}

function lowercase_hex_nibble(code) {
  return ((((48 <= code) && (code <= 57))) ? (code - 48) : (((97 <= code) && (code <= 102))) ? ((code - 97) + 10) : -1);
}

function decode_cwr1_hex(source) {
  if (($$bc$equiv(typeof source, "string"))) {
    const length = source.length;
    if ((($$bc$equiv(length, 0)) || (length > cwr1_hex_max_source_units))) {
      (() => { throw new Error("CWR1 hex transport is outside its source bound"); })();
    }
    return (() => { let index = 0; let high = -1; let bytes = []; while (true) {
    if ((index === length)) { return (((!($$bc$equiv(high, -1)))) ? (() => { throw new Error("CWR1 hex transport has an incomplete byte"); })() : (($$bc$equiv($$bc$count(bytes), 0))) ? (() => { throw new Error("CWR1 hex transport is empty"); })() : Object.freeze(bytes)); } else { const code = source.charCodeAt(index); const nibble = lowercase_hex_nibble(code); if (hex_whitespace_code_p(code)) { const _recur_0 = (index + 1); const _recur_1 = high; const _recur_2 = bytes; index = _recur_0; high = _recur_1; bytes = _recur_2; continue; } else if ((nibble < 0)) { return (() => { throw new Error("CWR1 hex transport contains a non-hex unit"); })(); } else if (($$bc$equiv(high, -1))) { const _recur_0 = (index + 1); const _recur_1 = nibble; const _recur_2 = bytes; index = _recur_0; high = _recur_1; bytes = _recur_2; continue; } else if (($$bc$count(bytes) >= cwr1_max_bytes)) { return (() => { throw new Error("CWR1 hex transport exceeds its byte bound"); })(); } else { const _recur_0 = (index + 1); const _recur_1 = -1; const _recur_2 = $$bc$conj_value(bytes, ((high * 16) + nibble)); index = _recur_0; high = _recur_1; bytes = _recur_2; continue; } }
  } })();
  } else {
    return (() => { throw new Error("CWR1 hex transport must be text"); })();
  }
}

function ExactProcessRequest(bytes) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/ExactProcessRequest", {_tag: "ExactProcessRequest", bytes});
}

function exactprocessrequest_bytes(r) { return r.bytes; }

function ExactProcessObservation(bytes) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/ExactProcessObservation", {_tag: "ExactProcessObservation", bytes});
}

function exactprocessobservation_bytes(r) { return r.bytes; }

function WasmCandidate(candidateId, base) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/WasmCandidate", {_tag: "WasmCandidate", candidateId, base});
}

function wasmcandidate_candidateId(r) { return r.candidateId; }

function wasmcandidate_base(r) { return r.base; }

function WasmSession(handle, packageId, sessionId, allocation, world, sequence, occurrences, disposed) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/WasmSession", {_tag: "WasmSession", handle, packageId, sessionId, allocation, world, sequence, occurrences, disposed});
}

function wasmsession_handle(r) { return r.handle; }

function wasmsession_packageId(r) { return r.packageId; }

function wasmsession_sessionId(r) { return r.sessionId; }

function wasmsession_allocation(r) { return r.allocation; }

function wasmsession_world(r) { return r.world; }

function wasmsession_sequence(r) { return r.sequence; }

function wasmsession_occurrences(r) { return r.occurrences; }

function wasmsession_disposed(r) { return r.disposed; }

function PersistentCartridge(openBytes, occurrences) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/PersistentCartridge", {_tag: "PersistentCartridge", openBytes, occurrences});
}

function persistentcartridge_openBytes(r) { return r.openBytes; }

function persistentcartridge_occurrences(r) { return r.occurrences; }

function Cwo1Observation(observationId, stateRevisionId, values) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/Cwo1Observation", {_tag: "Cwo1Observation", observationId, stateRevisionId, values});
}

function cwo1observation_observationId(r) { return r.observationId; }

function cwo1observation_stateRevisionId(r) { return r.stateRevisionId; }

function cwo1observation_values(r) { return r.values; }

function exact_byte_array_p(bytes, maximum) {
  return ((_logical) => (_logical !== false && _logical != null ? ((1 <= bytes.length) && ((bytes.length <= maximum) && (() => { let index = 0; while (true) {
    if (($$bc$equiv(index, bytes.length))) { return true; } else { const byte = bytes[index]; if (((_truthy) => _truthy !== false && _truthy != null)(((_logical) => (_logical !== false && _logical != null ? ((0 <= byte) && (byte <= 255)) : _logical))(Number.isInteger(byte)))) { const _recur_0 = (index + 1); index = _recur_0; continue; } else { return false; } }
  } })())) : _logical))(Array.isArray(bytes));
}

function require_request(request) {
  return (((!(request == null)) && exact_byte_array_p(request.bytes, cwr1_max_bytes)) ? ExactProcessRequest(frozen_byte_range(request.bytes, 0, request.bytes.length)) : (() => { throw new Error("cartridge request must carry bounded exact bytes"); })());
}

function process_status(status) {
  return (((_truthy) => _truthy !== false && _truthy != null)(Number.isSafeInteger(status)) ? status : -1);
}

function byte_at(bytes, index) {
  return bytes[index];
}

function little_u16(bytes, offset) {
  return (byte_at(bytes, offset) + (256 * byte_at(bytes, (offset + 1))));
}

function little_u32(bytes, offset) {
  return (byte_at(bytes, offset) + (256 * byte_at(bytes, (offset + 1))) + (65536 * byte_at(bytes, (offset + 2))) + (16777216 * byte_at(bytes, (offset + 3))));
}

function big_u32(bytes, offset) {
  return ((16777216 * byte_at(bytes, offset)) + (65536 * byte_at(bytes, (offset + 1))) + (256 * byte_at(bytes, (offset + 2))) + byte_at(bytes, (offset + 3)));
}

function little_safe_u64(bytes, offset) {
  const low = little_u32(bytes, offset);
  const high = little_u32(bytes, (offset + 4));
  return ((high > 2097151) ? (() => { throw new Error("64-bit transport value exceeds exact JavaScript range"); })() : (low + (high * 4294967296)));
}

function append_u32_bang(bytes, value) {
  bytes.push((value % 256));
  bytes.push((Math.trunc(value / 256) % 256));
  bytes.push((Math.trunc(value / 65536) % 256));
  return bytes.push((Math.trunc(value / 16777216) % 256));
}

function append_u64_bang(bytes, value) {
  append_u32_bang(bytes, (value % 4294967296));
  return append_u32_bang(bytes, Math.trunc(value / 4294967296));
}

function append_blob_bang(bytes, value) {
  append_u32_bang(bytes, value.length);
  value.forEach((byte) => {
  bytes.push(byte);
});
}

function require_range(bytes, offset, length, label) {
  const end = (offset + length);
  return (((offset < 0) || ((length < 0) || (end > bytes.length))) ? (() => { throw new Error($$bc$str(label, " is truncated")); })() : end);
}

function frozen_byte_range(bytes, start, end) {
  return (() => { let index = start; let result = []; while (true) {
    if ((index === end)) { return Object.freeze(result); } else { const _recur_0 = (index + 1); const _recur_1 = $$bc$conj_value(result, byte_at(bytes, index)); index = _recur_0; result = _recur_1; continue; }
  } })();
}

function finite_f64(bytes, offset) {
  const packed = new Uint8Array(frozen_byte_range(bytes, offset, (offset + 8)));
  const view = new DataView(packed.buffer);
  const value = view.getFloat64(0, true);
  return (((_truthy) => _truthy !== false && _truthy != null)(((_logical) => (_logical !== false && _logical != null ? (!(($$bc$equiv(value, 0.0)) && ($$bc$equiv(byte_at(bytes, (offset + 7)), 128)))) : _logical))(Number.isFinite(value))) ? value : (() => { throw new Error("CWO1 number is not canonical finite f64"); })());
}

function decode_cwo1_observation(incoming) {
  if (exact_byte_array_p(incoming, cwo1_max_bytes)) {
    const length = incoming.length;
    if ((length < (cwo1_prefix_bytes + 2))) {
      (() => { throw new Error("CWO1 response is truncated"); })();
    }
    if (((!($$bc$equiv(byte_at(incoming, 0), 67))) || ((!($$bc$equiv(byte_at(incoming, 1), 87))) || ((!($$bc$equiv(byte_at(incoming, 2), 79))) || (!($$bc$equiv(byte_at(incoming, 3), 49))))))) {
      (() => { throw new Error("CWO1 response magic is invalid"); })();
    }
    const observation_id = frozen_byte_range(incoming, 4, (4 + cwo1_identity_bytes));
    const state_revision_id = frozen_byte_range(incoming, (4 + cwo1_identity_bytes), cwo1_prefix_bytes);
    const count = little_u16(incoming, cwo1_prefix_bytes);
    if ((count > cwo1_max_values)) {
      (() => { throw new Error("CWO1 render value count is out of bounds"); })();
    }
    return (() => { let index = 0; let offset = (cwo1_prefix_bytes + 2); let values = []; while (true) {
    if ((index === count)) { return ((offset === length) ? Cwo1Observation(observation_id, state_revision_id, Object.freeze(values)) : (() => { throw new Error("CWO1 response has trailing bytes"); })()); } else { ((offset >= length) ? (() => { return (() => { throw new Error("CWO1 value is truncated"); })(); })() : null); const tag = byte_at(incoming, offset); if (($$bc$equiv(tag, 0))) { if (((offset + 9) > length)) { return (() => { throw new Error("CWO1 number is truncated"); })(); } else { const _recur_0 = (index + 1); const _recur_1 = (offset + 9); const _recur_2 = $$bc$conj_value(values, finite_f64(incoming, (offset + 1))); index = _recur_0; offset = _recur_1; values = _recur_2; continue; } } else if (($$bc$equiv(tag, 1))) { if (((offset + 2) > length)) { return (() => { throw new Error("CWO1 boolean is truncated"); })(); } else { const value = byte_at(incoming, (offset + 1)); if ((value > 1)) { return (() => { throw new Error("CWO1 boolean is invalid"); })(); } else { const _recur_0 = (index + 1); const _recur_1 = (offset + 2); const _recur_2 = $$bc$conj_value(values, ($$bc$equiv(value, 1))); index = _recur_0; offset = _recur_1; values = _recur_2; continue; } } } else { return (() => { throw new Error("CWO1 value tag is invalid"); })(); } }
  } })();
  } else {
    return (() => { throw new Error("CWO1 response must carry bounded exact bytes"); })();
  }
}

function project_frame(policy, observation) {
  return workbench["create-workbench-envelope"](policy, JSON.stringify(observation.values));
}

function dispatch_exact_request(module, request) {
  const checked = require_request(request);
  const reset = module.clause_process_v1_reset;
  const push = module.clause_process_v1_request_push;
  const dispatch = module.clause_process_v1_dispatch;
  const response_length = module.clause_process_v1_response_len;
  const response_byte = module.clause_process_v1_response_byte;
  reset();
  checked.bytes.forEach((byte) => {
  const status = process_status(push(byte));
  if ((!($$bc$equiv(status, 0)))) {
    (() => { throw new Error($$bc$str("CWR1 byte transfer rejected with status ", status)); })();
  }
});
  const status = process_status(dispatch());
  if ((!($$bc$equiv(status, 0)))) {
    (() => { throw new Error($$bc$str("CWR1 dispatch rejected with status ", status)); })();
  }
  const length = process_status(response_length());
  if (((length < (cwo1_prefix_bytes + 2)) || (length > cwo1_max_bytes))) {
    (() => { throw new Error("CWO1 response length is out of bounds"); })();
  }
  return (() => { let index = 0; let bytes = []; while (true) {
    if ((index === length)) { return ExactProcessObservation(Object.freeze(bytes)); } else { const byte = process_status(response_byte(index)); if (((byte < 0) || (byte > 255))) { return (() => { throw new Error("CWO1 response byte is out of bounds"); })(); } else { const _recur_0 = (index + 1); const _recur_1 = $$bc$conj_value(bytes, byte); index = _recur_0; bytes = _recur_1; continue; } }
  } })();
}

function parse_blob(bytes, offset, maximum, label) {
  const header_end = require_range(bytes, offset, 4, label);
  const length = little_u32(bytes, offset);
  const __bound = ((length > maximum) ? (() => { return (() => { throw new Error($$bc$str(label, " exceeds its bound")); })(); })() : null);
  const end = require_range(bytes, header_end, length, label);
  return {[$$bc$property_key($$bc$keyword("bytes"))]: frozen_byte_range(bytes, header_end, end), [$$bc$property_key($$bc$keyword("next"))]: end};
}

function require_allocation_epoch(record) {
  const bytes = record.bytes;
  return ((($$bc$equiv(bytes.length, allocation_epoch_bytes)) && ($$bc$equiv(frozen_byte_range(bytes, 0, 4), [82, 65, 69, 49]))) ? bytes : (() => { throw new Error("runtime allocation epoch has an invalid shape"); })());
}

function parse_persistent_cartridge_bang(request) {
  const checked = require_request(request);
  const bytes = checked.bytes;
  if (((bytes.length < 4) || ((!($$bc$equiv(byte_at(bytes, 0), 67))) || ((!($$bc$equiv(byte_at(bytes, 1), 87))) || ((!($$bc$equiv(byte_at(bytes, 2), 82))) || (!($$bc$equiv(byte_at(bytes, 3), 49)))))))) {
    (() => { throw new Error("persistent cartridge must carry exact CWR1 bytes"); })();
  }
  const package_record = parse_blob(bytes, 4, cwr1_max_bytes, "CWR1 package");
  const package_end = package_record.next;
  const application_end = require_range(bytes, package_end, 4, "CWR1 application");
  const physical_plan = parse_blob(bytes, application_end, cwr1_max_bytes, "CWR1 physical plan");
  const allocation = parse_blob(bytes, physical_plan.next, allocation_epoch_bytes, "CWR1 allocation epoch");
  const allocation_bytes = require_allocation_epoch(allocation);
  const authority_start = allocation.next;
  const identities_end = require_range(bytes, authority_start, (9 * identity_bytes), "CWR1 authority identities");
  const occurrence_evidence = parse_blob(bytes, identities_end, cwr1_max_bytes, "CWR1 occurrence evidence");
  const judgment_id_end = require_range(bytes, occurrence_evidence.next, identity_bytes, "CWR1 Judgment evidence identity");
  const judgment_evidence = parse_blob(bytes, judgment_id_end, cwr1_max_bytes, "CWR1 Judgment evidence");
  const admission_id_end = require_range(bytes, judgment_evidence.next, identity_bytes, "CWR1 Admission evidence identity");
  const admission_evidence = parse_blob(bytes, admission_id_end, cwr1_max_bytes, "CWR1 Admission evidence");
  const budget_end = require_range(bytes, admission_evidence.next, 8, "CWR1 budget");
  const count_end = require_range(bytes, budget_end, 2, "CWR1 occurrence count");
  const occurrence_count = little_u16(bytes, budget_end);
  const occurrences_result = (() => { let index = 0; let offset = count_end; let occurrences = []; while (true) {
    if ((index === occurrence_count)) { return {[$$bc$property_key($$bc$keyword("next"))]: offset, [$$bc$property_key($$bc$keyword("values"))]: occurrences}; } else { const occurrence = parse_blob(bytes, offset, cwr1_max_bytes, "CWR1 occurrence"); const _recur_0 = (index + 1); const _recur_1 = occurrence.next; const _recur_2 = $$bc$conj_value(occurrences, occurrence.bytes); index = _recur_0; offset = _recur_1; occurrences = _recur_2; continue; }
  } })();
  const slot_count_end = require_range(bytes, occurrences_result.next, 2, "CWR1 projection count");
  const slot_count = little_u16(bytes, occurrences_result.next);
  const final_offset = require_range(bytes, slot_count_end, (slot_count * 2), "CWR1 legacy projection");
  if ((($$bc$equiv(occurrence_count, 0)) || (!($$bc$equiv(final_offset, bytes.length))))) {
    (() => { throw new Error("CWR1 cartridge shape is incomplete"); })();
  }
  const open_bytes = [67, 87, 83, 49];
  bytes.slice(4, physical_plan.next).forEach((byte) => {
  open_bytes.push(byte);
});
  bytes.slice(authority_start, budget_end).forEach((byte) => {
  open_bytes.push(byte);
});
  open_bytes.push(0);
  append_u64_bang(open_bytes, session_command_limit);
  append_u32_bang(open_bytes, session_command_max_bytes);
  append_u32_bang(open_bytes, cse1_max_bytes);
  return PersistentCartridge(Object.freeze(open_bytes), Object.freeze(occurrences_result.values));
}

function session_module_functions(module) {
  const reset = module.clause_session_v1_io_reset;
  const push = module.clause_session_v1_request_push;
  const open = module.clause_session_v1_open;
  const command = module.clause_session_v1_command;
  const event_length = module.clause_session_v1_event_len;
  const event_byte = module.clause_session_v1_event_byte;
  if (((reset == null) || ((push == null) || ((open == null) || ((command == null) || ((event_length == null) || (event_byte == null))))))) {
    (() => { throw new Error("Wasm module lacks the persistent Clause session ABI"); })();
  }
  return {[$$bc$property_key($$bc$keyword("reset"))]: reset, [$$bc$property_key($$bc$keyword("push"))]: push, [$$bc$property_key($$bc$keyword("open"))]: open, [$$bc$property_key($$bc$keyword("command"))]: command, [$$bc$property_key($$bc$keyword("eventLength"))]: event_length, [$$bc$property_key($$bc$keyword("eventByte"))]: event_byte};
}

function dispatch_session_request(module, request, operation) {
  if (exact_byte_array_p(request, session_command_max_bytes)) {
    const api = session_module_functions(module);
    (api.reset)();
    request.forEach((byte) => {
  const status = process_status((api.push)(byte));
  if ((!($$bc$equiv(status, 0)))) {
    (() => { throw new Error($$bc$str("persistent byte transfer rejected with status ", status)); })();
  }
});
    const status = process_status((($$bc$equiv(operation, "open")) ? (api.open)() : (api.command)()));
    if ((!($$bc$equiv(status, 0)))) {
      (() => { throw new Error($$bc$str("persistent session ", operation, " rejected with status ", status)); })();
    }
    const length = process_status((api.eventLength)());
    if (((length < 21) || (length > cse1_max_bytes))) {
      (() => { throw new Error("CSE1 event length is out of bounds"); })();
    }
    return (() => { let index = 0; let event = []; while (true) {
    if ((index === length)) { return Object.freeze(event); } else { const byte = process_status((api.eventByte)(index)); if (((byte < 0) || (byte > 255))) { return (() => { throw new Error("CSE1 event byte is out of bounds"); })(); } else { const _recur_0 = (index + 1); const _recur_1 = $$bc$conj_value(event, byte); index = _recur_0; event = _recur_1; continue; } }
  } })();
  } else {
    return (() => { throw new Error("persistent session request must carry bounded exact bytes"); })();
  }
}

function decode_cse1_event(bytes) {
  if (exact_byte_array_p(bytes, cse1_max_bytes)) {
    if (((bytes.length < 21) || (!($$bc$equiv(frozen_byte_range(bytes, 0, 4), [67, 83, 69, 49]))))) {
      (() => { throw new Error("CSE1 event magic is invalid"); })();
    }
    const slot = little_u32(bytes, 4);
    const generation = little_u32(bytes, 8);
    const sequence = little_safe_u64(bytes, 12);
    const tag = byte_at(bytes, 20);
    const identity_at = (offset) => frozen_byte_range(bytes, offset, (offset + identity_bytes));
    return ((($$bc$equiv(tag, 1))) ? (() => { const allocation_offset = (21 + (5 * identity_bytes) + 4); const allocation = parse_blob(bytes, allocation_offset, allocation_epoch_bytes, "CSE1 allocation epoch"); const allocation_bytes = require_allocation_epoch(allocation); if ((!($$bc$equiv(allocation.next, bytes.length)))) {
  (() => { throw new Error("CSE1 Opened event has an invalid shape"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "opened", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence, [$$bc$property_key($$bc$keyword("packageId"))]: identity_at(21), [$$bc$property_key($$bc$keyword("sessionId"))]: identity_at(53), [$$bc$property_key($$bc$keyword("world"))]: identity_at(85), [$$bc$property_key($$bc$keyword("allocation"))]: allocation_bytes}; })() : (($$bc$equiv(tag, 3))) ? (() => { if ((!($$bc$equiv(bytes.length, (21 + (5 * identity_bytes) + 4))))) {
  (() => { throw new Error("CSE1 Candidate event has an invalid shape"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "candidate", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence, [$$bc$property_key($$bc$keyword("candidateId"))]: identity_at(53), [$$bc$property_key($$bc$keyword("base"))]: identity_at(85)}; })() : (($$bc$equiv(tag, 4))) ? (() => { if ((!($$bc$equiv(bytes.length, (21 + (5 * identity_bytes) + 4))))) {
  (() => { throw new Error("CSE1 issued authority event has an invalid shape"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "issued", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence, [$$bc$property_key($$bc$keyword("authorization"))]: identity_at(21), [$$bc$property_key($$bc$keyword("packageId"))]: identity_at(53), [$$bc$property_key($$bc$keyword("sessionId"))]: identity_at(85), [$$bc$property_key($$bc$keyword("base"))]: identity_at(117), [$$bc$property_key($$bc$keyword("candidateId"))]: identity_at(149)}; })() : (($$bc$equiv(tag, 5))) ? (() => { const prefix_end = (21 + (5 * identity_bytes) + 4); const projection_tag = byte_at(bytes, prefix_end); const successor = identity_at(53); return ((($$bc$equiv(projection_tag, 0))) ? (() => { if ((!($$bc$equiv(bytes.length, (prefix_end + 1))))) {
  (() => { throw new Error("CSE1 Admission event has trailing bytes"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "admission", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence, [$$bc$property_key($$bc$keyword("successor"))]: successor, [$$bc$property_key($$bc$keyword("projection"))]: null}; })() : (($$bc$equiv(projection_tag, 1))) ? (() => { const observation_offset = (prefix_end + 1); const term_record = parse_blob(bytes, (observation_offset + identity_bytes), cse1_max_bytes, "CSE1 projected Term"); if ((!($$bc$equiv(term_record.next, bytes.length)))) {
  (() => { throw new Error("CSE1 Admission projection has trailing bytes"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "admission", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence, [$$bc$property_key($$bc$keyword("successor"))]: successor, [$$bc$property_key($$bc$keyword("projection"))]: {[$$bc$property_key($$bc$keyword("observationId"))]: identity_at(observation_offset), [$$bc$property_key($$bc$keyword("termBytes"))]: term_record.bytes}}; })() : (() => { throw new Error("CSE1 Admission projection tag is invalid"); })()); })() : (($$bc$equiv(tag, 6))) ? (() => { if ((!($$bc$equiv(bytes.length, 21)))) {
  (() => { throw new Error("CSE1 Disposed event has trailing bytes"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "disposed", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence}; })() : (($$bc$equiv(tag, 7))) ? (() => { if ((!($$bc$equiv(bytes.length, 25)))) {
  (() => { throw new Error("CSE1 rejection has an invalid shape"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "rejected", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence, [$$bc$property_key($$bc$keyword("reason"))]: little_u32(bytes, 21)}; })() : {[$$bc$property_key($$bc$keyword("kind"))]: "input", [$$bc$property_key($$bc$keyword("slot"))]: slot, [$$bc$property_key($$bc$keyword("generation"))]: generation, [$$bc$property_key($$bc$keyword("sequence"))]: sequence});
  } else {
    return (() => { throw new Error("CSE1 event must carry bounded exact bytes"); })();
  }
}

function encode_session_command_bang(session, tag, payload) {
  const bytes = [67, 87, 73, 49];
  append_u32_bang(bytes, session.handle.slot);
  append_u32_bang(bytes, session.handle.generation);
  append_u64_bang(bytes, session.sequence.value);
  bytes.push(tag);
  if ((!(payload == null))) {
    payload.forEach((byte) => {
  bytes.push(byte);
});
  }
  return Object.freeze(bytes);
}

function blob_command_bang(session, tag, payload) {
  const blob = [];
  append_blob_bang(blob, payload);
  return encode_session_command_bang(session, tag, blob);
}

function ascii_bytes(value, label) {
  if (($$bc$equiv(typeof value, "string"))) {
    const length = value.length;
    if ((($$bc$equiv(length, 0)) || (length > 64))) {
      (() => { throw new Error($$bc$str(label, " is outside its byte bound")); })();
    }
    return (() => { let index = 0; let bytes = []; while (true) {
    if ((index === length)) { return Object.freeze(bytes); } else { const code = value.charCodeAt(index); if (((code < 33) || (code > 126))) { return (() => { throw new Error($$bc$str(label, " must be printable ASCII")); })(); } else { const _recur_0 = (index + 1); const _recur_1 = $$bc$conj_value(bytes, code); index = _recur_0; bytes = _recur_1; continue; } }
  } })();
  } else {
    return (() => { throw new Error($$bc$str(label, " must be text")); })();
  }
}

function decode_physical_observation(observation) {
  const envelope = observation.value;
  if (((_truthy) => _truthy !== false && _truthy != null)(((_logical) => (_logical !== false && _logical != null ? (($$bc$equiv(envelope.length, 1)) && ($$bc$equiv(typeof envelope[0], "string"))) : _logical))(Array.isArray(envelope)))) {
    const value = JSON.parse(envelope[0]);
    const kind = value.kind;
    return ((($$bc$equiv(kind, "keyboard"))) ? (() => { const phase = value.phase; const phase_tag = ((($$bc$equiv(phase, "down"))) ? 0 : (($$bc$equiv(phase, "up"))) ? 1 : -1); const repeat = value.repeat; if (((phase_tag < 0) || (!(($$bc$equiv(repeat, true)) || ($$bc$equiv(repeat, false)))))) {
  (() => { throw new Error("keyboard observation has an invalid phase"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "input", [$$bc$property_key($$bc$keyword("source"))]: {[$$bc$property_key($$bc$keyword("sequence"))]: observation.sequence, [$$bc$property_key($$bc$keyword("code"))]: ascii_bytes(value.code, "keyboard code"), [$$bc$property_key($$bc$keyword("phase"))]: phase_tag}}; })() : (($$bc$equiv(kind, "process-occurrence"))) ? (() => { const ordinal = value.ordinal; if (((!((_truthy) => _truthy !== false && _truthy != null)(Number.isSafeInteger(ordinal))) || (ordinal < 0))) {
  (() => { throw new Error("process occurrence ordinal is invalid"); })();
}
return {[$$bc$property_key($$bc$keyword("kind"))]: "candidate", [$$bc$property_key($$bc$keyword("ordinal"))]: ordinal}; })() : null);
  } else {
    return (() => { throw new Error("input observation lacks its exact physical envelope"); })();
  }
}

function physical_input_command_bang(session, input) {
  const payload = [];
  append_u64_bang(payload, input.sequence);
  payload.push(0);
  append_blob_bang(payload, input.code);
  payload.push(input.phase);
  return encode_session_command_bang(session, 3, payload);
}

function tick_candidate_command_bang(session, fixed_tick, configuration) {
  const milliseconds = fixed_tick.milliseconds;
  const payload = [];
  if (((!((_truthy) => _truthy !== false && _truthy != null)(Number.isSafeInteger(milliseconds))) || ((milliseconds <= 0) || (milliseconds > 4294967295)))) {
    (() => { throw new Error("fixed tick is outside the CWI1 bound"); })();
  }
  append_u64_bang(payload, configuration.revision);
  append_u32_bang(payload, milliseconds);
  return encode_session_command_bang(session, 4, payload);
}

function occurrence_candidate_command_bang(session, ordinal) {
  const occurrences = session.occurrences;
  if ((ordinal >= occurrences.length)) {
    (() => { throw new Error("process occurrence ordinal is outside the cartridge"); })();
  }
  return blob_command_bang(session, 2, occurrences[ordinal]);
}

function admission_scope_bytes_bang(session, candidate) {
  const payload = [];
  [session.packageId, session.sessionId, candidate.base, candidate.candidateId].forEach((identity) => {
  identity.forEach((byte) => {
  payload.push(byte);
});
});
  return payload;
}

function apply_session_command_bang(module, session, command) {
  const event = decode_cse1_event(dispatch_session_request(module, command, "command"));
  const current_sequence = session.sequence.value;
  if (((!($$bc$equiv(event.slot, session.handle.slot))) || ((!($$bc$equiv(event.generation, session.handle.generation))) || (!($$bc$equiv(event.sequence, (current_sequence + 1))))))) {
    (() => { throw new Error("CSE1 event does not advance the exact live session"); })();
  }
  (() => { const _a = session.sequence, _v = event.sequence; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  if (($$bc$equiv(event.kind, "rejected"))) {
    (() => { throw new Error($$bc$str("persistent Clause operation rejected with reason ", event.reason)); })();
  }
  return event;
}

function parse_canonical_blob(bytes, offset, label) {
  const header_end = require_range(bytes, offset, 4, label);
  const length = big_u32(bytes, offset);
  const end = require_range(bytes, header_end, length, label);
  return {[$$bc$property_key($$bc$keyword("bytes"))]: frozen_byte_range(bytes, header_end, end), [$$bc$property_key($$bc$keyword("next"))]: end};
}

function ascii_text(bytes, label) {
  if ((($$bc$equiv(bytes.length, 0)) || (bytes.length > 128))) {
    (() => { throw new Error($$bc$str(label, " is outside its text bound")); })();
  }
  return (() => { let index = 0; let result = ""; while (true) {
    if (($$bc$equiv(index, bytes.length))) { return result; } else { const byte = bytes[index]; if (((byte < 32) || (byte > 126))) { return (() => { throw new Error($$bc$str(label, " is not canonical ASCII")); })(); } else { const _recur_0 = (index + 1); const _recur_1 = $$bc$str(result, String.fromCharCode(byte)); index = _recur_0; result = _recur_1; continue; } }
  } })();
}

function decode_term_node(bytes, offset, depth) {
  if ((depth > 64)) {
    (() => { throw new Error("projected Term exceeds its depth bound"); })();
  }
  const tag_end = require_range(bytes, offset, 1, "projected Term node");
  const tag = byte_at(bytes, offset);
  return ((($$bc$equiv(tag, 0))) ? (() => { const kind = parse_canonical_blob(bytes, tag_end, "projected Atom kind"); const payload = parse_canonical_blob(bytes, kind.next, "projected Atom payload"); const equality_end = require_range(bytes, payload.next, 1, "projected Atom equality"); if ((!($$bc$equiv(byte_at(bytes, payload.next), 0)))) {
  (() => { throw new Error("projected Atom equality contract is invalid"); })();
}
return {[$$bc$property_key($$bc$keyword("node"))]: {[$$bc$property_key($$bc$keyword("kind"))]: "atom", [$$bc$property_key($$bc$keyword("atomKind"))]: kind.bytes, [$$bc$property_key($$bc$keyword("payload"))]: payload.bytes}, [$$bc$property_key($$bc$keyword("next"))]: equality_end}; })() : (($$bc$equiv(tag, 1))) ? (() => { const left = decode_term_node(bytes, tag_end, (depth + 1)); const operator = decode_term_node(bytes, left.next, (depth + 1)); const right = decode_term_node(bytes, operator.next, (depth + 1)); return {[$$bc$property_key($$bc$keyword("node"))]: {[$$bc$property_key($$bc$keyword("kind"))]: "triple", [$$bc$property_key($$bc$keyword("slots"))]: [left.node, operator.node, right.node]}, [$$bc$property_key($$bc$keyword("next"))]: right.next}; })() : (() => { throw new Error("projected Term node tag is invalid"); })());
}

function decode_canonical_term(bytes) {
  if (exact_byte_array_p(bytes, cse1_max_bytes)) {
    const node_start = require_range(bytes, 0, (2 * identity_bytes), "projected Term scope");
    const result = decode_term_node(bytes, node_start, 0);
    if ((!($$bc$equiv(result.next, bytes.length)))) {
      (() => { throw new Error("projected Term has trailing bytes"); })();
    }
    return result.node;
  } else {
    return (() => { throw new Error("projected Term bytes are outside the CSE1 bound"); })();
  }
}

function atom_kind_text(node) {
  return (($$bc$equiv(node.kind, "atom")) ? ascii_text(node.atomKind, "projected Atom kind") : (() => { throw new Error("projected realization expected an Atom"); })());
}

function projected_number(payload) {
  return (($$bc$equiv(payload.length, 8)) ? finite_f64(payload, 0) : (() => { throw new Error("projected F64 payload is invalid"); })());
}

function realize_object(realize_node, first) {
  return (() => { let node = first; let fields = []; let keys = new Set(); while (true) {
    if (($$bc$equiv(node.kind, "atom"))) { return (($$bc$equiv(atom_kind_text(node), "clause/js-object-end-v1")) ? Object.freeze(Object.fromEntries(fields)) : (() => { throw new Error("projected object has an invalid terminator"); })()); } else { const slots = node.slots; const field = slots[0]; const value = slots[1]; const rest = slots[2]; ((!($$bc$equiv(atom_kind_text(field), "clause/js-field-v1"))) ? (() => { return (() => { throw new Error("projected object entry lacks a field Atom"); })(); })() : null); const key = ascii_text(field.payload, "projected field"); (((_truthy) => _truthy !== false && _truthy != null)((($$bc$equiv(key, "__proto__")) || (($$bc$equiv(key, "prototype")) || (($$bc$equiv(key, "constructor")) || keys.has(key))))) ? (() => { return (() => { throw new Error("projected object field is unsafe or duplicated"); })(); })() : null); keys.add(key); const _recur_0 = rest; const _recur_1 = $$bc$conj_value(fields, [key, realize_node(value)]); const _recur_2 = keys; node = _recur_0; fields = _recur_1; keys = _recur_2; continue; }
  } })();
}

function realize_array(realize_node, first) {
  return (() => { let node = first; let values = []; while (true) {
    if (($$bc$equiv(node.kind, "atom"))) { return (($$bc$equiv(atom_kind_text(node), "clause/js-array-end-v1")) ? Object.freeze(values) : (() => { throw new Error("projected array has an invalid terminator"); })()); } else { const slots = node.slots; const item = slots[0]; ((!($$bc$equiv(atom_kind_text(item), "clause/js-item-v1"))) ? (() => { return (() => { throw new Error("projected array entry lacks an item Atom"); })(); })() : null); const _recur_0 = slots[2]; const _recur_1 = $$bc$conj_value(values, realize_node(slots[1])); node = _recur_0; values = _recur_1; continue; }
  } })();
}

function realize_projection_node(node) {
  if (($$bc$equiv(node.kind, "atom"))) {
    const kind = atom_kind_text(node);
    const payload = node.payload;
    return ((($$bc$equiv(kind, "clause/process-projected-f64-v1"))) ? projected_number(payload) : (($$bc$equiv(kind, "clause/process-projected-bool-v1"))) ? ((($$bc$equiv(payload.length, 1)) && (payload[0] <= 1)) ? ($$bc$equiv(payload[0], 1)) : (() => { throw new Error("projected Boolean payload is invalid"); })()) : (($$bc$equiv(kind, "clause/process-projected-symbol-v1"))) ? ascii_text(payload, "projected symbol") : (() => { throw new Error("projected scalar Atom is not realizable"); })());
  } else {
    const head = node.slots[0];
    const kind = atom_kind_text(head);
    return ((($$bc$equiv(kind, "clause/js-field-v1"))) ? realize_object(realize_projection_node, node) : (($$bc$equiv(kind, "clause/js-item-v1"))) ? realize_array(realize_projection_node, node) : (() => { throw new Error("projected Term lacks a realizable shape"); })());
  }
}

function decode_projected_term_frame(bytes) {
  return realize_projection_node(decode_canonical_term(bytes));
}

function require_session(value) {
  return (((!(value == null)) && ((!(value.handle == null)) && ((!(value.sequence == null)) && (!(value.disposed == null))))) ? value : (() => { throw new Error("Wasm session is invalid"); })());
}

function require_live_session(value) {
  const session = require_session(value);
  return (((_truthy) => _truthy !== false && _truthy != null)(session.disposed.value) ? (() => { throw new Error("Wasm session is disposed"); })() : session);
}

function require_candidate(value) {
  return (((!(value == null)) && ((!(value.candidateId == null)) && (!(value.base == null)))) ? value : (() => { throw new Error("Wasm candidate is invalid"); })());
}

function reject_reason(error) {
  const message = error.message;
  return (($$bc$equiv(typeof message, "string")) ? message : "Wasm cartridge boundary rejected");
}

function create_wasm_cartridge_port_bang(module, policy) {
  return workbench["->CartridgePort"]((package_candidate, complete) => (() => { try {
    return complete(workbench["->PackageAccepted"](parse_persistent_cartridge_bang(package_candidate)));
  } catch (_catch_0) {
    switch ($$bd$catch_dispatch(_catch_0, [Error])) {
      case 0: {
        const error = _catch_0;
        return complete(workbench["->PackageRejected"](reject_reason(error)));
        break;
      }
    }
  } })(), (accepted_package, __generation, complete) => (() => { try {
    const cartridge = accepted_package;
  const event = decode_cse1_event(dispatch_session_request(module, cartridge.openBytes, "open"));
  const __opened = (((!($$bc$equiv(event.kind, "opened"))) || (!($$bc$equiv(event.sequence, 0)))) ? (() => { return (() => { throw new Error("persistent session did not open exactly once"); })(); })() : null);
  const session = WasmSession(Object.freeze({[$$bc$property_key($$bc$keyword("slot"))]: event.slot, [$$bc$property_key($$bc$keyword("generation"))]: event.generation}), event.packageId, event.sessionId, event.allocation, ({value: event.world, watches: {}}), ({value: 0, watches: {}}), cartridge.occurrences, ({value: false, watches: {}}));
  const bootstrap_frame = workbench["create-workbench-envelope"](policy, "[]");
  return complete(workbench["->SessionStarted"](session, event.world, bootstrap_frame));
  } catch (_catch_1) {
    switch ($$bd$catch_dispatch(_catch_1, [Error])) {
      case 0: {
        const error = _catch_1;
        return complete(workbench["->SessionFailed"](reject_reason(error)));
        break;
      }
    }
  } })(), (incoming_session, fixed_tick, configuration, complete) => (() => { try {
    const session = require_live_session(incoming_session);
  const candidate_ordinal = ({value: null, watches: {}});
  (() => { configuration.observations.forEach((observation) => {
  const decoded = decode_physical_observation(observation);
  if ((!(decoded == null))) {
    if (($$bc$equiv(decoded.kind, "candidate"))) {
      if ((!(candidate_ordinal.value == null))) {
        (() => { throw new Error("one configuration may select only one process occurrence"); })();
      }
      (() => { const _a = candidate_ordinal, _v = decoded.ordinal; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
    } else {
      const event = apply_session_command_bang(module, session, physical_input_command_bang(session, decoded.source));
      if ((!($$bc$equiv(event.kind, "input")))) {
        (() => { throw new Error("CWI1 physical input did not produce InputAccepted"); })();
      }
    }
  }
}); })();
  const event = apply_session_command_bang(module, session, ((candidate_ordinal.value == null) ? tick_candidate_command_bang(session, fixed_tick, configuration) : occurrence_candidate_command_bang(session, candidate_ordinal.value)));
  if ((!($$bc$equiv(event.kind, "candidate")))) {
    (() => { throw new Error("CWI1 process occurrence did not produce CandidateAccepted"); })();
  }
  return complete(workbench["->CandidateProduced"](WasmCandidate(event.candidateId, event.base)));
  } catch (_catch_2) {
    switch ($$bd$catch_dispatch(_catch_2, [Error])) {
      case 0: {
        const error = _catch_2;
        return complete(workbench["->CandidateFailed"](reject_reason(error)));
        break;
      }
    }
  } })(), (incoming_session, incoming_candidate, complete) => (() => { try {
    const session = require_live_session(incoming_session);
  const candidate = require_candidate(incoming_candidate);
  const scope = admission_scope_bytes_bang(session, candidate);
  const issued = apply_session_command_bang(module, session, encode_session_command_bang(session, 5, scope));
  if ((!($$bc$equiv(issued.kind, "issued")))) {
    (() => { throw new Error("CWI1 issuance did not produce exact Admission authority"); })();
  }
  const payload = admission_scope_bytes_bang(session, candidate);
  (() => { issued.authorization.forEach((byte) => {
  payload.push(byte);
}); })();
  const event = apply_session_command_bang(module, session, encode_session_command_bang(session, 6, payload));
  const projection = event.projection;
  if (((!($$bc$equiv(event.kind, "admission"))) || (projection == null))) {
    (() => { throw new Error("Admission produced no package-declared frame Observation"); })();
  }
  (() => { const _a = session.world, _v = event.successor; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  const frame = workbench["create-workbench-envelope"](policy, JSON.stringify(projection.termBytes));
  return complete(workbench["->AdmissionAccepted"](session, event.successor, frame));
  } catch (_catch_3) {
    switch ($$bd$catch_dispatch(_catch_3, [Error])) {
      case 0: {
        const error = _catch_3;
        return complete(workbench["->AdmissionRejected"](reject_reason(error)));
        break;
      }
    }
  } })(), (incoming_session) => { const session = require_session(incoming_session);
if ((!((_truthy) => _truthy !== false && _truthy != null)(session.disposed.value))) {
  const event = apply_session_command_bang(module, session, encode_session_command_bang(session, 7, null));
  if ((!($$bc$equiv(event.kind, "disposed")))) {
    (() => { throw new Error("CWI1 disposal did not produce Disposed"); })();
  }
  return (() => { const _a = session.disposed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
} });
}

const create_wasm_cartridge_port = create_wasm_cartridge_port_bang;

export { Cwo1Observation as "->Cwo1Observation" };
export { ExactProcessObservation as "->ExactProcessObservation" };
export { ExactProcessRequest as "->ExactProcessRequest" };
export { Cwo1Observation as "Cwo1Observation" };
export { ExactProcessObservation as "ExactProcessObservation" };
export { ExactProcessRequest as "ExactProcessRequest" };
export { create_wasm_cartridge_port as "create-wasm-cartridge-port" };
export { cse1_projected_term_json_max_source_units as "cse1-projected-term-json-max-source-units" };
export { cse1_projected_term_max_properties as "cse1-projected-term-max-properties" };
export { cwo1observation_observationId as "cwo1observation-observationId" };
export { cwo1observation_stateRevisionId as "cwo1observation-stateRevisionId" };
export { cwo1observation_values as "cwo1observation-values" };
export { decode_cwo1_observation as "decode-cwo1-observation" };
export { decode_cwr1_hex as "decode-cwr1-hex" };
export { decode_projected_term_frame as "decode-projected-term-frame" };
export { exactprocessobservation_bytes as "exactprocessobservation-bytes" };
export { exactprocessrequest_bytes as "exactprocessrequest-bytes" };
//# sourceMappingURL=wasm-cartridge-port.js.map
