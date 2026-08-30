import * as shell from './shell.js';
import * as THREE from 'three';
import { keyword as $$bc$keyword, property_key as $$bc$property_key } from 'beagle/core.js';

function frozen_vec3(x, y, z) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("x"))]: x, [$$bc$property_key($$bc$keyword("y"))]: y, [$$bc$property_key($$bc$keyword("z"))]: z});
}

function frozen_platform(x, y, z, width, height, depth) {
  return Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(x, y, z), [$$bc$property_key($$bc$keyword("size"))]: frozen_vec3(width, height, depth)});
}

const sample_frame = Object.freeze({[$$bc$property_key($$bc$keyword("player"))]: Object.freeze({[$$bc$property_key($$bc$keyword("position"))]: frozen_vec3(0.0, 1.15, 0.0), [$$bc$property_key($$bc$keyword("velocity"))]: frozen_vec3(0.0, 0.0, 0.0), [$$bc$property_key($$bc$keyword("yaw"))]: 0.0, [$$bc$property_key($$bc$keyword("grounded"))]: true}), [$$bc$property_key($$bc$keyword("world"))]: Object.freeze({[$$bc$property_key($$bc$keyword("platforms"))]: Object.freeze([frozen_platform(0.0, -0.25, 0.0, 12.0, 0.5, 12.0), frozen_platform(4.0, 1.0, -2.0, 3.0, 0.5, 3.0), frozen_platform(-3.0, 2.2, -5.0, 2.5, 0.5, 2.5)])})});

const mount = document.querySelector("#arena");

const three = {[$$bc$property_key($$bc$keyword("Scene"))]: THREE.Scene, [$$bc$property_key($$bc$keyword("Color"))]: THREE.Color, [$$bc$property_key($$bc$keyword("PerspectiveCamera"))]: THREE.PerspectiveCamera, [$$bc$property_key($$bc$keyword("WebGLRenderer"))]: THREE.WebGLRenderer, [$$bc$property_key($$bc$keyword("HemisphereLight"))]: THREE.HemisphereLight, [$$bc$property_key($$bc$keyword("DirectionalLight"))]: THREE.DirectionalLight, [$$bc$property_key($$bc$keyword("BoxGeometry"))]: THREE.BoxGeometry, [$$bc$property_key($$bc$keyword("MeshStandardMaterial"))]: THREE.MeshStandardMaterial, [$$bc$property_key($$bc$keyword("Mesh"))]: THREE.Mesh, [$$bc$property_key($$bc$keyword("Group"))]: THREE.Group};

const arena = shell["create-jump-arena-shell!"](mount, window, three, (input) => mount.dispatchEvent(new CustomEvent("clause-input", {[$$bc$property_key($$bc$keyword("detail"))]: input})));

(arena.renderFrame)(sample_frame);

window.addEventListener("beforeunload", arena.dispose, {[$$bc$property_key($$bc$keyword("once"))]: true});
//# sourceMappingURL=demo.js.map
