# Nett Icon Viewer

Nett Icon Viewer is a program that displays icons from a GTK icon theme.

It is currently a work in progress and is a work of concept.

The word Nett is German for "Nice", so the literal translation of the name is "Nice Icon Viewer."

This name may change, but I think it is funny, so I might keep it.

## Building 

The main application is built with [`gtk-rs`](https://gtk-rs.org/) so you must have the required dependencies for it to be installed.

### Prerequisites

#### Fedora

```bash
sudo dnf install gtk4-devel
```

#### Debian/Ubuntu

```bash
sudo apt install libgtk-4-dev build-essential
```

#### Arch Linux

```bash
sudo pacman -S gtk4 base-devel
```

### Building

```bash
cargo build
```
or
```bash
cargo run
```


## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to use [conventional commit messages](https://www.conventionalcommits.org/en/v1.0.0/) when committing.

## License

[GNU General Public License v3.0](https://choosealicense.com/licenses/gpl-3.0/)
