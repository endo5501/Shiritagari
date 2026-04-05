import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

interface ThoughtPayload {
  inference: string;
  confidence: number;
}

const appWindow = getCurrentWindow();

function App() {
  const [thought, setThought] = useState<string | null>(null);
  const [question, setQuestion] = useState<string | null>(null);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const composingRef = useRef(false);

  const isAsking = question !== null;
  const bubbleText = isAsking ? question : thought;
  const bubbleClass = isAsking ? "speech" : "thought";

  useEffect(() => {
    const unlistenThought = listen<ThoughtPayload>(
      "shiritagari-thought",
      (event) => {
        setThought(event.payload.inference);
      }
    );

    const unlistenQuestion = listen<string>(
      "shiritagari-question",
      (event) => {
        setQuestion(event.payload);
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
      if (isAsking) {
        await invoke("answer_question", {
          answer: trimmed,
          questionContext: question,
        });
        setQuestion(null);
      } else {
        const response = await invoke<string>("send_message", {
          message: trimmed,
        });
        setThought(response);
      }
    } catch (err) {
      console.error("Failed to send message:", err);
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

  return (
    <div className="mascot-container">
      {bubbleText && (
        <div
          className={`bubble ${bubbleClass}`}
          data-testid="bubble"
        >
          <div className="bubble-text">{bubbleText}</div>
          <div className="bubble-tail" />
        </div>
      )}

      <div
        className="mascot-drag-area"
        onMouseDown={() => appWindow.startDragging()}
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
          placeholder={isAsking ? "質問に回答..." : "メッセージを入力..."}
          rows={1}
          disabled={isLoading}
        />
        <button onClick={handleSend} disabled={isLoading || !input.trim()}>
          {isAsking ? "回答" : "送信"}
        </button>
      </div>
    </div>
  );
}

export default App;
