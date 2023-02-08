# Orbital Decay

[![dependency status](https://deps.rs/repo/github/gridbugs/orbital-decay/status.svg)](https://deps.rs/repo/github/gridbugs/orbital-decay)
[![test status](https://github.com/gridbugs/orbital-decay/actions/workflows/test.yml/badge.svg)](https://github.com/gridbugs/orbital-decay/actions/workflows/test.yml)
[![ci status](https://github.com/gridbugs/orbital-decay/actions/workflows/ci.yml/badge.svg)](https://github.com/gridbugs/orbital-decay/actions/workflows/ci.yml)

A  a turn-based tactical roguelike with a focus on ranged combat.

![screenshot](/images/screenshot1.png)

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
