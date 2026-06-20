# Rust-STG フレームペーシング資産

このプロジェクトでは、Macroquad/macOS で 120Hz のフレーム間隔を安定させるために、更新処理や座標補間ではなく「最終的なフレーム提出タイミング」を主対象として調整した。

## 現在の安定デフォルト

- pacer: spin pacer
- macOS thread QoS: `QOS_CLASS_USER_INTERACTIVE`
- macOS thread policy: `THREAD_TIME_CONSTRAINT_POLICY`
- target refresh: 120Hz
- 診断 PASS 条件: `max_ms <= 9.0`

通常起動でこの構成が有効になる。

## 再現コマンド

```bash
./run.sh --diag-seconds 120 --diag-warmup-seconds 5 --visual-check
```

確認済み結果:

```text
verdict=PASS
avg_ms=8.333
p95_ms=8.334
p99_ms=8.334
range_ms=8.334-8.339
sd_ms=0.001
slow_pct=0.0
spikes=0
bg_delta range=2.000-2.000
```

120秒の診断中、slow frame event は発生しなかった。

## 重要な判断

以前は p99 がきれいでも、単発の 10-11ms フレームによって目視上ブルつくことがあった。

そのため、現在は `p99` だけでなく `max_ms <= 9.0` も PASS 条件に入れている。これにより、平均や p99 では隠れる単発ガクつきを WARN として検出できる。

## 効かなかった方向

- 背景 delta が固定された後は、可変 timestep や座標補間が主因ではなかった。
- `std::thread::sleep` + spin は CPU を下げられるが、まれに slow frame が出た。
- `thread::yield_now` + spin は spin-only より悪化した。
- QoS だけでは、まれな slow frame を完全には消せなかった。

## トレードオフ

現在の安定デフォルトは CPU 使用率が高い。

これはフレーム締切直前に OS sleep へ頼らず、spin pacer と macOS の time constraint policy で提出タイミングを固定するため。

低CPUモードを試す場合:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --sleep-pacer
```

time constraint を外して比較する場合:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --no-time-constraint
```

ただし、これらは検証用であり、安定デフォルトではない。

## 主要ファイル

- `src/frame_pacer.rs`: spin / sleep-spin pacer
- `src/platform_tuning.rs`: macOS QoS / time constraint setup
- `src/frame_stats.rs`: frame timing statistics
- `src/frame_log.rs`: diagnostic output
- `src/config.rs`: timing threshold / PASS criteria
