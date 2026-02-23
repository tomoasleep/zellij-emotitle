import { describe, expect, test } from "bun:test";
import type { Session } from "tuistory";
import {
  getInfo,
  launchZellijSession,
  queryTabNames,
  runPipe,
  sleep,
  zellijAction,
} from "./test-helpers";

interface PinOptions {
  context: Context;
  target?: "pane" | "tab";
  pane_id?: string;
  tab_index?: number;
  emojis?: string;
}

interface Context {
  session: Session;
  configDir: string;
  cacheDir: string;
  sessionName: string;
}

describe("emotitle plugin (tab target)", () => {
  const setupSession = async (): Promise<Context> => {
    const zellijSession = await launchZellijSession();
    const { session } = zellijSession;

    await session.press("esc");
    await sleep(200);

    return zellijSession;
  };

  const pinEmoji = async ({
    target,
    pane_id,
    tab_index,
    emojis = "",
    context: { session, configDir, cacheDir, sessionName },
  }: PinOptions) => {
    await runPipe(
      session,
      configDir,
      cacheDir,
      sessionName,
      `${target ? `target=${target},` : ""}${pane_id !== undefined ? `pane_id=${pane_id},` : ""}${tab_index !== undefined ? `tab_index=${tab_index},` : ""}emojis=${emojis}`,
    );
    await sleep(200);
  };

  const pinEmojiToTab = async (options: PinOptions) => {
    await pinEmoji({
      target: "tab",
      ...options,
    });
  };

  describe("when the target is specified by pane_id", () => {
    type PinOptionsWithPaneId = Omit<PinOptions, "pane_id"> & {
      pane_id: string;
    };

    const pinEmojiToTabByPaneId = async (options: PinOptionsWithPaneId) => {
      await pinEmojiToTab({
        target: "tab",
        ...options,
      });
    };

    describe("when the emoji type is pinned", () => {
      const pinPinnedEmojiToTabByPaneId = async (
        options: PinOptionsWithPaneId,
      ) => {
        await pinEmojiToTabByPaneId({
          target: "tab",
          emojis: "📌🚀",
          ...options,
        });
      };

      test("should apply emojis to the tab containing the pane", async () => {
        const context = await setupSession();
        const { session, configDir, cacheDir, sessionName } = context;

        await zellijAction(configDir, cacheDir, sessionName, "new-tab");
        await sleep(100);

        const info = await getInfo(configDir, cacheDir, sessionName);
        const firstPaneId = info.tabs[0].panes[0].id;
        const firstTabName = info.tabs[0].name;

        await pinPinnedEmojiToTabByPaneId({ context, pane_id: firstPaneId });
        await sleep(100);
        const text = await session.text();
        expect(text).toContain(`${firstTabName} | 📌🚀`);
      }, 60000);
    });

    describe("when the emoji type is non-pinned", () => {
      const pinNonPinnedEmojiToTabByPaneId = async (
        options: PinOptionsWithPaneId,
      ) => {
        await pinEmoji({
          target: "tab",
          emojis: "📚",
          ...options,
        });
      };

      describe("when multiple tab is created", () => {
        const createMultipleTabs = async ({
          configDir,
          cacheDir,
          sessionName,
        }: Context) => {
          await zellijAction(configDir, cacheDir, sessionName, "new-tab");
        };

        test("should apply emojis and remove after focusing", async () => {
          const context = await setupSession();
          await createMultipleTabs(context);
          const { session, configDir, cacheDir, sessionName } = context;

          const info = await getInfo(configDir, cacheDir, sessionName);
          const firstPaneId = info.tabs[0].panes[0].id;

          await pinNonPinnedEmojiToTabByPaneId({
            context,
            pane_id: firstPaneId,
          });
          await sleep(100);
          let text = await session.text();
          expect(text).toContain("📚");

          await sleep(1000);
          text = await session.text();
          expect(text).toContain("📚");

          await zellijAction(
            configDir,
            cacheDir,
            sessionName,
            "go-to-previous-tab",
          );
          await sleep(1000);

          text = await session.text();
          expect(text).not.toContain("📚");
        }, 60000);
      });

      describe("when some tab are deleted", () => {
        const createSomeTabsWithDeleted = async ({
          configDir,
          cacheDir,
          sessionName,
        }: Context) => {
          await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
            "TAB_A",
          ]);
          await zellijAction(configDir, cacheDir, sessionName, "new-tab");
          await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
            "TAB_B",
          ]);
          await zellijAction(configDir, cacheDir, sessionName, "new-tab");
          await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
            "TAB_C",
          ]);

          await zellijAction(
            configDir,
            cacheDir,
            sessionName,
            "go-to-tab-name",
            ["TAB_B"],
          );
          await zellijAction(configDir, cacheDir, sessionName, "close-tab");
        };

        test(
          "should apply emoji and delete after focusing",
          async () => {
            const context = await setupSession();
            await createSomeTabsWithDeleted(context);
            const { configDir, cacheDir, sessionName } = context;

            const info = await getInfo(configDir, cacheDir, sessionName);
            const tabCpaneId = info.tabs[1].panes[0].id;
            const tabCName = "TAB_C";

            await zellijAction(
              configDir,
              cacheDir,
              sessionName,
              "go-to-tab-name",
              [tabCName],
            );
            await sleep(300);

            await pinNonPinnedEmojiToTabByPaneId({
              context,
              pane_id: tabCpaneId,
            });
            await sleep(300);

            let tabNames = (
              await queryTabNames(configDir, cacheDir, sessionName)
            ).split("\n");
            expect(tabNames).toContain(`${tabCName} | 📚`);
            expect(tabNames).toContain("TAB_A");

            await sleep(1300);

            tabNames = (
              await queryTabNames(configDir, cacheDir, sessionName)
            ).split("\n");
            expect(tabNames).toContain(tabCName);
            expect(tabNames).not.toContain(`${tabCName} | 📚`);
          },
          60000,
        );
      });
    });
  });

  describe("when the target is specified by tab_index", () => {
    type PinOptionsWithTabIndex = Omit<PinOptions, "tab_index"> & {
      tab_index: number;
    };

    const pinEmojiToTabByTabIndex = async (options: PinOptionsWithTabIndex) => {
      await pinEmojiToTab({
        ...options,
      });
    };

    describe("when the emoji type is pinned", () => {
      test.todo(
        "should apply emojis to the tab specified by tab_index",
        () => {},
      );
    });

    describe("when the emoji type is non-pinned", () => {
      const pinNonPinnedEmojiToTabByTabIndex = async (
        options: PinOptionsWithTabIndex,
      ) => {
        await pinEmojiToTabByTabIndex({
          emojis: "📚",
          ...options,
        });
      };

      test("should remove emojis after 1 second on focused tab", async () => {
        const context = await setupSession();
        const { session } = context;

        await pinNonPinnedEmojiToTabByTabIndex({ context, tab_index: 0 });

        let text = await session.text();
        expect(text).toContain("📚");

        await sleep(1200);

        text = await session.text();
        expect(text).not.toContain("📚");
      }, 60000);

      test("should remove emojis after tab switch", async () => {
        const context = await setupSession();
        const { session, configDir, cacheDir, sessionName } = context;

        await zellijAction(configDir, cacheDir, sessionName, "new-tab");
        await sleep(100);

        await pinNonPinnedEmojiToTabByTabIndex({ context, tab_index: 0 });
        await sleep(100);
        let text = await session.text();
        expect(text).toContain("📚");

        await sleep(1000);
        text = await session.text();
        expect(text).toContain("📚");

        await zellijAction(
          configDir,
          cacheDir,
          sessionName,
          "go-to-previous-tab",
        );
        await sleep(1000);

        text = await session.text();
        expect(text).not.toContain("📚");
      }, 60000);

      describe("when some tab are deleted", () => {
        const createSomeTabsWithDeleted = async ({
          configDir,
          cacheDir,
          sessionName,
        }: Context) => {
          await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
            "TAB_A",
          ]);
          await zellijAction(configDir, cacheDir, sessionName, "new-tab");
          await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
            "TAB_B",
          ]);
          await zellijAction(configDir, cacheDir, sessionName, "new-tab");
          await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
            "TAB_C",
          ]);
          await zellijAction(configDir, cacheDir, sessionName, "new-tab");
          await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
            "TAB_D",
          ]);

          await zellijAction(
            configDir,
            cacheDir,
            sessionName,
            "go-to-tab-name",
            ["TAB_B"],
          );
          await zellijAction(configDir, cacheDir, sessionName, "close-tab");
        };

        test(
          "should apply emoji and delete after focusing",
          async () => {
            const context = await setupSession();
            await createSomeTabsWithDeleted(context);
            const { configDir, cacheDir, sessionName } = context;

            const info = await getInfo(configDir, cacheDir, sessionName);
            const tabCpaneId = info.tabs[2].position;
            const tabCName = "TAB_D";

            await pinNonPinnedEmojiToTabByTabIndex({
              context,
              tab_index: tabCpaneId,
            });

            let tabNames = (
              await queryTabNames(configDir, cacheDir, sessionName)
            ).split("\n");
            expect(tabNames).toContain(`${tabCName} | 📚`);
            expect(tabNames).toContain("TAB_A");

            await sleep(1300);

            tabNames = (
              await queryTabNames(configDir, cacheDir, sessionName)
            ).split("\n");
            expect(tabNames).toContain(tabCName);
            expect(tabNames).not.toContain(`${tabCName} | 📚`);
          },
          60000,
        );
      });
    });
  });

  describe("when the target is not specified", () => {
    const pinEmojiToCurrentTab = async (options: PinOptions) => {
      await pinEmojiToTab({
        ...options,
      });
    };

    describe("when the emoji type is pinned", () => {
      const pinNonPinnedEmojiToCurrentTab = async (options: PinOptions) => {
        await pinEmojiToCurrentTab({
          emojis: "📌🚀",
          ...options,
        });
      };

      test("should keep pinned emojis on tab switch back", async () => {
        const context = await setupSession();
        const { session, configDir, cacheDir, sessionName } = context;

        await pinNonPinnedEmojiToCurrentTab({ context });
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
        await zellijAction(
          configDir,
          cacheDir,
          sessionName,
          "go-to-previous-tab",
        );
        await sleep(1000);

        const text = await session.text();
        expect(text).toContain("📌🚀");
        expect(text).not.toContain("📌🚀 | 📚");
        expect(text).not.toContain("📚");
      }, 60000);

      test("should not rename wrong tab when tab index changes due to insertion", async () => {
        const context = await setupSession();
        const { configDir, cacheDir, sessionName } = context;

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

        await pinNonPinnedEmojiToCurrentTab({ context });

        await zellijAction(configDir, cacheDir, sessionName, "new-tab");
        await sleep(500);
        await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
          "TAB_C",
        ]);
        await sleep(300);
        await zellijAction(configDir, cacheDir, sessionName, "move-tab", [
          "left",
        ]);
        await sleep(1300);

        const tabNames = await queryTabNames(configDir, cacheDir, sessionName);
        expect(tabNames).toContain("TAB_C");
        expect(tabNames).not.toContain("TAB_B TAB_B");
        expect(tabNames).not.toContain("TAB_B | 📌🚀 TAB_B");
      }, 60000);
    });

    describe("when the emoji type is non-pinned", () => {
      test.todo("should remove emojis on tab switch", () => {});
    });
  });

  describe("when panes are deleted from tab", () => {
    const createTabWithMultiplePanes = async ({
      configDir,
      cacheDir,
      sessionName,
    }: Context) => {
      await zellijAction(configDir, cacheDir, sessionName, "rename-tab", [
        "MULTI_PANE",
      ]);
      await zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(100);
      await zellijAction(configDir, cacheDir, sessionName, "new-pane");
      await sleep(100);
    };

    describe("when the emoji type is pinned", () => {
      test("should keep pinned emojis after pane deletion", async () => {
        const context = await setupSession();
        await createTabWithMultiplePanes(context);
        const { session, configDir, cacheDir, sessionName } = context;

        await pinEmojiToTab({ context, emojis: "📌🚀" });
        await sleep(200);

        await zellijAction(configDir, cacheDir, sessionName, "close-pane");
        await sleep(100);
        await zellijAction(configDir, cacheDir, sessionName, "close-pane");
        await sleep(300);

        const tabNames = await queryTabNames(configDir, cacheDir, sessionName);
        expect(tabNames).toContain("MULTI_PANE | 📌🚀");
      }, 60000);
    });

    describe("when the emoji type is non-pinned", () => {
      test("should apply emoji and restore after pane deletion", async () => {
        const context = await setupSession();
        await createTabWithMultiplePanes(context);
        const { session, configDir, cacheDir, sessionName } = context;

        await pinEmojiToTab({ context, emojis: "📚" });
        await sleep(200);

        let tabNames = await queryTabNames(configDir, cacheDir, sessionName);
        expect(tabNames).toContain("MULTI_PANE | 📚");

        await zellijAction(configDir, cacheDir, sessionName, "close-pane");
        await sleep(100);
        await zellijAction(configDir, cacheDir, sessionName, "close-pane");
        await sleep(1300);

        tabNames = await queryTabNames(configDir, cacheDir, sessionName);
        expect(tabNames).toContain("MULTI_PANE");
        expect(tabNames).not.toContain("MULTI_PANE | 📚");
      }, 60000);
    });
  });
});
