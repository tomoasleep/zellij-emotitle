import { test, expect, describe } from "bun:test";
import { launchTerminal } from "tuistory";

import {
  WASM_PATH,
  setupConfigDir,
  setupCacheDir,
  cleanEnv,
  cleanupConfigDir,
  cleanupCacheDir,
  zellijAction,
  deleteSession,
  sleep,
} from "./test-helpers";

function runPipe(configDir: string, cacheDir: string, sessionName: string, args: string): Promise<void> {
  const proc = Bun.spawn(
    [
      "zellij",
      "--config-dir",
      configDir,
      "--session",
      sessionName,
      "pipe",
      "--name",
      "emotitle",
      "--plugin",
      `file:${WASM_PATH}`,
      "--args",
      args,
      "--",
      "",
    ],
    {
      env: cleanEnv(cacheDir),
    },
  );
  return proc.exited.then(() => {});
}

describe("pane deletion emoji behavior", () => {
  test("emoji is added to focused pane title", async () => {
    const configDir = setupConfigDir({ wasmPath: WASM_PATH, simplifiedUi: true, showStartupTips: false });
    const cacheDir = setupCacheDir({ wasmPath: WASM_PATH });
    const sessionName = `emotitle-test-${Date.now()}`;

    const session = await launchTerminal({
      command: "bash",
      args: [],
      cols: 140,
      rows: 35,
      env: cleanEnv(cacheDir),
    });

    try {
      await sleep(300);

      await session.type("unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME");
      await session.press("enter");
      await sleep(100);

      await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
      await session.press("enter");
      await sleep(100);

      await session.type(`zellij --config-dir ${configDir} -s ${sessionName} options --simplified-ui true`);
      await session.press("enter");
      await sleep(3000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📌🚀");
      await sleep(500);

      const text = await session.text();
      expect(text).toContain("📌🚀");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 30000);

  test("deleted pane emoji does not appear on new pane", async () => {
    const configDir = setupConfigDir({ wasmPath: WASM_PATH, simplifiedUi: true, showStartupTips: false });
    const cacheDir = setupCacheDir({ wasmPath: WASM_PATH });
    const sessionName = `emotitle-test-${Date.now()}`;

    const session = await launchTerminal({
      command: "bash",
      args: [],
      cols: 140,
      rows: 35,
      env: cleanEnv(cacheDir),
    });

    try {
      await sleep(300);

      await session.type("unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME");
      await session.press("enter");
      await sleep(100);

      await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
      await session.press("enter");
      await sleep(100);

      await session.type(`zellij --config-dir ${configDir} -s ${sessionName} options --simplified-ui true`);
      await session.press("enter");
      await sleep(3000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📌📚");
      await sleep(500);

      let text = await session.text();
      expect(text).toContain("📌📚");

      zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(500);

      zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
      await sleep(300);

      zellijAction(configDir, cacheDir, sessionName, "close-pane");
      await sleep(500);

      zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(500);

      text = await session.text();
      expect(text).not.toContain("📌📚");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 45000);

  test("other pane keeps emoji after pane deletion", async () => {
    const configDir = setupConfigDir({ wasmPath: WASM_PATH, simplifiedUi: true, showStartupTips: false });
    const cacheDir = setupCacheDir({ wasmPath: WASM_PATH });
    const sessionName = `emotitle-test-${Date.now()}`;

    const session = await launchTerminal({
      command: "bash",
      args: [],
      cols: 140,
      rows: 35,
      env: cleanEnv(cacheDir),
    });

    try {
      await sleep(300);

      await session.type("unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME");
      await session.press("enter");
      await sleep(100);

      await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
      await session.press("enter");
      await sleep(100);

      await session.type(`zellij --config-dir ${configDir} -s ${sessionName} options --simplified-ui true`);
      await session.press("enter");
      await sleep(3000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📌✅");
      await sleep(500);

      zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(500);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📌🎉");
      await sleep(500);

      let text = await session.text();
      expect(text).toContain("📌🎉");

      zellijAction(configDir, cacheDir, sessionName, "close-pane");
      await sleep(500);

      text = await session.text();
      expect(text).toContain("📌✅");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);
});
