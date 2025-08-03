use std::{
    fmt,
    net::IpAddr,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct HostInfo {
    pub name: String,
    pub ip: IpAddr,
}

#[derive(Debug)]
pub struct DM {
    pub host_info: HostInfo,
    pub send_method: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: u64,
    pub recv_time: Instant,
}

#[derive(Debug)]
pub enum ShModes {
    REC,
    SND,
}

impl fmt::Display for ShModes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for DM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size_str = crate::utils::human_readable_size(self.file_size);
        write!(
            f,
            "From {} with ip {} and {}: {} with size {} using send method: {}",
            self.host_info.name,
            self.host_info.ip,
            self.file_type,
            self.file_path,
            size_str,
            self.send_method
        )
    }
}

pub struct Config {
    pub send_method: String,
    pub follow_symlinks: bool,
}

pub struct PendingPacket {
    pub sequence_number: u64,
    pub data: Vec<u8>,
    pub last_sent: Instant,
    pub timeout_duration: Duration,
    pub retry_count: u32,
}
