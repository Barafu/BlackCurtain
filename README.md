# Black Curtain

I have two monitors. Sometimes I want to turn one of them off when it distracts me. But turning the monitor off causes OS to rearrange windows and sometimes even fail at that. Changing brightness programmatically also does not work reliably. So I made this. 

A GUI app that fills a monitor with a solid color — useful to quickly black out a specific screen. Supports custom colors, control through both mouse and keyboard.

## Controls

| Action              | Mouse                 | Keyboard |
| ------------------- | --------------------- | -------- |
| Toggle fullscreen   | Double-click          | Space    |
| Minimize window     | Right-click           | Enter    |

## Installation

On Linux, download the AppImage. Most likely your distro has an established way to install AppImages, like GearLevel or AppImageD. If you install it, you can customise the color by editing the menu entry. Or just doubleclick it. 

Windows version is like winter: it is coming. 

## Usage

```
black-curtain [COLOR]
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
