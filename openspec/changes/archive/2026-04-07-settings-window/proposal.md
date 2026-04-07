## Why

現在、アプリの設定は `~/.config/shiritagari/config.toml` を手動編集する必要がある。特に初回セットアップ時にLLMプロバイダの設定をTOMLファイルに記述するのはユーザにとって敷居が高い。また、Ollamaのモデル名を正確に手入力する必要があり、誤記によるエラーが起きやすい。GUIの設定画面を提供し、トレイアイコンから呼び出せるようにすることでユーザビリティを大幅に改善する。

## What Changes

- トレイメニューに「Settings」項目を追加し、クリックで設定ウィンドウを開く
- 設定ウィンドウ（別ウィンドウ、約500x600）をReactで実装し、以下のセクションを持つ：
  - **LLM設定**: プロバイダ選択、モデル選択/入力、APIキー環境変数名、Base URL
  - **監視設定**: ポーリング間隔、アプリ許可/拒否リスト
  - **外観設定**: マスコット画像パス
- Ollamaプロバイダ選択時は `/api/tags` からローカルのモデル一覧を取得し、ドロップダウンで選択可能にする
- プロバイダ種別（Ollama / Claude / OpenAI）に応じて動的にフォームフィールドを切り替える
- 設定保存時に `config.toml` への書き込みとランタイム設定（AppState）の更新を行い、再起動なしで反映する
- `AppState.config` を `Arc<RwLock<AppConfig>>` に変更し、ホットリロードを実現する

## Capabilities

### New Capabilities
- `settings-window`: トレイメニューから開く設定GUI。設定の読み込み・編集・保存とOllamaモデル一覧取得を提供する

### Modified Capabilities

(なし — 既存の `llm-provider` の要件自体は変更しない。設定の入力手段がTOML手動編集からGUIに拡張されるのみ)

## Impact

- **Rust (src-tauri)**:
  - `config.rs`: `Serialize` derive追加、config保存メソッド追加
  - `lib.rs`: `AppState.config` を `Arc<RwLock<AppConfig>>` に変更、新規Tauriコマンド追加（`get_config`, `save_config`, `list_ollama_models`）、トレイメニューに「Settings」追加、設定ウィンドウ作成
  - 既存のconfig参照箇所をRwLock対応に修正
- **Frontend (src/)**:
  - 新規Settingsコンポーネント群の追加
  - 設定ウィンドウ用のエントリポイント追加
- **Tauri設定**:
  - `tauri.conf.json` にマルチウィンドウ設定の追加（または動的ウィンドウ生成）
- **依存関係**:
  - Rust側: 追加なし（既存の `toml`, `serde`, `reqwest` で対応可能）
  - Frontend側: 追加なし（React標準のフォーム機能で十分）
