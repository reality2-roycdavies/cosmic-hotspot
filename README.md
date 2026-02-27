# WiFi Hotspot Applet for COSMIC Desktop

A WiFi hotspot toggle applet for the [COSMIC desktop environment](https://system76.com/cosmic) on Linux. Quickly create and manage a WiFi hotspot from your panel.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![COSMIC](https://img.shields.io/badge/desktop-COSMIC-purple.svg)

## Features

- **Native COSMIC Panel Applet**: Integrates directly into the COSMIC panel
- **Hotspot Toggle**: Enable/disable WiFi hotspot with one click
- **Status Icons**: Icon reflects hotspot state (active/inactive)
- **NAT Support**: Optional polkit policy for passwordless NAT configuration
- **Settings Page**: Configurable via the unified COSMIC applet settings app

## Requirements

**Two WiFi interfaces are required** to run a hotspot while staying connected to the internet. This typically means:
- A built-in WiFi card **plus** a USB WiFi dongle, or
- Two internal WiFi cards

One interface maintains your internet connection while the other broadcasts the hotspot.

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) toolchain (1.75+)
- [just](https://github.com/casey/just) command runner
- System libraries:

```bash
# Debian/Ubuntu/Pop!_OS
sudo apt install libwayland-dev libxkbcommon-dev libssl-dev pkg-config just

# Fedora
sudo dnf install wayland-devel libxkbcommon-devel openssl-devel just

# Arch
sudo pacman -S wayland libxkbcommon openssl just
```

### Build and Install

```bash
git clone https://github.com/reality2-roycdavies/cosmic-hotspot.git
cd cosmic-hotspot

# Build release binary
just build-release

# Install binary, desktop entry, and icons to ~/.local
just install-local
```

Then add the applet to your COSMIC panel via **Settings -> Desktop -> Panel -> Applets**.

### Optional: NAT Helper (for passwordless hotspot)

To allow the applet to configure NAT without prompting for a password each time:

```bash
just install-policy
```

This installs a NAT helper script and polkit policy (requires sudo). To remove:

```bash
just uninstall-policy
```

### Other just commands

```bash
just build-debug       # Debug build
just run               # Build debug and run
just run-release       # Build release and run
just check             # Run clippy checks
just fmt               # Format code
just clean             # Clean build artifacts
just uninstall-local   # Remove installed files
```

### Uninstalling

```bash
just uninstall-local
just uninstall-policy   # If NAT policy was installed
```

## Related COSMIC Applets

This is part of a suite of custom applets for the COSMIC desktop, configurable via the [unified settings app](https://github.com/reality2-roycdavies/cosmic-applet-settings):

| Applet | Description |
|--------|-------------|
| **[cosmic-applet-settings](https://github.com/reality2-roycdavies/cosmic-applet-settings)** | Unified settings app for the applet suite |
| **[cosmic-runkat](https://github.com/reality2-roycdavies/cosmic-runkat)** | Animated running cat CPU indicator for the panel |
| **[cosmic-bing-wallpaper](https://github.com/reality2-roycdavies/cosmic-bing-wallpaper)** | Daily Bing wallpaper manager with auto-update |
| **[cosmic-pie-menu](https://github.com/reality2-roycdavies/cosmic-pie-menu)** | Radial/pie menu app launcher with gesture support |
| **[cosmic-tailscale](https://github.com/reality2-roycdavies/cosmic-tailscale)** | Tailscale VPN status and control applet |

### Other Related Projects

| Project | Description |
|---------|-------------|
| **[cosmic-konnect](https://github.com/reality2-roycdavies/cosmic-konnect)** | Device connectivity and sync between Linux and Android |
| **[cosmic-konnect-android](https://github.com/reality2-roycdavies/cosmic-konnect-android)** | Android companion app for Cosmic Konnect |

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- [System76](https://system76.com/) for the COSMIC desktop environment
