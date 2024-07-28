<div align="center">
  <h2> TUI for managing wifi </h2>
</div>

## 📸 Demo

![](https://github.com/pythops/impala/assets/57548585/b96e7af4-cba4-49c7-a36f-12c83839134d)

## 💡 Prerequisites

A Linux based OS with [iwd](https://iwd.wiki.kernel.org/) installed.

> [!NOTE]
> You might need to install [nerdfonts](https://www.nerdfonts.com/) for the icons to be displayed correctly.

## 🚀 Installation

### 📥 Binary release

You can download the pre-built binaries from the release page [release page](https://github.com/pythops/impala/releases)

### 📦 crates.io

You can install `impala` from [crates.io](https://crates.io/crates/impala)

```shell
cargo install impala
```

### 🐧AUR

You can install `impala` from the [AUR](https://aur.archlinux.org/packages/impala) with using an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers).

```bash
paru -S impala
```

### Nixpkgs

```shell
nix-env -iA nixpkgs.impala
```

### ⚒️ Build from source

Run the following command:

```shell
git clone https://github.com/pythops/impala
cd impala
cargo build --release
```

This will produce an executable file at `target/release/impala` that you can copy to a directory in your `$PATH`.

## 🪄 Usage

### Global

`Tab`, `Shift + Tab`, `Left`, `Right`, `h`, `l`: Switch between different sections.

`j` or `Down` : Scroll down.

`k` or `Up`: Scroll up.

`ctrl+r`: Switch adapter mode.

`?`: Show help.

`esc`: Dismiss the different pop-ups.

`q` or `ctrl+c`: Quit the app.

### Device

`i`: Show device information.

`o`: Toggle device power.

### Station

`s`: Start scanning.

`Space`: Connect/Disconnect the network.

### Known Networks

`a`: Enable/Disable auto-connect.

`d`: Remove the network from the known networks list.

### Access Point

`n`: Start a new access point.

`x`: Stop the running access point.

## Custom keybindings

Keybindings can be customized in the config file `$HOME/.config/impala/config.toml`

```toml

switch = "r"
mode = "station"
color_mode = "auto"
unicode = true

[device]
infos = "i"
toggle_power = "o"

[access_point]
start = 'n'
stop = 'x'

[station]
toggle_scanning = "s"
toggle_connect = " "
auto_scan = true

[station.known_network]
toggle_autoconnect = "a"
remove = "d"
```

## ⚖️ License

GPLv3
