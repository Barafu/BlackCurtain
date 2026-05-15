# black-curtain

A GUI app that fills a monitor with a solid color — useful to quickly black out a specific screen. Supports custom colors, control through both mouse and keyboard.

## Installation

### Manual (recommended)

Download the binary and place it anywhere in PATH. Make sure it has execution permissions. Run it with `--install` argument to register the app so it appears in your desktop's menu:

```sh
black-curtain --install
```

This creates an XDG desktop entry and installs the application icon. To remove
it from the menu:

```sh
black-curtain --uninstall
```

Both commands require `xdg-desktop-menu` and `xdg-icon-resource` to be
available on your system.

## Usage

```
black-curtain [COLOR]
black-curtain --install
black-curtain --uninstall
```

`COLOR` is an optional hex color (e.g. `#000000`, `#ff0000`, `#333`). If
omitted, the curtain defaults to black.

### Controls

| Action              | Mouse                 | Keyboard |
| ------------------- | --------------------- | -------- |
| Toggle fullscreen   | Double-click          | Space    |
| Minimize window     | Right-click           | Enter    |

### Examples

```sh
# Full blackout
black-curtain

# Red dimmer
black-curtain '#ff0000'

# Dark gray (3-digit shorthand)
black-curtain '#222'
```

## License

MIT
