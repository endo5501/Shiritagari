import { describe, test, expect, vi, afterEach } from "vitest";
import { render, screen, cleanup, act } from "@testing-library/react";
import { fireEvent } from "@testing-library/react";
import App from "./App";

// Mock Tauri APIs
const mockInvoke = vi.fn().mockResolvedValue("mock response");
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}));

const { mockStartDragging } = vi.hoisted(() => ({
  mockStartDragging: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    startDragging: mockStartDragging,
  }),
}));

const listeners: Record<string, (event: { payload: unknown }) => void> = {};
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((eventName: string, callback: (event: { payload: unknown }) => void) => {
    listeners[eventName] = callback;
    return Promise.resolve(() => {
      delete listeners[eventName];
    });
  }),
}));

async function renderApp() {
  await act(async () => {
    render(<App />);
  });
}

function emitEvent(name: string, payload: unknown) {
  act(() => {
    listeners[name]?.({ payload });
  });
}

describe("Mascot UI", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    Object.keys(listeners).forEach((k) => delete listeners[k]);
  });

  test("renders character image and triggers drag on mousedown", async () => {
    await renderApp();
    const img = screen.getByAltText("mascot");
    expect(img).toBeInTheDocument();
    fireEvent.mouseDown(img.parentElement!);
    expect(mockStartDragging).toHaveBeenCalled();
  });

  test("uses custom character_image from config when available", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_mascot_config") {
        return Promise.resolve({ character_image: "/custom/char.png" });
      }
      return Promise.resolve("mock response");
    });
    await renderApp();
    const img = screen.getByAltText("mascot") as HTMLImageElement;
    expect(img.src).toContain("/custom/char.png");
  });

  test("uses default mascot image when character_image is null", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_mascot_config") {
        return Promise.resolve({ character_image: null });
      }
      return Promise.resolve("mock response");
    });
    await renderApp();
    const img = screen.getByAltText("mascot") as HTMLImageElement;
    expect(img.src).toContain("/default-mascot.png");
  });

  test("renders input area with send button", async () => {
    await renderApp();
    const input = screen.getByPlaceholderText("メッセージを入力...");
    expect(input).toBeInTheDocument();
    const button = screen.getByRole("button", { name: "送信" });
    expect(button).toBeInTheDocument();
  });

  test("does not show bubble when no thought received", async () => {
    await renderApp();
    const bubble = screen.queryByTestId("bubble");
    expect(bubble).not.toBeInTheDocument();
  });

  test("shows thought bubble when shiritagari-thought event received", async () => {
    await renderApp();
    emitEvent("shiritagari-thought", {
      inference: "コード書いてるな...",
      confidence: 0.7,
    });
    const bubble = screen.getByTestId("bubble");
    expect(bubble).toBeInTheDocument();
    expect(bubble).toHaveTextContent("コード書いてるな...");
    expect(bubble).toHaveClass("thought");
  });

  test("shows speech bubble when shiritagari-question event received", async () => {
    await renderApp();
    emitEvent("shiritagari-question", "これは何の作業ですか？");
    const bubble = screen.getByTestId("bubble");
    expect(bubble).toBeInTheDocument();
    expect(bubble).toHaveTextContent("これは何の作業ですか？");
    expect(bubble).toHaveClass("speech");
  });

  test("switches to answer mode when question received", async () => {
    await renderApp();
    emitEvent("shiritagari-question", "何をしていますか？");
    const input = screen.getByPlaceholderText("質問に回答...");
    expect(input).toBeInTheDocument();
    const button = screen.getByRole("button", { name: "回答" });
    expect(button).toBeInTheDocument();
  });

  test("sends answer and returns to thought mode", async () => {
    await renderApp();

    // Receive a question
    emitEvent("shiritagari-question", "何をしていますか？");

    const input = screen.getByPlaceholderText("質問に回答...");
    fireEvent.change(input, { target: { value: "コーディングです" } });

    await act(async () => {
      fireEvent.keyDown(input, { key: "Enter", isComposing: false });
    });

    expect(mockInvoke).toHaveBeenCalledWith("answer_question", {
      answer: "コーディングです",
      questionContext: "何をしていますか？",
    });

    // Should return to normal mode
    const normalInput = screen.getByPlaceholderText("メッセージを入力...");
    expect(normalInput).toBeInTheDocument();
  });

  test("shows error in thought bubble when send_message fails", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_mascot_config") {
        return Promise.resolve({ character_image: null });
      }
      if (cmd === "send_message") {
        return Promise.reject("不明なコマンドです: /unknown。/help で利用可能なコマンドを確認してください");
      }
      return Promise.resolve("mock response");
    });
    await renderApp();
    const textarea = screen.getByPlaceholderText("メッセージを入力...");
    fireEvent.change(textarea, { target: { value: "/unknown" } });

    await act(async () => {
      fireEvent.keyDown(textarea, { key: "Enter", isComposing: false });
    });

    const bubble = screen.getByTestId("bubble");
    expect(bubble).toBeInTheDocument();
    expect(bubble).toHaveTextContent("不明なコマンドです");
    expect(bubble).toHaveClass("thought");
  });

  test("IME: Enter during composition should NOT send", async () => {
    await renderApp();
    const textarea = screen.getByPlaceholderText("メッセージを入力...");

    fireEvent.change(textarea, { target: { value: "にほんご" } });
    fireEvent.keyDown(textarea, {
      key: "Enter",
      code: "Enter",
      nativeEvent: { isComposing: true },
      isComposing: true,
    });

    expect(textarea).toHaveValue("にほんご");
  });

  test("IME: Enter after compositionend guard should send", async () => {
    vi.useFakeTimers();
    await renderApp();
    const textarea = screen.getByPlaceholderText("メッセージを入力...");

    fireEvent.compositionStart(textarea);
    fireEvent.change(textarea, { target: { value: "こんにちは" } });
    fireEvent.compositionEnd(textarea);
    vi.advanceTimersByTime(50);

    fireEvent.keyDown(textarea, {
      key: "Enter",
      code: "Enter",
      isComposing: false,
    });

    expect(textarea).toHaveValue("");
    vi.useRealTimers();
  });
});
