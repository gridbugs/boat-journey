# Orbital Decay

[![Appveyor Status](https://ci.appveyor.com/api/projects/status/gitlab/stevebob/orbital-decay?branch=master&svg=true)](https://ci.appveyor.com/project/stevebob/orbital-decay)
[![dependency status](https://deps.rs/repo/gitlab/stevebob/orbital-decay/status.svg)](https://deps.rs/repo/gitlab/stevebob/orbital-decay)

## HIDPI

HIDPI scaling can make the game run larger than the screen size on some monitors.
The `WINIT_X11_SCALE_FACTOR` environment variable overrides the HIDPI scaling factor.

For example:
```
WINIT_X11_SCALE_FACTOR=3 cargo run --manifest-path graphical/Cargo.toml
```
