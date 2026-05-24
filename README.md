# PVM (PHP Version Manager)

PVM is a lightweight, high-performance command-line PHP Version Manager for Windows, rewritten in **Rust** for maximum speed, security, and portability. It allows you to easily install, switch between, and configure different versions of PHP on your machine.

---

## Features

* **Zero Config Setup**: Automatically clean up existing PHP paths and configure Windows Environment Variables system-wide.
* **Auto Import**: Automatically detect pre-existing PHP installations on your system during setup and register them under PVM.
* **Registry Refresh**: Instantly updates environment variables for newly opened terminal windows without needing a computer reboot.
* **Database Cache**: Uses a local SQLite database to persist installations and cached remote release URLs.
* **Extension Control**: Enable extensions (like `curl`) or open PHP folders natively in Notepad/Explorer.

---

## Setup & Installation

1. **Download PVM**: Extract the PVM application folder to a permanent directory of your choice on your system (e.g. `C:\tools\pvm`).
2. **Configure PATH**: Open a Command Prompt or PowerShell window **as an Administrator** and execute:
   ```powershell
   pvm setup
   ```
   * This command will detect if you have any existing PHP installations, import them, add PVM's directory to the System PATH, and clean up conflicting paths automatically.
3. **Restart Shell**: Open a new terminal window to begin using `pvm` globally.

---

## Usage

### 1. Show available remote versions
List all PHP versions available for download from `windows.php.net`:
```powershell
pvm list-remote
```

### 2. Install a specific version
Download, extract, and register a specific PHP version:
```powershell
pvm install 8.3.3
```
* Use the `--type` flag to download Thread Safe (`ts`) version instead of Non-Thread Safe (`nts` - default):
  ```powershell
  pvm install 8.3.3 --type ts
  ```

### 3. List installed versions
List all locally installed PHP versions registered under PVM:
```powershell
pvm list
```

### 4. Switch PHP version
Switch the active PHP version to the specified version:
```powershell
pvm use 8.3.3
```

### 5. Add a local PHP installation
Add a manually downloaded or pre-existing local PHP directory to PVM:
```powershell
pvm add --version 8.2.10 --path C:\path\to\php-8.2.10
```

### 6. Uninstall a version
Remove a registered PHP version and delete its local directory:
```powershell
pvm uninstall 8.3.3
```

### 7. Manage extensions
* **Enable extension**: Uncomments the extension inside the active `php.ini` file:
  ```powershell
  pvm ext-enable curl
  ```
* **Open extension folder**: Opens the active PHP version's extensions folder in Windows Explorer:
  ```powershell
  pvm ext
  ```

### 8. Edit Configuration
Open the active PHP configuration (`php.ini`) file directly in Notepad:
```powershell
pvm ini
```

---

## Compiling from Source

Ensure you have Rust installed (Edition 2024).

1. Clone the repository and navigate to the directory:
   ```powershell
   git clone https://github.com/sagarkarn/PVM.git
   cd PVM
   ```
2. Build the optimized release binary:
   ```powershell
   cargo build --release
   ```
3. The executable will be generated at `target/release/PVM.exe`.

---

## License

This project is licensed under the [MIT License](LICENSE).
