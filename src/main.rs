mod lib;
mod commons;
mod module;
use std::{ sync::Arc, time::Duration };
use clap::{ App, Arg, Command, SubCommand };
use crate::commons::output::Output;
use crate::module::crawler::Crawler;
use crate::module::crawler::{ CveDetails, GitHubSpider, QuotesSpider };

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
        let cli = App::new("vxsuite a command-line application")
            .version("0.0.1")
            .author("Author: seaung Github: <https://github.com/seaung>")
            .about("Dose awesome things.")
            .subcommand(
                SubCommand::with_name("scan")
                  .about("Provide the IP address of a target. e.g 192.168.10.1")
                  .arg(
                      Arg::with_name("domain")
                          .help("Provide the IP address of a target.")
                          .short('D')
                          .long("domain")
                  )
                  .arg(
                      Arg::with_name("port")
                          .short('p')
                          .long("port")
                          .help("Provide the port number of a target.")
                  )
            )
            .subcommand(
                Command::new("spiders").about("List all spiders")
            )
            .subcommand(
                Command::new("run").about("Run a Spider").arg(
                    Arg::new("spider")
                    .short('s')
                    .long("spider")
                    .help("run spider.")
                    .takes_value(true)
                    .required(true)
                ),
            )
            .get_matches();

        if let Some(_) = cli.subcommand_matches("spiders") {
            let spider_lists = vec!["cve", "github", "quotes"];
            println!("spider list : ");
            for spider in spider_lists {
                print!("\t\t\tspider name : {}\n", spider);
            }
        } else if let Some(matches) = cli.subcommand_matches("run") {
            let spider = matches.value_of("spider").unwrap();
            let crawler = Crawler::new(Duration::from_millis(200), 2, 500);

            match spider {
                "cve" => {
                    let s = Arc::new(CveDetails::new());
                    crawler.run(s).await;
                }
                "github" => {
                    let s = Arc::new(GitHubSpider::new());
                    crawler.run(s).await;
                }
                "quotes" => {
                    let s = Arc::new(QuotesSpider::new().await?);
                    crawler.run(s).await;
                }
                _ => Output::warning("select a spider please!")
            }
        }
        Ok(())
}
