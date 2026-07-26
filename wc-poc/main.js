// oxfmt WebContainer mini playground.
// Boots a WebContainer once, installs oxfmt (wasi build), starts a long-lived
// in-container format server, and talks to it over stdio (JSON lines).
// (HTTP via the preview URL is not usable from the host page: the proxy drops
// CORS headers. stdio avoids networking entirely.)
import { WebContainer } from "./node_modules/@webcontainer/api/dist/index.js";

const $ = (s) => document.querySelector(s);
const statusEl = $("#status");
const inputEl = $("#input");
const outputEl = $("#output");
statusEl.textContent = "";
const setStatus = (msg) => {
  statusEl.textContent = msg;
  console.log("[status]", msg);
};

const DEMO = {
  "demo.tsx": `const Button = styled.button\`
  color:red;background:   blue;
     padding:0 8px;
\`;
export const C = () => <div   className="p-4 text-white bg-blue-500 flex items-center">hi</div>;
`,
  "demo.ts": "const   x =   {a:1,   b:2};\nfunction  foo( a:number ){ return a }\n",
  "demo.css": ".a{color:red;background:blue}\n  .b   {   margin : 0 }\n",
  "demo.json": '{"b":1,   "a": [1,2,\n3]}\n',
  "demo.graphql": "type   Query { user(id:ID!):User,  }\ntype User{id:ID! name:String}\n",
  "demo.vue": `<template>
  <div   :class="ok?'a':'b'"  >{{   msg   }}</div>
</template>
<script setup>
const msg   =   "hi";
</script>
`,
  "demo.svelte": `<script>
  let count   = 0;
</script>
<button on:click={()=>count+=1}>
  clicked {count}   times</button>
`,
};

// Long-lived in-container process: one JSON request per line on stdin,
// one JSON response per line on stdout.
const FORMAT_SERVER = `
import readline from "node:readline";
import { format } from "oxfmt";
const rl = readline.createInterface({ input: process.stdin });
rl.on("line", async (line) => {
  line = line.trim();
  if (!line.startsWith("{")) return;
  const { id, filename, code, options } = JSON.parse(line);
  try {
    const t = performance.now();
    const r = await format(filename, code, options ?? {});
    process.stdout.write(
      "@@RES@@" + JSON.stringify({ id, code: r.code, errors: r.errors, ms: +(performance.now() - t).toFixed(1) }) + "\\n",
    );
  } catch (e) {
    process.stdout.write("@@RES@@" + JSON.stringify({ id, errors: [String(e)] }) + "\\n");
  }
});
console.log("[server-ready]");
`;

let writer = null;
let reqId = 0;
const pending = new Map();

function handleOutputChunk(buffer, chunk, onLine) {
  // jsh mixes in terminal escape sequences; strip them before line-parsing
  buffer.text += chunk.replace(/\x1b\[[0-9;?]*[A-Za-z]/g, "").replace(/\r/g, "");
  const lines = buffer.text.split("\n");
  buffer.text = lines.pop() ?? "";
  for (const line of lines) onLine(line);
}

async function boot() {
  setStatus("fetching tarball...");
  const res = await fetch("/oxfmt.tgz");
  if (!res.ok) throw new Error(`oxfmt.tgz not found (${res.status})`);
  const tgz = new Uint8Array(await res.arrayBuffer());

  setStatus("booting WebContainer...");
  const wc = await WebContainer.boot();

  await wc.mount({
    "package.json": {
      file: { contents: JSON.stringify({ name: "poc", type: "module", private: true }) },
    },
    "oxfmt.tgz": { file: { contents: tgz } },
    "format-server.mjs": { file: { contents: FORMAT_SERVER } },
  });

  setStatus("npm install (~10s)...");
  const install = await wc.spawn("npm", ["i", "./oxfmt.tgz", "svelte", "--no-audit", "--no-fund"]);
  if ((await install.exit) !== 0) throw new Error("npm install failed");

  setStatus("starting format server...");
  const proc = await wc.spawn("node", ["format-server.mjs"]);
  writer = proc.input.getWriter();

  const ready = Promise.withResolvers();
  const buffer = { text: "" };
  proc.output.pipeTo(
    new WritableStream({
      write: (chunk) =>
        handleOutputChunk(buffer, chunk, (line) => {
          if (line.includes("[server-ready]")) return ready.resolve();
          const idx = line.indexOf("@@RES@@");
          if (idx === -1) return;
          try {
            const msg = JSON.parse(line.slice(idx + 7));
            pending.get(msg.id)?.(msg);
            pending.delete(msg.id);
          } catch {
            console.log("[unparsable]", line.slice(0, 120));
          }
        }),
    }),
  );
  await ready.promise;
  setStatus("ready");
}

function request(payload, timeoutMs = 15000) {
  const id = ++reqId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error("format request timed out"));
    }, timeoutMs);
    pending.set(id, (msg) => {
      clearTimeout(timer);
      resolve(msg);
    });
    writer.write(JSON.stringify({ id, ...payload }) + "\n");
  });
}

let timer = null;
async function formatNow() {
  if (!writer) return;
  try {
    const filename = $("#filename").value;
    const options = $("#tailwind").checked ? { sortTailwindcss: {}, sortImports: {} } : {};
    // Tier 4 formatters are opt-in via config (runtime `svelte` package installed in-container)
    if (filename.endsWith(".svelte")) options.svelte = true;
    const { code, errors, ms } = await request({
      filename,
      code: inputEl.value,
      options,
    });
    const hasErrors = (errors ?? []).length > 0;
    outputEl.classList.toggle("error", hasErrors);
    outputEl.textContent = hasErrors ? JSON.stringify(errors, null, 2) : code;
    if (ms !== undefined) setStatus(`formatted in ${ms}ms (in-container)`);
  } catch (e) {
    outputEl.classList.add("error");
    outputEl.textContent = String(e);
  }
}
const schedule = () => {
  clearTimeout(timer);
  timer = setTimeout(formatNow, 300);
};

inputEl.addEventListener("input", schedule);
$("#tailwind").addEventListener("change", formatNow);
$("#filename").addEventListener("change", () => {
  inputEl.value = DEMO[$("#filename").value] ?? "";
  formatNow();
});

inputEl.value = DEMO["demo.tsx"];
boot()
  .then(formatNow)
  .catch((e) => setStatus("[boot error] " + e));
