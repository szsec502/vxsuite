use clap::{ App, Arg, SubCommand };
use crate::commons::output::Output;
//use crate::module::scanner::{ scan_ports };


pub struct CommandOptions;

impl CommandOptions {
    pub fn parse() {
        let matches = App::new("vxsuite a command-line application")
            .version("0.0.1")
            .author("Author: seaung Github: <https://github.com/seaung>")
            .about("Dose awesome things.")
            .subcommand(
                SubCommand::with_name("scan")
                  .about("Provide the IP address of a target. e.g 192.168.10.1")
                  .arg(
                      Arg::with_name("domain")
                          .help("Provide the IP address of a target.")
                          .short("D")
                          .long("domain")
                  )
                  .arg(
                      Arg::with_name("port")
                          .short("p")
                          .long("port")
                          .help("Provide the port number of a target.")
                  )
            )
            .subcommand(
                SubCommand::with_name("crawler")
                .about("Please provide an entry URL address. e.g https://www.example.com")
                .arg(
                    Arg::with_name("url")
                    .short("u")
                    .long("url")
                    .help("Please provide an entry URL address.")
                )
            )
            .get_matches();

        match matches.subcommand() {
            ("scan", Some(scan_match)) => {
                let target = match scan_match.value_of("target") {
                    Some(target) => target,
                    None => {
                        Output::warning("Enter target address please.");
                        std::process::exit(1);
                    },
                };
                
                let port: u16 = scan_match.value_of("port").unwrap_or("0").parse().unwrap_or(135);
            },
            ("crawler", Some(crawler_match)) => {
                let url = match crawler_match.value_of("url") {
                    Some(url) => url,
                    None => {
                        Output::warning("Can't get target url address.");
                        std::process::exit(1);
                    }
                };
                println!("crawler url from {}", url);
                Output::success("success");
            },
            _ => {
                Output::info("Please select an action.");
            },
        }
    }
}
