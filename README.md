# FiTui

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)
![Crates.io Version](https://img.shields.io/crates/v/FiTui?style=for-the-badge)
![Crates.io Downloads (latest version)](https://img.shields.io/crates/dv/fitui?style=flat-square)

A lightweight terminal-based personal finance tracker. Record transactions, track spending, and view financial insights from your terminal.

**Version:** 0.3.0

---

## Features

- Transaction management: add, edit, and delete credit/debit entries
- Stats view with totals and spending breakdowns by tag
- Recurring transactions for bills, salary, and subscriptions
- Local SQLite storage with configurable tags and currency
- Keyboard-driven interface

### Screenshots

#### Main Interface
![Main interface](assets/main_page.png)

#### Stats View
![Stats view](assets/stats_page.png)

---

## Installation

### Via Cargo

```bash
cargo install fitui
```

### Build from Source

Requires [Rust](https://rustup.rs/).

```bash
cargo build --release
```

Binary: `target/release/fitui` (Windows: `fitui.exe`)

**Linux / macOS**

```bash
mkdir -p ~/.local/bin
cp target/release/fitui ~/.local/bin/
chmod +x ~/.local/bin/fitui
fitui
```

Ensure `~/.local/bin` is in your `$PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**Windows**

1. Copy `fitui.exe` to a permanent location (e.g. `C:\Users\<you>\bin\`)
2. Add that folder to your `PATH` via System Properties
3. Run `fitui` from any terminal

**Termux (Android)**

```bash
pkg install rust
cargo build --release
cp target/release/fitui ~/.local/bin/
fitui
```

Note: first build may take 10-15 minutes on mobile.

---

## Configuration

Config is created automatically on first run.

| OS | Database | Config |
|----|----------|--------|
| Linux | `~/.local/share/fitui/budget.db` | `~/.config/fitui/config.yaml` |
| macOS | `~/Library/Application Support/com.ayan.fitui/budget.db` | `~/Library/Preferences/com.ayan.fitui/config.yaml` |
| Windows | `AppData\Roaming\ayan\fitui\data\budget.db` | `AppData\Roaming\ayan\fitui\config\config.yaml` |

### config.yaml

```yaml
currency: "$"  # $, EUR, GBP, JPY, INR, etc.
theme: "default"  # default, dracula, nord, gruvbox, or any custom theme

tags:
  - food
  - travel
  - shopping
  - bills
  - salary
  - other

custom_themes:
  dracula:
    accent: "#bd93f9"
    accent_soft: "#6272a4"
    credit: "#50fa7b"
    debit: "#ff5555"
    muted: "#6272a4"
    subtle: "#44475a"
    background: "#282a36"
    surface: "#343746"
    row_alt: "#2d2f3b"
    foreground: "#f8f8f2"
```

## Themes & Customization

FiTui supports out-of-the-box themes and full color customization using the `config.yaml` file.

### Built-in Themes
Change the `theme` field in `config.yaml` to one of the following:
* `default` - Ocean blue dark palette
* `dracula` - Vibrant vampire purple palette
* `nord` - Cool arctic frost blue palette
* `gruvbox` - Warm retro yellow/green palette

### Custom Themes
You can define your own themes under the `custom_themes` key or override the preconfigured ones. Colors can be defined as hex codes (e.g. `#ffffff`), standard color names (e.g. `red`, `blue`), or RGB format (e.g. `rgb(24, 24, 36)`):

| Property | Description |
|---|---|
| `accent` | Main accent color (headers, cursor, active tab, and main titles) |
| `accent_soft` | Border color for panel boundaries |
| `credit` | Text color for income (credits) |
| `debit` | Text color for expenses (debits) |
| `muted` | Labels and secondary metadata hints |
| `subtle` | Borders, vertical grids, and divider lines |
| `background` | Primary terminal view background |
| `surface` | Popups, highlights, and header row background |
| `row_alt` | Alternating table row background color |
| `foreground` | Standard text color |

---

## Planned

- CSV import from bank statements and payment apps
- Budget limits per tag
- Search and filter by amount, date, or tag
- Export to CSV/PDF
- Custom date range stats
- Multi-currency support
- Transaction notes
- Data backup and sync

Feature request or bug? [Open an issue](https://github.com/ayanchavand/fitui/issues).

---

## Changelog

### [0.3.0] - 2026-02-27

**Added**
- Tab-based navigation across views
- Active tab indicator
- Transaction date grouping
- Ratio-based column constraints for consistent table layout
- Improved empty state messaging

**Changed**
- Transactions list refactored to table layout
- UI formatting and spacing improvements
- Cleaner tab rendering

**Fixed**
- Quit now works regardless of current mode

---

## License

MIT