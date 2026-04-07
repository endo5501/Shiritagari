import { describe, test, expect, vi, afterEach, beforeEach } from "vitest";
import { render, screen, cleanup, act } from "@testing-library/react";
import { fireEvent } from "@testing-library/react";
import SettingsApp from "./SettingsApp";

const mockClose = vi.fn();
const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    close: mockClose,
  }),
}));

const defaultConfig = {
  polling: { interval_minutes: 10 },
  llm: {
    provider: "ollama",
    model: null,
    api_key_env: null,
    inference_provider: null,
    inference_model: null,
    inference_api_key_env: null,
    chat_provider: null,
    chat_model: null,
    chat_api_key_env: null,
    ollama_base_url: null,
    openai_base_url: null,
  },
  privacy: {
    allowlist_apps: [],
    blocklist_apps: ["Signal"],
    redaction_patterns: [],
  },
  confidence: {
    decay_rate: 0.99,
    threshold_silent: 0.8,
    threshold_re_ask: 0.5,
    threshold_soft_delete: 0.3,
  },
  mascot: { character_image: null },
};

function setupMocks(overrides?: Partial<typeof defaultConfig>) {
  const config = { ...defaultConfig, ...overrides };
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_config") return Promise.resolve(config);
    if (cmd === "list_ollama_models")
      return Promise.resolve([{ name: "llama3" }, { name: "mistral" }]);
    if (cmd === "save_config") return Promise.resolve();
    return Promise.resolve();
  });
}

async function renderSettings() {
  await act(async () => {
    render(<SettingsApp />);
  });
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("SettingsApp", () => {
  describe("基本: 読み込み、保存、キャンセル", () => {
    beforeEach(() => setupMocks());

    test("設定を読み込んで表示する", async () => {
      await renderSettings();

      expect(mockInvoke).toHaveBeenCalledWith("get_config");
      expect(screen.getByText("Settings")).toBeInTheDocument();
      expect(screen.getByText("LLM")).toBeInTheDocument();
      expect(screen.getByText("監視")).toBeInTheDocument();
      expect(screen.getByText("外観")).toBeInTheDocument();
    });

    test("保存ボタンでsave_configを呼びウィンドウを閉じる", async () => {
      await renderSettings();

      await act(async () => {
        fireEvent.click(screen.getByText("保存"));
      });

      expect(mockInvoke).toHaveBeenCalledWith("save_config", {
        newConfig: expect.objectContaining({
          llm: expect.objectContaining({ provider: "ollama" }),
        }),
      });
      expect(mockClose).toHaveBeenCalled();
    });

    test("キャンセルボタンでウィンドウを閉じる", async () => {
      await renderSettings();

      fireEvent.click(screen.getByText("キャンセル"));
      expect(mockClose).toHaveBeenCalled();
    });

    test("既存のブロックリストが表示される", async () => {
      await renderSettings();
      expect(screen.getByText("Signal")).toBeInTheDocument();
    });
  });

  describe("プロバイダ切り替え", () => {
    beforeEach(() => setupMocks());

    test("Ollama選択時にBase URLとモデルドロップダウンが表示される", async () => {
      await renderSettings();

      const providerSelect = screen.getByDisplayValue("ollama");
      expect(providerSelect).toBeInTheDocument();

      // Ollama shows base URL input and model dropdown
      expect(screen.getByPlaceholderText("http://localhost:11434")).toBeInTheDocument();
    });

    test("Claude選択時にAPI Key環境変数名とModelが表示される", async () => {
      await renderSettings();

      await act(async () => {
        fireEvent.change(screen.getByDisplayValue("ollama"), {
          target: { value: "claude" },
        });
      });

      expect(screen.getByPlaceholderText("ANTHROPIC_API_KEY")).toBeInTheDocument();
      expect(screen.getByPlaceholderText("claude-sonnet-4-20250514")).toBeInTheDocument();
    });

    test("OpenAI選択時にBase URL, API Key, Modelが表示される", async () => {
      await renderSettings();

      await act(async () => {
        fireEvent.change(screen.getByDisplayValue("ollama"), {
          target: { value: "openai" },
        });
      });

      expect(screen.getByPlaceholderText("https://api.openai.com")).toBeInTheDocument();
      expect(screen.getByPlaceholderText("OPENAI_API_KEY")).toBeInTheDocument();
      expect(screen.getByPlaceholderText("gpt-4o-mini")).toBeInTheDocument();
    });
  });

  describe("Ollamaモデル一覧", () => {
    test("モデル一覧を取得してドロップダウンに表示する", async () => {
      setupMocks();
      await renderSettings();

      expect(mockInvoke).toHaveBeenCalledWith("list_ollama_models", {
        baseUrl: null,
      });

      // Models should be in dropdown
      const options = screen.getAllByRole("option");
      const optionTexts = options.map((o) => o.textContent);
      expect(optionTexts).toContain("llama3");
      expect(optionTexts).toContain("mistral");
    });

    test("更新ボタンでモデル一覧を再取得する", async () => {
      setupMocks();
      await renderSettings();

      mockInvoke.mockClear();

      await act(async () => {
        fireEvent.click(screen.getByTitle("モデル一覧を更新"));
      });

      expect(mockInvoke).toHaveBeenCalledWith("list_ollama_models", {
        baseUrl: null,
      });
    });

    test("Ollama接続失敗時に手動入力フィールドにフォールバック", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "get_config") return Promise.resolve(defaultConfig);
        if (cmd === "list_ollama_models")
          return Promise.reject("Failed to connect to Ollama");
        return Promise.resolve();
      });

      await renderSettings();

      expect(screen.getByPlaceholderText("モデル名を入力")).toBeInTheDocument();
      expect(screen.getByText(/Failed to connect/)).toBeInTheDocument();
    });
  });
});
