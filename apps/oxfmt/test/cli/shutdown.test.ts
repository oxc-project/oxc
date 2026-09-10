import { afterEach, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  runCli: vi.fn(async () => ["format", 17]),
  disposeExternalServices: vi.fn(async () => {}),
}));

vi.mock("../../src-js/bindings", () => ({ runCli: mocks.runCli }));
vi.mock("../../src-js/cli/worker-proxy", () => ({
  disposeExternalServices: mocks.disposeExternalServices,
  initExternalServices: vi.fn(),
  formatFile: vi.fn(),
  formatEmbeddedCode: vi.fn(),
  formatEmbeddedDoc: vi.fn(),
  sortTailwindClasses: vi.fn(),
}));
vi.mock("../../src-js/cli/js_config", () => ({
  loadJsConfig: vi.fn(),
  loadVitePlusConfig: vi.fn(),
}));

afterEach(() => {
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

it.each([
  ["20.19.0", true],
  ["22.12.0", true],
  ["24.12.0", true],
  ["24.13.0", true],
  ["24.13.1", false],
  ["24.14.0", false],
  ["24.19.0", false],
  ["25.3.0", true],
  ["25.4.0", false],
  ["26.0.0", false],
])("uses the shutdown workaround on Node %s: %s", async (version, needsDelay) => {
  vi.resetModules();
  const exit = vi.fn();
  const timer = vi.fn();
  const cliProcess = {
    ...process,
    argv: [process.execPath, "oxfmt"],
    versions: { ...process.versions, node: version },
    stdout: { isTTY: true },
    stdin: { isTTY: true },
    exitCode: undefined,
    exit,
  };
  vi.stubGlobal("process", cliProcess);
  vi.stubGlobal("setTimeout", timer);

  await import("../../src-js/cli");

  expect(mocks.disposeExternalServices).toHaveBeenCalledOnce();
  expect(cliProcess.exitCode).toBe(17);
  expect(exit).not.toHaveBeenCalled();
  const delayedExit = [expect.any(Function), 50];
  expect(timer.mock.calls).toEqual(needsDelay ? [delayedExit] : []);
  timer.mock.calls[0]?.[0]();
  expect(exit.mock.calls).toEqual(needsDelay ? [[]] : []);
  expect(cliProcess.exitCode).toBe(17);
});
