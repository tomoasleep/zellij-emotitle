import { test, expect, afterAll, describe } from "bun:test";
import { launchTerminal, type TerminalSession } from "tuistory";
import { tmpdir } from "os";
import { join } from "path";
import { mkdirSync, rmSync, writeFileSync } from "fs";

const WASM_PATH = process.cwd() + "/../target/wasm32-wasip1/release/zellij-emotitle.wasm";

function setupConfigDir(): string {
  const dir = join(tmpdir(), `zellij-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
  mkdirSync(dir, { recursive: true });
  mkdirSync(join(dir, "layouts"), { recursive: true });
  
  writeFileSync(join(dir, "config.kdl"), `
keybinds {
    normal {}
}

load_plugins {
    file "${WASM_PATH}"
}
  `);
  writeFileSync(join(dir, "layouts", "default.kdl"), `
layout {
    pane size=1 split_direction="Vertical" borderless=true {
        plugin location="tab-bar"
    }
    pane
    pane size=1 borderless=true {
        plugin location="status-bar"
    }
}
  `);
  return dir;
}

function setupCacheDir(): string {
  const dir = join(tmpdir(), `zellij-cache-${Date.now()}-${Math.random().toString(36).slice(2)}`);
  mkdirSync(dir, { recursive: true });
  
  writeFileSync(join(dir, "permissions.kdl"), `"${WASM_PATH}" {
    ChangeApplicationState
    ReadApplicationState
}
`);
  
  return dir;
}

function cleanEnv(cacheDir: string): Record<string, string> {
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (value !== undefined && !key.startsWith("ZELLIJ")) {
      env[key] = value;
    }
  }
  env["ZELLIJ_CACHE_DIR"] = cacheDir;
  return env;
}

async function sleep(ms: number): Promise<void> {
  await new Promise((r) => setTimeout(r, ms));
}

function runPipe(configDir: string, cacheDir: string, sessionName: string, args: string): Promise<void> {
  const proc = Bun.spawn([
    "zellij",
    "--config-dir", configDir,
    "--session", sessionName,
    "pipe",
    "--name", "emotitle",
    "--plugin", `file:${WASM_PATH}`,
    "--args", args,
    "--", "",
  ], {
    env: cleanEnv(cacheDir),
  });
  return proc.exited.then(() => {});
}

describe("pane deletion emoji behavior", () => {
  test("emoji is added to focused pane title", async () => {
    const configDir = setupConfigDir();
    const cacheDir = setupCacheDir();
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
      
      await session.type(`unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME`);
      await session.press("enter");
      await sleep(100);
      
      await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
      await session.press("enter");
      await sleep(100);
      
      await session.type(`zellij --config-dir ${configDir} -s ${sessionName}`);
      await session.press("enter");
      await sleep(3000);
      
      await session.press("esc");
      await sleep(200);
      
      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=🚀,mode=permanent");
      await sleep(500);
      
      const text = await session.text();
      expect(text).toContain("Pane #1 | 🚀");
    } finally {
      try {
        Bun.spawnSync(["zellij", "--config-dir", configDir, "--session", sessionName, "kill-session"], { 
          timeout: 5000,
          env: cleanEnv(cacheDir),
        });
      } catch {}
      session.close();
      rmSync(configDir, { recursive: true, force: true });
      rmSync(cacheDir, { recursive: true, force: true });
    }
  }, 30000);

  test("deleted pane emoji does not appear on new pane", async () => {
    const configDir = setupConfigDir();
    const cacheDir = setupCacheDir();
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
      
      await session.type(`unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME`);
      await session.press("enter");
      await sleep(100);
      
      await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
      await session.press("enter");
      await sleep(100);
      
      await session.type(`zellij --config-dir ${configDir} -s ${sessionName}`);
      await session.press("enter");
      await sleep(3000);
      
      await session.press("esc");
      await sleep(200);
      
      // Add emoji to pane 1
      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=📚,mode=permanent");
      await sleep(500);
      
      let text = await session.text();
      expect(text).toContain("Pane #1 | 📚");
      
      // Close pane 1
      await session.type(`zellij --config-dir ${configDir} --session ${sessionName} action close-pane`);
      await session.press("enter");
      await sleep(500);
      
      // Create new pane
      await session.type(`zellij --config-dir ${configDir} --session ${sessionName} action new-pane`);
      await session.press("enter");
      await sleep(500);
      
      // New pane should NOT have the emoji from deleted pane
      text = await session.text();
      expect(text).not.toContain("📚");
    } finally {
      try {
        Bun.spawnSync(["zellij", "--config-dir", configDir, "--session", sessionName, "kill-session"], { 
          timeout: 5000,
          env: cleanEnv(cacheDir),
        });
      } catch {}
      session.close();
      rmSync(configDir, { recursive: true, force: true });
      rmSync(cacheDir, { recursive: true, force: true });
    }
  }, 45000);

  test("other pane keeps emoji after pane deletion", async () => {
    const configDir = setupConfigDir();
    const cacheDir = setupCacheDir();
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
      
      await session.type(`unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME`);
      await session.press("enter");
      await sleep(100);
      
      await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
      await session.press("enter");
      await sleep(100);
      
      await session.type(`zellij --config-dir ${configDir} -s ${sessionName}`);
      await session.press("enter");
      await sleep(3000);
      
      await session.press("esc");
      await sleep(200);
      
      // Add emoji to pane 1
      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=✅,mode=permanent");
      await sleep(500);
      
      // Create pane 2
      await session.type(`zellij --config-dir ${configDir} --session ${sessionName} action new-pane`);
      await session.press("enter");
      await sleep(500);
      
      // Add emoji to pane 2
      await runPipe(configDir, cacheDir, sessionName, "target=pane,emojis=🎉,mode=permanent");
      await sleep(500);
      
      let text = await session.text();
      expect(text).toContain("🎉");
      
      // Close pane 2
      await session.type(`zellij --config-dir ${configDir} --session ${sessionName} action close-pane`);
      await session.press("enter");
      await sleep(500);
      
      // Pane 1 should still have its emoji
      text = await session.text();
      expect(text).toContain("✅");
    } finally {
      try {
        Bun.spawnSync(["zellij", "--config-dir", configDir, "--session", sessionName, "kill-session"], { 
          timeout: 5000,
          env: cleanEnv(cacheDir),
        });
      } catch {}
      session.close();
      rmSync(configDir, { recursive: true, force: true });
      rmSync(cacheDir, { recursive: true, force: true });
    }
  }, 60000);
});
