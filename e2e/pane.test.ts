import { describe, expect, test } from "bun:test";
import type { Session } from "tuistory";
import {
  launchZellijSession,
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
  [Symbol.dispose](): void;
}

describe("emotitle plugin (pane target)", () => {
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
      `${target ? `target=${target},` : ""}${pane_id ? `pane_id=${pane_id},` : ""}${tab_index !== undefined ? `tab_index=${tab_index},` : ""}emojis=${emojis}`,
    );
    await sleep(200);
  };

  const pinEmojiToPane = async (options: PinOptions) => {
    await pinEmoji({
      target: "pane",
      ...options,
    });
  };

  describe("when the target is specified by pane_id", () => {
    describe("when the emoji type is pinned", () => {
      test.todo(
        "should apply emojis to the pane specified by pane_id",
        () => {},
      );
    });

    describe("when the emoji type is non-pinned", () => {
      test.todo(
        "should apply emojis to the pane specified by pane_id",
        () => {},
      );
    });
  });

  describe("when the target is not specified", () => {
    const pinEmojiToCurrent = async (options: PinOptions) => {
      await pinEmojiToPane({
        ...options,
      });
    };

    describe("when the emoji type is pinned", () => {
      const pinPinnedEmoji = async (options: PinOptions) => {
        await pinEmojiToCurrent({
          emojis: "📌🚀",
          ...options,
        });
      };

      test("should apply emojis to focused pane", async () => {
        const context = await setupSession();
        await pinPinnedEmoji({ context });
        const { session } = context;

        const text = await session.text();
        expect(text).toContain("📌🚀");
      }, 30000);

      test("should keep pinned emojis on focus regain", async () => {
        const context = await setupSession();
        await pinPinnedEmoji({ context });
        const { session, configDir, cacheDir, sessionName } = context;

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
        await zellijAction(
          configDir,
          cacheDir,
          sessionName,
          "focus-previous-pane",
        );
        await sleep(1000);

        const text = await session.text();
        expect(text).toContain("📌🚀");
        expect(text).not.toContain("📌🚀 | 📚");
        expect(text).not.toContain("📚");
      }, 60000);

      test("should keep other pane emojis although pane is deleted", async () => {
        const context = await setupSession();
        await pinPinnedEmoji({ context });
        const { session, configDir, cacheDir, sessionName } = context;

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

        await zellijAction(
          configDir,
          cacheDir,
          sessionName,
          "focus-previous-pane",
        );
        await sleep(500);

        text = await session.text();
        expect(text).toContain("📌🚀");
      }, 45000);

      test("should not carry emojis to new pane when pane is deleted", async () => {
        const context = await setupSession();
        await pinPinnedEmoji({ context });
        const { session, configDir, cacheDir, sessionName } = context;

        let text = await session.text();
        expect(text).toContain("📌🚀");

        await zellijAction(configDir, cacheDir, sessionName, "new-pane");
        await sleep(500);

        await zellijAction(
          configDir,
          cacheDir,
          sessionName,
          "focus-previous-pane",
        );
        await sleep(300);

        await zellijAction(configDir, cacheDir, sessionName, "close-pane");
        await sleep(500);

        await zellijAction(configDir, cacheDir, sessionName, "new-pane");
        await sleep(500);

        text = await session.text();
        expect(text).not.toContain("📌🚀");
      }, 30000);
    });

    describe("when multiple emojis with pinned emojis are set", () => {
      const pinMultipleEmojis = async (options: PinOptions) => {
        await pinEmojiToCurrent({
          emojis: "📌🚀 | 📚 | 🚗",
          ...options,
        });
      };

      test("should keep only pinned segments", async () => {
        const context = await setupSession();
        await pinMultipleEmojis({ context });
        const { session, configDir, cacheDir, sessionName } = context;

        let text = await session.text();
        expect(text).toContain("📌🚀");

        await zellijAction(configDir, cacheDir, sessionName, "new-pane");
        await sleep(500);
        await zellijAction(
          configDir,
          cacheDir,
          sessionName,
          "focus-previous-pane",
        );
        await sleep(1700);

        text = await session.text();
        expect(text).toContain("📌🚀");
        expect(text).not.toContain("📌🚀 | 📚");
        expect(text).not.toContain("📚");
        expect(text).not.toContain("🚗");
      }, 60000);
    });

    describe("when the emoji type is non-pinned", () => {
      const pinNonPinnedEmoji = async (options: PinOptions) => {
        await pinEmojiToCurrent({
          emojis: "📚️",
          ...options,
        });
      };

      test("should remove emojis after 1 second on focus change", async () => {
        const context = await setupSession();
        const { session, configDir, cacheDir, sessionName } = context;

        await zellijAction(configDir, cacheDir, sessionName, "new-pane");
        await sleep(500);

        await pinNonPinnedEmoji({ context });
        await zellijAction(
          configDir,
          cacheDir,
          sessionName,
          "focus-previous-pane",
        );
        await sleep(100);

        let text = await session.text();
        expect(text).toContain("📚");
        await sleep(1200);

        text = await session.text();
        expect(text).not.toContain("📚");
      }, 60000);

      test("should not resurrect deleted emojis on setting new emojis", async () => {
        const context = await setupSession();
        await pinNonPinnedEmoji({ context });
        const { session, configDir, cacheDir, sessionName } = context;

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
  });
});
