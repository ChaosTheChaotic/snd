use gethostname::gethostname;

pub fn is_vpn(name: &str) -> bool {
    if cfg!(windows) {
        let patterns = ["TAP", "OpenVPN", "WireGuard", "ZeroTier", "Tailscale"];
        patterns.iter().any(|p| name.to_uppercase().contains(p))
    } else if cfg!(unix) {
        let patterns = ["tun", "tap", "ppp", "zt", "tailscale", "utun", "vpn"];
        patterns.iter().any(|p| name.starts_with(p))
    } else {
        false
    }
}

// Since im not bothered to handle this each and every time
pub fn gen_cname() -> String {
    gethostname()
        .to_str()
        .unwrap_or("nohostnameerror")
        .to_string()
}

pub fn extract_hostname(message: &str) -> String {
    if let Some(start) = message.find("from ") {
        if let Some(end) = message.find('!') {
            if start + 5 < end {
                return message[start + 5..end].to_string();
            }
        }
    }
    message.trim_end_matches('!').to_string()
}
