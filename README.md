# Rustris

Tetris with the guideline ruleset and a retro theme.
Written in SDL2 and Rust for fun.

## Building

Requires vcpkg to build.

```bash
cargo install cargo-vcpkg
cargo vcpkg build
cargo build --release
```

All resources are embedded into the binary.

### macOS

The linker will fail to link SDL2 haptics. You will need to add the following to `~/.cargo/config.toml`:

```toml
[target.aarch64-apple-darwin]
rustflags = ["-C", "link-args=-weak_framework CoreHaptics"]
```

### Retro handhelds

I have built this successfully for [ArkOS](https://github.com/christianhaitian/arkos) on the [Anberic rg353m](https://anbernic.com/products/rg353m).
For this we need an ancient linux distro having glibc <2.30 but a fairly new SDL2 build.
It can be cross compiled in Docker with script:

```shell
./build-aarch64-cross.sh
```

## Config

Config is stored in yaml:

* Windows: `$HOME\AppData\Roaming\rustris`
* MacOS: `$HOME/Library/Application Support/rustris`
* Linux: `$XDG_CONFIG_HOME/rustris` or `$HOME/.config/rustris`

Most of it you can ignore except:

### Video Mode

* `Window` (default) - note if your screen is not at least 720p then rustris may not even load on first attempt.
    ```yaml
    video:
      mode: !Window
        width: 1280
        height: 720
    ```
* `FullScreen` - native fullscreen (recommended), note rustris should scale to any weird resolution but was designed for 1080p & 4k.
    ```yaml
    video:
      mode: !FullScreen
        width: 1920
        height: 1080
    ```  
* `FullScreenDesktop` - fullscreen in windowed mode
    ```yaml
    video:
      mode: !FullScreenDesktop
    ```  

### Controls

Only keyboard controls are supported (I play this on a custom arcade cabinet with a programmable keyboard encoder).

```yaml
input:
  menu:
    up: Up
    down: Down
    left: Left
    right: Right
    select: X
    start: Return
  player1:
    move_left: Left
    move_right: Right
    soft_drop: Down
    hard_drop: Up
    rotate_clockwise: X
    rotate_anticlockwise: Z
    hold: LShift
  player2: ~
  pause: F1
  next_theme: F2
  quit: Escape
```

All key names are defined in [src/config.rs](src/config.rs).

There are no default player 2 controls.

## TODO
* t-spin
* critical music
* game boy color theme