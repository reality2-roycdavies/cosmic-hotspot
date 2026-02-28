//! CLI settings protocol for cosmic-applet-settings hub integration.

use crate::config::Config;
use crate::hotspot;

pub fn describe() {
    let config = Config::load();

    let wifi_interfaces = hotspot::list_wifi_interfaces();
    let wifi_opts: Vec<serde_json::Value> = wifi_interfaces
        .iter()
        .map(|i| serde_json::json!({"value": i, "label": i}))
        .collect();

    let net_interfaces = hotspot::list_network_interfaces();
    let net_opts: Vec<serde_json::Value> = net_interfaces
        .iter()
        .map(|i| serde_json::json!({"value": i, "label": i}))
        .collect();

    let schema = serde_json::json!({
        "title": "WiFi Hotspot Settings",
        "description": "Configure and manage a WiFi hotspot using NetworkManager.",
        "sections": [
            {
                "title": "Network",
                "items": [
                    {
                        "type": "text",
                        "key": "ssid",
                        "label": "SSID",
                        "value": config.ssid,
                        "placeholder": "Network name"
                    },
                    {
                        "type": "text",
                        "key": "password",
                        "label": "Password",
                        "value": config.password,
                        "placeholder": "WPA2 password"
                    },
                    {
                        "type": "select",
                        "key": "band",
                        "label": "Band",
                        "value": config.band,
                        "options": [
                            {"value": "bg", "label": "2.4 GHz"},
                            {"value": "a", "label": "5 GHz"}
                        ]
                    }
                ]
            },
            {
                "title": "Interfaces",
                "items": [
                    {
                        "type": "select",
                        "key": "hotspot_interface",
                        "label": "Hotspot Interface",
                        "value": config.hotspot_interface,
                        "options": wifi_opts
                    },
                    {
                        "type": "select",
                        "key": "internet_interface",
                        "label": "Internet Interface",
                        "value": config.internet_interface,
                        "options": net_opts
                    }
                ]
            },
            {
                "title": "Advanced",
                "items": [
                    {
                        "type": "text",
                        "key": "connection_name",
                        "label": "Connection Name",
                        "value": config.connection_name,
                        "placeholder": "NM connection name"
                    },
                    {
                        "type": "text",
                        "key": "gateway_ip",
                        "label": "Gateway IP",
                        "value": config.gateway_ip,
                        "placeholder": "192.168.44.1/24"
                    }
                ]
            }
        ],
        "actions": [
            {"id": "reset", "label": "Reset to Defaults", "style": "destructive"},
            {"id": "refresh_interfaces", "label": "Refresh Interfaces", "style": "standard"}
        ]
    });

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}

pub fn set(key: &str, value: &str) {
    let mut config = Config::load();

    let result: Result<&str, String> = match key {
        "ssid" => parse_string(value).map(|v| { config.ssid = v; "Updated SSID" }),
        "password" => parse_string(value).map(|v| { config.password = v; "Updated password" }),
        "band" => parse_string(value).and_then(|v| {
            if v == "bg" || v == "a" {
                config.band = v;
                Ok("Updated band")
            } else {
                Err(format!("Invalid band: must be 'bg' or 'a'"))
            }
        }),
        "hotspot_interface" => parse_string(value).map(|v| { config.hotspot_interface = v; "Updated hotspot interface" }),
        "internet_interface" => parse_string(value).map(|v| { config.internet_interface = v; "Updated internet interface" }),
        "connection_name" => parse_string(value).map(|v| { config.connection_name = v; "Updated connection name" }),
        "gateway_ip" => parse_string(value).map(|v| { config.gateway_ip = v; "Updated gateway IP" }),
        _ => Err(format!("Unknown key: {key}")),
    };

    match result {
        Ok(msg) => match config.save() {
            Ok(()) => {
                // Restart the hotspot if active so changes take effect immediately
                if hotspot::is_hotspot_active(&config) {
                    let _ = hotspot::stop_hotspot(&config);
                    match hotspot::start_hotspot(&config) {
                        Ok(_) => print_response(true, msg),
                        Err(e) => print_response(false, &format!("{msg} (restart failed: {e})")),
                    }
                } else {
                    print_response(true, msg);
                }
            }
            Err(e) => print_response(false, &format!("Save failed: {e}")),
        },
        Err(e) => print_response(false, &e),
    }
}

pub fn action(id: &str) {
    match id {
        "save" => {
            // Each --settings-set already saves and restarts; this is kept for compat
            print_response(true, "Configuration saved");
        }
        "reset" => {
            let config = Config::default();
            match config.save() {
                Ok(()) => print_response(true, "Reset to defaults"),
                Err(e) => print_response(false, &format!("Reset failed: {e}")),
            }
        }
        "refresh_interfaces" => {
            // Just re-describe will show fresh interfaces
            print_response(true, "Interfaces refreshed");
        }
        _ => print_response(false, &format!("Unknown action: {id}")),
    }
}

fn parse_string(value: &str) -> Result<String, String> {
    serde_json::from_str::<String>(value).map_err(|e| format!("Invalid string: {e}"))
}

fn print_response(ok: bool, message: &str) {
    let resp = serde_json::json!({"ok": ok, "message": message});
    println!("{}", resp);
}
