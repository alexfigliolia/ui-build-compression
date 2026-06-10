use std::env::args;

use futures::executor;

use crate::{compress::compressor::Compressor, logger::logger::Logger};

mod compress;
mod logger;

fn main() {
    let argv: Vec<String> = args().collect();
    if argv.len() < 2 {
        Logger::exit_with_info("Please specify an absolute path to a directory");
    }
    executor::block_on(Compressor::new(&argv[1]).compress());
}
