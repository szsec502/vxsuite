mod lib;

use lib::{ options::CommandOptions };

fn main() {
    CommandOptions::parse();
}
