# Rust-STG フレームペーシング資産

このプロジェクトでは、Macroquad/macOS で 60Hz / 120Hz のフレーム間隔を安定させるために、更新処理や座標補間ではなく「最終的なフレーム提出タイミング」を主対象として調整した。

## 現在の安定デフォルト

- pacer: Mach wait + balanced final spin
- macOS thread QoS: `QOS_CLASS_USER_INTERACTIVE`
- macOS thread policy: `THREAD_TIME_CONSTRAINT_POLICY`
- default target refresh: 120Hz
- 60Hz verification: `--target-hz 60`
- default background: `assets/br_01.png`
- diagnostics: `next_frame`, OS wait, spin wait, and total pacer wait are measured separately
- 診断 PASS 条件: `max_ms <= target_ms + 0.7`

通常起動でこの構成が有効になる。

## 再現コマンド

```bash
./run.sh --diag-seconds 120 --diag-warmup-seconds 5 --visual-check
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check --target-hz 60 --bg-step 2
```

`--visual-check` はデフォルト背景とスクロールの測定に集中する。画面上のHUDを重ねて確認したい場合は `--hud` を追加する。

120Hz 診断結果:

```text
verdict=WARN
avg_ms=8.362
p95_ms=8.334
p99_ms=8.469
range_ms=8.334-24.829
sd_ms=0.459
slow_pct=0.4
spikes=1
cpu=55.5%
bg_delta range=2.000-2.000
pacer_spin_ms avg=2.221 range=0.000-4.499
pacer_os_wait_ms avg=0.002 range=0.000-0.266
next_frame_ms avg=6.045 range=3.519-24.679
```

120Hz では `next_frame().await` がフレーム境界待ちの大部分を消費することがある。この場合、manual pacer の `mach_wait_until` はほぼ動かず、`next_frame` の戻り遅れがそのまま slow frame になる。

60Hz 確認済み結果:

```text
verdict=PASS
avg_ms=16.667
p95_ms=16.667
p99_ms=16.667
range_ms=16.667-16.670
sd_ms=0.000
slow_pct=0.0
spikes=0
cpu=30.3%
bg_delta range=4.000-4.000
pacer_spin_ms avg=4.490 range=4.448-4.497
pacer_os_wait_ms avg=11.313 range=9.465-11.504
next_frame_ms avg=0.756 range=0.602-2.595
```

60Hz では、120Hz基準の frame-step 量を `120 / target_hz` で補正する。デフォルト背景の 2px/frame は 60Hz では 4px/frame になり、秒間スクロール量は同じになる。

60Hz では `next_frame().await` の待ち時間が短く、`mach_wait_until` と spin の役割分担が設計通りに出ている。

## 重要な判断

以前は p99 がきれいでも、単発の 10-11ms フレームによって目視上ブルつくことがあった。

そのため、現在は `p99` だけでなく `max_ms <= target_ms + 0.7` も PASS 条件に入れている。これにより、平均や p99 では隠れる単発ガクつきを WARN として検出できる。

## ホットパス監査

CPU使用率が上がり続けて見える問題を確認するため、ループ内処理を見直した。

- `mach_timebase_info` は毎フレーム呼ばず、初回だけ取得してキャッシュする。
- sleep-spin pacer の計測は、spin中に余計な計測処理を挟まない。
- HUD文字列は毎フレーム `format!` せず、HUDサンプル間隔で更新する。
- HUD用のフレーム統計は毎秒 `Vec` 確保をせず、固定配列でソートする。
- HUD用リングバッファは一周後も正しい時系列で snapshot を作る。
- HUD非表示かつ診断/ログ無効の通常実行では、統計計測を動かさない。

60Hz / HUDあり / 60秒診断では CPU は 30-31% 台で横ばいだった。上がり続ける蓄積挙動は確認されなかった。

120Hz で CPU が時間とともに上がるように見えるケースは、処理量増加ではなく `next_frame` 後に残る待ち時間が spin 側へ寄ることで説明できる。

## 効かなかった方向

- 背景 delta が固定された後は、可変 timestep や座標補間が主因ではなかった。
- `std::thread::sleep` + spin は CPU を下げられるが、まれに slow frame が出た。
- Mach wait の spin margin を 2-4ms に下げると CPU は下がるが、30秒診断でも slow frame が出た。
- 120Hz では spin margin を 5ms にしても `next_frame` の戻り遅れは消えなかった。
- `thread::yield_now` + spin は spin-only より悪化した。
- QoS だけでは、まれな slow frame を完全には消せなかった。
- pure spin は安定しやすいが CPU 使用率が高すぎた。

## トレードオフ

現在のペーシングは、`next_frame().await` の後に macOS の `mach_wait_until` で粗く待機し、最後だけ balanced spin する。

これにより、pure spin よりCPU使用率を下げながら、`std::thread::sleep` より締切直前の精度を保つ。

参考値:

- 60Hz / 4.5ms balanced: PASS, cpu=30.3%
- 120Hz / 4.5ms balanced: WARN, next_frame_ms max=24.679
- 120Hz / 5.0ms precision: WARN, next_frame_ms max=23.929
- 120Hz / 4.0ms: WARN, slow frameあり
- 120Hz / 3.0ms: WARN, slow frameあり
- 120Hz / 2.0ms eco: WARN, slow frameあり

pure spin と比較する場合:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --spin-pacer
```

sleep-spin と比較する場合:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --sleep-pacer
```

低CPU側に寄せた eco pacer と比較する場合:

```bash
./run.sh --diag-seconds 30 --diag-warmup-seconds 5 --visual-check --eco-pacer
```

time constraint を外して比較する場合:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --no-time-constraint
```

ただし、これらは検証用であり、安定デフォルトではない。

## 主要ファイル

- `src/frame_pacer.rs`: Mach wait / spin / sleep-spin pacer
- `src/platform_tuning.rs`: macOS QoS / time constraint setup
- `src/frame_stats.rs`: frame timing statistics
- `src/frame_log.rs`: diagnostic output
- `src/config.rs`: timing threshold / PASS criteria

## デフォルト背景

通常背景は `assets/br_01.png` を使う。

この画像は 960x540 で、`BACKGROUND_DRAW_SCALE=2.0` により 1920x1080 に一致する。フルスクリーン相当の背景では、動的な source 矩形切り出しを使わず、フル画像2枚を縦に並べる単純なループ描画にする。実測ではこの方式の方がフレームペーシングが安定した。

## フォント

HUD フォントは Google Fonts の Silkscreen Regular に差し替えた。

- source: https://github.com/google/fonts/tree/main/ofl/silkscreen
- license: SIL Open Font License 1.1
- local license: `assets/fonts/Silkscreen-OFL.txt`
