# Contributing

Macroquad Smooth2D Lab is currently an experimental test version.

Issues and diagnostic reports are welcome, especially when they include enough
environment information and frame timing output to reproduce or compare the
result.

API-level contributions may be delayed until the reusable library shape is more
stable.

## Useful Reports

Please include:

- OS, CPU, GPU, and display refresh rate
- command used
- profile used: `stable60`, `smooth120`, or custom
- final diagnostic output
- visible symptom
- whether an external monitor, screen recorder, or battery-saving mode was used

## Development Check

Before submitting changes, run:

```bash
./check.sh
```

For timing behavior, prefer release builds:

```bash
./run.sh --profile stable60
./run.sh --profile smooth120
```
