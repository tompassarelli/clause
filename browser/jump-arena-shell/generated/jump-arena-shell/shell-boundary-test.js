import * as shell from './shell.js';
import * as test from 'bun:test';
import { keyword as $$bc$keyword, property_key as $$bc$property_key } from 'beagle/core.js';

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

const pixel_ratio = ({value: 0.0, watches: {}});

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

function fakeMesh(geometry, material) {
  return Object.assign({}, node(), {[$$bc$property_key($$bc$keyword("geometry"))]: geometry, [$$bc$property_key($$bc$keyword("material"))]: material});
}

function fakeGroup() {
  return node();
}

function fakeCanvas_bang() {
  return {[$$bc$property_key($$bc$keyword("addEventListener"))]: (kind, handler) => canvas_listeners.value.set(kind, handler), [$$bc$property_key($$bc$keyword("removeEventListener"))]: (kind, __handler) => canvas_listeners.value.delete(kind), [$$bc$property_key($$bc$keyword("getBoundingClientRect"))]: () => ({[$$bc$property_key($$bc$keyword("left"))]: 10.0, [$$bc$property_key($$bc$keyword("top"))]: 20.0, [$$bc$property_key($$bc$keyword("width"))]: 200.0, [$$bc$property_key($$bc$keyword("height"))]: 100.0}), [$$bc$property_key($$bc$keyword("setPointerCapture"))]: (pointer_id) => pointer_captures.value.add(pointer_id), [$$bc$property_key($$bc$keyword("hasPointerCapture"))]: (pointer_id) => pointer_captures.value.has(pointer_id), [$$bc$property_key($$bc$keyword("releasePointerCapture"))]: (pointer_id) => { pointer_captures.value.delete(pointer_id);
return (() => { const _a = released_pointers; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })(); }, [$$bc$property_key($$bc$keyword("setAttribute"))]: (__name, __value) => null, [$$bc$property_key($$bc$keyword("remove"))]: () => (() => { const _a = canvas_removals; const _old = _a.value; _a.value = (((_x) => (_x + 1)))(_old); for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _a.value); return _a.value; })()};
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

function sample_frame() {
  return Object.freeze({[$$bc$property_key($$bc$keyword("player"))]: Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(2.0, 3.0, 4.0), [$$bc$property_key($$bc$keyword("velocity"))]: frozen_vec3(5.0, 6.0, 7.0), [$$bc$property_key($$bc$keyword("yaw"))]: 0.5, [$$bc$property_key($$bc$keyword("grounded"))]: false}), [$$bc$property_key($$bc$keyword("world"))]: Object.freeze({[$$bc$property_key($$bc$keyword("platforms"))]: Object.freeze([frozen_platform()])})});
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
  return (() => { const _a = pixel_ratio, _v = 0.0; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
}

test.test("production shell emits input without advancing player state and tears down", () => { reset_fixture_bang();
const three = {[$$bc$property_key($$bc$keyword("Scene"))]: fakeScene, [$$bc$property_key($$bc$keyword("Color"))]: fakeColor, [$$bc$property_key($$bc$keyword("PerspectiveCamera"))]: fakeCamera, [$$bc$property_key($$bc$keyword("WebGLRenderer"))]: fakeRenderer_bang, [$$bc$property_key($$bc$keyword("HemisphereLight"))]: fakeLight, [$$bc$property_key($$bc$keyword("DirectionalLight"))]: fakeLight, [$$bc$property_key($$bc$keyword("BoxGeometry"))]: fakeResource_bang, [$$bc$property_key($$bc$keyword("MeshStandardMaterial"))]: fakeResource_bang, [$$bc$property_key($$bc$keyword("Mesh"))]: fakeMesh, [$$bc$property_key($$bc$keyword("Group"))]: fakeGroup};
const arena = shell["create-jump-arena-shell!"](fakeMount(), fakeBrowser_bang(), three, (input) => emitted_inputs.value.push(input));
const frame = sample_frame();
const before = JSON.stringify(frame);
(arena.renderFrame)(frame);
(browser_listeners.value.get("keydown"))({[$$bc$property_key($$bc$keyword("code"))]: "Space", [$$bc$property_key($$bc$keyword("repeat"))]: false});
(canvas_listeners.value.get("pointerdown"))({[$$bc$property_key($$bc$keyword("clientX"))]: 110.0, [$$bc$property_key($$bc$keyword("clientY"))]: 70.0, [$$bc$property_key($$bc$keyword("pointerId"))]: 9, [$$bc$property_key($$bc$keyword("button"))]: 0, [$$bc$property_key($$bc$keyword("buttons"))]: 1});
test.expect(JSON.stringify(frame)).toBe(before);
test.expect(frame.player.position).toEqual({[$$bc$property_key($$bc$keyword("x"))]: 2.0, [$$bc$property_key($$bc$keyword("y"))]: 3.0, [$$bc$property_key($$bc$keyword("z"))]: 4.0});
test.expect(frame.player.velocity).toEqual({[$$bc$property_key($$bc$keyword("x"))]: 5.0, [$$bc$property_key($$bc$keyword("y"))]: 6.0, [$$bc$property_key($$bc$keyword("z"))]: 7.0});
test.expect(frame.player.grounded).toBe(false);
test.expect(render_count.value).toBe(1);
test.expect(emitted_inputs.value.length).toBe(2);
test.expect(pixel_ratio.value).toBe(2.0);
(arena.dispose)();
(arena.dispose)();
test.expect(browser_listeners.value.size).toBe(0);
test.expect(canvas_listeners.value.size).toBe(0);
test.expect(released_pointers.value).toBe(1);
test.expect(resource_disposals.value).toBe(4);
test.expect(renderer_disposals.value).toBe(1);
test.expect(context_losses.value).toBe(1);
return test.expect(canvas_removals.value).toBe(1); });
//# sourceMappingURL=shell-boundary-test.js.map
