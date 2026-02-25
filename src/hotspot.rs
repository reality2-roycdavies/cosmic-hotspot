use crate::config::Config;
use std::process::Command;

pub fn start_hotspot(config: &Config) -> Result<String, String> {
    // Remove any existing connection with the same name
    let _ = Command::new("nmcli")
        .args(["connection", "delete", &config.connection_name])
        .output();

    // Create the AP connection
    let output = Command::new("nmcli")
        .args([
            "connection", "add",
            "type", "wifi",
            "ifname", &config.hotspot_interface,
            "con-name", &config.connection_name,
            "ssid", &config.ssid,
            "--",
            "wifi.mode", "ap",
            "wifi.band", &config.band,
            "wifi-sec.key-mgmt", "wpa-psk",
            "wifi-sec.proto", "rsn",
            "wifi-sec.pairwise", "ccmp",
            "wifi-sec.group", "ccmp",
            "wifi-sec.psk", &config.password,
            "ipv4.method", "shared",
            "ipv4.addresses", &config.gateway_ip,
            "ipv6.method", "disabled",
        ])
        .output()
        .map_err(|e| format!("Failed to run nmcli: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create hotspot: {stderr}"));
    }

    // Activate the connection
    let output = Command::new("nmcli")
        .args(["connection", "up", &config.connection_name])
        .output()
        .map_err(|e| format!("Failed to run nmcli: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to activate hotspot: {stderr}"));
    }

    // NetworkManager's "ipv4.method shared" already handles:
    //   - DHCP server on the hotspot interface
    //   - IP forwarding (sysctl net.ipv4.ip_forward=1)
    //   - iptables MASQUERADE NAT rule
    //
    // For cases where NM's built-in sharing isn't enough (e.g. Quest 3),
    // explicit NAT rules can be set up if a polkit policy is installed:
    //   sudo install -m644 resources/io.github.reality2_roycdavies.cosmic-hotspot.policy \
    //     /usr/share/polkit-1/actions/
    setup_nat_if_authorized(config);

    Ok(format!(
        "Hotspot '{}' active on {}",
        config.ssid, config.hotspot_interface,
    ))
}

pub fn stop_hotspot(config: &Config) -> Result<String, String> {
    let _ = Command::new("nmcli")
        .args(["connection", "down", &config.connection_name])
        .output();

    let _ = Command::new("nmcli")
        .args(["connection", "delete", &config.connection_name])
        .output();

    Ok("Hotspot stopped".to_string())
}

pub fn is_hotspot_active(config: &Config) -> bool {
    Command::new("nmcli")
        .args(["-t", "-f", "GENERAL.STATE", "connection", "show", &config.connection_name])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("activated")
        })
        .unwrap_or(false)
}

const NAT_HELPER: &str = "/usr/local/bin/cosmic-hotspot-nat";

/// Set up explicit NAT rules using the helper script + polkit policy.
/// If the helper script isn't installed, this is a no-op — NM shared mode still works.
///
/// Install with: just install-policy
fn setup_nat_if_authorized(config: &Config) {
    // Only attempt if the helper script is installed
    if !std::path::Path::new(NAT_HELPER).exists() {
        eprintln!("NAT helper not installed — relying on NM shared mode");
        return;
    }

    // pkexec with the helper script: the polkit policy (allow_active=yes) means
    // no password dialog for active sessions
    match Command::new("pkexec")
        .args([NAT_HELPER, &config.hotspot_interface, &config.internet_interface])
        .output()
    {
        Ok(output) if output.status.success() => {
            eprintln!("Explicit NAT rules applied via helper");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("NAT helper warning: {stderr}");
        }
        Err(e) => {
            eprintln!("NAT helper error: {e}");
        }
    }
}

pub fn get_connected_clients(config: &Config) -> Vec<String> {
    // Use "ip neigh show dev <interface>" which is more reliable than arp on modern Linux.
    // Output format: "192.168.44.2 lladdr aa:bb:cc:dd:ee:ff REACHABLE"
    let ip_result = Command::new("ip")
        .args(["neigh", "show", "dev", &config.hotspot_interface])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    // Skip entries in FAILED state (stale/unreachable)
                    if parts.len() >= 4 && !line.contains("FAILED") {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if !ip_result.is_empty() {
        return ip_result;
    }

    // Fallback: try reading /proc/net/arp directly
    std::fs::read_to_string("/proc/net/arp")
        .map(|content| {
            content
                .lines()
                .skip(1) // Skip header
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    // Format: IP HW-type Flags HW-address Mask Device
                    if parts.len() >= 6 && parts[5] == config.hotspot_interface {
                        // Skip incomplete entries (flags 0x0)
                        if parts[2] != "0x0" {
                            Some(parts[0].to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// List available WiFi interfaces from NetworkManager
pub fn list_wifi_interfaces() -> Vec<String> {
    Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE", "device"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 && parts[1] == "wifi" {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// List all network interfaces (for internet interface selection)
pub fn list_network_interfaces() -> Vec<String> {
    Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE,STATE", "device"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let device = parts[0];
                        let dev_type = parts[1];
                        // Include wifi and ethernet devices, skip loopback and bridge
                        if dev_type == "wifi" || dev_type == "ethernet" {
                            Some(device.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
