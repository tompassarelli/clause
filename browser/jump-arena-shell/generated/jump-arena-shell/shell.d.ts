export interface ArenaVec3 {
    readonly x: number;
    readonly y: number;
    readonly z: number;
}
export interface ArenaPlayerFrame {
    readonly position: ArenaVec3;
    readonly velocity: ArenaVec3;
    readonly yaw: number;
    readonly grounded: boolean;
}
export interface ArenaPlatformFrame {
    readonly position: ArenaVec3;
    readonly size: ArenaVec3;
}
export interface ArenaCollectibleFrame {
    readonly position: ArenaVec3;
    readonly state: string;
}
export interface ArenaWorldFrame {
    readonly platforms: readonly ArenaPlatformFrame[];
    readonly collectibles: readonly ArenaCollectibleFrame[];
}
export interface ArenaFrame {
    readonly player: ArenaPlayerFrame;
    readonly world: ArenaWorldFrame;
}
export type ArenaInput = Readonly<{
    kind: "keyboard";
    phase: "down" | "up";
    code: string;
    repeat: boolean;
}> | Readonly<{
    kind: "pointer";
    phase: "down" | "move" | "up" | "cancel";
    pointerId: number;
    button: number;
    buttons: number;
    x: number;
    y: number;
}>;
interface RectLike {
    readonly left: number;
    readonly top: number;
    readonly width: number;
    readonly height: number;
}
export interface ArenaCanvas {
    getBoundingClientRect(): RectLike;
    setAttribute(name: string, value: string): void;
    focus(options: {
        readonly preventScroll: boolean;
    }): void;
    setPointerCapture(pointerId: number): void;
    hasPointerCapture(pointerId: number): boolean;
    releasePointerCapture(pointerId: number): void;
    addEventListener<TEvent>(type: string, listener: (event: TEvent) => unknown): void;
    removeEventListener<TEvent>(type: string, listener: (event: TEvent) => unknown): void;
    remove(): void;
}
export interface JumpArenaShell {
    readonly canvas: ArenaCanvas;
    readonly renderFrame: (frame: unknown) => ArenaFrame;
    readonly dispose: () => void;
}
declare function create_jump_arena_shell_bang(mount_value: unknown, browser_value: unknown, three_value: unknown, emit_input: (input: ArenaInput) => unknown): JumpArenaShell;
export { create_jump_arena_shell_bang as "create-jump-arena-shell!" };
