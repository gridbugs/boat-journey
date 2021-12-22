# Orbital Decay

[![Appveyor Status](https://ci.appveyor.com/api/projects/status/gitlab/stevebob/orbital-decay?branch=main&svg=true)](https://ci.appveyor.com/project/stevebob/orbital-decay)
[![dependency status](https://deps.rs/repo/gitlab/stevebob/orbital-decay/status.svg)](https://deps.rs/repo/gitlab/stevebob/orbital-decay)

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

For nightly rust:
```
nix-shell nix/nightly.nix
```

## Debug Environment

Source the script `debug_env_linux.sh` to set cargo environment variables for faster builds:
```
. debug_env_linux.sh
```
