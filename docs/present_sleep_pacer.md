# Present Sleep Pacer

This note documents the post-frame sleep pacer. It is now the default pacer for
the runnable lab app.

The mode is intentionally generic. It does not include third-party framework
source code, and it is not a port of another engine's main loop. It sleeps after
Macroquad's `next_frame().await` to reduce CPU use while keeping visible frame
pacing measurable.

## Command

```bash
./run.sh --present-sleep-pacer
./run.sh --post-frame-sleep
```

The default architecture is equivalent to:

```bash
./run.sh --profile smooth120 --logic60-draw --present-sleep-pacer --hud
```

## Behavior

1. Run the normal update and draw path for the selected loop mode.
2. Call `next_frame().await`.
3. Record `next_frame_ms`.
4. Sleep for `PACER_PRESENT_SLEEP_SECS`.
5. Do not spin.

This mode is not a precise frame limiter by itself. It is best understood as a
low-CPU presentation experiment. On a 120Hz display it can still render near
120 FPS.

## Recommended Pairing

The default setup is:

- fixed 60Hz game logic via `--logic60-draw`
- display-rate rendering with interpolation
- post-frame sleep via `--present-sleep-pacer`

This separates the responsibilities:

- game correctness: fixed 60Hz logic
- visual smoothness: display-rate rendering plus interpolation
- CPU cost: small post-frame sleep
- display jitter: measured separately as frame pacing

## Measured Results

These results were recorded on the current development machine in release mode.
They are useful as local evidence, not as a universal hardware guarantee.

### `smooth120` + present sleep

Command:

```bash
./run.sh --profile smooth120 --diag-seconds 30 --diag-warmup-seconds 5 --visual-check --hud --present-sleep-pacer
```

Result:

- `verdict=WARN`
- `avg_fps=120.0`
- `avg_ms=8.333`
- `p95_ms=9.251`
- `p99_ms=9.701`
- `range_ms=3.274-12.379`
- `sd_ms=0.498`
- `slow_pct=0.1`
- `spikes=0`
- `cpu=14.7%`
- `pacer_spin_ms avg=0.000`
- `pacer_os_wait_ms avg=1.016`
- `next_frame_ms avg=7.166 range=1.881-11.116 sd=0.505`

Interpretation: CPU use drops because there is no spin wait, but frame pacing is
less tight than the balanced/default pacer.

### `stable60` + present sleep

Command:

```bash
./run.sh --profile stable60 --diag-seconds 30 --diag-warmup-seconds 5 --visual-check --hud --present-sleep-pacer
```

Result:

- `verdict=PASS`
- `avg_fps=120.0`
- `avg_ms=8.333`
- `p95_ms=8.850`
- `p99_ms=8.953`
- `range_ms=6.438-10.230`
- `sd_ms=0.283`
- `slow_pct=0.0`
- `spikes=0`
- `cpu=8.2%`
- `pacer_spin_ms avg=0.000`
- `pacer_os_wait_ms avg=1.009`
- `next_frame_ms avg=7.231 range=5.335-9.137 sd=0.283`

Interpretation: this mode still follows the display/frame boundary and did not
hold 60 FPS on the tested 120Hz display. A fixed one millisecond sleep after
`next_frame` is not a frame-rate limiter by itself.

### `smooth120` + fixed 60 logic + present sleep

Command:

```bash
./run.sh --profile smooth120 --logic60-draw --present-sleep-pacer --diag-seconds 10 --diag-warmup-seconds 2 --visual-check --hud
```

Result:

- `verdict=WARN`
- `avg_fps=120.0`
- `avg_ms=8.333`
- `p99_ms=9.643`
- `sd_ms=0.478`
- `slow_pct=0.2`
- `spikes=0`
- `cpu=12.6%`
- `bg_delta avg=4.000`

Interpretation: this is the current default architecture because visible motion
separates fixed logic from display-rate rendering.
