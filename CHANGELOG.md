# Changelog

All notable changes to this project will be documented in this file.

This project is currently experimental. Version history may describe
investigation milestones rather than stable library API changes.

## [0.1.0-experimental.1] - 2026-06-21

### Added

- Experimental `src/lib.rs` entry point.
- `prelude` exports for early reusable frame pacing and diagnostics types.
- `examples/basic_pacing.rs` minimal library-use example.
- Diagnostic HUD screenshot in README.
- Bilingual article draft.

### Changed

- Demo app now uses reusable modules through the crate library boundary.
- GitHub Actions now runs format check, all-target check, and all-target clippy.
- Diagonal movement defaults to `RAW`, with `NORM` still available through `V`.
- HUD readability improved with colored diagnostic rows.

## [0.1.0-experimental] - 2026-06-21

### Added

- Macroquad frame pacing investigation app.
- `stable60` and `smooth120` runtime profiles.
- Diagnostic HUD for frame timing, CPU, pacer wait, `next_frame` wait, slow
  frames, and spikes.
- Manual pacer modes for timing experiments.
- Looping background scroll and sprite movement test scene.
- Startup warmup before measurement.
- Public metadata, license files, experimental disclaimer, and AI-readable
  `llms.txt`.

### Known Limitations

- Not a finalized library API.
- Not a guaranteed stutter-free solution for all hardware, displays, operating
  systems, drivers, or compositors.
- Tuned mainly from local visual inspection and diagnostics.
