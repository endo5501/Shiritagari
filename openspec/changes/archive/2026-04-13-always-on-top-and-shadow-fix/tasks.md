## 1. Windows透明枠の修正

- [x] 1.1 `tauri.conf.json` の main ウィンドウ設定に `"shadow": false` を追加する
- [x] 1.2 Windows環境で透明枠が消えていることを目視確認する

## 2. AppState に最前面モード状態を追加

- [x] 2.1 `AppState` 構造体に `always_on_top: AtomicBool` フィールドを追加する（初期値 `false`）
- [x] 2.2 `AtomicBool` の use 宣言を追加する

## 3. トレイメニューに CheckMenuItem を追加

- [x] 3.1 `CheckMenuItemBuilder` の use 宣言を追加する
- [x] 3.2 トレイメニューに「Always on Top」チェックメニュー項目を追加する（Settings と Quit の間に配置）
- [x] 3.3 メニューイベントハンドラで「always_on_top」イベントを処理し、`AtomicBool` のトグルと `set_always_on_top` の呼び出しを実装する

## 4. 回答後の最前面モード分岐

- [x] 4.1 `submit_answer` 内の `set_always_on_top(false)` を、最前面モードの状態に応じて分岐するよう変更する
- [x] 4.2 最前面モードON時は `set_always_on_top(false)` を呼ばず、OFF時のみ呼ぶようにする

## 5. テスト

- [x] 5.1 `AtomicBool` のトグル動作に関するユニットテストを追加する（必要に応じて） — Tauriコマンド統合のためユニットテスト不要、動作確認で代替
- [x] 5.2 アプリを起動し、トレイメニューから「Always on Top」のON/OFFが動作することを確認する
- [x] 5.3 最前面モードON時に質問回答後もウィンドウが最前面に留まることを確認する
- [x] 5.4 最前面モードOFF時に質問回答後にウィンドウの最前面が解除されることを確認する

## 6. 最終確認

- [x] 6.1 `/simplify`スキルを使用してコードレビューを実施
- [x] 6.2 `/codex:review --scope branch --background` スキルを使用して現在開発中のコードレビューを実施
- [x] 6.3 `/opsx:verify`でchangeを検証
