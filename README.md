# Rust-STG Macroquad Frame Pacing Lab

Rust-STG is a small Macroquad test project for investigating arcade-quality 2D frame pacing at 120 Hz.

The current default is tuned for frame-time stability on macOS:

- 120 Hz target frame pacing
- spin pacer
- `QOS_CLASS_USER_INTERACTIVE`
- `THREAD_TIME_CONSTRAINT_POLICY`
- strict diagnostics that fail on single-frame max-time spikes

The goal is not high average FPS. The goal is stable frame intervals with no visible single-frame shake.

## Run

```bash
./run.sh
```

## Verify

```bash
./check.sh
./run.sh --diag-seconds 120 --diag-warmup-seconds 5 --visual-check
```

Verified result on the reference machine:

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

## Diagnostic Modes

Use the stable default:

```bash
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check
```

Compare without macOS time constraint:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --no-time-constraint
```

Compare lower-CPU sleep pacing:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --sleep-pacer
```

## Tradeoff

The stable default intentionally uses high CPU. `std::thread::sleep` and `thread::yield_now` reduced or shifted CPU usage, but both allowed occasional slow frames during testing.

For the detailed investigation notes, see [docs/frame_pacing_asset.md](docs/frame_pacing_asset.md).
