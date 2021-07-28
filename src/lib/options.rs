use clap::{ App, Arg, SubCommand };

pub struct CommandOptions;

impl CommandOptions {
    pub fn parse() {
        let matches = App::new("vxsuite a command-line application")
            .version("0.0.1")
            .author("Author: seaung Github: <https://github.com/seaung>")
            .about("")
            .subcommand(
                SubCommand::with_name("scan")
                  .about("e.g http://www.example.com")
                  .arg(
                      Arg::with_name("target")
                          .index(1)
                          .value_name("TARGET")
                          .help("enter a target site.")
                  )
            )
            .get_matches();

        match matches.subcommand() {
            ("scan", Some(scan_match)) => {
                let target = match scan_match.value_of("target") {
                    Some(target) => target,
                    None => {
                        std::process::exit(1);
                    },
                };
                println!("target value : {}", target);
            },
            _ => {},
        }
    }
}
