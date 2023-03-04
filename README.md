# Gridbugs Template 2023

[![dependency status](https://deps.rs/repo/github/gridbugs/gridbugs-template-2023/status.svg)](https://deps.rs/repo/github/gridbugs/gridbugs-template-2023)
[![test status](https://github.com/gridbugs/gridbugs-template-2023/actions/workflows/test.yml/badge.svg)](https://github.com/gridbugs/gridbugs-template-2023/actions/workflows/test.yml)

Starting point for programs made with the gridbugs library

## HIDPI

HIDPI scaling can make the game run larger than the screen size on some monitors.
The `WINIT_X11_SCALE_FACTOR` environment variable overrides the HIDPI scaling factor.

For example:
```
WINIT_X11_SCALE_FACTOR=3 cargo run --manifest-path wgpu/Cargo.toml
```

## Nix

To set up a shell with an installation of rust and external dependencies:
```
nix-shell
```
...or for nix flakes users:
```
nix develop
```
