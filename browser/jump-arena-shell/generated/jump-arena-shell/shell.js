import { keyword as $$bc$keyword, property_key as $$bc$property_key, record_value as $$bc$record_value, str as $$bc$str } from 'beagle/core.js';

function ArenaVec3(x, y, z) {
  return $$bc$record_value("jump-arena-shell.shell/ArenaVec3", {_tag: "ArenaVec3", x, y, z});
}

function arenavec3_x(r) { return r.x; }

function arenavec3_y(r) { return r.y; }

function arenavec3_z(r) { return r.z; }

function ArenaPlayerFrame(position, velocity, yaw, grounded) {
  return $$bc$record_value("jump-arena-shell.shell/ArenaPlayerFrame", {_tag: "ArenaPlayerFrame", position, velocity, yaw, grounded});
}

function arenaplayerframe_position(r) { return r.position; }

function arenaplayerframe_velocity(r) { return r.velocity; }

function arenaplayerframe_yaw(r) { return r.yaw; }

function arenaplayerframe_grounded(r) { return r.grounded; }

function ArenaPlatformFrame(position, size) {
  return $$bc$record_value("jump-arena-shell.shell/ArenaPlatformFrame", {_tag: "ArenaPlatformFrame", position, size});
}

function arenaplatformframe_position(r) { return r.position; }

function arenaplatformframe_size(r) { return r.size; }

function ArenaWorldFrame(platforms) {
  return $$bc$record_value("jump-arena-shell.shell/ArenaWorldFrame", {_tag: "ArenaWorldFrame", platforms});
}

function arenaworldframe_platforms(r) { return r.platforms; }

function ArenaFrame(player, world) {
  return $$bc$record_value("jump-arena-shell.shell/ArenaFrame", {_tag: "ArenaFrame", player, world});
}

function arenaframe_player(r) { return r.player; }

function arenaframe_world(r) { return r.world; }

function ArenaPointerEvent(clientX, clientY, pointerId, button, buttons) {
  return $$bc$record_value("jump-arena-shell.shell/ArenaPointerEvent", {_tag: "ArenaPointerEvent", clientX, clientY, pointerId, button, buttons});
}

function arenapointerevent_clientX(r) { return r.clientX; }

function arenapointerevent_clientY(r) { return r.clientY; }

function arenapointerevent_pointerId(r) { return r.pointerId; }

function arenapointerevent_button(r) { return r.button; }

function arenapointerevent_buttons(r) { return r.buttons; }

function numeric_value(value) {
  return Number.parseFloat($$bc$str(value));
}

function require_frozen_frame(frame) {
  return (((_truthy) => _truthy !== false && _truthy != null)(((_logical) => (_logical !== false && _logical != null ? ((_logical) => (_logical !== false && _logical != null ? ((_logical) => (_logical !== false && _logical != null ? ((_logical) => (_logical !== false && _logical != null ? ((_logical) => (_logical !== false && _logical != null ? ((_logical) => (_logical !== false && _logical != null ? frame.world.platforms.every((platform) => ((_logical) => (_logical !== false && _logical != null ? ((_logical) => (_logical !== false && _logical != null ? Object.isFrozen(platform.size) : _logical))(Object.isFrozen(platform.position)) : _logical))(Object.isFrozen(platform))) : _logical))(Object.isFrozen(frame.world.platforms)) : _logical))(Object.isFrozen(frame.world)) : _logical))(Object.isFrozen(frame.player.velocity)) : _logical))(Object.isFrozen(frame.player.position)) : _logical))(Object.isFrozen(frame.player)) : _logical))(Object.isFrozen(frame))) ? frame : (() => { throw new Error("renderFrame requires a deeply frozen arena frame"); })());
}

function keyboard_input(phase, event) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("kind"))]: "keyboard", [$$bc$property_key($$bc$keyword("phase"))]: phase, [$$bc$property_key($$bc$keyword("code"))]: event.code, [$$bc$property_key($$bc$keyword("repeat"))]: event.repeat});
}

function pointer_input(phase, event, canvas) {
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1.0, rect.width);
  const height = Math.max(1.0, rect.height);
  const x = ((((event.clientX - rect.left) * 2.0) / width) - 1.0);
  const y = (1.0 - (((event.clientY - rect.top) * 2.0) / height));
  return Object.freeze({[$$bc$property_key($$bc$keyword("kind"))]: "pointer", [$$bc$property_key($$bc$keyword("phase"))]: phase, [$$bc$property_key($$bc$keyword("pointerId"))]: event.pointerId, [$$bc$property_key($$bc$keyword("button"))]: event.button, [$$bc$property_key($$bc$keyword("buttons"))]: event.buttons, [$$bc$property_key($$bc$keyword("x"))]: x, [$$bc$property_key($$bc$keyword("y"))]: y});
}

function create_jump_arena_shell_bang(mount, browser, three, emit_input) {
  const scene_ctor = three.Scene;
  const color_ctor = three.Color;
  const camera_ctor = three.PerspectiveCamera;
  const renderer_ctor = three.WebGLRenderer;
  const hemisphere_light_ctor = three.HemisphereLight;
  const directional_light_ctor = three.DirectionalLight;
  const box_geometry_ctor = three.BoxGeometry;
  const material_ctor = three.MeshStandardMaterial;
  const mesh_ctor = three.Mesh;
  const group_ctor = three.Group;
  const scene = new scene_ctor();
  const camera = new camera_ctor(52.0, 1.0, 0.1, 160.0);
  const renderer = new renderer_ctor({[$$bc$property_key($$bc$keyword("antialias"))]: true});
  const canvas = renderer.domElement;
  const player_geometry = new box_geometry_ctor(0.8, 1.8, 0.8);
  const player_material = new material_ctor({[$$bc$property_key($$bc$keyword("color"))]: 4697343, [$$bc$property_key($$bc$keyword("roughness"))]: 0.42});
  const platform_geometry = new box_geometry_ctor(1.0, 1.0, 1.0);
  const platform_material = new material_ctor({[$$bc$property_key($$bc$keyword("color"))]: 2967637, [$$bc$property_key($$bc$keyword("roughness"))]: 0.78});
  const player_mesh = new mesh_ctor(player_geometry, player_material);
  const platform_group = new group_ctor();
  const disposed = ({value: false, watches: {}});
  const canvas_focused = ({value: false, watches: {}});
  const has_frame = ({value: false, watches: {}});
  const platform_meshes = ({value: [], watches: {}});
  const active_pointers = new Set();
  (scene.background = new color_ctor(1116716));
  scene.add(new hemisphere_light_ctor(10144255, 1250849, 2.4));
  const sun = new directional_light_ctor(16777215, 3.2);
  sun.position.set(6.0, 12.0, 8.0);
  scene.add(sun);
  scene.add(platform_group);
  scene.add(player_mesh);
  canvas.setAttribute("tabindex", "0");
  mount.appendChild(canvas);
  const resize = () => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  const width = Math.max(1.0, numeric_value(mount.clientWidth));
  const height = Math.max(1.0, numeric_value(mount.clientHeight));
  const ratio = Math.min(2.0, numeric_value(browser.devicePixelRatio));
  renderer.setPixelRatio(ratio);
  renderer.setSize(width, height, false);
  (camera.aspect = (width / height));
  camera.updateProjectionMatrix();
  if (((_truthy) => _truthy !== false && _truthy != null)(has_frame.value)) {
    return renderer.render(scene, camera);
  }
} };
  const focus_canvas = () => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  return (() => { const _a = canvas_focused, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
} };
  const blur_canvas = () => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  return (() => { const _a = canvas_focused, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
} };
  const key_down = (event) => { if (((_truthy) => _truthy !== false && _truthy != null)(((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value)) && canvas_focused.value))) {
  return emit_input(keyboard_input("down", event));
} };
  const key_up = (event) => { if (((_truthy) => _truthy !== false && _truthy != null)(((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value)) && canvas_focused.value))) {
  return emit_input(keyboard_input("up", event));
} };
  const pointer_event = (phase, event) => emit_input(pointer_input(phase, event, canvas));
  const pointer_down = (event) => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  canvas.focus({[$$bc$property_key($$bc$keyword("preventScroll"))]: true});
  active_pointers.add(event.pointerId);
  canvas.setPointerCapture(event.pointerId);
  return pointer_event("down", event);
} };
  const pointer_move = (event) => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  return pointer_event("move", event);
} };
  const pointer_up = (event) => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  if (((_truthy) => _truthy !== false && _truthy != null)(canvas.hasPointerCapture(event.pointerId))) {
    canvas.releasePointerCapture(event.pointerId);
  }
  active_pointers.delete(event.pointerId);
  return pointer_event("up", event);
} };
  const pointer_cancel = (event) => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  if (((_truthy) => _truthy !== false && _truthy != null)(canvas.hasPointerCapture(event.pointerId))) {
    canvas.releasePointerCapture(event.pointerId);
  }
  active_pointers.delete(event.pointerId);
  return pointer_event("cancel", event);
} };
  const render_frame = (incoming) => { if (((_truthy) => _truthy !== false && _truthy != null)(disposed.value)) {
  return (() => { throw new Error("jump arena shell is disposed"); })();
} else {
  const frame = require_frozen_frame(incoming);
  const player = frame.player;
  const position = player.position;
  const yaw = player.yaw;
  player_mesh.position.set(position.x, position.y, position.z);
  (player_mesh.rotation.y = yaw);
  (() => { platform_meshes.value.forEach((mesh) => {
  platform_group.remove(mesh);
}); })();
  (() => { const _a = platform_meshes, _v = []; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { frame.world.platforms.forEach((platform) => {
  const mesh = new mesh_ctor(platform_geometry, platform_material);
  const platform_position = platform.position;
  const size = platform.size;
  mesh.position.set(platform_position.x, platform_position.y, platform_position.z);
  mesh.scale.set(size.x, size.y, size.z);
  platform_group.add(mesh);
  platform_meshes.value.push(mesh);
}); })();
  camera.position.set((position.x + (Math.sin(yaw) * 7.5)), (position.y + 4.8), (position.z + (Math.cos(yaw) * 7.5)));
  camera.lookAt(position.x, (position.y + 1.0), position.z);
  renderer.render(scene, camera);
  (() => { const _a = has_frame, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  return frame;
} };
  const dispose = () => { if ((!((_truthy) => _truthy !== false && _truthy != null)(disposed.value))) {
  (() => { const _a = disposed, _v = true; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = canvas_focused, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  (() => { const _a = has_frame, _v = false; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  browser.removeEventListener("resize", resize);
  canvas.removeEventListener("focus", focus_canvas);
  canvas.removeEventListener("blur", blur_canvas);
  canvas.removeEventListener("keydown", key_down);
  canvas.removeEventListener("keyup", key_up);
  canvas.removeEventListener("pointerdown", pointer_down);
  canvas.removeEventListener("pointermove", pointer_move);
  canvas.removeEventListener("pointerup", pointer_up);
  canvas.removeEventListener("pointercancel", pointer_cancel);
  active_pointers.forEach((pointer_id) => { if (((_truthy) => _truthy !== false && _truthy != null)(canvas.hasPointerCapture(pointer_id))) {
  return canvas.releasePointerCapture(pointer_id);
} });
  active_pointers.clear();
  (() => { const _a = platform_meshes, _v = []; const _old = _a.value; _a.value = _v; for (const _k in _a.watches) _a.watches[_k](_k, _a, _old, _v); return _v; })();
  platform_group.clear();
  player_geometry.dispose();
  player_material.dispose();
  platform_geometry.dispose();
  platform_material.dispose();
  scene.clear();
  renderer.dispose();
  renderer.forceContextLoss();
  return canvas.remove();
} };
  browser.addEventListener("resize", resize);
  canvas.addEventListener("focus", focus_canvas);
  canvas.addEventListener("blur", blur_canvas);
  canvas.addEventListener("keydown", key_down);
  canvas.addEventListener("keyup", key_up);
  canvas.addEventListener("pointerdown", pointer_down);
  canvas.addEventListener("pointermove", pointer_move);
  canvas.addEventListener("pointerup", pointer_up);
  canvas.addEventListener("pointercancel", pointer_cancel);
  resize();
  return Object.freeze({[$$bc$property_key($$bc$keyword("canvas"))]: canvas, [$$bc$property_key($$bc$keyword("renderFrame"))]: render_frame, [$$bc$property_key($$bc$keyword("dispose"))]: dispose});
}

export { create_jump_arena_shell_bang as "create-jump-arena-shell!" };
//# sourceMappingURL=shell.js.map
