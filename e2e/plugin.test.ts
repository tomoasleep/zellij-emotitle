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
  const proc = Bun.spawnSync(
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
      stdout: "pipe",
      stderr: "pipe",
      env: cleanEnv(cacheDir),
    },
  );
  if (proc.exitCode !== 0) {
    throw new Error(`zellij pipe failed: ${proc.stderr.toString()}`);
  }
  return Promise.resolve();
}

describe("emotitle plugin e2e", () => {
  test("should apply emojis to the focused pane via pipe", async () => {
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
      await sleep(5000);

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

  test("should not carry pane emojis to a newly created pane after deletion", async () => {
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
      await sleep(5000);

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
  }, 30000);

  test("should keep other pane emojis after deleting the current pane", async () => {
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
      await sleep(5000);

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

      zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
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
  }, 45000);

  test("should keep only pinned segments on focus after stacked emojis", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📌🚀 | 📚 | 🚗");
      await sleep(500);

      let text = await session.text();
      expect(text).toContain("📌🚀");

      zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(500);
      zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
      await sleep(1700);

      text = await session.text();
      expect(text).toContain("📌🚀");
      expect(text).not.toContain("📌🚀 | 📚");
      expect(text).not.toContain("📚");
      expect(text).not.toContain("🚗");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should keep pinned emojis on focus after two consecutive pane pipes", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📌🚀");
      await sleep(200);
      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📚");
      await sleep(400);

      zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(500);
      zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
      await sleep(1000);

      const text = await session.text();
      expect(text).toContain("📌🚀");
      expect(text).not.toContain("📌🚀 | 📚");
      expect(text).not.toContain("📚");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should remove non-pinned pane emojis on focus", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📚");
      await sleep(400);

      zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(500);
      zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
      await sleep(1000);

      const text = await session.text();
      expect(text).not.toContain("📚");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should keep non-pinned pane emojis before one second and remove after one second", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=🛼");
      await sleep(300);

      let text = await session.text();
      expect(text).toContain("🛼");

      await sleep(1200);

      text = await session.text();
      expect(text).not.toContain("🛼");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should keep all pinned pane segments and remove non-pinned on focus", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📌🚀 | 📌✅ | 📚");
      await sleep(400);

      zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(500);
      zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
      await sleep(1000);

      const text = await session.text();
      expect(text).toContain("📌🚀 | 📌✅");
      expect(text).not.toContain("📚");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should remove non-pinned tab emojis after tab switch", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=tab,emojis=📚");
      await sleep(400);

      zellijAction(configDir, cacheDir, sessionName, "new-tab");
      await sleep(700);
      zellijAction(configDir, cacheDir, sessionName, "go-to-previous-tab");
      await sleep(1000);

      const text = await session.text();
      expect(text).not.toContain("📚");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should keep pinned tab emojis with explicit tab_index after focus", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=tab,tab_index=0,emojis=📌🚀");
      await sleep(200);
      await runPipe(configDir, cacheDir, sessionName, "target=tab,tab_index=0,emojis=📚");
      await sleep(400);

      zellijAction(configDir, cacheDir, sessionName, "new-tab");
      await sleep(700);
      zellijAction(configDir, cacheDir, sessionName, "go-to-previous-tab");
      await sleep(1000);

      const text = await session.text();
      expect(text).toContain("📌🚀");
      expect(text).not.toContain("📌🚀 | 📚");
      expect(text).not.toContain("📚");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should keep non-pinned tab emojis before one second and remove after one second", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=tab,emojis=🛼");
      await sleep(300);

      let text = await session.text();
      expect(text).toContain("🛼");

      await sleep(1200);

      text = await session.text();
      expect(text).not.toContain("🛼");
    } finally {
      try {
        deleteSession(sessionName);
      } catch {}
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    }
  }, 60000);

  test("should not resurrect previous temp emojis after timer cleanup on focused pane", async () => {
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
      await sleep(5000);

      await session.press("esc");
      await sleep(200);

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📚");
      await sleep(1300);

      let text = await session.text();
      expect(text).not.toContain("📚");

      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=✅");
      await sleep(300);

      text = await session.text();
      expect(text).toContain("✅");
      expect(text).not.toContain("📚");
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
