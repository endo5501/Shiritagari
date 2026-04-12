## Why

マスコットウィンドウを常に手前に表示したいケースがあるが、現在は質問表示時のみ一時的に最前面化する仕組みしかなく、ユーザーが任意にON/OFFを制御できない。また、Windows環境で透明ウィンドウにDWMのシャドウが残り、不自然な「枠」が表示される視覚的問題がある。

## What Changes

- トレイメニューに「Always on Top」チェックメニュー項目を追加し、最前面モードのON/OFFをトグルできるようにする
- 最前面モードON時は、質問への回答後もウィンドウが最前面に留まる
- 最前面モードOFF時は、現在と同じ挙動（質問時のみ前面、回答後は解除）
- `tauri.conf.json` に `shadow: false` を追加し、Windowsでの透明枠問題を解消する

## Capabilities

### New Capabilities
- `always-on-top`: トレイメニューからマスコットウィンドウの最前面表示モードをON/OFFで切り替える機能

### Modified Capabilities
- `mascot-display`: ウィンドウ設定に `shadow: false` を追加し、Windows環境での透明枠を解消

## Impact

- `src-tauri/tauri.conf.json`: ウィンドウ設定に `shadow` プロパティ追加
- `src-tauri/src/lib.rs`: `AppState` に最前面モード状態の追加、トレイメニュー構築の変更、回答後の `set_always_on_top` 分岐追加
- `src-tauri/src/events.rs`: 変更なし（`bring_window_to_front` は既存のまま）
- 新規依存ライブラリなし
