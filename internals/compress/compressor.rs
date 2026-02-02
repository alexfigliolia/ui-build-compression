use std::{
    fs::File,
    io::{BufReader, Read, Write, copy},
    path::{Path, PathBuf},
};

use brotli::CompressorWriter;
use flate2::{
    Compression,
    write::{DeflateEncoder, GzEncoder},
};
use futures::{TryStreamExt, stream::FuturesUnordered};
use jwalk::WalkDir;

use crate::{concurrency::thread_pool::ThreadPool, logger::logger::Logger};

pub struct Compressor {
    directory: String,
}

impl Compressor {
    pub fn new(directory: String) -> Compressor {
        Compressor { directory }
    }

    pub async fn compress(&self) {
        let path = Path::new(&self.directory);
        if !path.is_absolute() || !path.exists() {
            Logger::exit_with_info("Please specify an absolute path to a directory");
        }
        Logger::info(format!("Compressing {}", self.directory).as_str());
        let mut total = 0;
        let mut compressed = 0;
        let mut futures = FuturesUnordered::new();
        let mut pool = ThreadPool::new(None, None, None);
        for entry in WalkDir::new(path).into_iter().filter_map(|e| {
            if e.is_err() {
                return None;
            }
            let option = e.ok();
            option.filter(|file| file.file_type().is_file())
        }) {
            total += 1;
            self.progress(total, compressed);
            futures.push(pool.spawn_blocking(move || {
                let tasks: [(PathBuf, fn(&PathBuf)); 4] = [
                    (entry.path(), Compressor::compress_brotli),
                    (entry.path(), Compressor::compress_deflate),
                    (entry.path(), Compressor::compress_gzip),
                    (entry.path(), Compressor::compress_zstd),
                ];
                tasks.map(|(path, task)| task(&path))
            }));
        }
        while let Ok(Some(_)) = futures.try_next().await {
            compressed += 1;
            self.progress(total, compressed);
        }
        pool.pool.shutdown_background();
        Logger::info(
            format!(
                "Finished! Compressed {} files",
                Logger::green(total.to_string().as_str())
            )
            .as_str(),
        );
    }

    fn progress(&self, total: i32, compressed: i32) {
        let percentage = (compressed * 100) / total;
        let max_bars = 25;
        let filled_bars = (compressed * max_bars) / total;
        let remaining = max_bars - filled_bars;
        print!(
            "\rCompressing [{}{}] {percentage}%",
            "=".repeat(filled_bars as usize),
            " ".repeat(remaining as usize)
        );
        if percentage == 100 {
            println!();
        }
    }

    fn compress_zstd(path: &PathBuf) {
        let mut output_file =
            File::create(format!("{}.zstd", path.to_str().expect("str"))).expect("will exist");
        let mut encoder = zstd::stream::Encoder::new(&mut output_file, 0).expect("created");
        copy(&mut Compressor::input_file(path), &mut encoder).expect("copy complete");
        encoder.finish().expect("done");
    }

    fn compress_brotli(path: &PathBuf) {
        let mut encoder = CompressorWriter::new(Compressor::output_file(path, ".br"), 4096, 11, 22);
        copy(&mut Compressor::input_file(path), &mut encoder).expect("copy complete");
        encoder.flush().expect("done");
    }

    fn compress_gzip(path: &PathBuf) {
        let mut input = Compressor::input_file(path);
        let mut reader = BufReader::new(&mut input);
        let mut encoder = GzEncoder::new(Compressor::output_file(path, ".gz"), Compression::new(9));
        copy(&mut reader, &mut encoder).expect("copy complete");
        encoder.finish().expect("done");
    }

    fn compress_deflate(path: &PathBuf) {
        let mut input = Compressor::input_file(path);
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer).expect("done");
        let mut encoder = DeflateEncoder::new(
            Compressor::output_file(path, ".deflate"),
            Compression::default(),
        );
        encoder.write_all(&buffer).expect("done");
        encoder.finish().expect("done");
    }

    fn input_file(path: &PathBuf) -> File {
        File::open(path).expect("exists")
    }

    fn output_file(path: &PathBuf, extension: &str) -> File {
        let path_string = path.to_str().expect("str");
        let output = File::create(format!("{}{extension}", path_string));
        if output.is_err() {
            panic!("Failed to create file at: \n\n\t{}", path_string)
        }
        output.expect("created")
    }
}
