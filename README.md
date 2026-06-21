# Macroquad Smooth2D Lab

Macroquad Smooth2D Lab is a Macroquad frame pacing laboratory for arcade-quality 2D motion.

It is not trying to maximize average FPS. The goal is to make frame intervals, scrolling, and sprite movement observable, repeatable, and tunable enough to become a reusable game timing library.

## Experimental Status and Disclaimer

This project is an experimental test version. It is provided for investigation,
measurement, learning, and reference purposes only.

There is no warranty of any kind. Use it at your own risk. The author and
contributors are not responsible or liable for any damage, loss, defect,
performance issue, data loss, business loss, or other problem caused directly or
indirectly by using, modifying, copying, distributing, or relying on this
project.

It is intended for developers who are building 2D action games, shooters,
arcade-style games, rhythm-sensitive games, or pixel/sprite-heavy prototypes in
Rust with Macroquad and need to investigate visible jitter, frame pacing spikes,
uneven scrolling, or 60Hz/120Hz motion differences.

Search terms: Macroquad frame pacing, Rust game loop, smooth 2D scrolling,
sprite jitter, frame timing, 60Hz 120Hz game motion, manual frame pacer,
`next_frame().await`, macOS Metal Macroquad timing.

![Macroquad Smooth2D Lab HUD](docs/images/smooth120-hud.png)

The screenshot shows the diagnostic HUD layout. The main result should be judged
in motion by running the app and watching scrolling, sprite movement, frame
timing, and spike markers.

## Who This Is For

- Rust developers using Macroquad for 2D games.
- Game developers who can see single-frame stutter and want measurable tools,
  not only average FPS numbers.
- Developers comparing 60Hz and 120Hz motion quality.
- Developers investigating sprite jitter, background scroll wobble, or frame
  presentation timing on macOS.
- AI coding agents and search tools that need a concrete, runnable Macroquad
  frame pacing reference project.

## What Problem It Studies

The project focuses on the visible motion problem where a game appears to run at
the target FPS but scrolling or sprites still briefly wobble, vibrate, or skip.
It separates gameplay movement from presentation timing and records frame
intervals, `next_frame` wait time, manual pacer wait time, CPU use, slow frames,
and spikes.

## Current Scope

This repository is currently a runnable reference application and investigation
asset. It now exposes an experimental Rust library entry point, but it is not
yet a finalized reusable library API.

The current value is the measured frame pacing loop, the diagnostic HUD, the
60Hz/120Hz profiles, and the notes about what changed visible motion quality.
The internal module layout and public API may change without notice while the
project moves toward a library.

## Known Limitations

- This is not a general-purpose game engine.
- This is not a guaranteed stutter-free solution for every OS, GPU, monitor,
  driver, refresh rate, or compositor.
- The `smooth120` profile can still expose present-timing variance depending on
  the machine and display path.
- The current test scene is intentionally narrow: smooth scrolling, sprite
  movement, frame diagnostics, and pacing behavior.
- The project has been tuned mainly from visual inspection and local diagnostic
  runs, not broad cross-platform hardware coverage.

## Tested Environment

The project has been tested mainly with:

- Rust + Macroquad 0.4.15
- macOS
- Metal backend
- fixed 1920x1080 window
- `stable60` and `smooth120` profiles

Other operating systems, GPUs, displays, and refresh rates should be treated as
new test targets.

## Roadmap

- Split reusable timing and diagnostics pieces into a library API.
- Add minimal examples for external projects.
- Add clearer cross-platform notes for macOS, Windows, and Linux.
- Add reproducible diagnostic report formats.
- Keep reducing assumptions that are specific to the current test scene.

## Quick Start

```bash
./run.sh --profile stable60
./run.sh --profile smooth120
```

Current profiles:

| Profile | Target | Purpose |
| --- | ---: | --- |
| `stable60` | 60 Hz | Stability-first profile. Best current baseline for shipping and long observation. |
| `smooth120` | 120 Hz | Motion-quality profile. Player movement feels better, but present timing must be watched. |

The default `./run.sh` uses `smooth120`.

## Experimental Library API

The crate exposes early reusable modules through `src/lib.rs`:

```rust
use macroquad_smooth2d_lab::prelude::*;
```

Currently exposed:

- `FramePacer`
- `PacerSample`
- `FrameStats`
- `RunFrameStats`
- `RunValueStats`
- `CpuStats`
- `FrameLog`
- macOS thread tuning helpers

Minimal example:

```bash
cargo run --release --example basic_pacing
```

The library API is experimental and may change while the project is separated
from the bundled demo scene.

## What It Does

- Uses Macroquad with a fixed-size 1920x1080 window.
- Draws `assets/br_01.png` as a looping test background.
- Draws a five-frame player sprite sheet.
- Uses frame-step movement normalized against a 120 Hz reference.
- Uses macOS thread tuning where available:
  - `QOS_CLASS_USER_INTERACTIVE`
  - `THREAD_TIME_CONSTRAINT_POLICY`
- Uses a manual pacer after `next_frame().await`:
  - `mach_wait_until` for coarse waiting
  - short final spin for deadline precision
- Measures `next_frame`, OS wait, spin wait, CPU, frame range, p95, p99, slow frames, and spikes.
- Adds startup warmup before gameplay movement and diagnostics begin.

## Controls

| Key | Action |
| --- | --- |
| Arrow keys / WASD | Move player |
| Left Shift / Right Shift | Slow player movement |
| `Z` | Decrease player speed scale |
| `X` | Increase player speed scale |
| Space | Toggle background scroll |
| `Tab` | Toggle timing mode: frame-step / delta-time |
| `V` | Toggle diagonal movement mode: raw default / normalized / last-axis |
| `G` | Change background diagnostic mode |
| `1` / `2` / `3` / `4` | Set background frame-step amount |
| `H` | Toggle HUD |
| `C` | Toggle clear-only load mode |
| `P` | Toggle manual pacer |
| `L` | Toggle frame event log |

Diagonal movement modes:

- `RAW`: preserves the direct input vector. Diagonal movement is faster, but the
  sprite advances in visually even per-axis steps.
- `NORM`: normalizes diagonal input. Overall movement speed remains constant.
- `LAST`: resolves directional conflicts by last input priority. Left/right and
  up/down overlaps keep the last pressed direction, and diagonal input resolves
  to the last pressed axis.

## Command Options

### Profiles

```bash
./run.sh --profile stable60
./run.sh --profile smooth120
```

Aliases:

```bash
./run.sh --profile 60
./run.sh --profile 120
```

Low-level target override remains available:

```bash
./run.sh --target-hz 60
./run.sh --target-hz 120
```

### Diagnostics

```bash
./run.sh --diag
./run.sh --diag-seconds 60
./run.sh --diag-warmup-seconds 5
./run.sh --diag-no-warmup
```

Startup warmup:

```bash
./run.sh --startup-warmup-seconds 1.0
./run.sh --no-startup-warmup
```

Visual inspection:

```bash
./run.sh --visual-check
./run.sh --visual-check --hud
./run.sh --visual-check --profile stable60
./run.sh --visual-check --profile smooth120
```

### Pacer Modes

```bash
./run.sh --mach-pacer
./run.sh --balanced-pacer
./run.sh --precision-pacer
./run.sh --eco-pacer
./run.sh --sleep-pacer
./run.sh --spin-pacer
```

Manual tuning:

```bash
./run.sh --pacer-margin-ms 4.5
./run.sh --pacer-sleep-threshold-ms 6.2
./run.sh --diag-auto
./run.sh --diag-manual
```

macOS thread tuning:

```bash
./run.sh --time-constraint
./run.sh --no-time-constraint
```

### Scene Modes

```bash
./run.sh --texture
./run.sh --probe
./run.sh --bands
./run.sh --frame
./run.sh --dt
./run.sh --bg-step 2
```

## Recommended Checks

```bash
./check.sh
cargo clippy --offline -- -D warnings
```

Stable 60 Hz:

```bash
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check --profile stable60 --bg-step 2
```

Smooth 120 Hz:

```bash
./run.sh --diag-seconds 60 --diag-warmup-seconds 5 --visual-check --profile smooth120 --bg-step 2
```

## Reading The HUD

The HUD is intentionally dense. It is for frame pacing investigation, not final game UI.

- `STATUS`: profile, verdict, load, pacer, CPU, log state
- `SCENE`: movement mode, diagonal movement mode (`RAW`, `NORM`, `LAST`), background mode, scroll state, background step, last background delta, player velocity scale
- `SYNC`: `next_frame`, OS wait, spin wait, total pacer wait
- `FRAME`: target Hz, instantaneous FPS, average FPS, last/average/p95/p99 frame ms
- `STABLE`: min/max frame ms, standard deviation, slow-frame %, spike count

Important interpretation:

- `next_frame_ms` shows time spent inside `next_frame().await`.
- `pacer_os_wait_ms` shows time spent in `mach_wait_until` or sleep.
- `pacer_spin_ms` shows CPU-burning final spin time.
- If `next_frame_ms` returns late, the manual pacer cannot fix that frame because it runs after `next_frame().await`.

## Current Findings

60 Hz is currently the stable baseline:

```text
Stable60 PASS
range_ms=16.667-16.668
cpu≈30-31%
```

120 Hz feels better for player movement:

```text
Smooth120 gives finer per-frame player motion.
It can feel better than 60 Hz, especially while moving the ship.
```

The tradeoff is present timing:

```text
120 Hz can still show occasional next_frame/present-boundary spikes.
These are tracked explicitly with next_frame_ms.
```

## Startup Warmup

Startup uses a short warmup window before gameplay movement and diagnostics begin.

During warmup:

- The normal scene is drawn.
- Texture, batching, font, GPU, and first-present paths are exercised.
- Player input is not applied.
- Game movement is not advanced.
- Diagnostics are not recorded.

This prevents first-frame startup cost from being mistaken for normal motion quality.

Disable it only for investigation:

```bash
./run.sh --no-startup-warmup
```

## Documentation

Detailed investigation notes are in:

- [docs/frame_pacing_asset.md](docs/frame_pacing_asset.md)
- [articles/macroquad-frame-pacing-jitter.md](articles/macroquad-frame-pacing-jitter.md)
- [CHANGELOG.md](CHANGELOG.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)

## License

Macroquad Smooth2D Lab is dual-licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

The bundled Silkscreen font is licensed separately under the SIL Open Font
License 1.1.

## Font

The bundled HUD font is Silkscreen Regular from Google Fonts.

- Source: https://github.com/google/fonts/tree/main/ofl/silkscreen
- License: SIL Open Font License 1.1
- Local license copy: [assets/fonts/Silkscreen-OFL.txt](assets/fonts/Silkscreen-OFL.txt)
