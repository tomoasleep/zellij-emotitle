import { tmpdir } from "os";
import { join } from "path";
import { mkdirSync, rmSync, writeFileSync } from "fs";

export const WASM_PATH = process.cwd() + "/../target/wasm32-wasip1/release/zellij-emotitle.wasm";

type SetupConfigOptions = {
  wasmPath?: string;
  simplifiedUi?: boolean;
  showStartupTips?: boolean;
};

type SetupCacheOptions = {
  wasmPath?: string;
};

export function setupConfigDir(options: SetupConfigOptions = {}): string {
  const configDir = join(tmpdir(), `zellij-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
  mkdirSync(configDir, { recursive: true });
  mkdirSync(join(configDir, "layouts"), { recursive: true });

  const loadPlugins = options.wasmPath
    ? `
load_plugins {
  file "${options.wasmPath}"
}
`
    : "";
  const uiConfig = options.simplifiedUi
    ? `
ui {
  simplified_ui true
}
`
    : "";
  const startupTips = options.showStartupTips === false ? "\nshow_startup_tips false\n" : "";
  
  writeFileSync(join(configDir, "config.kdl"), `
keybinds {
  normal {}
}
${loadPlugins}
${uiConfig}
${startupTips}
  `);
  writeFileSync(join(configDir, "layouts", "default.kdl"), `
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
  
  return configDir;
}

export function setupCacheDir(options: SetupCacheOptions = {}): string {
  const cacheDir = join(tmpdir(), `zellij-cache-${Date.now()}-${Math.random().toString(36).slice(2)}`);
  mkdirSync(cacheDir, { recursive: true });

  if (options.wasmPath) {
    writeFileSync(join(cacheDir, "permissions.kdl"), `"${options.wasmPath}" {
    ChangeApplicationState
    ReadApplicationState
}
`);
  }

  return cacheDir;
}

export function cleanEnv(cacheDir: string): Record<string, string> {
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (value !== undefined && !key.startsWith("ZELLIJ")) {
      env[key] = value;
    }
  }
  env["ZELLIJ_CACHE_DIR"] = cacheDir;
  return env;
}

export function cleanupConfigDir(configDir: string): void {
  try {
    rmSync(configDir, { recursive: true, force: true });
  } catch {}
}

export function cleanupCacheDir(cacheDir: string): void {
  try {
    rmSync(cacheDir, { recursive: true, force: true });
  } catch {}
}

export function zellijAction(configDir: string, cacheDir: string, sessionName: string, action: string, args: string[] = []): void {
  const result = Bun.spawnSync(
    ["zellij", "--config-dir", configDir, "--session", sessionName, "action", action, ...args],
    { encoding: "utf-8", env: cleanEnv(cacheDir) }
  );
  if (result.exitCode !== 0) {
    throw new Error(`zellij action ${action} failed: ${result.stderr}`);
  }
}

export function deleteSession(sessionName: string): void {
  Bun.spawnSync(["zellij", "delete-session", "-f", sessionName], {
    timeout: 5000,
  });
}

export async function sleep(ms: number): Promise<void> {
  await new Promise((r) => setTimeout(r, ms));
}
