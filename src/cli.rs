use clap::{Command, arg, crate_version};

pub fn cli() -> Command {
    Command::new("impala")
        .about("TUI For managing wifi")
        .version(crate_version!())
        .arg(
            arg!(--mode <mode>)
                .short('m')
                .required(false)
                .help("Device mode")
                .value_parser(["station", "ap"]),
        )
}
