# Loresmith Installation Guide

## System Requirements

Before installing Loresmith, ensure your system meets these requirements:

### Minimum Requirements
- **Operating System**: Linux, macOS, or Windows 10+
- **Processor**: 1 GHz or faster
- **Memory**: 512 MB RAM
- **Storage**: 100 MB available space
- **Display**: Terminal/console with UTF-8 support

### Recommended Requirements
- **Operating System**: Linux or macOS (latest versions)
- **Processor**: 2 GHz dual-core or better
- **Memory**: 2 GB RAM
- **Storage**: 500 MB available space
- **Display**: Terminal with 256-color support

## Installation Methods

### Method 1: Pre-built Binaries (Recommended)

#### Linux
```bash
# Download the latest Linux release (replace <version> with e.g. 6.0.0)
curl -L https://github.com/James-HoneyBadger/Loresmith/releases/download/v<version>/Loresmith-v<version>-linux-x86_64.tar.gz -o loresmith.tar.gz

# Extract the archive
tar -xzf loresmith.tar.gz

# Move to a directory in your PATH (optional)
sudo mv loresmith-* /usr/local/bin/

# Verify installation
./loresmith_player examples/adventures/demo_adventure.json
```

#### macOS
```bash
# Download the latest macOS release (replace <version> and <arch>)
curl -L https://github.com/James-HoneyBadger/Loresmith/releases/download/v<version>/Loresmith-v<version>-macos-<arch>.tar.gz -o loresmith.tar.gz

# Extract the archive
tar -xzf loresmith.tar.gz

# Move to a directory in your PATH (optional)
sudo mv loresmith-* /usr/local/bin/

# Verify installation
./loresmith_player examples/adventures/demo_adventure.json
```

#### Windows
```powershell
# Download the latest release
Invoke-WebRequest -Uri "https://github.com/James-HoneyBadger/Loresmith/releases/download/v<version>/Loresmith-v<version>-windows-x86_64.zip" -OutFile "loresmith.zip"

# Extract the archive
Expand-Archive -Path "loresmith.zip" -DestinationPath "C:\Program Files\Loresmith"

# Add to PATH (optional)
# Go to System Properties > Environment Variables > Path > Edit > Add "C:\Program Files\Loresmith"

# Verify installation
.\loresmith_player.exe examples/adventures/demo_adventure.json
```

### Method 2: Building from Source

#### Prerequisites

**Rust Installation**
```bash
# Install Rust using rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

**System Dependencies**

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# GUI editor dependencies (eframe/winit/rfd)
sudo apt install libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev \
  libxrandr-dev libxi-dev libgl1-mesa-dev
```

**CentOS/RHEL/Fedora:**
```bash
sudo dnf install gcc gcc-c++ make openssl-devel
# or on CentOS/RHEL:
sudo yum install gcc gcc-c++ make openssl-devel
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (optional, for additional tools)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Windows:**
```powershell
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Or use Chocolatey:
choco install visualstudio2019buildtools
```

#### Building Loresmith

```bash
# Clone the repository
git clone https://github.com/James-HoneyBadger/Loresmith.git
cd Loresmith

# Build all components in release mode
cargo build --release

# Verify the build
./target/release/loresmith_player examples/adventures/demo_adventure.json

# Optional: Run tests
cargo test
```

#### Installing Built Binaries

```bash
# Copy binaries to system location (Linux/macOS)
sudo cp target/release/loresmith_* /usr/local/bin/

# Or create a local bin directory
mkdir -p ~/bin
cp target/release/loresmith_* ~/bin/
export PATH="$HOME/bin:$PATH"
```

### Method 3: Using Package Managers

#### Cargo (Rust Package Manager)
```bash
# Loresmith is not published on crates.io. Install the binaries from git:
cargo install --git https://github.com/James-HoneyBadger/Loresmith.git --bin loresmith_player
cargo install --git https://github.com/James-HoneyBadger/Loresmith.git --bin loresmith_console
cargo install --git https://github.com/James-HoneyBadger/Loresmith.git --bin loresmith_workbench
cargo install --git https://github.com/James-HoneyBadger/Loresmith.git --bin loresmith_studio
```

#### Snap and Homebrew

Snap and Homebrew packages are not currently published. Use the pre-built assets or
build from source.

## Post-Installation Setup

Loresmith does not require a configuration directory or config file. All settings are contained within the adventure JSON files themselves.

### Adventure Files

Place your adventure JSON files anywhere and pass the path to either player:

```bash
cargo run --bin loresmith_player -- path/to/my_adventure.json
cargo run --bin loresmith_console -- path/to/my_adventure.json
```

Bundled demo adventures are included under examples/adventures, including demo_adventure.json and shattered_realms_demo.json.

## Running Loresmith

### Graphical Player

```bash
# Choose from the adventure library
loresmith_player

# Open a specific adventure
loresmith_player path/to/adventure.json
```

### Command Line Player

```bash
# Play the demo adventure
loresmith_console examples/adventures/demo_adventure.json

# Load specific adventure
loresmith_console path/to/adventure.json
```

### Terminal UI Editor

```bash
# Start the editor
loresmith_workbench

# Open specific adventure
loresmith_workbench path/to/adventure.json
```

### Graphical Editor (Studio)

```bash
# Start the graphical editor
loresmith_studio
```

## Troubleshooting Installation

### Common Issues

#### "Command not found" Error

**Problem:** Loresmith commands are not recognized.

**Solutions:**
1. Check if binaries are in your PATH:
   ```bash
   which loresmith_console
   ```
2. Add to PATH manually:
   ```bash
   export PATH="$HOME/bin:$PATH"  # Add to ~/.bashrc or ~/.zshrc
   ```
3. Use full paths:
   ```bash
   /usr/local/bin/loresmith_console
   ```

#### Permission Denied

**Problem:** Cannot execute binaries.

**Solutions:**
```bash
# Make binaries executable
chmod +x loresmith_console
chmod +x loresmith_workbench
chmod +x loresmith_studio

# Or for all binaries
chmod +x loresmith_*
```

#### Build Failures

**Problem:** `cargo build` fails.

**Common causes and solutions:**

1. **Missing dependencies:**
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential pkg-config libssl-dev

   # macOS
   xcode-select --install
   ```

2. **Outdated Rust:**
   ```bash
   rustup update
   ```

3. **Network issues:**
   ```bash
   # Clear cargo cache
   cargo clean
   # Try again
   cargo build
   ```

#### GUI Editor Won't Start

**Problem:** `loresmith_studio` fails to launch.

**Solutions:**
1. Check display server (Linux):
   ```bash
   echo $DISPLAY
   ```
2. Install GUI libraries (Linux):
   ```bash
   sudo apt install libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev
   ```
3. Use terminal editor instead:
   ```bash
   loresmith_workbench
   ```

### Dependency Issues

#### Rust Version Too Old

```bash
# Update Rust
rustup update

# Check version
rustc --version  # Should be 1.95 or newer
```

#### Missing System Libraries

**Linux:**
```bash
# Install common development libraries
sudo apt install libfontconfig-dev libxcb-render0-dev libxcb-shape0-dev
```

**macOS:**
```bash
# Install additional tools
brew install fontconfig
```

### Performance Issues

#### Slow Startup

**Solutions:**
1. Use release builds:
   ```bash
   cargo build --release
   ```
2. Check available memory:
   ```bash
   free -h  # Linux
   vm_stat   # macOS
   ```

#### High Memory Usage

**Solutions:**
1. Close other applications
2. Use smaller adventures
3. Monitor with system tools:
   ```bash
   top -p $(pgrep loresmith)
   ```

## Advanced Configuration

### Environment Variables

```bash
# Enable debug logging
export RUST_LOG=debug
```

### Custom Build Options

```bash
# Build with optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Build for different architectures
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-apple-darwin

# Create distributable packages
cargo install cargo-deb
cargo deb
```

### Development Setup

For contributors and developers:

```bash
# Clone with submodules (if any)
git clone --recursive https://github.com/James-HoneyBadger/Loresmith.git

# Install development tools
cargo install cargo-watch cargo-expand cargo-flamegraph

# Set up pre-commit hooks
cp pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Run development server
cargo watch -x 'run --bin loresmith_console'
```

## Updating Loresmith

### Automatic Updates

```bash
# Using pre-built binaries
curl -L https://github.com/James-HoneyBadger/Loresmith/releases/latest/download/loresmith-update.sh | bash
```

### Manual Updates

```bash
# Pull latest changes
git pull origin main

# Rebuild
cargo build --release

# Install new binaries
sudo cp target/release/loresmith_* /usr/local/bin/
```

### Checking for Updates

```bash
# Check current version
loresmith_console --version

# Check latest release on GitHub
curl -s https://api.github.com/repos/James-HoneyBadger/Loresmith/releases/latest | grep '"tag_name"'
```

## Uninstalling Loresmith

### Remove Binaries

```bash
# Remove from system locations
sudo rm /usr/local/bin/loresmith_*

# Remove from local bin
rm ~/bin/loresmith_*
```

### Remove Configuration and Data

```bash
# Remove config directory
rm -rf ~/.config/loresmith

# Remove save files (optional)
rm -rf ~/Documents/Loresmith
```

### Clean Build Artifacts

```bash
# Remove source and build files
rm -rf Loresmith/

# Clean cargo cache (optional)
cargo cache --autoclean
```

## Getting Help

### Documentation
- [User Manual](User_Manual.md) - Complete player guide
- [Game Designer Manual](Game_Designer_Manual.md) - Creating adventures
- [Technical Reference](Technical_Reference.md) - API and architecture
- [Development Guide](Development_Guide.md) - Contributing

### Community Support
- **GitHub Issues**: [Report bugs](https://github.com/James-HoneyBadger/Loresmith/issues)
- **Discussions**: [Community forum](https://github.com/James-HoneyBadger/Loresmith/discussions)
- **Discord**: Join our community server (when available)

### System Information for Bug Reports

When reporting issues, include:

```bash
# System information
uname -a
rustc --version
cargo --version

# Loresmith version (no --version flag; just run with a demo file)
loresmith_console examples/adventures/demo_adventure.json

# Error logs
RUST_LOG=debug loresmith_console 2>&1 | head -50
```

---

**Installation complete!** You're now ready to explore the world of Loresmith. Start with the [User Manual](User_Manual.md) to learn how to play, or dive into the [Game Designer Manual](Game_Designer_Manual.md) to create your own adventures.