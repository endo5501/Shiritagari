import { describe, test, expect, vi, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import { fireEvent } from "@testing-library/react";
import App from "./App";

// Mock Tauri APIs
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue("mock response"),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe("Chat input IME handling", () => {
  afterEach(() => {
    cleanup();
  });

  test("Enter during IME composition (isComposing=true) should NOT send", () => {
    render(<App />);
    const textarea = screen.getByPlaceholderText("メッセージを入力...");

    fireEvent.change(textarea, { target: { value: "にほんご" } });

    // Firefox-style: keydown fires while still composing
    fireEvent.keyDown(textarea, {
      key: "Enter",
      code: "Enter",
      nativeEvent: { isComposing: true },
      isComposing: true,
    });

    expect(textarea).toHaveValue("にほんご");
  });

  test("Enter immediately after compositionend should NOT send (Chrome/WebKit)", () => {
    render(<App />);
    const textarea = screen.getByPlaceholderText("メッセージを入力...");

    // User types with IME
    fireEvent.compositionStart(textarea);
    fireEvent.change(textarea, { target: { value: "あなたは" } });

    // Chrome/WebKit event order: compositionend fires BEFORE keydown
    fireEvent.compositionEnd(textarea);
    fireEvent.keyDown(textarea, {
      key: "Enter",
      code: "Enter",
      isComposing: false,
    });

    // Should NOT send — this Enter was for confirming IME, not sending
    expect(textarea).toHaveValue("あなたは");
  });

  test("Enter well after composition ends should send the message", async () => {
    vi.useFakeTimers();
    render(<App />);
    const textarea = screen.getByPlaceholderText("メッセージを入力...");

    // Complete an IME composition
    fireEvent.compositionStart(textarea);
    fireEvent.change(textarea, { target: { value: "こんにちは" } });
    fireEvent.compositionEnd(textarea);

    // Wait for the guard period to expire
    vi.advanceTimersByTime(50);

    // Now press Enter — this is a deliberate send action
    fireEvent.keyDown(textarea, {
      key: "Enter",
      code: "Enter",
      isComposing: false,
    });

    expect(textarea).toHaveValue("");
    vi.useRealTimers();
  });

  test("Enter without any IME composition should send the message", () => {
    render(<App />);
    const textarea = screen.getByPlaceholderText("メッセージを入力...");

    fireEvent.change(textarea, { target: { value: "hello" } });

    fireEvent.keyDown(textarea, {
      key: "Enter",
      code: "Enter",
      isComposing: false,
    });

    expect(textarea).toHaveValue("");
  });
});
