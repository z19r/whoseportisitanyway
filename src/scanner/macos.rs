use super::{ScanError, Scanner};
use crate::model::RawPort;

pub struct MacosScanner;

impl Scanner for MacosScanner {
    fn scan(&self) -> Result<Vec<RawPort>, ScanError> {
        // TODO: use libproc APIs or parse lsof output
        Ok(vec![])
    }
}
