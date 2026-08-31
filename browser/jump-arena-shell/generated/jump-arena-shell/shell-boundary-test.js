import * as shell from "./shell.js";
import * as test from "bun:test";
import { equivV as $$bc$equiv, keyword as $$bc$keyword, property_key as $$bc$property_key } from 'beagle/core.js';
import { catch_dispatch as $$bd$catch_dispatch } from 'beagle/exception-dispatch.js';

function fail_expectation_bang(message) {
  return (() => { throw new Error(message); })();
}

function expect_same_bang(actual, expected) {
  return ((actual === expected) ? null : fail_expectation_bang("expected values to be identical"));
}

function expect_equal_bang(actual, expected) {
  return ((JSON.stringify(actual) === JSON.stringify(expected)) ? null : fail_expectation_bang("expected values to be structurally equal"));
}

function expect_throws_message_bang(thunk, expected) {
  const matched = (() => { try {
    thunk();
  return false;
  } catch (_catch_0) {
    switch ($$bd$catch_dispatch(_catch_0, [Error])) {
      case 0: {
        const error = _catch_0;
        return (error.message === expected);
        break;
      }
    }
  } })();
  return (matched ? null : fail_expectation_bang("expected function to throw the exact message"));
}

function empty_listener_map() {
  return new Map();
}

const browser_listeners = ({value: empty_listener_map(), watches: {}});

const canvas_listeners = ({value: empty_listener_map(), watches: {}});

const pointer_captures = ({value: new Set(), watches: {}});

const emitted_inputs = ({value: [], watches: {}});

const render_count = ({value: 0, watches: {}});

const resource_disposals = ({value: 0, watches: {}});

const renderer_disposals = ({value: 0, watches: {}});

const context_losses = ({value: 0, watches: {}});

const canvas_removals = ({value: 0, watches: {}});

const released_pointers = ({value: 0, watches: {}});

const focus_requests = ({value: 0, watches: {}});

const pixel_ratio = ({value: 0.0, watches: {}});

const material_colors = ({value: [], watches: {}});

function vector3() {
  return {[$$bc$property_key($$bc$keyword("x"))]: 0.0, [$$bc$property_key($$bc$keyword("y"))]: 0.0, [$$bc$property_key($$bc$keyword("z"))]: 0.0, [$$bc$property_key($$bc$keyword("set"))]: (__x, __y, __z) => null};
}

function node() {
  return {[$$bc$property_key($$bc$keyword("position"))]: vector3(), [$$bc$property_key($$bc$keyword("rotation"))]: {[$$bc$property_key($$bc$keyword("x"))]: 0.0, [$$bc$property_key($$bc$keyword("y"))]: 0.0, [$$bc$property_key($$bc$keyword("z"))]: 0.0}, [$$bc$property_key($$bc$keyword("scale"))]: vector3(), [$$bc$property_key($$bc$keyword("add"))]: (__child) => null, [$$bc$property_key($$bc$keyword("remove"))]: (__child) => null, [$$bc$property_key($$bc$keyword("clear"))]: () => null, [$$bc$property_key($$bc$keyword("lookAt"))]: (__x, __y, __z) => null, [$$bc$property_key($$bc$keyword("updateProjectionMatrix"))]: () => null};
}

function fakeScene() {
  return node();
}

function fakeColor(__color) {
  return {};
}

function fakeCamera(__fov, __aspect, __near, __far) {
  return node();
}

function fakeLight(__color, __ground_or_intensity, ...__rest) {
  return node();
}

function fakeResource_bang(...__arguments) {
  return {[$$bc$property_key($$bc$keyword("dispose"))]: () => (() => { const _a = resource_disposals; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()};
}

function fakeMaterial_bang(options) {
  material_colors.value.push(options.color);
  return fakeResource_bang(options);
}

function fakeMesh(geometry, material) {
  return Object.assign({}, node(), {[$$bc$property_key($$bc$keyword("geometry"))]: geometry, [$$bc$property_key($$bc$keyword("material"))]: material});
}

function fakeGroup() {
  return node();
}

function fakeCanvas_bang() {
  return {[$$bc$property_key($$bc$keyword("addEventListener"))]: (kind, handler) => canvas_listeners.value.set(kind, handler), [$$bc$property_key($$bc$keyword("removeEventListener"))]: (kind, __handler) => canvas_listeners.value.delete(kind), [$$bc$property_key($$bc$keyword("getBoundingClientRect"))]: () => ({[$$bc$property_key($$bc$keyword("left"))]: 10.0, [$$bc$property_key($$bc$keyword("top"))]: 20.0, [$$bc$property_key($$bc$keyword("width"))]: 200.0, [$$bc$property_key($$bc$keyword("height"))]: 100.0}), [$$bc$property_key($$bc$keyword("setPointerCapture"))]: (pointer_id) => pointer_captures.value.add(pointer_id), [$$bc$property_key($$bc$keyword("hasPointerCapture"))]: (pointer_id) => pointer_captures.value.has(pointer_id), [$$bc$property_key($$bc$keyword("releasePointerCapture"))]: (pointer_id) => { pointer_captures.value.delete(pointer_id);
return (() => { const _a = released_pointers; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })(); }, [$$bc$property_key($$bc$keyword("focus"))]: (__options) => { (() => { const _a = focus_requests; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })();
return (canvas_listeners.value.get("focus"))({}); }, [$$bc$property_key($$bc$keyword("setAttribute"))]: (__name, __value) => null, [$$bc$property_key($$bc$keyword("remove"))]: () => (() => { const _a = canvas_removals; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()};
}

function fakeRenderer_bang(__options) {
  return {[$$bc$property_key($$bc$keyword("domElement"))]: fakeCanvas_bang(), [$$bc$property_key($$bc$keyword("setPixelRatio"))]: (ratio) => (() => { const _a = pixel_ratio, _v = ratio; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })(), [$$bc$property_key($$bc$keyword("setSize"))]: (__width, __height, __update_style) => null, [$$bc$property_key($$bc$keyword("render"))]: (__scene, __camera) => (() => { const _a = render_count; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })(), [$$bc$property_key($$bc$keyword("dispose"))]: () => (() => { const _a = renderer_disposals; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })(), [$$bc$property_key($$bc$keyword("forceContextLoss"))]: () => (() => { const _a = context_losses; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()};
}

function fakeBrowser_bang() {
  return {[$$bc$property_key($$bc$keyword("devicePixelRatio"))]: 4.0, [$$bc$property_key($$bc$keyword("addEventListener"))]: (kind, handler) => browser_listeners.value.set(kind, handler), [$$bc$property_key($$bc$keyword("removeEventListener"))]: (kind, __handler) => browser_listeners.value.delete(kind)};
}

function fakeMount() {
  return {[$$bc$property_key($$bc$keyword("clientWidth"))]: 640.0, [$$bc$property_key($$bc$keyword("clientHeight"))]: 360.0, [$$bc$property_key($$bc$keyword("appendChild"))]: (__canvas) => null};
}

function frozen_vec3(x, y, z) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("x"))]: x, [$$bc$property_key($$bc$keyword("y"))]: y, [$$bc$property_key($$bc$keyword("z"))]: z});
}

function frozen_platform() {
  return Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(0.0, -0.25, 0.0), [$$bc$property_key($$bc$keyword("size"))]: frozen_vec3(12.0, 0.5, 12.0)});
}

function frozen_collectible(state) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(2.0, 3.0, 4.0), [$$bc$property_key($$bc$keyword("state"))]: state});
}

function sample_frame(state) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("player"))]: Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(2.0, 3.0, 4.0), [$$bc$property_key($$bc$keyword("velocity"))]: frozen_vec3(5.0, 6.0, 7.0), [$$bc$property_key($$bc$keyword("yaw"))]: 0.5, [$$bc$property_key($$bc$keyword("grounded"))]: false}), [$$bc$property_key($$bc$keyword("world"))]: Object.freeze({[$$bc$property_key($$bc$keyword("platforms"))]: Object.freeze([frozen_platform()]), [$$bc$property_key($$bc$keyword("collectibles"))]: Object.freeze([frozen_collectible(state)])})});
}

function reset_fixture_bang() {
  (() => { const _a = browser_listeners, _v = empty_listener_map(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = canvas_listeners, _v = empty_listener_map(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = pointer_captures, _v = new Set(); const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = emitted_inputs, _v = []; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = render_count, _v = 0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = resource_disposals, _v = 0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = renderer_disposals, _v = 0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = context_losses, _v = 0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = canvas_removals, _v = 0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = released_pointers, _v = 0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = focus_requests, _v = 0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = pixel_ratio, _v = 0.0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return (() => { const _a = material_colors, _v = []; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
}

test["test"]("production shell emits input without advancing player state and tears down", () => { reset_fixture_bang();
const three = {[$$bc$property_key($$bc$keyword("Scene"))]: fakeScene, [$$bc$property_key($$bc$keyword("Color"))]: fakeColor, [$$bc$property_key($$bc$keyword("PerspectiveCamera"))]: fakeCamera, [$$bc$property_key($$bc$keyword("WebGLRenderer"))]: fakeRenderer_bang, [$$bc$property_key($$bc$keyword("HemisphereLight"))]: fakeLight, [$$bc$property_key($$bc$keyword("DirectionalLight"))]: fakeLight, [$$bc$property_key($$bc$keyword("BoxGeometry"))]: fakeResource_bang, [$$bc$property_key($$bc$keyword("MeshStandardMaterial"))]: fakeMaterial_bang, [$$bc$property_key($$bc$keyword("Mesh"))]: fakeMesh, [$$bc$property_key($$bc$keyword("Group"))]: fakeGroup};
const arena = shell["create-jump-arena-shell!"](fakeMount(), fakeBrowser_bang(), three, (input) => emitted_inputs.value.push(input));
const frame = sample_frame("active");
const collected_frame = sample_frame("collected");
const before = JSON.stringify(frame);
const resize_handler = browser_listeners.value.get("resize");
const focus_handler = canvas_listeners.value.get("focus");
const blur_handler = canvas_listeners.value.get("blur");
const key_down = canvas_listeners.value.get("keydown");
const key_up = canvas_listeners.value.get("keyup");
const pointer_down = canvas_listeners.value.get("pointerdown");
const pointer_move = canvas_listeners.value.get("pointermove");
const pointer_up = canvas_listeners.value.get("pointerup");
const pointer_cancel = canvas_listeners.value.get("pointercancel");
(arena.renderFrame)(frame);
const active_color = material_colors.value[2];
(arena.renderFrame)(collected_frame);
expect_same_bang(($$bc$equiv(active_color, material_colors.value[3])), false);
key_down({[$$bc$property_key($$bc$keyword("code"))]: "Space", [$$bc$property_key($$bc$keyword("repeat"))]: false});
expect_same_bang(emitted_inputs.value.length, 0);
focus_handler({});
key_down({[$$bc$property_key($$bc$keyword("code"))]: "Space", [$$bc$property_key($$bc$keyword("repeat"))]: false});
expect_same_bang(emitted_inputs.value.length, 1);
blur_handler({});
key_down({[$$bc$property_key($$bc$keyword("code"))]: "Space", [$$bc$property_key($$bc$keyword("repeat"))]: false});
expect_same_bang(emitted_inputs.value.length, 1);
pointer_down({[$$bc$property_key($$bc$keyword("clientX"))]: 110.0, [$$bc$property_key($$bc$keyword("clientY"))]: 70.0, [$$bc$property_key($$bc$keyword("pointerId"))]: 9, [$$bc$property_key($$bc$keyword("button"))]: 0, [$$bc$property_key($$bc$keyword("buttons"))]: 1});
expect_same_bang(JSON.stringify(frame), before);
expect_equal_bang(frame.player.position, {[$$bc$property_key($$bc$keyword("x"))]: 2.0, [$$bc$property_key($$bc$keyword("y"))]: 3.0, [$$bc$property_key($$bc$keyword("z"))]: 4.0});
expect_equal_bang(frame.player.velocity, {[$$bc$property_key($$bc$keyword("x"))]: 5.0, [$$bc$property_key($$bc$keyword("y"))]: 6.0, [$$bc$property_key($$bc$keyword("z"))]: 7.0});
expect_same_bang(frame.player.grounded, false);
expect_same_bang(render_count.value, 2);
expect_same_bang(emitted_inputs.value.length, 2);
expect_same_bang(pixel_ratio.value, 2.0);
expect_same_bang(browser_listeners.value.has("keydown"), false);
expect_same_bang(browser_listeners.value.has("keyup"), false);
expect_same_bang(focus_requests.value, 1);
(arena.dispose)();
(arena.dispose)();
key_down({[$$bc$property_key($$bc$keyword("code"))]: "KeyW", [$$bc$property_key($$bc$keyword("repeat"))]: false});
focus_handler({});
blur_handler({});
key_down({[$$bc$property_key($$bc$keyword("code"))]: "KeyW", [$$bc$property_key($$bc$keyword("repeat"))]: false});
key_up({[$$bc$property_key($$bc$keyword("code"))]: "KeyW", [$$bc$property_key($$bc$keyword("repeat"))]: false});
pointer_down({[$$bc$property_key($$bc$keyword("clientX"))]: 110.0, [$$bc$property_key($$bc$keyword("clientY"))]: 70.0, [$$bc$property_key($$bc$keyword("pointerId"))]: 10, [$$bc$property_key($$bc$keyword("button"))]: 0, [$$bc$property_key($$bc$keyword("buttons"))]: 1});
pointer_move({[$$bc$property_key($$bc$keyword("clientX"))]: 111.0, [$$bc$property_key($$bc$keyword("clientY"))]: 71.0, [$$bc$property_key($$bc$keyword("pointerId"))]: 9, [$$bc$property_key($$bc$keyword("button"))]: 0, [$$bc$property_key($$bc$keyword("buttons"))]: 1});
pointer_up({[$$bc$property_key($$bc$keyword("clientX"))]: 111.0, [$$bc$property_key($$bc$keyword("clientY"))]: 71.0, [$$bc$property_key($$bc$keyword("pointerId"))]: 9, [$$bc$property_key($$bc$keyword("button"))]: 0, [$$bc$property_key($$bc$keyword("buttons"))]: 0});
pointer_cancel({[$$bc$property_key($$bc$keyword("clientX"))]: 111.0, [$$bc$property_key($$bc$keyword("clientY"))]: 71.0, [$$bc$property_key($$bc$keyword("pointerId"))]: 9, [$$bc$property_key($$bc$keyword("button"))]: 0, [$$bc$property_key($$bc$keyword("buttons"))]: 0});
resize_handler();
expect_throws_message_bang(() => (arena.renderFrame)(frame), "jump arena shell is disposed");
expect_same_bang(browser_listeners.value.size, 0);
expect_same_bang(canvas_listeners.value.size, 0);
expect_same_bang(pointer_captures.value.size, 0);
expect_same_bang(emitted_inputs.value.length, 2);
expect_same_bang(focus_requests.value, 1);
expect_same_bang(render_count.value, 2);
expect_same_bang(released_pointers.value, 1);
expect_same_bang(resource_disposals.value, 7);
expect_same_bang(renderer_disposals.value, 1);
expect_same_bang(context_losses.value, 1);
return expect_same_bang(canvas_removals.value, 1); });
//# sourceMappingURL=shell-boundary-test.js.map
