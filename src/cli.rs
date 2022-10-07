use clap::Parser;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelFilter;
use simplelog::TermLogger;
use simplelog::TerminalMode;

use self::structs::VendorCli;
use self::structs::VendorCommand;
use crate::core::Controller;

mod structs;

pub fn run() {
    let cli = VendorCli::parse();

    setup_logging(cli.debug);

    let controller = Controller::new();
    match cli.command {
        VendorCommand::Init {} => controller.init(),
        VendorCommand::Add {
            url,
            refname,
            extensions,
            targets,
            ignores,
        } => controller.add(&url, &refname, extensions, targets, ignores),
        VendorCommand::Install {} => controller.install(),
        VendorCommand::Update {} => controller.update(),
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
