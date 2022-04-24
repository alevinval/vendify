use super::commands;
use super::commands::VendorCli;
use super::commands::VendorCommand;
use clap::Parser;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::LevelFilter;
use simplelog::TermLogger;
use simplelog::TerminalMode;

pub fn run() {
    let cli = VendorCli::parse();

    setup_logging(cli.debug);

    match cli.command {
        VendorCommand::Init {} => {
            commands::init::run();
        }
        VendorCommand::Add {
            url,
            refname,
            extensions,
            targets,
            ignores,
        } => {
            commands::add::run(&url, &refname, extensions, targets, ignores);
        }
        VendorCommand::Install {} => {
            commands::install::run();
        }
        VendorCommand::Update {} => {
            commands::update::run();
        }
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
