# FiTui
FiTui is a small terminal-based personal finance tracker built in Rust.  
It lets you record transactions, view totals, and see spending breakdowns by tag.  
The interface is keyboard-driven and runs entirely inside the terminal.

Version: 0.1.4
---

## Features
[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

- Add credit or debit transactions
- Navigate and delete entries
- View overall stats (earned, spent, balance)
- Spending breakdown grouped by tag
- Local persistence using SQLite
- Configurable tags and cuurency stored in a YAML file
- Recurring transactions auto-insert monthly transactions (salary, bills, etc.)

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
- `← / →` : Toggle type, cycle tags, or toggle recurring  
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

### Termux (Android)

1. Install Rust (if not already installed):
   ```bash
   pkg update && pkg upgrade
   pkg install rust
   ```

2. Clone or download the FiTui source code into Termux

3. Build the release binary:
   ```bash
   cargo build --release
   ```

4. The binary will be located at `target/release/fitui`. Run it with:
   ```bash
   ./target/release/fitui
   ```

   Or copy it to your PATH:
   ```bash
   cp target/release/fitui ~/.local/bin/
   fitui
   ```

**Note:** First build in Termux may take 10-15 minutes due to compilation time on mobile devices.

---

## Recurring Transactions

FiTui supports recurring transactions that are automatically inserted each month.

### How It Works

1. When adding a transaction, press Tab to reach the "Recurring" field
2. Press `←` or `→` to toggle it to "🔄 Yes"
3. Save the transaction with `Enter`
4. On app startup each new month, the recurring transaction is automatically re-created
5. Recurring entries are only inserted once per month, preventing duplicates

### Example Use Cases

- **Monthly salary**: Automatically add income each month
- **Rent/Mortgage**: Auto-insert fixed housing payments
- **Subscriptions**: Bill payments (internet, streaming services, etc.)
- **Fixed expenses**: Insurance, utilities, gym membership

### Current Behavior

Recurring transactions are inserted on the **same date each month** as they were originally created. For example:
- Create on Feb 15 → Auto-inserts on Mar 15, Apr 15, May 15, etc.

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

## Currency

You can customize the currency symbol displayed throughout FiTui by setting the `currency` field in `config.yaml`. The default currency is `₹`.

Example:

```yaml
currency: €
```

Common currency symbols: `$`, `€`, `£`, `¥`, `₹`, `₽`, `₩`, `฿`, `₪`, `₦`, `₱`, `₡`, `₲`, `₴`, `₵`

## License

MIT
