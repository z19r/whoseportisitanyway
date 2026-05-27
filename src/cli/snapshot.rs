use anyhow::Result;

use crate::config::Config;

use super::scan_and_classify;

pub fn run(config: &Config, json: bool) -> Result<()> {
    let entries = scan_and_classify(config)?;

    if json {
        let output = serde_json::to_string_pretty(&entries)?;
        println!("{output}");
    } else {
        println!(
            "{:<7} {:<5} {:<20} {:<12} {:<20} {:<8} {:<12}",
            "PORT", "PROTO", "PROCESS", "TYPE", "PROJECT", "PID", "STATE"
        );
        for e in &entries {
            println!(
                "{:<7} {:<5} {:<20} {:<12} {:<20} {:<8} {}",
                e.port,
                e.protocol,
                e.process_name,
                e.classification,
                e.project.as_ref().map(|p| p.name.as_str()).unwrap_or("—"),
                e.pid,
                e.state,
            );
        }
    }

    Ok(())
}
