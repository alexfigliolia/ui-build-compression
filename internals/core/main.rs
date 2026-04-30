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
    let path = &argv[1];
    executor::block_on(Compressor::new(path.clone()).compress());
}
