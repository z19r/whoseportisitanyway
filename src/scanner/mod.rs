use crate::model::RawPort;
use thiserror::Error;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("failed to read process info: {0}")]
    ProcessInfo(#[from] std::io::Error),
    #[error("failed to parse port data: {0}")]
    Parse(String),
    #[error("permission denied — try running with elevated privileges")]
    PermissionDenied,
}

pub trait Scanner {
    fn scan(&self) -> Result<Vec<RawPort>, ScanError>;
}

pub fn create_scanner() -> Box<dyn Scanner> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxScanner)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacosScanner)
    }
}
