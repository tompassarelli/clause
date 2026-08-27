const [root, host, three] = Bun.argv.slice(2);
if (!root || !host || !three) throw new Error("root, host, and Three.js module paths are required");

const files = new Map([
  ["/", `${root}/index.html`],
  ["/artifact.mjs", `${root}/artifact.mjs`],
  ["/host.mjs", host],
  ["/three.mjs", three],
  ["/three.core.js", three.replace(/three\.module\.js$/, "three.core.js")],
]);

const server = Bun.serve({
  hostname: "127.0.0.1",
  port: 0,
  async fetch(request) {
    const pathname = new URL(request.url).pathname;
    const path = files.get(pathname);
    return path
      ? new Response(await Bun.file(path).arrayBuffer(), { headers: { "Content-Type": pathname === "/" ? "text/html; charset=utf-8" : "text/javascript; charset=utf-8" } })
      : new Response("not found", { status: 404 });
  },
});

console.log(server.url.href);
