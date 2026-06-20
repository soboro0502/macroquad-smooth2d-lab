# Rust-STG Macroquad Frame Pacing Lab

Rust-STG is a small Macroquad test project for investigating arcade-quality 2D frame pacing at 60 Hz and 120 Hz.

The current default is tuned for frame-time investigation on macOS:

- 120 Hz default target frame pacing, with `--target-hz 60` for 60 Hz verification
- `assets/br_01.png` as the default looping background
- Mach wait + balanced final spin pacer
- HUD and diagnostic logs split `next_frame`, OS wait, and spin wait timing
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
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check --target-hz 60 --bg-step 2
```

120 Hz diagnostic result on the reference machine:

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

At 120 Hz, `next_frame().await` can consume most of the frame boundary wait before the manual pacer runs. In that case the Mach wait path has almost no room to operate, and occasional `next_frame` return delays show up as visible frame-time spikes.

60 Hz verified result on the reference machine:

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

At 60 Hz, the frame-step movement is scaled against the 120 Hz reference. The default 2 px/frame background step becomes 4 px/frame so the per-second scroll speed stays the same.

The runtime hot path has been audited for library use. HUD strings are cached, Mach timebase conversion is cached, HUD statistics avoid per-sample heap allocation, and normal non-diagnostic execution does not update frame statistics while the HUD and frame log are off.

## Diagnostic Modes

Use the stable default:

```bash
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check
```

Show the on-screen HUD when inspecting diagnostics manually:

```bash
./run.sh --visual-check --hud
```

Run the same visual check at 60 Hz:

```bash
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check --target-hz 60 --bg-step 2
```

Compare without macOS time constraint:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --no-time-constraint
```

Compare pure spin pacing:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --spin-pacer
```

Compare sleep-spin pacing:

```bash
./run.sh --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --sleep-pacer
```

Compare lower CPU pacing:

```bash
./run.sh --diag-seconds 30 --diag-warmup-seconds 5 --visual-check --eco-pacer
```

## Tradeoff

The manual pacer only controls the time left after `next_frame().await` returns. At 60 Hz, that leaves enough room for `mach_wait_until` plus a short spin, and the frame pacing is stable at about 30% CPU on the reference MacBook.

At 120 Hz, `next_frame().await` sometimes returns late. Increasing the spin margin does not fix that path, because the slow frame has already happened before the manual pacer gets control. This is now tracked explicitly with `next_frame_ms`, `pacer_os_wait_ms`, and `pacer_spin_ms`.

For the detailed investigation notes, see [docs/frame_pacing_asset.md](docs/frame_pacing_asset.md).

## Font

The bundled HUD font is Silkscreen Regular from Google Fonts.

- Source: https://github.com/google/fonts/tree/main/ofl/silkscreen
- License: SIL Open Font License 1.1
- Local license copy: [assets/fonts/Silkscreen-OFL.txt](assets/fonts/Silkscreen-OFL.txt)
