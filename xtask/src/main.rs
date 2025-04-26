use anyhow::Result;
use clap::Parser;
use settings::Settings;

mod osx;
mod settings;

fn main() -> Result<()> {
    println!("Hello, world!");

    let settings = Settings::try_parse()?;

    Ok(())
}
