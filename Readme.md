<div align="center">
  <h2> TUI for managing wifi </h2>
  <br>
  <img src="https://github.com/pythops/impala/assets/57548585/3cfdf034-9b00-41ba-85ac-a079b5cf6b65"/>
</div>

## 💡 Prerequisites

A Linux based OS with [iwd](https://iwd.wiki.kernel.org/) installed.

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

`Tab`: Switch between different sections.

`j` or `Down` : Scroll down.

`k` or `Up`: Scroll up.

`s`: Start scanning.

`Space`: Connect/Disconnect the network.

`?`: Show help.

`esc`: Dismiss the help pop-up.

`q` or `ctrl+c`: Quit the app.

### Device

`i`: Show device information.

### Known Networks

`d`: Remove the network from the known networks list.

## Custom keybindings

Keybindings can be customized in the config file `$HOME/.config/impala/config.toml`

```toml

[device]
infos = "i"

[access_point]
start = 's'
stop = 'x'

[station]
toggle_scanning = "s"
toggle_connect = " "

[station.known_network]
remove = "d"
```

## ⚖️ License

GPLv3
