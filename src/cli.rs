use anyhow::Result;
use clap::Parser;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelFilter;
use simplelog::TermLogger;
use simplelog::TerminalMode;

use self::structs::Cli;
use self::structs::Commands;
use crate::control::Controller;
use crate::preset::Preset;

mod structs;

/// Entry point to run the vendify CLI
///
/// # Errors
///
/// Will return `Err` if the operation has not succeeded.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    setup_logging(cli.debug);

    let preset = Preset::new();
    let controller = Controller::new(preset);
    match cli.command {
        Commands::Init {} => controller.init(),
        Commands::Add {
            url,
            refname,
            extensions,
            targets,
            ignores,
        } => controller.add(&url, &refname, extensions, targets, ignores),
        Commands::Install {} => controller.install()?,
        Commands::Update {} => controller.update()?,
    };
    Ok(())
}

fn setup_logging(is_debug: bool) {
    let logging_config = ConfigBuilder::default()
        .set_time_level(LevelFilter::Off)
        .build();
    TermLogger::init(
        if is_debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        logging_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
