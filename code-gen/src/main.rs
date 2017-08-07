use std::fs::File;
use std::io::{BufReader, BufRead};
use std::env;

#[macro_use]
extern crate nom;

fn main() {
    let filename = env::args().nth(1)?;
    let file = File::open(&filename)?;
    let input =
}
