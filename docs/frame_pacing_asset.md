# Macroquad Smooth2D Lab フレームペーシング資産

このプロジェクトでは、Macroquad/macOS で 60Hz / 120Hz のフレーム間隔を安定させるために、更新処理や座標補間ではなく「最終的なフレーム提出タイミング」を主対象として調整した。

## 実験版としての位置づけと免責

このプロジェクトは実験・検証用のテストバージョンであり、調査、計測、学習、参考を目的として提供する。

いかなる保証もない。利用、改変、複製、配布、参照によって直接または間接的に発生した損害、不具合、性能問題、データ損失、事業上の損失、その他一切の問題について、作者およびコントリビューターはいかなる責任も負わない。

## 現在の安定デフォルト

- pacer: Mach wait + balanced final spin
- macOS thread QoS: `QOS_CLASS_USER_INTERACTIVE`
- macOS thread policy: `THREAD_TIME_CONSTRAINT_POLICY`
- default profile: `smooth120`
- stable profile: `stable60`
- default background: `assets/br_01.png`
- diagnostics: `next_frame`, OS wait, spin wait, and total pacer wait are measured separately
- 診断 PASS 条件: `max_ms <= target_ms + 0.7`

通常起動でこの構成が有効になる。

## 再現コマンド

```bash
./run.sh --diag-seconds 120 --diag-warmup-seconds 5 --visual-check --profile smooth120
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check --profile stable60 --bg-step 2
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

## プロファイル設計

このプロジェクトでは「本命を1つに決める」のではなく、用途別プロファイルとして整理する。

| profile | target | 目的 |
| --- | ---: | --- |
| `stable60` | 60Hz | 安定性優先。現時点のリリース基準候補。 |
| `smooth120` | 120Hz | 自機移動の気持ちよさ優先。present境界の監視が必要。 |

プロファイルは `--profile stable60` / `--profile smooth120` で選ぶ。`--target-hz` は低レベル検証用として残す。

ゲーム速度は `REFERENCE_GAME_HZ=120` を基準にする。frame-step mode では `120 / target_hz` で1フレームあたりの移動量を補正し、60Hzと120Hzで秒間速度が変わらないようにする。

例:

- 背景 `2px/frame @ 120Hz` は、60Hzでは `4px/frame`
- 自機 `4px/frame @ 120Hz` は、60Hzでは `8px/frame`
- `Z/X` の player speed scale は、この基準移動量に倍率を掛ける

## 設定できるもの

主な起動オプション:

| option | 内容 |
| --- | --- |
| `--profile stable60` | 60Hz安定プロファイル |
| `--profile smooth120` | 120Hz滑らかさプロファイル |
| `--target-hz <hz>` | 低レベルtarget Hz指定 |
| `--hud` | HUD表示 |
| `--visual-check` | デフォルト背景とframe-step modeで視覚確認 |
| `--diag` | デフォルト秒数で診断 |
| `--diag-seconds <sec>` | 診断時間 |
| `--diag-warmup-seconds <sec>` | 診断前ウォームアップ時間 |
| `--startup-warmup-seconds <sec>` | 起動直後ウォームアップ時間 |
| `--no-startup-warmup` | 起動ウォームアップ無効 |
| `--bg-step <px>` | 背景frame-step量 |
| `--frame` | frame-step mode |
| `--dt` | delta-time mode |
| `--texture` | 通常背景 |
| `--probe` | 検証用背景 |
| `--bands` | 帯背景 |
| `--mach-pacer` | Mach wait + spin |
| `--balanced-pacer` | balanced spin margin |
| `--precision-pacer` | precision spin margin |
| `--eco-pacer` | low CPU比較用 |
| `--sleep-pacer` | sleep + spin比較用 |
| `--spin-pacer` | pure spin比較用 |
| `--pacer-margin-ms <ms>` | spin margin指定 |
| `--pacer-sleep-threshold-ms <ms>` | sleep-spin閾値 |
| `--time-constraint` | macOS time constraint有効 |
| `--no-time-constraint` | macOS time constraint無効 |

実行中の操作:

| key | 内容 |
| --- | --- |
| Arrow / WASD | 自機移動 |
| Shift | 低速移動 |
| `Z` / `X` | 自機速度倍率を下げる / 上げる |
| Space | 背景スクロールON/OFF |
| `Tab` | frame-step / delta-time切り替え |
| `V` | 斜め移動モード切り替え: raw default / normalized / last-axis |
| `G` | 背景モード切り替え |
| `1`-`4` | 背景frame-step量切り替え |
| `H` | HUD表示切り替え |
| `C` | clear-only切り替え |
| `P` | manual pacer ON/OFF |
| `L` | frame log ON/OFF |

## HUDの読み方

HUDは検証用であり、ゲーム内UIではない。

- `STATUS`: プロファイル、判定、描画負荷、pacer、CPU、ログ状態
- `SCENE`: 移動モード、斜め移動モード、背景モード、背景step、背景delta、自機速度倍率
- `SYNC`: `next_frame` / OS wait / spin / total wait
- `FRAME`: target Hz、FPS、平均、last/p95/p99 ms
- `STABLE`: min/max、標準偏差、slow%、spike数

特に重要なのは `SYNC` 行。

- `next`: `next_frame().await` 内で待った時間
- `os`: `mach_wait_until` または sleep で待った時間
- `spin`: CPUを使って最後に合わせた時間
- `total`: manual pacer全体の待ち時間

`next` が遅れているフレームは、manual pacerの前にすでに遅れている。ここは現在のMacroquad構成で120Hzを詰める時の最大注意点。

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

## 起動ウォームアップ

起動直後は、テクスチャ初回描画、Macroquad内部バッチ、フォント、GPU/WindowServer/Metalの初回presentなどが動く可能性がある。

そのため、通常起動では短い startup warmup を挟む。この間も通常シーンは描画するが、ゲーム更新、入力反映、診断計測は開始しない。診断モードでは `startup_warmup_seconds + diag_warmup_seconds` の後から測定を始める。

検証でウォームアップを外したい場合は `--no-startup-warmup` を使う。

## 効かなかった方向

- 背景 delta が固定された後は、可変 timestep や座標補間が主因ではなかった。
- `std::thread::sleep` + spin は CPU を下げられるが、まれに slow frame が出た。
- Mach wait の spin margin を 2.5ms 以下に下げると CPU は下がるが、`next_frame` の戻り遅れが出やすくなった。
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
- 120Hz / 3.0ms balanced: PASS, cpu=41.5%, 30秒診断
- 120Hz / 2.5ms: WARN, next_frame_ms max=9.489
- 120Hz / 2.0ms eco: WARN, slow frameまたは `next_frame` 戻り遅れあり

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
