use clap::Parser;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelFilter;
use simplelog::TermLogger;
use simplelog::TerminalMode;

use self::structs::VendorCli;
use self::structs::VendorCommand;
use crate::control::Controller;

mod structs;

pub fn run() {
    let cli = VendorCli::parse();

    setup_logging(cli.debug);

    match cli.command {
        VendorCommand::Init {} => Controller::init(),
        VendorCommand::Add {
            url,
            refname,
            extensions,
            targets,
            ignores,
        } => Controller::add(&url, &refname, extensions, targets, ignores),
        VendorCommand::Install {} => Controller::install(),
        VendorCommand::Update {} => Controller::update(),
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
