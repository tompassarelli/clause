function is_property_bag(value) {
    return ((typeof value === "object" && value !== null) ||
        typeof value === "function");
}
function is_frozen_vec3(value) {
    return (is_property_bag(value) &&
        Object.isFrozen(value) &&
        typeof value.x === "number" &&
        typeof value.y === "number" &&
        typeof value.z === "number");
}
function is_frozen_player_frame(value) {
    return (is_property_bag(value) &&
        Object.isFrozen(value) &&
        is_frozen_vec3(value.position) &&
        is_frozen_vec3(value.velocity) &&
        typeof value.yaw === "number" &&
        typeof value.grounded === "boolean");
}
function is_frozen_platform_frame(value) {
    return (is_property_bag(value) &&
        Object.isFrozen(value) &&
        is_frozen_vec3(value.position) &&
        is_frozen_vec3(value.size));
}
function is_frozen_collectible_frame(value) {
    return (is_property_bag(value) &&
        Object.isFrozen(value) &&
        is_frozen_vec3(value.position) &&
        typeof value.state === "string");
}
function is_frozen_world_frame(value) {
    if (!is_property_bag(value) || !Object.isFrozen(value))
        return false;
    const { platforms, collectibles } = value;
    return (Array.isArray(platforms) &&
        Object.isFrozen(platforms) &&
        platforms.every(is_frozen_platform_frame) &&
        Array.isArray(collectibles) &&
        Object.isFrozen(collectibles) &&
        collectibles.every(is_frozen_collectible_frame));
}
function is_frozen_frame(value) {
    return (is_property_bag(value) &&
        Object.isFrozen(value) &&
        is_frozen_player_frame(value.player) &&
        is_frozen_world_frame(value.world));
}
function require_frozen_frame(value) {
    if (is_frozen_frame(value))
        return value;
    throw new Error("renderFrame requires a deeply frozen arena frame");
}
function is_mount_like(value) {
    return (is_property_bag(value) &&
        "clientWidth" in value &&
        "clientHeight" in value &&
        typeof value.appendChild === "function");
}
function require_mount(value) {
    if (is_mount_like(value))
        return value;
    throw new Error("jump arena shell requires a mount");
}
function is_browser_like(value) {
    return (is_property_bag(value) &&
        "devicePixelRatio" in value &&
        typeof value.addEventListener === "function" &&
        typeof value.removeEventListener === "function");
}
function require_browser(value) {
    if (is_browser_like(value))
        return value;
    throw new Error("jump arena shell requires a browser host");
}
function is_three_like(value) {
    return (is_property_bag(value) &&
        typeof value.Scene === "function" &&
        typeof value.Color === "function" &&
        typeof value.PerspectiveCamera === "function" &&
        typeof value.WebGLRenderer === "function" &&
        typeof value.HemisphereLight === "function" &&
        typeof value.DirectionalLight === "function" &&
        typeof value.BoxGeometry === "function" &&
        typeof value.MeshStandardMaterial === "function" &&
        typeof value.Mesh === "function" &&
        typeof value.Group === "function");
}
function require_three(value) {
    if (is_three_like(value))
        return value;
    throw new Error("jump arena shell requires a Three.js host");
}
function numeric_value(value) {
    return Number.parseFloat(String(value));
}
function projected_symbol_color(value) {
    let hash = 216613;
    for (let index = 0; index < value.length; index += 1) {
        hash = (hash * 33 + value.charCodeAt(index)) % 13619151;
    }
    return 2105376 + (hash % 13619151);
}
function keyboard_input(phase, event) {
    return Object.freeze({
        kind: "keyboard",
        phase,
        code: event.code,
        repeat: event.repeat,
    });
}
function pointer_input(phase, event, canvas) {
    const rect = canvas.getBoundingClientRect();
    const width = Math.max(1, rect.width);
    const height = Math.max(1, rect.height);
    return Object.freeze({
        kind: "pointer",
        phase,
        pointerId: event.pointerId,
        button: event.button,
        buttons: event.buttons,
        x: ((event.clientX - rect.left) * 2) / width - 1,
        y: 1 - ((event.clientY - rect.top) * 2) / height,
    });
}
function create_jump_arena_shell_bang(mount_value, browser_value, three_value, emit_input) {
    const mount = require_mount(mount_value);
    const browser = require_browser(browser_value);
    const three = require_three(three_value);
    const scene = new three.Scene();
    const camera = new three.PerspectiveCamera(52, 1, 0.1, 160);
    const renderer = new three.WebGLRenderer({ antialias: true });
    const canvas = renderer.domElement;
    const player_geometry = new three.BoxGeometry(0.8, 1.8, 0.8);
    const player_material = new three.MeshStandardMaterial({
        color: 4697343,
        roughness: 0.42,
    });
    const platform_geometry = new three.BoxGeometry(1, 1, 1);
    const platform_material = new three.MeshStandardMaterial({
        color: 2967637,
        roughness: 0.78,
    });
    const collectible_geometry = new three.BoxGeometry(0.45, 0.45, 0.45);
    const player_mesh = new three.Mesh(player_geometry, player_material);
    const platform_group = new three.Group();
    const collectible_group = new three.Group();
    let disposed = false;
    let canvas_focused = false;
    let has_frame = false;
    let platform_meshes = [];
    let collectible_meshes = [];
    const active_pointers = new Set();
    scene.background = new three.Color(1116716);
    scene.add(new three.HemisphereLight(10144255, 1250849, 2.4));
    const sun = new three.DirectionalLight(16777215, 3.2);
    sun.position.set(6, 12, 8);
    scene.add(sun);
    scene.add(platform_group);
    scene.add(collectible_group);
    scene.add(player_mesh);
    canvas.setAttribute("tabindex", "0");
    mount.appendChild(canvas);
    const resize = () => {
        if (disposed)
            return;
        const width = Math.max(1, numeric_value(mount.clientWidth));
        const height = Math.max(1, numeric_value(mount.clientHeight));
        const ratio = Math.min(2, numeric_value(browser.devicePixelRatio));
        renderer.setPixelRatio(ratio);
        renderer.setSize(width, height, false);
        camera.aspect = width / height;
        camera.updateProjectionMatrix();
        if (has_frame)
            renderer.render(scene, camera);
    };
    const focus_canvas = () => {
        if (!disposed)
            canvas_focused = true;
    };
    const blur_canvas = () => {
        if (!disposed)
            canvas_focused = false;
    };
    const key_down = (event) => {
        if (!disposed && canvas_focused) {
            return emit_input(keyboard_input("down", event));
        }
    };
    const key_up = (event) => {
        if (!disposed && canvas_focused) {
            return emit_input(keyboard_input("up", event));
        }
    };
    const pointer_event = (phase, event) => emit_input(pointer_input(phase, event, canvas));
    const pointer_down = (event) => {
        if (disposed)
            return;
        canvas.focus({ preventScroll: true });
        active_pointers.add(event.pointerId);
        canvas.setPointerCapture(event.pointerId);
        return pointer_event("down", event);
    };
    const pointer_move = (event) => {
        if (!disposed)
            return pointer_event("move", event);
    };
    const release_pointer = (event) => {
        if (canvas.hasPointerCapture(event.pointerId)) {
            canvas.releasePointerCapture(event.pointerId);
        }
        active_pointers.delete(event.pointerId);
    };
    const pointer_up = (event) => {
        if (disposed)
            return;
        release_pointer(event);
        return pointer_event("up", event);
    };
    const pointer_cancel = (event) => {
        if (disposed)
            return;
        release_pointer(event);
        return pointer_event("cancel", event);
    };
    const clear_collectibles = () => {
        for (const mesh of collectible_meshes) {
            collectible_group.remove(mesh);
            mesh.material.dispose();
        }
        collectible_meshes = [];
    };
    const render_frame = (incoming) => {
        if (disposed)
            throw new Error("jump arena shell is disposed");
        const frame = require_frozen_frame(incoming);
        const { position, yaw } = frame.player;
        player_mesh.position.set(position.x, position.y, position.z);
        player_mesh.rotation.y = yaw;
        for (const mesh of platform_meshes)
            platform_group.remove(mesh);
        platform_meshes = [];
        for (const platform of frame.world.platforms) {
            const mesh = new three.Mesh(platform_geometry, platform_material);
            mesh.position.set(platform.position.x, platform.position.y, platform.position.z);
            mesh.scale.set(platform.size.x, platform.size.y, platform.size.z);
            platform_group.add(mesh);
            platform_meshes.push(mesh);
        }
        clear_collectibles();
        for (const collectible of frame.world.collectibles) {
            const material = new three.MeshStandardMaterial({
                color: projected_symbol_color(collectible.state),
                roughness: 0.3,
            });
            const mesh = new three.Mesh(collectible_geometry, material);
            mesh.position.set(collectible.position.x, collectible.position.y, collectible.position.z);
            collectible_group.add(mesh);
            collectible_meshes.push(mesh);
        }
        camera.position.set(position.x + Math.sin(yaw) * 7.5, position.y + 4.8, position.z + Math.cos(yaw) * 7.5);
        camera.lookAt(position.x, position.y + 1, position.z);
        renderer.render(scene, camera);
        has_frame = true;
        return frame;
    };
    const dispose = () => {
        if (disposed)
            return;
        disposed = true;
        canvas_focused = false;
        has_frame = false;
        browser.removeEventListener("resize", resize);
        canvas.removeEventListener("focus", focus_canvas);
        canvas.removeEventListener("blur", blur_canvas);
        canvas.removeEventListener("keydown", key_down);
        canvas.removeEventListener("keyup", key_up);
        canvas.removeEventListener("pointerdown", pointer_down);
        canvas.removeEventListener("pointermove", pointer_move);
        canvas.removeEventListener("pointerup", pointer_up);
        canvas.removeEventListener("pointercancel", pointer_cancel);
        for (const pointer_id of active_pointers) {
            if (canvas.hasPointerCapture(pointer_id)) {
                canvas.releasePointerCapture(pointer_id);
            }
        }
        active_pointers.clear();
        platform_meshes = [];
        platform_group.clear();
        clear_collectibles();
        collectible_group.clear();
        player_geometry.dispose();
        player_material.dispose();
        platform_geometry.dispose();
        platform_material.dispose();
        collectible_geometry.dispose();
        scene.clear();
        renderer.dispose();
        renderer.forceContextLoss();
        canvas.remove();
    };
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
    return Object.freeze({
        canvas,
        renderFrame: render_frame,
        dispose,
    });
}
export { create_jump_arena_shell_bang as "create-jump-arena-shell!" };
//# sourceMappingURL=shell.js.map