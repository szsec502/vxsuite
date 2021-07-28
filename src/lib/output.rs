use ansi_term::Colour::{ Red, Blue, Yellow, Green };

pub struct Output;

impl Output {
    pub fn error(msg: &str) {
        println!("{} - {}", Red.bold().paint("[-]"), msg);
    }

    pub fn info(msg: &str) {
        println!("{} - {}", Blue.bold().paint("[*]"), msg);
    }

    pub fn warning(msg: &str) {
        println!("{} - {}", Yellow.bold().paint("[!]"), msg);
    }

    pub fn success(msg: &str) {
        println!("{} - {}", Green.bold().paint("[+]"), msg);
    }
}

