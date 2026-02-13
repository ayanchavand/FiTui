# FiTui
FiTui is a small terminal-based personal finance tracker built in Rust.  
It lets you record transactions, view totals, and see spending breakdowns by tag.  
The interface is keyboard-driven and runs entirely inside the terminal.

---

## Features
[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

- Add credit or debit transactions
- Navigate and delete entries
- View overall stats (earned, spent, balance)
- Spending breakdown grouped by tag
- Local persistence using SQLite
- Configurable tags stored in a YAML file

### Screenshots

Main interface:

![Main interface](assets/main_page.png)

Stats view:

![Stats view](assets/stats_page.png)

---

## Controls

### Normal Mode

- `↑ / ↓` : Navigate transactions  
- `a`     : Add transaction  
- `d`     : Delete selected transaction  
- `s`     : Open stats view  
- `q`     : Quit  

### Form Mode

- `Tab`   : Next field  
- `← / →` : Toggle type or cycle tags  
- `Enter` : Save transaction  
- `Esc`   : Cancel  

### Stats Mode

- `Esc`   : Back to main view  

---

## Data Storage

FiTui stores files in standard OS-specific locations.

### Database (`budget.db`)

Stored in the system data directory:

- **Linux**: `~/.local/share/fitui/budget.db`
- **macOS**: `~/Library/Application Support/com.ayan.fitui/budget.db`
- **Windows**: `AppData\Roaming\ayan\fitui\data\budget.db`

### Config (`config.yaml`)

Stored in the system config directory:

- **Linux**: `~/.config/fitui/config.yaml`
- **macOS**: `~/Library/Preferences/com.ayan.fitui/config.yaml`
- **Windows**: `AppData\Roaming\ayan\fitui\config\config.yaml`

The config file is automatically created on first run.

---

## Building

Requires Rust installed.

```bash
cargo build --release
```

Binary will be located at:

```
target/release/fitui
```

(on Windows: `fitui.exe`)

---

## Installing

### Windows

1. Build the release binary
2. Copy `fitui.exe` somewhere permanent, e.g.
   ```
   C:\Users\<you>\cli\
   ```
3. Add that folder to your PATH
4. Then run:
   ```powershell
   fitui
   ```

### Linux / macOS

1. Copy the binary into a directory on your PATH:
   ```bash
   mkdir -p ~/.local/bin
   cp target/release/fitui ~/.local/bin/
   chmod +x ~/.local/bin/fitui
   ```
2. Run:
   ```bash
   fitui
   ```

---

## Tags

Transaction tags are loaded from `config.yaml`.

Example:

```yaml
tags:
  - food
  - travel
  - shopping
  - bills
  - salary
  - other
```

---

## License

MIT
