import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import {
  debugPrint,
  debugSessionPrint,
  getInfo,
  launchZellijSession,
  runPipe,
  sleep,
} from "./test-helpers";

describe("emotitle info command", () => {
  test("should return valid JSON with tabs and panes info", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await debugSessionPrint(session);

    await debugPrint("=== config.kdl ===");
    await debugPrint(() => {
      return readFileSync(`${configDir}/config.kdl`, "utf-8");
    });

    await session.press("esc");
    await sleep(200);

    const info = await getInfo(configDir, cacheDir, sessionName);

    await debugPrint("=== pipe output ===");
    await debugPrint(info);

    expect(info.tabs).toBeInstanceOf(Array);
    expect(info.tabs.length).toBeGreaterThan(0);
    expect(info.tabs[0]).toHaveProperty("position");
    expect(info.tabs[0]).toHaveProperty("name");
    expect(info.tabs[0]).toHaveProperty("active");
    expect(info.tabs[0]).toHaveProperty("panes");
    expect(info.tabs[0].panes).toBeInstanceOf(Array);
    expect(info.focused_tab_index).toBeDefined();
    expect(info.focused_pane).toBeDefined();
  }, 30000);

  test("should return ok for pane command", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await session.press("esc");
    await sleep(200);

    const output = await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌🚀",
    );
    expect(output).toBe("ok");
  }, 30000);
});
