pub mod list;
pub mod snapshot;
pub mod why;

use anyhow::Result;

use crate::classifier;
use crate::config::Config;
use crate::docker::DockerIndex;
use crate::model::PortEntry;
use crate::scanner;

pub fn scan_and_classify(config: &Config) -> Result<Vec<PortEntry>> {
    let scanner = scanner::create_scanner();
    let raw_ports = scanner.scan()?;
    let docker = DockerIndex::probe();
    Ok(classifier::classify_all(
        raw_ports,
        &config.watched_ports,
        docker.as_ref(),
    ))
}
