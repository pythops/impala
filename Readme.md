<div align="center">
  <h2> TUI for managing wifi </h2>
</div>

## 📸 Demo

![](https://github.com/user-attachments/assets/55c800ff-d0aa-4454-aa6b-3990833ce530)

## ✨ Features

- WPA Enterprise (802.1X) Support
- Station & Access Point Modes
- QR Code Network Sharing

## 💡 Prerequisites

- A Linux based OS
- [iwd](https://iwd.wiki.kernel.org/) running.
- [nerdfonts](https://www.nerdfonts.com/) (Optional) for icons.

> [!IMPORTANT]
> To avoid conflicts, ensure wireless management services like NetworkManager or wpa_supplicant are disabled.

## 🚀 Installation

### 📥 Binary release

You can download the pre-built binaries from the release page [release page](https://github.com/pythops/impala/releases)

### 📦 crates.io

You can install `impala` from [crates.io](https://crates.io/crates/impala)

```shell
cargo install impala
```

### 🐧Arch Linux

You can install `impala` from the [official repositories](https://archlinux.org/packages/extra/x86_64/impala/) with using [pacman](https://wiki.archlinux.org/title/pacman).

```bash
pacman -S impala
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

`Tab` or `Shift + Tab`: Switch between different sections.

`j` or `Down` : Scroll down.

`k` or `Up`: Scroll up.

`ctrl+r`: Switch adapter mode.

`?`: Show help.

`esc`: Dismiss the different pop-ups.

`q` or `ctrl+c`: Quit the app. (Note: `<Esc>` can also quit if `esc_quit = true` is set in config)

### Device

`i`: Show device information.

`o`: Toggle device power.

### Station

`s`: Start scanning.

`Space or Enter`: Connect/Disconnect the network.

### New Networks

`a`: Show all the new networks.

`h`: Add a hidden network manually.

### Known Networks

`t`: Enable/Disable auto-connect.

`d`: Remove the network from the known networks list.

`a`: Show all the known networks.

`p`: Share via QR Code.

### Access Point

`n`: Start a new access point.

`x`: Stop the running access point.

## Custom keybindings

Keybindings can be customized in the config file `$HOME/.config/impala/config.toml`

```toml

switch = "r"
mode = "station"
esc_quit = false  # Set to true to enable Esc key to quit the app

[device]
infos = "i"
toggle_power = "o"

[access_point]
start = 'n'
stop = 'x'

[station]
toggle_scanning = "s"

[station.new_network]
show_all = "a"
add_hidden = "h"

[station.known_network]
toggle_autoconnect = "t"
remove = "d"
show_all = "a"
share = "p"
```

## ⚖️ License

GPLv3
