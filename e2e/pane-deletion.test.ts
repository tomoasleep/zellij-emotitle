import { test, expect, beforeAll, afterAll, describe } from "bun:test";
import { launchTerminal, type TerminalSession } from "tuistory";

import {
  setupConfigDir,
  setupCacheDir,
  cleanEnv,
  cleanupConfigDir,
  cleanupCacheDir,
  zellijAction,
  deleteSession,
  sleep,
} from "./test-helpers";

const SESSION_NAME = `emotitle-e2e-${Date.now()}`;
let configDir: string;
let cacheDir: string;

describe("pane deletion emoji behavior", () => {
  let session: TerminalSession;

  beforeAll(async () => {
    configDir = setupConfigDir();
    cacheDir = setupCacheDir();

    session = await launchTerminal({
      command: "bash",
      args: [],
      cols: 140,
      rows: 35,
      env: cleanEnv(cacheDir),
    });

    await sleep(300);

    await session.type("unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME");
    await session.press("enter");
    await sleep(100);

    await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
    await session.press("enter");
    await sleep(100);

    await session.type(`zellij --config-dir ${configDir} -s ${SESSION_NAME}`);
    await session.press("enter");

    await sleep(2000);
  }, 10000);

  afterAll(() => {
    try {
      deleteSession(SESSION_NAME);
    } catch {}
    session?.close();
    cleanupConfigDir(configDir);
    cleanupCacheDir(cacheDir);
  });

  test("scenario A: pane deletion and emoji should not persist", async () => {
    await session.press("enter");
    await sleep(300);

    zellijAction(configDir, cacheDir, SESSION_NAME, "rename-pane", ["🚀"]);
    await sleep(500);

    let text = await session.text();
    expect(text).toContain("🚀");

    zellijAction(configDir, cacheDir, SESSION_NAME, "close-pane");
    await sleep(500);

    zellijAction(configDir, cacheDir, SESSION_NAME, "new-pane");
    await sleep(500);

    text = await session.text();
    expect(text).not.toContain("🚀");
  }, 30000);

  test("scenario B: other pane keeps emoji after pane deletion", async () => {
    zellijAction(configDir, cacheDir, SESSION_NAME, "rename-pane", ["📚"]);
    await sleep(500);

    zellijAction(configDir, cacheDir, SESSION_NAME, "new-pane");
    await sleep(500);

    zellijAction(configDir, cacheDir, SESSION_NAME, "rename-pane", ["✅"]);
    await sleep(500);

    let text = await session.text();
    expect(text).toContain("✅");

    zellijAction(configDir, cacheDir, SESSION_NAME, "close-pane");
    await sleep(500);

    zellijAction(configDir, cacheDir, SESSION_NAME, "focus-previous-pane");
    await sleep(500);

    text = await session.text();
    expect(text).toContain("📚");
  }, 30000);

  test("scenario C: multiple pane deletion and creation", async () => {
    zellijAction(configDir, cacheDir, SESSION_NAME, "new-pane");
    await sleep(200);

    zellijAction(configDir, cacheDir, SESSION_NAME, "rename-pane", ["🎉"]);
    await sleep(500);

    zellijAction(configDir, cacheDir, SESSION_NAME, "close-pane");
    await sleep(500);

    for (let i = 0; i < 2; i++) {
      zellijAction(configDir, cacheDir, SESSION_NAME, "new-pane");
      await sleep(500);

      const text = await session.text();
      expect(text).not.toContain("🎉");

      zellijAction(configDir, cacheDir, SESSION_NAME, "close-pane");
      await sleep(200);
    }
  }, 30000);
});
