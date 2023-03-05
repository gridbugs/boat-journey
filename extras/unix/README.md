# Boat Journey

## Package Contents

- boat-journey-graphical: Graphical version of the game, rendering with metal on macos and vulkan on linux
- boat-journey-terminal: Terminal version of the game, rendering as text in an ansi terminal

## HIDPI

HIDPI scaling can make the game run larger than the screen size on some monitors.
The `WINIT_X11_SCALE_FACTOR` environment variable overrides the HIDPI scaling factor.

For example:
```
WINIT_X11_SCALE_FACTOR=1 ./boat-journey-graphical
```

## MacOS

In order to run binaries on MacOS, you may need to navigate to this directory
in Finder and right click -> Open on the binaries, then choose Open at the
prompt.
