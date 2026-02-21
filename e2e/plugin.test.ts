import { describe, expect, test } from "bun:test";

import {
  getInfo,
  launchZellijSession,
  queryTabNames,
  runPipe,
  sleep,
  zellijAction,
} from "./test-helpers";

describe("emotitle plugin e2e", () => {
  test("should apply emojis to the focused pane via pipe", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await session.press("esc");
    await sleep(200);

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌🚀",
    );
    await sleep(500);

    const text = await session.text();
    expect(text).toContain("📌🚀");
  }, 30000);

  test("should not carry pane emojis to a newly created pane after deletion", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌📚",
    );
    await sleep(500);

    let text = await session.text();
    expect(text).toContain("📌📚");

    await zellijAction(configDir, cacheDir, sessionName, "new-pane");
    await sleep(500);

    await zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
    await sleep(300);

    await zellijAction(configDir, cacheDir, sessionName, "close-pane");
    await sleep(500);

    await zellijAction(configDir, cacheDir, sessionName, "new-pane");
    await sleep(500);

    text = await session.text();
    expect(text).not.toContain("📌📚");
  }, 30000);

  test("should keep other pane emojis after deleting the current pane", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌✅",
    );
    await sleep(500);

    await zellijAction(configDir, cacheDir, sessionName, "new-pane");
    await sleep(500);

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌🎉",
    );
    await sleep(500);

    let text = await session.text();
    expect(text).toContain("📌🎉");

    await zellijAction(configDir, cacheDir, sessionName, "close-pane");
    await sleep(500);

    await zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
    await sleep(500);

    text = await session.text();
    expect(text).toContain("📌✅");
  }, 45000);

  test("should keep only pinned segments on focus after stacked emojis", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌🚀 | 📚 | 🚗",
    );
    await sleep(500);

    let text = await session.text();
    expect(text).toContain("📌🚀");

    await zellijAction(configDir, cacheDir, sessionName, "new-pane");
    await sleep(500);
    await zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
    await sleep(1700);

    text = await session.text();
    expect(text).toContain("📌🚀");
    expect(text).not.toContain("📌🚀 | 📚");
    expect(text).not.toContain("📚");
    expect(text).not.toContain("🚗");
  }, 60000);

  test("should keep pinned emojis on focus after two consecutive pane pipes", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌🚀",
    );
    await sleep(200);
    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📚",
    );
    await sleep(400);

    await zellijAction(configDir, cacheDir, sessionName, "new-pane");
    await sleep(500);
    await zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
    await sleep(1000);

    const text = await session.text();
    expect(text).toContain("📌🚀");
    expect(text).not.toContain("📌🚀 | 📚");
    expect(text).not.toContain("📚");
  }, 60000);

  test("should remove non-pinned pane emojis on focus", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📚",
    );
    await sleep(400);

    await zellijAction(configDir, cacheDir, sessionName, "new-pane");
    await sleep(500);
    await zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
    await sleep(1200);

    const text = await session.text();
    expect(text).not.toContain("📚");
  }, 60000);

  test("should keep non-pinned pane emojis before one second and remove after one second", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=🛼",
    );
    await sleep(100);

    let text = await session.text();
    expect(text).toContain("🛼");

    await sleep(1500);

    text = await session.text();
    expect(text).not.toContain("🛼");
  }, 60000);

  test("should keep all pinned pane segments and remove non-pinned on focus", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📌🚀 | 📌✅ | 📚",
    );
    await sleep(400);

    await zellijAction(configDir, cacheDir, sessionName, "new-pane");
    await sleep(500);
    await zellijAction(configDir, cacheDir, sessionName, "focus-previous-pane");
    await sleep(1000);

    const text = await session.text();
    expect(text).toContain("📌🚀 | 📌✅");
    expect(text).not.toContain("📚");
  }, 60000);

  test("should remove non-pinned tab emojis after tab switch", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=tab,tab_index=0,emojis=📚",
    );
    await sleep(400);

    await zellijAction(configDir, cacheDir, sessionName, "new-tab");
    await sleep(700);
    await zellijAction(configDir, cacheDir, sessionName, "go-to-previous-tab");
    await sleep(1000);

    const text = await session.text();
    expect(text).not.toContain("📚");
  }, 60000);

  test("should keep pinned tab emojis on focused tab after focus", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=tab,emojis=📌🚀",
    );
    await sleep(200);
    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=tab,emojis=📚",
    );
    await sleep(400);

    await zellijAction(configDir, cacheDir, sessionName, "new-tab");
    await sleep(700);
    await zellijAction(configDir, cacheDir, sessionName, "go-to-previous-tab");
    await sleep(1000);

    const text = await session.text();
    expect(text).toContain("📌🚀");
    expect(text).not.toContain("📌🚀 | 📚");
    expect(text).not.toContain("📚");
  }, 60000);

  test("should not rename inserted tab after tab insertion shifts tracked tab index", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
      "TAB_A",
    ]);
    await sleep(300);

    await zellijAction(configDir, cacheDir, sessionName, "new-tab");
    await sleep(700);
    await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
      "TAB_B",
    ]);
    await sleep(300);

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=tab,emojis=📌📚",
    );
    await sleep(300);

    await zellijAction(configDir, cacheDir, sessionName, "new-tab");
    await sleep(500);
    await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
      "TAB_C",
    ]);
    await sleep(300);
    await zellijAction(configDir, cacheDir, sessionName, "move-tab", ["left"]);
    await sleep(1300);

    const tabNames = await queryTabNames(configDir, cacheDir, sessionName);
    expect(tabNames).toContain("TAB_C");
    expect(tabNames).not.toContain("TAB_B TAB_B");
    expect(tabNames).not.toContain("TAB_B | 📌📚 TAB_B");
  }, 60000);

  test("should keep tab names aligned after deleting tab before tracked tab", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
      "TAB_A",
    ]);
    await sleep(300);

    await zellijAction(configDir, cacheDir, sessionName, "new-tab");
    await sleep(700);
    await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
      "TAB_B",
    ]);
    await sleep(300);

    await zellijAction(configDir, cacheDir, sessionName, "new-tab");
    await sleep(700);
    await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
      "TAB_C",
    ]);
    await sleep(300);

    const info = await getInfo(configDir, cacheDir, sessionName);

    await zellijAction(configDir, cacheDir, sessionName, "go-to-tab-name", [
      "TAB_B",
    ]);
    await sleep(300);
    await zellijAction(configDir, cacheDir, sessionName, "close-tab");
    await sleep(700);

    await zellijAction(configDir, cacheDir, sessionName, "go-to-tab-name", [
      "TAB_C",
    ]);
    await sleep(300);
    const targetTabName = `TAB_C_ACTIVE_${Date.now()}`;
    await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
      targetTabName,
    ]);
    await sleep(300);

    const tabCpaneId = info.tabs[2].panes[0].id;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      `target=tab,pane_id=${tabCpaneId},emojis=📚`,
    );
    await sleep(300);

    let tabNames = (
      await queryTabNames(configDir, cacheDir, sessionName)
    ).split("\n");
    expect(tabNames).toContain(`${targetTabName} | 📚`);
    expect(tabNames).toContain("TAB_A");

    await sleep(1300);

    tabNames = (await queryTabNames(configDir, cacheDir, sessionName)).split(
      "\n",
    );
    expect(tabNames).toContain(targetTabName);
    expect(tabNames).not.toContain(`${targetTabName} | 📚`);

    await sleep(1300);

    tabNames = (await queryTabNames(configDir, cacheDir, sessionName)).split(
      "\n",
    );
    expect(tabNames).toContain(targetTabName);
    expect(tabNames).not.toContain(`${targetTabName} | 📚`);
  }, 60000);

  test("should keep non-pinned tab emojis before one second and remove after one second", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=tab,tab_index=0,emojis=🛼",
    );
    await sleep(300);

    let text = await session.text();
    expect(text).toContain("🛼");

    await sleep(1200);

    text = await session.text();
    expect(text).not.toContain("🛼");
  }, 60000);

  test("should not resurrect previous temp emojis after timer cleanup on focused pane", async () => {
    await using zellijSession = await launchZellijSession();
    const { session, configDir, cacheDir, sessionName } = zellijSession;

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=📚",
    );
    await sleep(1300);

    let text = await session.text();
    expect(text).not.toContain("📚");

    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      "target=pane,emojis=✅",
    );
    await sleep(300);

    text = await session.text();
    expect(text).toContain("✅");
    expect(text).not.toContain("📚");
  }, 60000);
});
