use std::time::Duration;

use pug::clap::{App, Arg, SubCommand};
use serialport::{self, SerialPortSettings};

use super::{
    errors::Result,
    tty::{publisher, BaudRate, DataBits, FlowControl, Parity, Pseudo, StopBits},
};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
pub const HOMEPAGE: &'static str = env!("CARGO_PKG_HOMEPAGE");
pub const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
pub const BANNER: &'static str = include_str!("banner.txt");

pub fn launch() -> Result<()> {
    let path = Arg::with_name("path")
        .required(true)
        .short("p")
        .long("path")
        .help("Directory to load protocols")
        .takes_value(true);
    let delay = Arg::with_name("delay")
        .short("d")
        .long("delay")
        .help("Delay in microseconds")
        .takes_value(true);
    let interval = Arg::with_name("interval")
        .required(true)
        .short("i")
        .long("interval")
        .help("Interval in seconds")
        .takes_value(true);
    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .before_help(BANNER)
        .after_help(HOMEPAGE)
        .subcommand(
            SubCommand::with_name("pseudo")
                .about("Create a pseudo serial-port device")
                .arg(&path)
                .arg(&interval)
                .arg(&delay),
        ).subcommand(
            SubCommand::with_name("publisher")
                .about("Publisher message to serial-port")
                .arg(&path)
                .arg(&interval)
                .arg(&delay)
                .arg(
                    Arg::with_name("name")
                        .required(true)
                        .short("n")
                        .long("name")
                        .help("Device name(/dev/serial0,/dev/ttyUSB0,/dev/pts/1,COM1)")
                        .takes_value(true),
                ).arg(
                    Arg::with_name("baud_rate")
                        .short("B")
                        .long("baud-rate")
                        .help(&format!(
                            "The baud rate in symbols-per-second({})",
                            BaudRate
                                .clone()
                                .into_iter()
                                .map(|i| i.to_string())
                                .collect::<Vec<String>>()
                                .join(",")
                        )).takes_value(true),
                ).arg(
                    Arg::with_name("data_bits")
                        .short("D")
                        .long("data-bits")
                        .help(&format!(
                            "Number of bits used to represent a character sent on the line({})",
                            DataBits
                                .keys()
                                .cloned()
                                .collect::<Vec<&'static str>>()
                                .join(",")
                        )).takes_value(true),
                ).arg(
                    Arg::with_name("flow_control")
                        .short("f")
                        .long("flow-control")
                        .help(&format!(
                            "The type of signalling to use for controlling data transfer({})",
                            FlowControl
                                .keys()
                                .cloned()
                                .collect::<Vec<&'static str>>()
                                .join(",")
                        )).takes_value(true),
                ).arg(
                    Arg::with_name("parity")
                        .short("P")
                        .long("parity")
                        .help(&format!(
                            "The type of parity to use for error checking({})",
                            Parity
                                .keys()
                                .cloned()
                                .collect::<Vec<&'static str>>()
                                .join(",")
                        )).takes_value(true),
                ).arg(
                    Arg::with_name("stop_bits")
                        .short("s")
                        .long("stop-bits")
                        .help(&format!(
                            "Number of bits to use to signal the end of a character({})",
                            &StopBits
                                .keys()
                                .cloned()
                                .collect::<Vec<&'static str>>()
                                .join(",")
                        )).takes_value(true),
                ).arg(
                    Arg::with_name("timeout")
                        .short("t")
                        .long("timeout")
                        .help("Amount of time to wait to receive data before timing out")
                        .takes_value(true),
                ),
        ).get_matches();

    if let Some(matches) = matches.subcommand_matches("pseudo") {
        let path = matches.value_of("path").unwrap();
        let interval = Duration::from_secs(
            matches
                .value_of("interval")
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        );
        let delay = match matches.value_of("delay") {
            Some(v) => Some(Duration::from_micros(v.parse::<u64>().unwrap())),
            None => None,
        };

        let port = Pseudo::new()?;
        return port.start(path, interval, delay);
    }

    if let Some(matches) = matches.subcommand_matches("publisher") {
        let name = matches.value_of("name").unwrap();
        let path = matches.value_of("path").unwrap();
        let interval = Duration::from_secs(
            matches
                .value_of("interval")
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        );
        let delay = match matches.value_of("delay") {
            Some(v) => Some(Duration::from_micros(v.parse::<u64>().unwrap())),
            None => None,
        };

        let mut settings = SerialPortSettings::default();
        settings.baud_rate = matches
            .value_of("baud_rate")
            .unwrap_or("9600")
            .parse::<u32>()
            .unwrap()
            .into();

        settings.data_bits = match matches.value_of("data_bits").unwrap_or("Eight") {
            "Five" => serialport::DataBits::Five,
            "Six" => serialport::DataBits::Six,
            "Seven" => serialport::DataBits::Seven,
            "Eight" => serialport::DataBits::Eight,
            _ => panic!("bad data bits"),
        };
        settings.flow_control = match matches.value_of("flow_control").unwrap_or("None") {
            "None" => serialport::FlowControl::None,
            "Software" => serialport::FlowControl::Software,
            "Hardware" => serialport::FlowControl::Hardware,
            _ => panic!("bad flow control"),
        };
        settings.parity = match matches.value_of("parity").unwrap_or("None") {
            "None" => serialport::Parity::None,
            "Odd" => serialport::Parity::Odd,
            "Even" => serialport::Parity::Even,
            _ => panic!("bad parity"),
        };
        settings.stop_bits = match matches.value_of("stop_bits").unwrap_or("One") {
            "One" => serialport::StopBits::One,
            "Two" => serialport::StopBits::Two,
            _ => panic!("bad stop bits"),
        };

        return publisher(path, &name.to_string(), &settings, interval, delay);
    }

    Ok(())
}
