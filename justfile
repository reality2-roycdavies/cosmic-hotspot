name := 'cosmic-hotspot'
appid := 'io.github.reality2_roycdavies.cosmic-hotspot'

# Default recipe: build release
default: build-release

# Build in debug mode
build-debug:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run in debug mode
run:
    cargo run

# Run in release mode
run-release:
    cargo run --release

# Check code with clippy
check:
    cargo clippy --all-features

# Format code
fmt:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean

# Install to local user
install-local:
    #!/bin/bash
    set -e

    echo "Stopping any running instances..."
    pkill -x "cosmic-hotspot" 2>/dev/null || true
    sleep 1

    # Install binary
    mkdir -p ~/.local/bin
    rm -f ~/.local/bin/{{name}}
    cp target/release/{{name}} ~/.local/bin/

    # Install desktop entry
    mkdir -p ~/.local/share/applications
    cp resources/{{appid}}.desktop ~/.local/share/applications/

    # Install icons
    mkdir -p ~/.local/share/icons/hicolor/scalable/apps
    cp resources/{{appid}}.svg ~/.local/share/icons/hicolor/scalable/apps/
    mkdir -p ~/.local/share/icons/hicolor/symbolic/apps
    cp resources/{{appid}}-symbolic.svg ~/.local/share/icons/hicolor/symbolic/apps/
    cp resources/{{appid}}-active-symbolic.svg ~/.local/share/icons/hicolor/symbolic/apps/
    cp resources/{{appid}}-inactive-symbolic.svg ~/.local/share/icons/hicolor/symbolic/apps/

    echo "Installation complete!"
    echo "Add the applet to your COSMIC panel to use it."

# Install NAT helper + polkit policy (enables passwordless NAT setup, requires sudo)
install-policy:
    sudo install -Dm755 resources/cosmic-hotspot-nat /usr/local/bin/cosmic-hotspot-nat
    sudo install -Dm644 resources/{{appid}}.policy /usr/share/polkit-1/actions/{{appid}}.policy
    @echo "NAT helper and polkit policy installed."
    @echo "Explicit NAT rules will now be applied silently when starting the hotspot."

# Remove NAT helper + polkit policy
uninstall-policy:
    sudo rm -f /usr/local/bin/cosmic-hotspot-nat
    sudo rm -f /usr/share/polkit-1/actions/{{appid}}.policy

# Uninstall from local user
uninstall-local:
    rm -f ~/.local/bin/{{name}}
    rm -f ~/.local/share/applications/{{appid}}.desktop
    rm -f ~/.local/share/icons/hicolor/scalable/apps/{{appid}}.svg
    rm -f ~/.local/share/icons/hicolor/symbolic/apps/{{appid}}-symbolic.svg
    rm -f ~/.local/share/icons/hicolor/symbolic/apps/{{appid}}-active-symbolic.svg
    rm -f ~/.local/share/icons/hicolor/symbolic/apps/{{appid}}-inactive-symbolic.svg

# Build and run
br: build-debug run
