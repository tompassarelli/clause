import * as workbench from './workbench.js';
import { conj_value as $$bc$conj_value, equivV as $$bc$equiv, record_value as $$bc$record_value, str as $$bc$str } from 'beagle/core.js';
import { catch_dispatch as $$bd$catch_dispatch } from 'beagle/exception-dispatch.js';

const cwr1_max_bytes = (4 * 1024 * 1024);

const cwo1_max_bytes = (64 * 1024);

const cwo1_prefix_bytes = (4 + 32 + 32);

const cwo1_identity_bytes = 32;

const cwo1_max_values = 256;

function ExactProcessRequest(bytes) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/ExactProcessRequest", {_tag: "ExactProcessRequest", bytes});
}

function exactprocessrequest_bytes(r) { return r.bytes; }

function ExactProcessObservation(bytes) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/ExactProcessObservation", {_tag: "ExactProcessObservation", bytes});
}

function exactprocessobservation_bytes(r) { return r.bytes; }

function WasmCandidate(request) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/WasmCandidate", {_tag: "WasmCandidate", request});
}

function wasmcandidate_request(r) { return r.request; }

function WasmSession(initial, disposed) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/WasmSession", {_tag: "WasmSession", initial, disposed});
}

function wasmsession_initial(r) { return r.initial; }

function wasmsession_disposed(r) { return r.disposed; }

function Cwo1Observation(observationId, stateRevisionId, values) {
  return $$bc$record_value("jump-arena-shell.wasm-cartridge-port/Cwo1Observation", {_tag: "Cwo1Observation", observationId, stateRevisionId, values});
}

function cwo1observation_observationId(r) { return r.observationId; }

function cwo1observation_stateRevisionId(r) { return r.stateRevisionId; }

function cwo1observation_values(r) { return r.values; }

function exact_byte_array_p(bytes, maximum) {
  return ((_logical) => (_logical !== false && _logical != null ? ((1 <= bytes.length) && ((bytes.length <= maximum) && bytes.every((byte) => ((_logical) => (_logical !== false && _logical != null ? ((0 <= byte) && (byte <= 255)) : _logical))(Number.isInteger(byte))))) : _logical))(Array.isArray(bytes));
}

function require_request(request) {
  return (((!(request == null)) && exact_byte_array_p(request.bytes, cwr1_max_bytes)) ? ExactProcessRequest(Object.freeze(request.bytes.slice())) : (() => { throw new Error("cartridge request must carry bounded exact bytes"); })());
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

function frozen_byte_range(bytes, start, end) {
  return Object.freeze(bytes.slice(start, end));
}

function finite_f64(bytes, offset) {
  const packed = new Uint8Array(bytes.slice(offset, (offset + 8)));
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

function require_session(value) {
  return (((!(value == null)) && ((!(value.initial == null)) && (!(value.disposed == null)))) ? value : (() => { throw new Error("Wasm session is invalid"); })());
}

function require_live_session(value) {
  const session = require_session(value);
  return (((_truthy) => _truthy !== false && _truthy != null)(session.disposed.value) ? (() => { throw new Error("Wasm session is disposed"); })() : session);
}

function require_candidate(value) {
  return (((!(value == null)) && (!(value.request == null))) ? value : (() => { throw new Error("Wasm candidate is invalid"); })());
}

function reject_reason(error) {
  const message = error.message;
  return (($$bc$equiv(typeof message, "string")) ? message : "Wasm cartridge boundary rejected");
}

function create_wasm_cartridge_port_bang(module, policy) {
  return workbench["->CartridgePort"]((package_candidate, complete) => (() => { try {
    return complete(workbench["->PackageAccepted"](require_request(package_candidate)));
  } catch (_catch_0) {
    switch ($$bd$catch_dispatch(_catch_0, [Error])) {
      case 0: {
        const error = _catch_0;
        return complete(workbench["->PackageRejected"](reject_reason(error)));
        break;
      }
    }
  } })(), (accepted_package, __generation, complete) => (() => { try {
    const request = require_request(accepted_package);
  const session = WasmSession(request, ({value: false, watches: {}}));
  return complete(workbench["->SessionStarted"](session, null, null));
  } catch (_catch_1) {
    switch ($$bd$catch_dispatch(_catch_1, [Error])) {
      case 0: {
        const error = _catch_1;
        return complete(workbench["->SessionFailed"](reject_reason(error)));
        break;
      }
    }
  } })(), (incoming_session, __fixed_tick, __configuration, complete) => (() => { try {
    const session = require_live_session(incoming_session);
  return complete(workbench["->CandidateProduced"](WasmCandidate(session.initial)));
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
  const response = dispatch_exact_request(module, candidate.request);
  const observation = decode_cwo1_observation(response.bytes);
  const frame = project_frame(policy, observation);
  return complete(workbench["->AdmissionAccepted"](session, observation.stateRevisionId, frame));
  } catch (_catch_3) {
    switch ($$bd$catch_dispatch(_catch_3, [Error])) {
      case 0: {
        const error = _catch_3;
        return complete(workbench["->AdmissionRejected"](reject_reason(error)));
        break;
      }
    }
  } })(), (incoming_session) => { const session = require_session(incoming_session);
return (() => { const _a = session.disposed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })(); });
}

const create_wasm_cartridge_port = create_wasm_cartridge_port_bang;

export { Cwo1Observation as "->Cwo1Observation" };
export { ExactProcessObservation as "->ExactProcessObservation" };
export { ExactProcessRequest as "->ExactProcessRequest" };
export { Cwo1Observation as "Cwo1Observation" };
export { ExactProcessObservation as "ExactProcessObservation" };
export { ExactProcessRequest as "ExactProcessRequest" };
export { create_wasm_cartridge_port as "create-wasm-cartridge-port" };
export { cwo1observation_observationId as "cwo1observation-observationId" };
export { cwo1observation_stateRevisionId as "cwo1observation-stateRevisionId" };
export { cwo1observation_values as "cwo1observation-values" };
export { decode_cwo1_observation as "decode-cwo1-observation" };
export { exactprocessobservation_bytes as "exactprocessobservation-bytes" };
export { exactprocessrequest_bytes as "exactprocessrequest-bytes" };
//# sourceMappingURL=wasm-cartridge-port.js.map
