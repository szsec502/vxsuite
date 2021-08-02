mod lib;
mod commons;
mod module;

use lib::{ options::CommandOptions };

fn main() {
    CommandOptions::parse();
}
