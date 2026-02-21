import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { $ } from "bun";
import { launchTerminal, type Session } from "tuistory";

export const WASM_PATH = `${process.cwd()}/../target/wasm32-wasip1/release/zellij-emotitle.wasm`;
const PIPE_PAYLOAD = "_";

type SetupConfigOptions = {
  wasmPath?: string;
  simplifiedUi?: boolean;
  showStartupTips?: boolean;
};

type SetupCacheOptions = {
  wasmPath?: string;
};

export function setupConfigDir(options: SetupConfigOptions = {}): string {
  const configDir = join(
    tmpdir(),
    `zellij-test-${Date.now()}-${Math.random().toString(36).slice(2)}`,
  );
  mkdirSync(configDir, { recursive: true });
  mkdirSync(join(configDir, "layouts"), { recursive: true });

  const plugins = `
plugins {
  emotitle location="file:${options.wasmPath}"
}
`;

  const loadPlugins = options.wasmPath
    ? `
load_plugins {
  emotitle
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
  const startupTips =
    options.showStartupTips === false ? "\nshow_startup_tips false\n" : "";

  writeFileSync(
    join(configDir, "config.kdl"),
    `
keybinds {
  normal {}
}
${plugins}
${loadPlugins}
${uiConfig}
${startupTips}
  `,
  );
  writeFileSync(
    join(configDir, "layouts", "default.kdl"),
    `
  layout {
    pane size=1 split_direction="Vertical" borderless=true {
        plugin location="tab-bar"
    }
    pane
    pane size=1 borderless=true {
        plugin location="status-bar"
    }
  }
  `,
  );

  return configDir;
}

function setupCacheDir(options: SetupCacheOptions = {}): string {
  const cacheDir = join(
    tmpdir(),
    `zellij-cache-${Date.now()}-${Math.random().toString(36).slice(2)}`,
  );
  mkdirSync(cacheDir, { recursive: true });

  if (options.wasmPath) {
    writeFileSync(
      join(cacheDir, "permissions.kdl"),
      `"${options.wasmPath}" {
    ChangeApplicationState
    ReadApplicationState
    ReadCliPipes
}
`,
    );
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
  env.ZELLIJ_CACHE_DIR = cacheDir;
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

export async function launchZellijSession() {
  const configDir = setupConfigDir({
    wasmPath: WASM_PATH,
    simplifiedUi: true,
    showStartupTips: false,
  });
  const cacheDir = setupCacheDir({ wasmPath: WASM_PATH });
  const sessionName = `emotitle-test-${Date.now()}`;

  await debugPrint("=== LaunchTerminal");
  const session = await launchTerminal({
    command: "bash",
    args: [],
    cols: 140,
    rows: 35,
    env: cleanEnv(cacheDir),
  });
  await debugPrint("=== Done LaunchTerminal");

  await session.type("unset ZELLIJ ZELLIJ_PANE_ID ZELLIJ_SESSION_NAME");
  await session.press("enter");
  await sleep(100);

  await session.type(`export ZELLIJ_CACHE_DIR=${cacheDir}`);
  await session.press("enter");
  await sleep(100);
  await debugSessionPrint(session);

  await debugPrint("=== Launch zellij session");
  await session.type(
    `zellij --config-dir ${configDir} -s ${sessionName} options --simplified-ui true`,
  );
  await session.press("enter");
  await session.waitForText(`Zellij (${sessionName})`, { timeout: 5000 });
  await debugPrint("=== Done launching zellij session");
  await debugSessionPrint(session);

  const sessionText = await session.text();
  await debugPrint(sessionText);
  if (sessionText.includes("Allow? (y/n)")) {
    await debugPrint("=== Allow permission for plugin");
    await session.type("y");
    await sleep(100);
    await debugPrint("=== Done allowing permission");
  }

  return {
    session,
    configDir,
    cacheDir,
    sessionName,

    [Symbol.dispose]() {
      deleteSession(sessionName);
      session.close();
      cleanupConfigDir(configDir);
      cleanupCacheDir(cacheDir);
    },
  };
}

export async function zellijAction(
  configDir: string,
  cacheDir: string,
  sessionName: string,
  action: string,
  args: string[] = [],
) {
  await debugPrint(`=== Running zellij action: ${action} ${args.join(" ")}`);
  return $`zellij --config-dir ${configDir} --session ${sessionName} action ${action} ${args.join(" ")}`
    .env(cleanEnv(cacheDir))
    .throws(true)
    .text();
}

export async function queryTabNames(
  configDir: string,
  cacheDir: string,
  sessionName: string,
): Promise<string> {
  await debugPrint(`=== Querying tab names for session ${sessionName}`);
  return $`zellij --config-dir ${configDir} --session ${sessionName} action query-tab-names`
    .env(cleanEnv(cacheDir))
    .throws(true)
    .text();
}

export async function deleteSession(sessionName: string) {
  return $`zellij delete-session -f ${sessionName}`.throws(false).text();
}

export async function sleep(ms: number): Promise<void> {
  await new Promise((r) => setTimeout(r, ms));
}

export async function debugPrint(
  text: string | (() => string | Promise<string>),
) {
  if (process.env.DEBUG) {
    if (typeof text === "function") {
      const textResult = await text();
      console.error(textResult);
    } else {
      console.error(text);
    }
  }
}

export async function debugSessionPrint(session: Session) {
  if (process.env.DEBUG) {
    const sessionText = await session.text();
    console.error(sessionText);
  }
}

export async function runPipe(
  session: Session,
  configDir: string,
  cacheDir: string,
  sessionName: string,
  args: string,
): Promise<string> {
  await debugPrint(`=== Running zellij pipe with args: ${args}`);
  const output =
    await $`zellij --config-dir ${configDir} --session ${sessionName} pipe --name emotitle --plugin emotitle --args ${args} -- ${PIPE_PAYLOAD}`
      .env(cleanEnv(cacheDir))
      .throws(true)
      .text();
  await debugPrint(`=== Done running zellij pipe with args: ${args}`);
  await debugSessionPrint(session);

  if (output.length > 0 && output !== "ok") {
    throw new Error(`zellij pipe returned plugin error: ${output}`);
  }

  return output;
}

export async function getInfo(
  configDir: string,
  cacheDir: string,
  sessionName: string,
) {
  const output =
    await $`zellij --config-dir ${configDir} --session ${sessionName} pipe --name emotitle --plugin emotitle --args info=true -- ${PIPE_PAYLOAD}`
      .env(cleanEnv(cacheDir))
      .throws(true)
      .text();

  const info = JSON.parse(output);

  await debugPrint("=== Fetched info ===");
  await debugPrint(JSON.stringify(info, null, 2));

  return info;
}
