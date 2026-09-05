const hostname = "127.0.0.1";
const port = 4174;
const shellRoot = new URL("../", import.meta.url);
const scheduleBuild = new URL("../../build/scheduling/", shellRoot);
const entrypoint = new URL("./scheduling-app.ts", import.meta.url).pathname;

let browserBundle: Promise<Blob> | null = null;

function bundleSchedulingApp(): Promise<Blob> {
  browserBundle ??= Bun.build({
    entrypoints: [entrypoint],
    format: "esm",
    target: "browser",
    sourcemap: "inline",
    minify: false,
  }).then(result => {
    if (!result.success || result.outputs.length !== 1) {
      for (const log of result.logs) console.error(log);
      throw new Error("scheduling browser bundle failed");
    }
    return result.outputs[0];
  });
  return browserBundle;
}

async function existingFile(url: URL, contentType: string): Promise<Response> {
  const asset = Bun.file(url);
  if (!(await asset.exists())) {
    console.error(`Missing scheduling asset: ${url.pathname}`);
    return new Response("The schedule is not ready.", { status: 503 });
  }
  return new Response(asset, { headers: { "content-type": contentType } });
}

async function route(request: Request): Promise<Response> {
  const path = new URL(request.url).pathname;
  if (path === "/" || path === "/schedule.html") {
    return existingFile(new URL("scheduling.html", shellRoot), "text/html; charset=utf-8");
  }
  if (path === "/scheduling.css") {
    return existingFile(new URL("scheduling.css", shellRoot), "text/css; charset=utf-8");
  }
  if (path === "/assets/scheduling-app.js") {
    try {
      return new Response(await bundleSchedulingApp(), {
        headers: { "content-type": "text/javascript; charset=utf-8" },
      });
    } catch (error) {
      console.error(error);
      return new Response("The schedule could not be prepared.", { status: 500 });
    }
  }
  if (path === "/schedule/initial.cwr1") {
    return existingFile(new URL("initial.cwr1", scheduleBuild), "application/octet-stream");
  }
  if (path === "/wasm/clause_runtime.js") {
    return existingFile(new URL("wasm/clause_runtime.js", scheduleBuild), "text/javascript; charset=utf-8");
  }
  if (path === "/wasm/clause_runtime_bg.wasm") {
    return existingFile(new URL("wasm/clause_runtime_bg.wasm", scheduleBuild), "application/wasm");
  }
  return new Response("Not found", { status: 404 });
}

try {
  // The bind is the atomic availability check. If another service owns 4174,
  // Bun throws and this process leaves it untouched.
  Bun.serve({ hostname, port, fetch: route });
  console.log(`Scheduling interface listening on http://${hostname}:${port}`);
} catch (error) {
  console.error(`Cannot start scheduling interface: ${hostname}:${port} is unavailable.`);
  throw error;
}
