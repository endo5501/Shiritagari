## Context

現在のマスコットウィンドウは `decorations: false` / `transparent: true` の frameless 透過ウィンドウとして動作する。質問イベント発生時に `set_always_on_top(true)` で一時的に最前面化し、回答後に `set_always_on_top(false)` で解除する仕組みになっている。

Windows環境では DWM（Desktop Window Manager）が透過ウィンドウにシャドウを付与するため、`shadow` プロパティが未設定（デフォルト `true`）のままだと不自然な透明枠が表示される。macOS では `macOSPrivateApi: true` により回避されている。

## Goals / Non-Goals

**Goals:**
- ユーザーがトレイメニューから最前面モードをON/OFFできるようにする
- Windowsでの透明枠問題を解消する

**Non-Goals:**
- 最前面モードの設定を永続化する（アプリ再起動でOFFに戻る）
- フロントエンドUIでの最前面トグル操作
- macOSやLinux固有のウィンドウ表示問題の対応

## Decisions

### 1. 状態管理: 2つの `AtomicBool` で関心を分離

**選択**: `AppState` 構造体に `user_pinned: AtomicBool`（ユーザーの最前面設定）と `question_topmost: AtomicBool`（質問表示による一時的な最前面化）の2つのフラグを追加する。実効状態は `user_pinned || question_topmost` で計算する。

**理由**: 単一の `always_on_top` フラグでは「ユーザーの意思による最前面設定」と「質問表示による一時的な最前面化」が混在し、質問中にトレイメニューを操作すると状態の不整合が生じる。2つのフラグに分離することで、各操作が独立して状態を管理でき、常に正しい実効状態を計算できる。

**代替案**:
- 単一の `AtomicBool`: 質問中のトレイ操作で状態が不整合になるレースコンディションあり
- `Mutex<bool>`: ロックのオーバーヘッドがあり、単純なフラグには過剰

### 2. トレイメニュー: `CheckMenuItemBuilder` を使用

**選択**: Tauri 2 の `CheckMenuItemBuilder` でチェック付きメニュー項目を追加する

**理由**: OS標準のチェックマーク付きメニューが表示され、現在のON/OFF状態がユーザーに明確に伝わる。追加の `use` 宣言だけで使える。

**代替案**:
- 通常の `MenuItemBuilder` でテキスト切り替え（"Always on Top ✓" / "Always on Top"）: OS非標準でプラットフォーム間の見た目が不統一

### 3. shadow: false をウィンドウ設定に追加

**選択**: `tauri.conf.json` の main ウィンドウ設定に `"shadow": false` を追加する

**理由**: 最もシンプルな方法で DWM シャドウを無効化できる。Tauri 2 がネイティブにサポートするプロパティであり、Win32 APIを直接呼ぶ必要がない。macOS では `macOSPrivateApi` が既に設定されており影響なし。

**代替案**:
- Win32 API (`DwmExtendFrameIntoClientArea`) を直接呼ぶ: 複雑で OS 依存コードが増える
- `WS_EX_LAYERED` スタイル: パフォーマンス影響があり過剰

### 4. `bring_window_to_front()` は変更しない

**選択**: 既存の `bring_window_to_front()` 関数はそのまま維持する

**理由**: この関数は質問表示時に呼ばれ、常に `set_always_on_top(true)` を設定する。最前面モードがON/OFFどちらでも、質問時は前面に来るべきなので変更不要。モードによる分岐は回答後の処理（`lib.rs` の `submit_answer` 内）だけで行う。

## Risks / Trade-offs

- **[Risk] `shadow: false` で他のOSの見た目に影響する可能性** → macOSでは `macOSPrivateApi: true` が既に設定されておりシャドウ制御は独立。Linux は WM 依存だが `shadow: false` が悪影響を及ぼすケースは稀。
- **[Risk] 最前面モードON時にユーザーが他のウィンドウを操作しづらくなる** → ユーザー自身がトグルで制御するため許容範囲。初期状態はOFFで現行動作と同一。
- **[Trade-off] 最前面モードの設定が永続化されない** → 設計をシンプルに保つため意図的に非永続。需要があれば将来追加可能。
