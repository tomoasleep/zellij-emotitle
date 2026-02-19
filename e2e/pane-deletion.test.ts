import { test, expect, beforeAll, afterAll, describe } from "bun:test";
import { launchTerminal, type TerminalSession } from "tuistory";
import { tmpdir } from "os";
import { join } from "path";
import { mkdirSync, rmSync, writeFileSync } from "fs";

const SESSION_NAME = `emotitle-e2e-${Date.now()}`;
let configDir: string;

function setupConfigDir(): string {
  const dir = join(tmpdir(), `zellij-test-${Date.now()}`);
  mkdirSync(dir, { recursive: true });
  mkdirSync(join(dir, "layouts"), { recursive: true });
  
  writeFileSync(join(dir, "config.kdl"), `
keybinds {
    normal {}
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

function cleanEnv(): Record<string, string> {
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (value !== undefined && !key.startsWith("ZELLIJ")) {
      env[key] = value;
    }
  }
  return env;
}

function zellijAction(action: string, args: string[] = []): void {
  const result = Bun.spawnSync(
    ["zellij", "--config-dir", configDir, "--session", SESSION_NAME, "action", action, ...args],
    { encoding: "utf-8", env: cleanEnv() }
  );
  if (result.exitCode !== 0) {
    throw new Error(`zellij action ${action} failed: ${result.stderr}`);
  }
}

async function sleep(ms: number): Promise<void> {
  await new Promise((r) => setTimeout(r, ms));
}

describe("pane deletion emoji behavior", () => {
  let session: TerminalSession;

  beforeAll(async () => {
    configDir = setupConfigDir();
    
    session = await launchTerminal({
      command: "bash",
      args: [],
      cols: 140,
      rows: 35,
      env: cleanEnv(),
    });
    
    await sleep(300);
    
    await session.type(`unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME`);
    await session.press("enter");
    await sleep(100);
    
    await session.type(`zellij --config-dir ${configDir} -s ${SESSION_NAME} options --simplified-ui true`);
    await session.press("enter");
    
    await sleep(2000);
  }, 10000);

  afterAll(() => {
    try {
      Bun.spawnSync(["zellij", "--config-dir", configDir, "--session", SESSION_NAME, "kill-session"], { 
        timeout: 5000,
        env: cleanEnv(),
      });
    } catch {}
    session?.close();
    try {
      rmSync(configDir, { recursive: true, force: true });
    } catch {}
  });

  test("scenario A: pane deletion and emoji should not persist", async () => {
    await session.press("enter");
    await sleep(300);
    
    zellijAction("rename-pane", ["🚀"]);
    await sleep(500);
    
    let text = await session.text();
    expect(text).toContain("🚀");
    
    zellijAction("close-pane");
    await sleep(500);
    
    zellijAction("new-pane");
    await sleep(500);
    
    text = await session.text();
    expect(text).not.toContain("🚀");
  }, 30000);

  test("scenario B: other pane keeps emoji after pane deletion", async () => {
    zellijAction("rename-pane", ["📚"]);
    await sleep(500);
    
    zellijAction("new-pane");
    await sleep(500);
    
    zellijAction("rename-pane", ["✅"]);
    await sleep(500);
    
    let text = await session.text();
    expect(text).toContain("✅");
    
    zellijAction("close-pane");
    await sleep(500);
    
    zellijAction("focus-previous-pane");
    await sleep(500);
    
    text = await session.text();
    expect(text).toContain("📚");
  }, 30000);

  test("scenario C: multiple pane deletion and creation", async () => {
    zellijAction("new-pane");
    await sleep(200);
    
    zellijAction("rename-pane", ["🎉"]);
    await sleep(500);
    
    zellijAction("close-pane");
    await sleep(500);
    
    for (let i = 0; i < 2; i++) {
      zellijAction("new-pane");
      await sleep(500);
      
      const text = await session.text();
      expect(text).not.toContain("🎉");
      
      zellijAction("close-pane");
      await sleep(200);
    }
  }, 30000);
});
