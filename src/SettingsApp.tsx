import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./Settings.css";

interface AppConfig {
  polling: { interval_minutes: number };
  llm: {
    provider: string;
    model: string | null;
    api_key_env: string | null;
    inference_provider: string | null;
    inference_model: string | null;
    inference_api_key_env: string | null;
    chat_provider: string | null;
    chat_model: string | null;
    chat_api_key_env: string | null;
    ollama_base_url: string | null;
    openai_base_url: string | null;
  };
  privacy: {
    allowlist_apps: string[];
    blocklist_apps: string[];
    redaction_patterns: string[];
  };
  confidence: {
    decay_rate: number;
    threshold_silent: number;
    threshold_re_ask: number;
    threshold_soft_delete: number;
  };
  mascot: { character_image: string | null };
}

interface OllamaModel {
  name: string;
}

const PROVIDERS = ["ollama", "claude", "openai"] as const;

function ListEditor({
  title,
  items,
  onAdd,
  onRemove,
  placeholder,
}: {
  title: string;
  items: string[];
  onAdd: (value: string) => void;
  onRemove: (index: number) => void;
  placeholder: string;
}) {
  const [inputValue, setInputValue] = useState("");

  const handleAdd = () => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    onAdd(trimmed);
    setInputValue("");
  };

  return (
    <div className="list-editor">
      <h3>{title}</h3>
      <div className="list-items">
        {items.map((item, i) => (
          <span key={i} className="list-tag">
            {item}
            <button type="button" onClick={() => onRemove(i)}>
              ×
            </button>
          </span>
        ))}
      </div>
      <div className="list-add">
        <input
          type="text"
          value={inputValue}
          placeholder={placeholder}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleAdd();
          }}
        />
        <button type="button" onClick={handleAdd}>
          追加
        </button>
      </div>
    </div>
  );
}

function SettingsApp() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [ollamaModels, setOllamaModels] = useState<OllamaModel[]>([]);
  const [ollamaError, setOllamaError] = useState<string | null>(null);
  const [loadingModels, setLoadingModels] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig);
  }, []);

  const fetchOllamaModels = useCallback(
    async (baseUrl?: string | null) => {
      setLoadingModels(true);
      setOllamaError(null);
      try {
        const models = await invoke<OllamaModel[]>("list_ollama_models", {
          baseUrl: baseUrl || null,
        });
        setOllamaModels(models);
      } catch (e) {
        setOllamaError(String(e));
        setOllamaModels([]);
      } finally {
        setLoadingModels(false);
      }
    },
    [],
  );

  useEffect(() => {
    if (config?.llm.provider === "ollama") {
      fetchOllamaModels(config.llm.ollama_base_url);
    }
  }, [config?.llm.provider]); // eslint-disable-line react-hooks/exhaustive-deps

  if (!config) return <div className="settings-loading">Loading...</div>;

  const updateLlm = (field: keyof AppConfig["llm"], value: string | null) => {
    setConfig({
      ...config,
      llm: { ...config.llm, [field]: value === "" ? null : value },
    });
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke("save_config", { newConfig: config });
      getCurrentWindow().close();
    } catch (e) {
      alert(`保存に失敗しました: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    getCurrentWindow().close();
  };

  const updateList = (
    field: "allowlist_apps" | "blocklist_apps",
    newItems: string[],
  ) => {
    setConfig({
      ...config,
      privacy: { ...config.privacy, [field]: newItems },
    });
  };

  return (
    <div className="settings-container">
      <h1>Settings</h1>

      <section className="settings-section">
        <h2>LLM</h2>

        <label>
          Provider
          <select
            value={config.llm.provider}
            onChange={(e) => updateLlm("provider", e.target.value)}
          >
            {PROVIDERS.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>
        </label>

        {config.llm.provider === "ollama" && (
          <>
            <label>
              Base URL
              <input
                type="text"
                value={config.llm.ollama_base_url ?? ""}
                placeholder="http://localhost:11434"
                onChange={(e) => updateLlm("ollama_base_url", e.target.value)}
                onBlur={() => fetchOllamaModels(config.llm.ollama_base_url)}
              />
            </label>

            <label>
              Model
              {ollamaError ? (
                <>
                  <input
                    type="text"
                    value={config.llm.model ?? ""}
                    placeholder="モデル名を入力"
                    onChange={(e) => updateLlm("model", e.target.value)}
                  />
                  <span className="error-text">{ollamaError}</span>
                </>
              ) : (
                <div className="select-with-button">
                  <select
                    value={config.llm.model ?? ""}
                    onChange={(e) => updateLlm("model", e.target.value)}
                    disabled={loadingModels}
                  >
                    <option value="">
                      {loadingModels ? "読み込み中..." : "選択してください"}
                    </option>
                    {ollamaModels.map((m) => (
                      <option key={m.name} value={m.name}>
                        {m.name}
                      </option>
                    ))}
                  </select>
                  <button
                    type="button"
                    onClick={() => fetchOllamaModels(config.llm.ollama_base_url)}
                    disabled={loadingModels}
                    className="refresh-button"
                    title="モデル一覧を更新"
                  >
                    ↻
                  </button>
                </div>
              )}
            </label>
          </>
        )}

        {config.llm.provider === "claude" && (
          <>
            <label>
              API Key 環境変数名
              <input
                type="text"
                value={config.llm.api_key_env ?? ""}
                placeholder="ANTHROPIC_API_KEY"
                onChange={(e) => updateLlm("api_key_env", e.target.value)}
              />
            </label>
            <label>
              Model
              <input
                type="text"
                value={config.llm.model ?? ""}
                placeholder="claude-sonnet-4-20250514"
                onChange={(e) => updateLlm("model", e.target.value)}
              />
            </label>
          </>
        )}

        {config.llm.provider === "openai" && (
          <>
            <label>
              Base URL
              <input
                type="text"
                value={config.llm.openai_base_url ?? ""}
                placeholder="https://api.openai.com"
                onChange={(e) => updateLlm("openai_base_url", e.target.value)}
              />
            </label>
            <label>
              API Key 環境変数名
              <input
                type="text"
                value={config.llm.api_key_env ?? ""}
                placeholder="OPENAI_API_KEY"
                onChange={(e) => updateLlm("api_key_env", e.target.value)}
              />
            </label>
            <label>
              Model
              <input
                type="text"
                value={config.llm.model ?? ""}
                placeholder="gpt-4o-mini"
                onChange={(e) => updateLlm("model", e.target.value)}
              />
            </label>
          </>
        )}
      </section>

      <section className="settings-section">
        <h2>監視</h2>

        <label>
          ポーリング間隔（分）
          <input
            type="number"
            min={1}
            value={config.polling.interval_minutes}
            onChange={(e) =>
              setConfig({
                ...config,
                polling: {
                  ...config.polling,
                  interval_minutes: Math.max(1, parseInt(e.target.value) || 1),
                },
              })
            }
          />
        </label>

        <ListEditor
          title="許可リスト (Allowlist)"
          items={config.privacy.allowlist_apps}
          onAdd={(value) =>
            !config.privacy.allowlist_apps.includes(value) &&
            updateList("allowlist_apps", [...config.privacy.allowlist_apps, value])
          }
          onRemove={(i) =>
            updateList("allowlist_apps", config.privacy.allowlist_apps.filter((_, idx) => idx !== i))
          }
          placeholder="アプリ名を追加"
        />

        <ListEditor
          title="拒否リスト (Blocklist)"
          items={config.privacy.blocklist_apps}
          onAdd={(value) =>
            !config.privacy.blocklist_apps.includes(value) &&
            updateList("blocklist_apps", [...config.privacy.blocklist_apps, value])
          }
          onRemove={(i) =>
            updateList("blocklist_apps", config.privacy.blocklist_apps.filter((_, idx) => idx !== i))
          }
          placeholder="アプリ名を追加"
        />
      </section>

      <section className="settings-section">
        <h2>外観</h2>

        <label>
          マスコット画像パス
          <input
            type="text"
            value={config.mascot.character_image ?? ""}
            placeholder="画像ファイルのパス"
            onChange={(e) =>
              setConfig({
                ...config,
                mascot: {
                  ...config.mascot,
                  character_image: e.target.value || null,
                },
              })
            }
          />
        </label>
      </section>

      <div className="settings-actions">
        <button type="button" onClick={handleCancel} className="btn-cancel">
          キャンセル
        </button>
        <button
          type="button"
          onClick={handleSave}
          disabled={saving}
          className="btn-save"
        >
          {saving ? "保存中..." : "保存"}
        </button>
      </div>
    </div>
  );
}

export default SettingsApp;
