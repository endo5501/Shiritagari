import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface Message {
  role: "user" | "assistant" | "system";
  content: string;
  isQuestion?: boolean;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [pendingQuestion, setPendingQuestion] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = listen<string>("shiritagari-question", (event) => {
      setPendingQuestion(event.payload);
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: event.payload, isQuestion: true },
      ]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSend = async () => {
    const trimmed = input.trim();
    if (!trimmed || isLoading) return;

    setMessages((prev) => [...prev, { role: "user", content: trimmed }]);
    setInput("");
    setIsLoading(true);

    try {
      if (pendingQuestion) {
        // This is an answer to a Shiritagari question — save as episode
        await invoke("answer_question", {
          answer: trimmed,
          questionContext: pendingQuestion,
        });
        setPendingQuestion(null);
        setMessages((prev) => [
          ...prev,
          { role: "assistant", content: "ありがとうございます！覚えておきます。" },
        ]);
      } else {
        // Regular chat message
        const response = await invoke<string>("send_message", {
          message: trimmed,
        });
        setMessages((prev) => [
          ...prev,
          { role: "assistant", content: response },
        ]);
      }
    } catch (err) {
      setMessages((prev) => [
        ...prev,
        {
          role: "system",
          content: `Error: ${err}`,
        },
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="chat-container">
      <div className="chat-header">
        <h1>Shiritagari</h1>
        {pendingQuestion && (
          <span className="pending-indicator">質問に回答待ち</span>
        )}
      </div>
      <div className="chat-messages">
        {messages.length === 0 && (
          <div className="empty-state">
            Shiritagariが動作しています。質問があれば聞いてきます。
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}${msg.isQuestion ? " question" : ""}`}>
            <div className="message-bubble">{msg.content}</div>
          </div>
        ))}
        {isLoading && (
          <div className="message assistant">
            <div className="message-bubble loading">...</div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      <div className="chat-input">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={pendingQuestion ? "質問に回答..." : "メッセージを入力..."}
          rows={1}
          disabled={isLoading}
        />
        <button onClick={handleSend} disabled={isLoading || !input.trim()}>
          {pendingQuestion ? "回答" : "送信"}
        </button>
      </div>
    </div>
  );
}

export default App;
