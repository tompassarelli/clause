import { expect, test } from "bun:test";
import * as shell from "./shell.js";
class EventRegistry {
    listeners = new Map();
    add(type, listener) {
        this.listeners.set(type, listener);
    }
    remove(type, _listener) {
        this.listeners.delete(type);
    }
    require(type) {
        const listener = this.listeners.get(type);
        if (typeof listener !== "function") {
            throw new Error(`missing ${type} listener`);
        }
        return (event) => Reflect.apply(listener, undefined, [event]);
    }
    has(type) {
        return this.listeners.has(type);
    }
    clear() {
        this.listeners.clear();
    }
    get size() {
        return this.listeners.size;
    }
}
const browserListeners = new Map();
const canvasListeners = new EventRegistry();
const pointerCaptures = new Set();
const emittedInputs = [];
const materialColors = [];
let renderCount = 0;
let resourceDisposals = 0;
let rendererDisposals = 0;
let contextLosses = 0;
let canvasRemovals = 0;
let releasedPointers = 0;
let focusRequests = 0;
let pixelRatio = 0;
class FakeVector {
    x = 0;
    y = 0;
    z = 0;
    set(x, y, z) {
        this.x = x;
        this.y = y;
        this.z = z;
    }
}
class FakeNode {
    position = new FakeVector();
    rotation = { x: 0, y: 0, z: 0 };
    scale = new FakeVector();
    add(_child) { }
    remove(_child) { }
    clear() { }
    lookAt(_x, _y, _z) { }
    updateProjectionMatrix() { }
}
class FakeScene extends FakeNode {
    background = null;
}
class FakeColor {
    color;
    constructor(color) {
        this.color = color;
    }
}
class FakeCamera extends FakeNode {
    fov;
    aspect;
    near;
    far;
    constructor(fov, aspect, near, far) {
        super();
        this.fov = fov;
        this.aspect = aspect;
        this.near = near;
        this.far = far;
    }
}
class FakeLight extends FakeNode {
    material = { dispose() { } };
    constructor(..._arguments) {
        super();
    }
}
class FakeResource {
    dispose() {
        resourceDisposals += 1;
    }
}
class FakeGeometry extends FakeResource {
    width;
    height;
    depth;
    constructor(width, height, depth) {
        super();
        this.width = width;
        this.height = height;
        this.depth = depth;
    }
}
class FakeMaterial extends FakeResource {
    options;
    constructor(options) {
        super();
        this.options = options;
        materialColors.push(options.color);
    }
}
class FakeMesh extends FakeNode {
    geometry;
    material;
    constructor(geometry, material) {
        super();
        this.geometry = geometry;
        this.material = material;
    }
}
class FakeGroup extends FakeNode {
}
class FakeCanvas {
    addEventListener(type, listener) {
        canvasListeners.add(type, listener);
    }
    removeEventListener(type, listener) {
        canvasListeners.remove(type, listener);
    }
    getBoundingClientRect() {
        return { left: 10, top: 20, width: 200, height: 100 };
    }
    setPointerCapture(pointerId) {
        pointerCaptures.add(pointerId);
    }
    hasPointerCapture(pointerId) {
        return pointerCaptures.has(pointerId);
    }
    releasePointerCapture(pointerId) {
        pointerCaptures.delete(pointerId);
        releasedPointers += 1;
    }
    focus(_options) {
        focusRequests += 1;
        canvasListeners.require("focus")({});
    }
    setAttribute(_name, _value) { }
    remove() {
        canvasRemovals += 1;
    }
}
class FakeRenderer {
    options;
    domElement = new FakeCanvas();
    constructor(options) {
        this.options = options;
    }
    setPixelRatio(ratio) {
        pixelRatio = ratio;
    }
    setSize(_width, _height, _updateStyle) { }
    render(_scene, _camera) {
        renderCount += 1;
    }
    dispose() {
        rendererDisposals += 1;
    }
    forceContextLoss() {
        contextLosses += 1;
    }
}
class FakeBrowser {
    devicePixelRatio = 4;
    addEventListener(type, listener) {
        browserListeners.set(type, listener);
    }
    removeEventListener(type, _listener) {
        browserListeners.delete(type);
    }
}
class FakeMount {
    clientWidth = 640;
    clientHeight = 360;
    appendChild(_canvas) { }
}
function frozenVec3(x, y, z) {
    return Object.freeze({ _tag: "ArenaVec3", x, y, z });
}
function frozenPlatform() {
    return Object.freeze({
        _tag: "ArenaPlatformFrame",
        position: frozenVec3(0, -0.25, 0),
        size: frozenVec3(12, 0.5, 12),
    });
}
function frozenCollectible(state) {
    return Object.freeze({
        _tag: "ArenaCollectibleFrame",
        position: frozenVec3(2, 3, 4),
        state,
    });
}
function sampleFrame(state) {
    return Object.freeze({
        _tag: "ArenaFrame",
        player: Object.freeze({
            _tag: "ArenaPlayerFrame",
            position: frozenVec3(2, 3, 4),
            velocity: frozenVec3(5, 6, 7),
            yaw: 0.5,
            grounded: false,
        }),
        world: Object.freeze({
            _tag: "ArenaWorldFrame",
            platforms: Object.freeze([frozenPlatform()]),
            collectibles: Object.freeze([frozenCollectible(state)]),
        }),
    });
}
function requireBrowserListener(type) {
    const listener = browserListeners.get(type);
    if (listener === undefined) {
        throw new Error(`missing ${type} listener`);
    }
    return listener;
}
function resetFixture() {
    browserListeners.clear();
    canvasListeners.clear();
    pointerCaptures.clear();
    emittedInputs.length = 0;
    materialColors.length = 0;
    renderCount = 0;
    resourceDisposals = 0;
    rendererDisposals = 0;
    contextLosses = 0;
    canvasRemovals = 0;
    releasedPointers = 0;
    focusRequests = 0;
    pixelRatio = 0;
}
test("production shell emits input without advancing player state and tears down", () => {
    resetFixture();
    const three = {
        Scene: FakeScene,
        Color: FakeColor,
        PerspectiveCamera: FakeCamera,
        WebGLRenderer: FakeRenderer,
        HemisphereLight: FakeLight,
        DirectionalLight: FakeLight,
        BoxGeometry: FakeGeometry,
        MeshStandardMaterial: FakeMaterial,
        Mesh: FakeMesh,
        Group: FakeGroup,
    };
    const arena = shell["create-jump-arena-shell!"](new FakeMount(), new FakeBrowser(), three, (input) => {
        emittedInputs.push(input);
    });
    const frame = sampleFrame("active");
    const collectedFrame = sampleFrame("collected");
    const before = JSON.stringify(frame);
    const resizeHandler = requireBrowserListener("resize");
    const focusHandler = canvasListeners.require("focus");
    const blurHandler = canvasListeners.require("blur");
    const keyDown = canvasListeners.require("keydown");
    const keyUp = canvasListeners.require("keyup");
    const pointerDown = canvasListeners.require("pointerdown");
    const pointerMove = canvasListeners.require("pointermove");
    const pointerUp = canvasListeners.require("pointerup");
    const pointerCancel = canvasListeners.require("pointercancel");
    arena.renderFrame(frame);
    const activeColor = materialColors[2];
    arena.renderFrame(collectedFrame);
    expect(Object.is(activeColor, materialColors[3])).toBe(false);
    keyDown({ code: "Space", repeat: false });
    expect(emittedInputs).toHaveLength(0);
    focusHandler({});
    keyDown({ code: "Space", repeat: false });
    expect(emittedInputs).toHaveLength(1);
    blurHandler({});
    keyDown({ code: "Space", repeat: false });
    expect(emittedInputs).toHaveLength(1);
    pointerDown({
        clientX: 110,
        clientY: 70,
        pointerId: 9,
        button: 0,
        buttons: 1,
    });
    expect(JSON.stringify(frame)).toBe(before);
    expect(frame.player.position).toEqual(frozenVec3(2, 3, 4));
    expect(frame.player.velocity).toEqual(frozenVec3(5, 6, 7));
    expect(frame.player.grounded).toBe(false);
    expect(renderCount).toBe(2);
    expect(emittedInputs).toHaveLength(2);
    expect(pixelRatio).toBe(2);
    expect(browserListeners.has("keydown")).toBe(false);
    expect(browserListeners.has("keyup")).toBe(false);
    expect(focusRequests).toBe(1);
    arena.dispose();
    arena.dispose();
    keyDown({ code: "KeyW", repeat: false });
    focusHandler({});
    blurHandler({});
    keyDown({ code: "KeyW", repeat: false });
    keyUp({ code: "KeyW", repeat: false });
    pointerDown({
        clientX: 110,
        clientY: 70,
        pointerId: 10,
        button: 0,
        buttons: 1,
    });
    pointerMove({
        clientX: 111,
        clientY: 71,
        pointerId: 9,
        button: 0,
        buttons: 1,
    });
    pointerUp({
        clientX: 111,
        clientY: 71,
        pointerId: 9,
        button: 0,
        buttons: 0,
    });
    pointerCancel({
        clientX: 111,
        clientY: 71,
        pointerId: 9,
        button: 0,
        buttons: 0,
    });
    resizeHandler();
    expect(() => arena.renderFrame(frame)).toThrow("jump arena shell is disposed");
    expect(browserListeners.size).toBe(0);
    expect(canvasListeners.size).toBe(0);
    expect(pointerCaptures.size).toBe(0);
    expect(emittedInputs).toHaveLength(2);
    expect(focusRequests).toBe(1);
    expect(renderCount).toBe(2);
    expect(releasedPointers).toBe(1);
    expect(resourceDisposals).toBe(7);
    expect(rendererDisposals).toBe(1);
    expect(contextLosses).toBe(1);
    expect(canvasRemovals).toBe(1);
});
//# sourceMappingURL=shell-boundary-test.js.map