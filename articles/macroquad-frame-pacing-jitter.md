# MacroquadでFPSは出ているのに2Dスクロールがガタつく問題を調べています

Rust + Macroquadで2Dゲームを作っているときに、FPSは60/120出ているのに、背景スクロールやスプライトが一瞬ブルっと震える現象に悩んでいました。

Love2Dでは滑らかに見えるのに、同じPCでもMacroquadやBevyでは違和感が出ることがありました。

この手の問題は、単に平均FPSを見るだけでは分かりません。FPS表示は安定して見えても、実際にはフレームの提出タイミング、`next_frame().await` の待ち時間、OS側の待機、CPU負荷、モニターのリフレッシュレートなどが絡んで、見た目の動きだけが一瞬ガタつくことがあります。

そこで、検証用のリポジトリを公開しました。

## 何を調べているか

このリポジトリでは、以下を可視化・計測しています。

- frame time
- `next_frame().await` の待ち時間
- manual pacer の待ち時間
- slow frame
- spike
- CPU使用率
- 60Hz / 120Hz の違い
- 背景スクロール
- スプライト移動

目的は「平均FPSを高くすること」ではありません。

背景スクロールやスプライト移動が本当に滑らかに見えるか、そのときフレーム間隔や待機時間に何が起きているかを観察できるようにすることです。

## 現在の状態

まだ完全な解決策ではありません。

このプロジェクトは実験版です。完成したライブラリAPIでも、すべての環境で必ず滑らかになることを保証するものでもありません。

現時点では、以下の2つのプロファイルを用意しています。

- `stable60`: 60Hz安定性優先
- `smooth120`: 120Hzの見た目の滑らかさ優先

自分の環境では、120Hzの方が自機の動きは気持ちよく見える場面があります。ただし、環境によってはpresent timingや`next_frame().await`側の揺れが見える可能性があります。

## なぜ公開したか

同じように「FPSは出ているのに、なぜか2Dスクロールやスプライトがガタつく」と悩んでいる人がいると思ったからです。

この問題は検索語もばらけます。

- jitter
- stutter
- frame pacing
- sprite shaking
- smooth scrolling
- `next_frame().await`
- Rust game loop
- Macroquad timing

もし同じような問題を調べている人がいれば、実験材料として役に立つかもしれません。

GitHub:

https://github.com/soboro0502/macroquad-smooth2d-lab

## 免責

このプロジェクトは実験・検証用のテストバージョンです。

いかなる保証もありません。利用、改変、複製、配布、参照によって直接または間接的に発生した損害、不具合、性能問題、データ損失、事業上の損失、その他一切の問題について、作者およびコントリビューターはいかなる責任も負いません。

---

# Investigating 2D scrolling jitter in Macroquad even when FPS looks fine

While making a 2D game with Rust + Macroquad, I kept running into a problem where the FPS counter showed 60 or 120 FPS, but the background scrolling or sprites would still briefly jitter or shake.

Love2D looked smooth on the same PC, but Macroquad and Bevy could still show visible motion issues.

This kind of problem is hard to understand by looking only at average FPS. Even when the FPS counter looks stable, visible motion can still be affected by frame presentation timing, `next_frame().await` wait time, OS waiting behavior, CPU load, and display refresh rate.

I published an experimental repository to investigate this.

## What it measures

The repository visualizes and measures:

- frame time
- `next_frame().await` wait time
- manual pacer wait time
- slow frames
- spikes
- CPU usage
- 60Hz / 120Hz behavior
- background scrolling
- sprite movement

The goal is not to maximize average FPS.

The goal is to observe whether scrolling and sprite movement actually look smooth, and what happens to frame intervals and wait times when they do not.

## Current status

This is not a complete solution yet.

It is an experimental test project. It is not a finalized reusable library API, and it does not guarantee perfectly smooth motion on every environment.

The current profiles are:

- `stable60`: stability-first 60Hz profile
- `smooth120`: motion-quality-focused 120Hz profile

On my environment, 120Hz can feel better for player movement. However, depending on the machine and display path, present timing or `next_frame().await` variance may still be visible.

## Why I published it

I think there may be other people struggling with the same problem: "the FPS looks fine, but 2D scrolling or sprites still feel wrong."

The search terms for this problem are scattered:

- jitter
- stutter
- frame pacing
- sprite shaking
- smooth scrolling
- `next_frame().await`
- Rust game loop
- Macroquad timing

If someone is investigating a similar issue, this repository may be useful as a runnable reference.

GitHub:

https://github.com/soboro0502/macroquad-smooth2d-lab

## Disclaimer

This project is an experimental test version.

There is no warranty of any kind. The author and contributors are not responsible or liable for any damage, loss, defect, performance issue, data loss, business loss, or other problem caused directly or indirectly by using, modifying, copying, distributing, or relying on this project.
