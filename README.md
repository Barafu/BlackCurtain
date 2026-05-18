# black-curtain

I have two monitors. Sometimes I want to turn one of them off when it distracts me. But turning the monitor off causes OS to rearrange windows and sometimes even fail at that. Changing brightness programmatically also does not work reliably. So I made this. 

A GUI app that fills a monitor with a solid color — useful to quickly black out a specific screen. Supports custom colors, control through both mouse and keyboard.

## Controls

| Action              | Mouse                 | Keyboard |
| ------------------- | --------------------- | -------- |
| Toggle fullscreen   | Double-click          | Space    |
| Minimize window     | Right-click           | Enter    |

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
The application was made using DeepSeek LLM
