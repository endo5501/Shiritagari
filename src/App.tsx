import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

type BubbleMode = "idle" | "thinking" | "asking";

interface ThoughtPayload {
  inference: string;
  confidence: number;
}

function App() {
  const [thought, setThought] = useState<string | null>(null);
  const [question, setQuestion] = useState<string | null>(null);
  const [bubbleMode, setBubbleMode] = useState<BubbleMode>("idle");
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const composingRef = useRef(false);

  useEffect(() => {
    const unlistenThought = listen<ThoughtPayload>(
      "shiritagari-thought",
      (event) => {
        setThought(event.payload.inference);
        if (bubbleMode !== "asking") {
          setBubbleMode("thinking");
        }
      }
    );

    const unlistenQuestion = listen<string>(
      "shiritagari-question",
      (event) => {
        setQuestion(event.payload);
        setBubbleMode("asking");
      }
    );

    return () => {
      unlistenThought.then((fn) => fn());
      unlistenQuestion.then((fn) => fn());
    };
  }, []);

  const handleSend = async () => {
    const trimmed = input.trim();
    if (!trimmed || isLoading) return;

    setInput("");
    setIsLoading(true);

    try {
      if (question && bubbleMode === "asking") {
        await invoke("answer_question", {
          answer: trimmed,
          questionContext: question,
        });
        setQuestion(null);
        setBubbleMode(thought ? "thinking" : "idle");
      } else {
        const response = await invoke<string>("send_message", {
          message: trimmed,
        });
        setThought(response);
        setBubbleMode("thinking");
      }
    } catch {
      // Error handling - stay in current mode
    } finally {
      setIsLoading(false);
    }
  };

  const handleCompositionStart = () => {
    composingRef.current = true;
  };

  const handleCompositionEnd = () => {
    setTimeout(() => {
      composingRef.current = false;
    }, 20);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (
      e.key === "Enter" &&
      !e.shiftKey &&
      !e.nativeEvent.isComposing &&
      !composingRef.current
    ) {
      e.preventDefault();
      handleSend();
    }
  };

  const bubbleText =
    bubbleMode === "asking" ? question : bubbleMode === "thinking" ? thought : null;

  return (
    <div className="mascot-container">
      {bubbleText && (
        <div
          className={`bubble ${bubbleMode === "asking" ? "speech" : "thought"}`}
          data-testid="bubble"
        >
          <div className="bubble-text">{bubbleText}</div>
          <div className="bubble-tail" />
        </div>
      )}

      <div
        className="mascot-drag-area"
        onMouseDown={() => getCurrentWindow().startDragging()}
      >
        <img
          src="/default-mascot.png"
          alt="mascot"
          className="mascot-image"
          draggable={false}
        />
      </div>

      <div className="mascot-input">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          onCompositionStart={handleCompositionStart}
          onCompositionEnd={handleCompositionEnd}
          placeholder={
            bubbleMode === "asking" ? "質問に回答..." : "メッセージを入力..."
          }
          rows={1}
          disabled={isLoading}
        />
        <button onClick={handleSend} disabled={isLoading || !input.trim()}>
          {bubbleMode === "asking" ? "回答" : "送信"}
        </button>
      </div>
    </div>
  );
}

export default App;
