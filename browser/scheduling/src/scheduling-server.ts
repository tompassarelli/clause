const hostname = "127.0.0.1";
const port = Number.parseInt(Bun.env.SCHEDULING_PORT ?? "4174", 10);
const shellRoot = new URL("../", import.meta.url);
const laneRoot = new URL("../../../", import.meta.url);
const sourcePath = new URL("test-vectors/authoring/scheduling.clause", laneRoot).pathname;
const sourceServer = Bun.env.SCHEDULING_SOURCE_SERVER
  ?? new URL("target/scheduling-live-workbench/debug/scheduling_source_server", laneRoot).pathname;
const entrypoint = new URL("./scheduling-app.ts", import.meta.url).pathname;

interface ScalarEffectPayload {
  readonly index: number;
  readonly entry: number;
  readonly start: number;
  readonly end: number;
  readonly artifact: string;
  readonly expression: string;
  readonly designation: string;
}

interface GenerationPayload {
  readonly generation: number;
  readonly compilerMicros: number;
  readonly cwr1: string;
  readonly cet1: string | null;
  readonly scalarEffects: readonly ScalarEffectPayload[];
  readonly completeEntry: number;
}

function decodeHexText(value: string): string {
  return new TextDecoder().decode(Uint8Array.from(
    value.match(/../g) ?? [],
    pair => Number.parseInt(pair, 16),
  ));
}

function spawnSourceServer() {
  const child = Bun.spawn({
    cmd: [sourceServer, sourcePath],
    stdin: "pipe",
    stdout: "pipe",
    stderr: "inherit",
  });
  const reader = child.stdout.getReader();
  const decoder = new TextDecoder();
  let buffered = "";
  let latest: GenerationPayload | null = null;
  let boundary = Promise.resolve();

  function serialized<T>(operation: () => Promise<T>): Promise<T> {
    const result = boundary.then(operation);
    boundary = result.then(() => undefined, () => undefined);
    return result;
  }

  async function readLine(): Promise<string> {
    for (;;) {
      const newline = buffered.indexOf("\n");
      if (newline >= 0) {
        const line = buffered.slice(0, newline);
        buffered = buffered.slice(newline + 1);
        return line;
      }
      const chunk = await reader.read();
      if (chunk.done) throw new Error("resident scheduling source process ended");
      buffered += decoder.decode(chunk.value, { stream: true });
    }
  }

  function parse(line: string): GenerationPayload {
    const fields = line.split("\t");
    if (fields[0] === "error") throw new Error(decodeHexText(fields[1] ?? ""));
    if (fields[0] !== "generation" || fields.length !== 7) {
      throw new Error("resident scheduling source protocol failed");
    }
    const catalog = fields[5] ?? "";
    return Object.freeze({
      generation: Number.parseInt(fields[1]!, 10),
      compilerMicros: Number.parseInt(fields[2]!, 10),
      cwr1: fields[3]!,
      cet1: fields[4]!.length === 0 ? null : fields[4]!,
      scalarEffects: Object.freeze(catalog.length === 0 ? [] : catalog.split(";").map(value => {
        const [index, entry, start, end, artifact, expression, designation] = value.split(",");
        if ([index, entry, start, end, artifact, expression, designation].some(field => field === undefined)) {
          throw new Error("resident scheduling edit catalog failed");
        }
        return Object.freeze({
          index: Number.parseInt(index!, 10),
          entry: Number.parseInt(entry!, 10),
          start: Number.parseInt(start!, 10),
          end: Number.parseInt(end!, 10),
          artifact: artifact!,
          expression: decodeHexText(expression!),
          designation: decodeHexText(designation!),
        });
      })),
      completeEntry: Number.parseInt(fields[6]!, 10),
    });
  }

  async function current(): Promise<GenerationPayload> {
    if (latest === null) latest = parse(await readLine());
    return latest;
  }

  async function editUnserialized(
    capturedGeneration: number,
    catalogIndex: number,
    expression: string,
  ): Promise<GenerationPayload> {
    if ((await current()).generation !== capturedGeneration) {
      throw new Error("This schedule rule changed before your edit. Review the current rule and try again.");
    }
    const bytes = new TextEncoder().encode(expression);
    if (bytes.length > 4096 || expression.includes("\n") || expression.includes("\r")) {
      throw new Error("The rule expression is too long.");
    }
    const encoded = Array.from(bytes, byte => byte.toString(16).padStart(2, "0")).join("");
    child.stdin.write(`edit\t${capturedGeneration}\t${catalogIndex}\t${encoded}\n`);
    child.stdin.flush();
    latest = parse(await readLine());
    return latest;
  }

  return {
    child,
    current: () => serialized(current),
    edit: (capturedGeneration: number, catalogIndex: number, expression: string) =>
      serialized(() => editUnserialized(capturedGeneration, catalogIndex, expression)),
  };
}

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

const resident = spawnSourceServer();

async function route(request: Request): Promise<Response> {
  const url = new URL(request.url);
  if (url.pathname === "/" || url.pathname === "/schedule.html") {
    return existingFile(new URL("scheduling.html", shellRoot), "text/html; charset=utf-8");
  }
  if (url.pathname === "/scheduling.css") {
    return existingFile(new URL("scheduling.css", shellRoot), "text/css; charset=utf-8");
  }
  if (url.pathname === "/assets/scheduling-app.js") {
    try {
      return new Response(await bundleSchedulingApp(), {
        headers: { "content-type": "text/javascript; charset=utf-8" },
      });
    } catch (error) {
      console.error(error);
      return new Response("The schedule could not be prepared.", { status: 500 });
    }
  }
  if (url.pathname === "/resident-generation") {
    try {
      return Response.json(await resident.current());
    } catch (error) {
      console.error(error);
      return Response.json({ error: "The schedule source could not be checked." }, { status: 503 });
    }
  }
  if (url.pathname === "/resident-edit" && request.method === "POST") {
    if (request.headers.get("origin") !== url.origin) {
      return Response.json({ error: "The schedule edit did not come from this workbench." }, { status: 403 });
    }
    try {
      const body: unknown = await request.json();
      if (typeof body !== "object" || body === null || Array.isArray(body)) {
        throw new Error("The schedule edit is malformed.");
      }
      const value = body as Record<string, unknown>;
      if (!Number.isSafeInteger(value.capturedGeneration)
        || !Number.isSafeInteger(value.catalogIndex)
        || typeof value.expression !== "string") {
        throw new Error("The schedule edit is malformed.");
      }
      return Response.json(await resident.edit(
        value.capturedGeneration as number,
        value.catalogIndex as number,
        value.expression,
      ));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      return Response.json({ error: message }, { status: 422 });
    }
  }
  if (url.pathname === "/wasm/clause_runtime.js") {
    return existingFile(new URL("target/scheduling-browser-wasm/clause_runtime.js", laneRoot), "text/javascript; charset=utf-8");
  }
  if (url.pathname === "/wasm/clause_runtime_bg.wasm") {
    return existingFile(new URL("target/scheduling-browser-wasm/clause_runtime_bg.wasm", laneRoot), "application/wasm");
  }
  return new Response("Not found", { status: 404 });
}

try {
  Bun.serve({ hostname, port, fetch: route });
  console.log(`Scheduling workbench listening on http://${hostname}:${port}`);
} catch (error) {
  resident.child.kill();
  console.error(`Cannot start scheduling workbench: ${hostname}:${port} is unavailable.`);
  throw error;
}

function stopResident(): void {
  try {
    resident.child.stdin.write("quit\n");
    resident.child.stdin.flush();
  } finally {
    resident.child.kill();
    process.exit(0);
  }
}

process.on("SIGINT", stopResident);
process.on("SIGTERM", stopResident);
