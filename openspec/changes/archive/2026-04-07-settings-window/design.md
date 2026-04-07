## Context

Shiritagari は Tauri (Rust + React) で構築された常駐型AIエージェント。現在、全設定は `~/.config/shiritagari/config.toml` の手動編集で管理されている。メインウィンドウは 300x420 の小型マスコット表示用で、リサイズ不可。トレイメニューには Show / Knowledge Base / Quit の3項目がある。

`AppState.config` は `AppConfig` を値で保持し、起動時に一度だけ読み込む。ポーリングループにも `config.clone()` で渡している。ただし、LLMプロバイダは `send_message` やポーリングサイクルのたびに `create_*_provider(&config.llm)` で毎回生成しているため、config参照さえ更新すればプロバイダは自動的に新設定を使う。

## Goals / Non-Goals

**Goals:**
- トレイメニューから設定ウィンドウを開き、GUIで設定を編集・保存できる
- Ollamaプロバイダ選択時にローカルのモデル一覧から選択できる
- 設定保存時にconfig.tomlへの永続化とランタイム設定の更新を行う（再起動不要）
- プロバイダ種別に応じた動的フォーム切り替え

**Non-Goals:**
- 初回セットアップウィザード（将来の拡張として検討）
- inference/chat個別プロバイダの設定UI（上級者向け設定はTOML手動編集のまま）
- Confidence パラメータの設定UI（上級者向け）
- Redaction patterns の設定UI（正規表現の入力UIは複雑すぎる）
- APIキーの直接入力・保存（現行の環境変数名指定方式を維持）

## Decisions

### 1. 別ウィンドウで設定画面を開く

メインウィンドウは 300x420 のマスコット表示専用でリサイズ不可。設定画面にはテキスト入力やドロップダウンが多数必要で、この幅では収まらない。

**選択: Tauriの `WebviewWindowBuilder` で別ウィンドウ (約500x600) を動的に生成する**

代替案:
- メインウィンドウ内でルーティング → 幅300pxでは入力フォームが窮屈
- メインウィンドウをリサイズ → マスコット表示との切り替えが煩雑

### 2. AppState.config を Arc<RwLock<AppConfig>> に変更

現状 `AppState.config` は `AppConfig` を値で保持。設定画面から保存時にランタイム設定を更新するには、共有参照が必要。

**選択: `Arc<RwLock<AppConfig>>` に変更し、read時はRwLockのreadロック、保存時はwriteロックで更新**

理由:
- `send_message` と polling loop の両方が毎回 `create_*_provider` を呼んでいるため、configを差し替えるだけで次回呼び出しから新設定が反映される
- ポーリング間隔の変更は `tokio::watch` チャネルで polling loop に通知し、ticker を再作成する

代替案:
- アプリ再起動で反映 → ユーザ体験が悪い
- イベントベースで各コンポーネントに通知 → 過剰な複雑さ

### 3. 設定ウィンドウのエントリポイント

Tauriマルチウィンドウでは、各ウィンドウに異なるURLを割り当てられる。

**選択: `/settings.html` を別エントリポイントとして作成し、SettingsApp コンポーネントをマウントする**

理由:
- メインウィンドウの App.tsx とは完全に独立した関心事
- React Router を導入するほどの複雑さではない
- Viteのマルチページ設定で対応可能

### 4. Ollamaモデル一覧取得

Ollama は `GET /api/tags` でローカルインストール済みモデル一覧を返す。

**選択: Rust側に `list_ollama_models` Tauriコマンドを追加し、Ollama APIを呼び出してモデル名一覧を返す**

理由:
- CORSの問題を回避（フロントエンドから直接Ollama APIを呼ぶとブラウザ環境のCORS制約に引っかかる可能性）
- Base URL のカスタマイズにも対応しやすい
- エラーハンドリング（Ollama未起動時など）をRust側で統一的に処理

### 5. 設定画面のセクション構成

全設定を一画面に出すと煩雑なため、よく使う項目に絞る。

**選択: 3セクション構成**
- **LLM設定**: provider, model, api_key_env, ollama_base_url, openai_base_url
- **監視設定**: interval_minutes, allowlist_apps, blocklist_apps
- **外観設定**: character_image

上級者向け設定（inference/chat個別設定、confidence、redaction_patterns）は対象外とし、TOML手動編集のまま残す。

### 6. config.toml の保存戦略

**選択: 設定画面で管理する項目のみを書き込み、未管理項目は既存値を保持する**

手順:
1. 保存時に既存の config.toml を読み込む
2. 設定画面の値で該当フィールドを上書き
3. 全体を TOML にシリアライズして書き込む

理由:
- 上級者がTOMLで直接設定した inference/chat 個別設定等を消さない
- AppConfig 全体を Serialize 可能にすることで、読み込み→更新→書き込みの一貫した処理が可能

## Risks / Trade-offs

- **[RwLock の競合]** → 設定の読み取りは高頻度だが書き込みは稀（ユーザが保存ボタンを押した時のみ）なので、RwLock の read/write 非対称性が適している。実質的な競合リスクは極めて低い
- **[Polling interval のホットリロード]** → `tokio::watch` チャネルで通知する設計を採用。ポーリングループ側で `tokio::select!` により interval 変更を検知し ticker を再作成する。やや複雑だが、再起動なしの設定反映というゴールのために必要
- **[Ollama未起動時のモデル一覧取得失敗]** → フロントエンドでエラーメッセージを表示し、手動入力へのフォールバックを用意する
- **[TOML保存時のデータ損失]** → 既存ファイルを読み込んでからマージすることで、設定画面で管理しない項目を保持する。ただし、TOML内のコメントは失われる
