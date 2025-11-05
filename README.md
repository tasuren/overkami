# overkami: オーバーレイ壁紙アプリ

このアプリは、特定のアプリに壁紙を設定するための、tauri製の実験的なアプリです。

仕組み上、どんなアプリでも基本的には壁紙を再現できます。
ただ、このアプリはベータ段階で、なおかつ、壁紙の表示方法がトリッキーで不安定です。
このため、**このアプリを使って起きたあらゆる損害を開発者は一切、負いません。**

なお、このプロジェクトのオーナーは、このプロジェクトを実験的段階以降に引き上げる予定はありません。
このため、これからサポートが継続される保証がないです。
ですが、Pull Requestは歓迎しますし、このリポジトリの運用を引き継いでくれる人がいれば譲渡します。
（場合によってはこの方針が変わる可能性もあります。）

<img width="400" alt="Twier with wallpaper" src="https://github.com/user-attachments/assets/3eb1ffd3-06e9-49b8-bece-4a6039a2d29c" />

## 仕組み

半透明でタイトルバーを持たない、クリックが貫通するウィンドウをまず作ります。
そしてこのウィンドウに壁紙を写し、壁紙を設定したいアプリのウィンドウにサイズを合わせ追従させます。
これにより、壁紙表現を擬似的に再現します。

ウィンドウを追従させるのは、WindowsではSetWinEventHook、macOSではAXObserverCreateを使うことで実現しています。
これらは、ウィンドウの動きを監視するのに使い、ウィンドウが動いた際に通知を受け取ることができます。
それにより、ウィンドウが動いた際に壁紙ウィンドウも同じ場所に追従して動かせるわけです。

### 技術スタック

- GUIフレームワーク: tauri
- フロントエンド
    - TypeScript
    - SolidJS
    - Tailwind CSS
    - Tailwind Variants
    - Modular Forms: フォームバリデーション

## 対応プラットフォーム

- [x] Windows: 現在かなり不安定 ⚠️
- [x] macOS
- [ ] Linux系OS

Linux系OSに対応する予定はないですが、Pull Requestは歓迎します。

## スクリーンショット

<img width="600" alt="overkami screen shot" src="https://github.com/user-attachments/assets/1b0d07b3-b432-40ad-be41-c132a9a062c3" />

<img width="600" alt="メモ帳に壁紙を設定した例" src="https://github.com/user-attachments/assets/90790964-74fb-4c59-b095-ba50e557fc9d" />

https://github.com/user-attachments/assets/1b59cb07-8329-4dc5-82f5-0b7ddd741584

## ライセンス

このプロジェクト・アプリは、[GNU General Public License v3.0 or later](./LICENSE)に基づいて提供されます。
