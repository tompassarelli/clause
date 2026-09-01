import { expect, test } from "bun:test";

import * as shell from "./shell.js";

interface Disposable {
  dispose(): void;
}

interface KeyboardEventLike {
  readonly code: string;
  readonly repeat: boolean;
}

interface PointerEventLike {
  readonly clientX: number;
  readonly clientY: number;
  readonly pointerId: number;
  readonly button: number;
  readonly buttons: number;
}

type EmptyEvent = Readonly<Record<string, never>>;

class EventRegistry {
  private readonly listeners = new Map<string, unknown>();

  add<TEvent>(type: string, listener: (event: TEvent) => unknown): void {
    this.listeners.set(type, listener);
  }

  remove<TEvent>(type: string, _listener: (event: TEvent) => unknown): void {
    this.listeners.delete(type);
  }

  require<TEvent>(type: string): (event: TEvent) => unknown {
    const listener = this.listeners.get(type);
    if (typeof listener !== "function") {
      throw new Error(`missing ${type} listener`);
    }
    return (event: TEvent): unknown =>
      Reflect.apply(listener, undefined, [event]);
  }

  has(type: string): boolean {
    return this.listeners.has(type);
  }

  clear(): void {
    this.listeners.clear();
  }

  get size(): number {
    return this.listeners.size;
  }
}

const browserListeners = new Map<string, () => unknown>();
const canvasListeners = new EventRegistry();
const pointerCaptures = new Set<number>();
const emittedInputs: shell.ArenaInput[] = [];
const materialColors: number[] = [];

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

  set(x: number, y: number, z: number): void {
    this.x = x;
    this.y = y;
    this.z = z;
  }
}

class FakeNode {
  readonly position = new FakeVector();
  readonly rotation = { x: 0, y: 0, z: 0 };
  readonly scale = new FakeVector();

  add(_child: unknown): void {}

  remove(_child: unknown): void {}

  clear(): void {}

  lookAt(_x: number, _y: number, _z: number): void {}

  updateProjectionMatrix(): void {}
}

class FakeScene extends FakeNode {
  background: unknown = null;
}

class FakeColor {
  constructor(readonly color: number) {}
}

class FakeCamera extends FakeNode {
  constructor(
    readonly fov: number,
    public aspect: number,
    readonly near: number,
    readonly far: number,
  ) {
    super();
  }
}

class FakeLight extends FakeNode {
  readonly material: Disposable = { dispose(): void {} };

  constructor(..._arguments: unknown[]) {
    super();
  }
}

class FakeResource implements Disposable {
  dispose(): void {
    resourceDisposals += 1;
  }
}

class FakeGeometry extends FakeResource {
  constructor(
    readonly width: number,
    readonly height: number,
    readonly depth: number,
  ) {
    super();
  }
}

class FakeMaterial extends FakeResource {
  constructor(
    readonly options: Readonly<{ color: number; roughness: number }>,
  ) {
    super();
    materialColors.push(options.color);
  }
}

class FakeMesh extends FakeNode {
  constructor(
    readonly geometry: Disposable,
    readonly material: Disposable,
  ) {
    super();
  }
}

class FakeGroup extends FakeNode {}

class FakeCanvas {
  addEventListener<TEvent>(
    type: string,
    listener: (event: TEvent) => unknown,
  ): void {
    canvasListeners.add(type, listener);
  }

  removeEventListener<TEvent>(
    type: string,
    listener: (event: TEvent) => unknown,
  ): void {
    canvasListeners.remove(type, listener);
  }

  getBoundingClientRect(): Readonly<{
    left: number;
    top: number;
    width: number;
    height: number;
  }> {
    return { left: 10, top: 20, width: 200, height: 100 };
  }

  setPointerCapture(pointerId: number): void {
    pointerCaptures.add(pointerId);
  }

  hasPointerCapture(pointerId: number): boolean {
    return pointerCaptures.has(pointerId);
  }

  releasePointerCapture(pointerId: number): void {
    pointerCaptures.delete(pointerId);
    releasedPointers += 1;
  }

  focus(_options: Readonly<{ preventScroll: boolean }>): void {
    focusRequests += 1;
    canvasListeners.require<EmptyEvent>("focus")({});
  }

  setAttribute(_name: string, _value: string): void {}

  remove(): void {
    canvasRemovals += 1;
  }
}

class FakeRenderer {
  readonly domElement = new FakeCanvas();

  constructor(readonly options: Readonly<{ antialias: boolean }>) {}

  setPixelRatio(ratio: number): void {
    pixelRatio = ratio;
  }

  setSize(_width: number, _height: number, _updateStyle: boolean): void {}

  render(_scene: unknown, _camera: unknown): void {
    renderCount += 1;
  }

  dispose(): void {
    rendererDisposals += 1;
  }

  forceContextLoss(): void {
    contextLosses += 1;
  }
}

class FakeBrowser {
  readonly devicePixelRatio = 4;

  addEventListener(type: string, listener: () => unknown): void {
    browserListeners.set(type, listener);
  }

  removeEventListener(type: string, _listener: () => unknown): void {
    browserListeners.delete(type);
  }
}

class FakeMount {
  readonly clientWidth = 640;
  readonly clientHeight = 360;

  appendChild(_canvas: unknown): void {}
}

function frozenVec3(x: number, y: number, z: number): shell.ArenaVec3 {
  return Object.freeze({ _tag: "ArenaVec3", x, y, z });
}

function frozenPlatform(): shell.ArenaPlatformFrame {
  return Object.freeze({
    _tag: "ArenaPlatformFrame",
    position: frozenVec3(0, -0.25, 0),
    size: frozenVec3(12, 0.5, 12),
  });
}

function frozenCollectible(state: string): shell.ArenaCollectibleFrame {
  return Object.freeze({
    _tag: "ArenaCollectibleFrame",
    position: frozenVec3(2, 3, 4),
    state,
  });
}

function sampleFrame(state: string): shell.ArenaFrame {
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

function requireBrowserListener(type: string): () => unknown {
  const listener = browserListeners.get(type);
  if (listener === undefined) {
    throw new Error(`missing ${type} listener`);
  }
  return listener;
}

function resetFixture(): void {
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

test(
  "production shell emits input without advancing player state and tears down",
  () => {
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
    const arena = shell["create-jump-arena-shell!"](
      new FakeMount(),
      new FakeBrowser(),
      three,
      (input) => {
        emittedInputs.push(input);
      },
    );
    const frame = sampleFrame("active");
    const collectedFrame = sampleFrame("collected");
    const before = JSON.stringify(frame);
    const resizeHandler = requireBrowserListener("resize");
    const focusHandler = canvasListeners.require<EmptyEvent>("focus");
    const blurHandler = canvasListeners.require<EmptyEvent>("blur");
    const keyDown = canvasListeners.require<KeyboardEventLike>("keydown");
    const keyUp = canvasListeners.require<KeyboardEventLike>("keyup");
    const pointerDown =
      canvasListeners.require<PointerEventLike>("pointerdown");
    const pointerMove =
      canvasListeners.require<PointerEventLike>("pointermove");
    const pointerUp = canvasListeners.require<PointerEventLike>("pointerup");
    const pointerCancel =
      canvasListeners.require<PointerEventLike>("pointercancel");

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

    expect(() => arena.renderFrame(frame)).toThrow(
      "jump arena shell is disposed",
    );
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
  },
);
