<p align="center">
  <img src="static/logo.png" alt="Zephyr" width="120" />
</p>

<h1 align="center">Zephyr</h1>
<p align="center"><strong>A fast, modern mod manager for all your games</strong></p>
<p align="center">by <a href="https://prismo-studios.com">Prismo Studio</a></p>

<p align="center">
  <a href="https://github.com/prismo-studio/zephyr/releases"><img src="https://img.shields.io/badge/version-0.2.0-blue?style=flat" alt="Version 0.2.0" /></a>
  <a href="https://github.com/prismo-studio/zephyr/blob/main/LICENSE.md"><img src="https://img.shields.io/badge/license-GPL--3.0-green?style=flat" alt="License GPL-3.0" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey?style=flat" alt="Platform Windows | Linux" />
</p>

---

## Screenshots

<!-- Add screenshots here -->
<!-- ![Zephyr main view](docs/screenshots/main.png) -->

## Features

- **Multi-game support** -- 198+ games from Thunderstore, including Lethal Company, R.E.P.O., and Risk of Rain 2
- **Thunderstore integration** -- Browse, install, and update mods directly from Thunderstore
- **Profile management** -- Create and switch between mod profiles for each game
- **Config editor** -- Edit mod configuration files with a built-in, structured editor
- **Custom themes** -- 4 built-in themes with full CSS custom property support
- **Multi-language support** -- Available in 7 languages
- **Fast Rust backend** -- Native performance powered by Tauri 2 and Rust

## Quick Start

### Download

Head to the [Releases](https://github/prismo-studio/zephyr/releases) page and grab the latest installer for your platform:

- **Windows** -- `.msi` installer
- **Linux** -- AppImage, `.deb`, or `.rpm`

> **Note:** On Windows, you may see a SmartScreen prompt. Click "More Info" then "Run Anyway" to proceed.

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)

#### Commands

```bash
git clone https://github/prismo-studio/zephyr.git
cd zephyr
pnpm install
pnpm tauri dev
```

To create a production build:

```bash
pnpm tauri build
```

## Tech Stack

| Layer     | Technology            |
| --------- | --------------------- |
| Framework | Tauri 2               |
| Frontend  | Svelte 5              |
| Backend   | Rust                  |
| Styling   | CSS Custom Properties |

## Fork Info

Zephyr is forked from [Gale](https://github/Kesomannen/gale) by [Kesomannen](https://github/Kesomannen).

## License

This project is licensed under [GPL-3.0](LICENSE.md).

Material icons are licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0.html).

## Credits

Built and maintained by [Prismo Studio](https://prismo-studio).
