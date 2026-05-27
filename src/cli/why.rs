use anyhow::{bail, Result};

use crate::config::Config;

use super::scan_and_classify;

pub fn run(config: &Config, port: u16) -> Result<()> {
    let entries = scan_and_classify(config)?;

    let matches: Vec<_> = entries.iter().filter(|e| e.port == port).collect();

    if matches.is_empty() {
        bail!("nothing is listening on port {port}");
    }

    for entry in &matches {
        println!(
            "Port {port} is used by {} (PID {})",
            entry.process_name, entry.pid
        );
        println!("  Type: {}", entry.classification);
        println!("  Command: {}", entry.command_line);
        println!("  Address: {}", entry.local_addr);
        if let Some(project) = &entry.project {
            println!("  Project: {} ({})", project.name, project.root.display());
            if let Some(fw) = &project.framework {
                println!("  Framework: {fw}");
            }
        }
    }

    Ok(())
}
