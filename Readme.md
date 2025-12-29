<div align="center">
  <h2> TUI for managing wifi </h2>
</div>

## 📸 Demo

![](https://github.com/user-attachments/assets/55c800ff-d0aa-4454-aa6b-3990833ce530)

## ✨ Features

- WPA Enterprise (802.1X) Support
- Station & Access Point Modes
- QR Code Network Sharing
- Support hidden networks

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

```bash
$ impala
```

## 🛠️Custom keybindings

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

[station.known_network]
toggle_autoconnect = "t"
remove = "d"
show_all = "a"
share = "p"

[station.new_network]
show_all = "a"
connect_hidden = ""
```

## Contributing

- No AI slop.
- Only submit a pull request after having a prior issue or discussion.
- Keep PRs small and focused.

## ⚖️ License

GPLv3
