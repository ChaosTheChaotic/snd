use std::{fmt, net::IpAddr};

#[derive(Debug)]
pub struct HostInfo {
    pub name: String,
    pub ip: IpAddr,
}

#[derive(Debug)]
pub struct DM {
    pub host_info: HostInfo,
    pub file_path: String,
    pub file_type: String,
    pub file_size: u64,
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
            "From {} with ip {} and {}: {} with size {}",
            self.host_info.name, self.host_info.ip, self.file_type, self.file_path, size_str
        )
    }
}
