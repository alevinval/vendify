use clap::Parser;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelFilter;
use simplelog::TermLogger;
use simplelog::TerminalMode;

use self::structs::Cli;
use self::structs::Commands;
use crate::control::Controller;

mod structs;

pub fn run() {
    let cli = Cli::parse();

    setup_logging(cli.debug);

    match cli.command {
        Commands::Init {} => Controller::init(),
        Commands::Add {
            url,
            refname,
            extensions,
            targets,
            ignores,
        } => Controller::add(&url, &refname, extensions, targets, ignores),
        Commands::Install {} => Controller::install(),
        Commands::Update {} => Controller::update(),
    };
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
