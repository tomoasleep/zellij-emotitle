import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import {
  debugPrint,
  debugSessionPrint,
  getInfo,
  launchZellijSession,
  runPipe,
  sleep,
  zellijAction,
} from "./test-helpers";

describe("emotitle info command", () => {
  test("should return valid JSON with tabs and panes info", async () => {
    using zellijSession = await launchZellijSession();
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
    using zellijSession = await launchZellijSession();
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

  describe("event_history", () => {
    test("should record TabAdded event when tab is created", async () => {
      using zellijSession = await launchZellijSession();
      const { session, configDir, cacheDir, sessionName } = zellijSession;

      await session.press("esc");
      await sleep(200);

      const info = await getInfo(configDir, cacheDir, sessionName);

      expect(info.event_history).toBeInstanceOf(Array);
      expect(info.event_history.length).toBeGreaterThan(0);
      expect(info.event_history[0]).toHaveProperty("seq");
      expect(info.event_history[0]).toHaveProperty("event_type");
      expect(info.event_history[0]).toHaveProperty("pane_keys");
      expect(info.event_history[0]).toHaveProperty("internal_index");
    }, 30000);

    test("should record TabAdded events for multiple tabs", async () => {
      using zellijSession = await launchZellijSession();
      const { session, configDir, cacheDir, sessionName } = zellijSession;

      await session.press("esc");
      await sleep(200);

      await zellijAction(configDir, cacheDir, sessionName, "new-tab");
      await sleep(100);

      const info = await getInfo(configDir, cacheDir, sessionName);

      const addedEvents = info.event_history.filter(
        (e: { event_type: string }) => e.event_type === "TabAdded",
      );
      expect(addedEvents.length).toBeGreaterThanOrEqual(2);
    }, 30000);

    test("should record TabRemoved event when tab is closed", async () => {
      using zellijSession = await launchZellijSession();
      const { session, configDir, cacheDir, sessionName } = zellijSession;

      await session.press("esc");
      await sleep(200);

      await zellijAction(configDir, cacheDir, sessionName, "new-tab");
      await sleep(100);
      await zellijAction(configDir, cacheDir, sessionName, "close-tab");
      await sleep(200);

      const info = await getInfo(configDir, cacheDir, sessionName);

      const removedEvents = info.event_history.filter(
        (e: { event_type: string }) => e.event_type === "TabRemoved",
      );
      expect(removedEvents.length).toBeGreaterThanOrEqual(1);
    }, 30000);

    test("should record TabKeyUpdated event when pane is closed", async () => {
      using zellijSession = await launchZellijSession();
      const { session, configDir, cacheDir, sessionName } = zellijSession;

      await session.press("esc");
      await sleep(200);

      await zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(100);
      await zellijAction(configDir, cacheDir, sessionName, "close-pane");
      await sleep(200);

      const info = await getInfo(configDir, cacheDir, sessionName);

      const updatedEvents = info.event_history.filter(
        (e: { event_type: string }) => e.event_type === "TabKeyUpdated",
      );
      expect(updatedEvents.length).toBeGreaterThanOrEqual(1);
    }, 30000);

    test("should maintain sequential event order", async () => {
      using zellijSession = await launchZellijSession();
      const { session, configDir, cacheDir, sessionName } = zellijSession;

      await session.press("esc");
      await sleep(200);

      await zellijAction(configDir, cacheDir, sessionName, "new-tab");
      await sleep(100);
      await zellijAction(configDir, cacheDir, sessionName, "close-tab");
      await sleep(200);

      const info = await getInfo(configDir, cacheDir, sessionName);

      for (let i = 1; i < info.event_history.length; i++) {
        expect(info.event_history[i].seq).toBeGreaterThan(
          info.event_history[i - 1].seq,
        );
      }
    }, 30000);

    test("should limit event history to 200 entries", async () => {
      using zellijSession = await launchZellijSession();
      const { session, configDir, cacheDir, sessionName } = zellijSession;

      await session.press("esc");
      await sleep(200);

      for (let i = 0; i < 50; i++) {
        await zellijAction(configDir, cacheDir, sessionName, "new-pane");
        await sleep(50);
        await zellijAction(configDir, cacheDir, sessionName, "close-pane");
        await sleep(50);
      }

      const info = await getInfo(configDir, cacheDir, sessionName);

      expect(info.event_history.length).toBeLessThanOrEqual(200);
    }, 120000);
  });
});
