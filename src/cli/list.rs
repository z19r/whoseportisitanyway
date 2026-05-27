use anyhow::Result;

use crate::config::Config;

use super::scan_and_classify;

pub fn run(config: &Config, plain: bool) -> Result<()> {
    let entries = scan_and_classify(config)?;

    if plain {
        for e in &entries {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                e.port, e.protocol, e.process_name, e.classification, e.pid,
            );
        }
    } else {
        let output = serde_json::to_string_pretty(&entries)?;
        println!("{output}");
    }

    Ok(())
}
