use std::{
    fs::{File, remove_file},
    io::{BufReader, Read, Write, copy},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use brotli::CompressorWriter;
use flate2::{
    Compression,
    write::{DeflateEncoder, GzEncoder},
};
use futures::{TryStreamExt, stream::FuturesUnordered};
use jwalk::WalkDir;
use tokio_thread_pool::ThreadPool;

use crate::logger::logger::Logger;

static BROTLI_EXTENSION: &str = "br";
static GZIP_EXTENSION: &str = "gz";
static ZSTANDARD_EXTENSION: &str = "zst";
static DEFLATE_EXTENSION: &str = "deflate";

static EXTENSION_BLACKLIST: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        BROTLI_EXTENSION,
        GZIP_EXTENSION,
        ZSTANDARD_EXTENSION,
        DEFLATE_EXTENSION,
    ]
});

pub struct Compressor {
    target: String,
    total: usize,
    compressed: usize,
    prev_print_length: i32,
}

impl Compressor {
    pub fn new(target: &str) -> Compressor {
        Compressor {
            total: 0,
            compressed: 0,
            prev_print_length: 0,
            target: target.to_owned(),
        }
    }

    pub async fn compress(&mut self) {
        let path = self.validate_path();
        let mut tasks = FuturesUnordered::new();
        let files = self.collect_files(&path);
        let encodings = [
            Compressor::compress_brotli,
            Compressor::compress_gzip,
            Compressor::compress_zst,
            Compressor::compress_deflate,
        ];
        self.total = files.len() * encodings.len();
        let mut thread_pool = ThreadPool::new(None, None, None);
        for file in files {
            for encoder in encodings {
                let file = file.clone();
                tasks.push(thread_pool.spawn(move || {
                    encoder(&file);
                }));
            }
        }
        while let Ok(Some(_)) = tasks.try_next().await {
            self.compressed += 1;
            self.progress();
        }
        thread_pool.pool.shutdown_background();
        Logger::info(
            format!(
                "Finished! Compressed {} files",
                Logger::green((self.total).to_string().as_str())
            )
            .as_str(),
        );
    }

    fn collect_files(&self, path: &Path) -> Vec<PathBuf> {
        let mut result: Vec<PathBuf> = Vec::new();
        for entry in WalkDir::new(path).into_iter().filter_map(|e| {
            if e.is_err() {
                return None;
            }
            let option = e.ok();
            option.filter(|file| {
                file.file_type().is_file()
                    && file.path().extension().is_some_and(|value| {
                        if let Some(ext) = value.to_str() {
                            return !EXTENSION_BLACKLIST.contains(&ext);
                        }
                        false
                    })
            })
        }) {
            result.push(entry.path());
        }
        result
    }

    fn progress(&mut self) {
        let percentage = (self.compressed * 100) / self.total;
        let max_bars = 25;
        let filled_bars = (self.compressed * max_bars) / self.total;
        let remaining = max_bars - filled_bars;
        let text = format!(
            "Compressing [{}{}] {percentage}% ({} files remaining)",
            "=".repeat(filled_bars),
            " ".repeat(remaining),
            self.total - self.compressed
        );
        let line_length = text.len() as i32;
        if self.prev_print_length == 0 {
            self.prev_print_length = line_length;
        }
        let diff = self.prev_print_length - line_length;
        let mut appended_spaces = diff;
        if appended_spaces < 0 {
            appended_spaces = 0;
        }
        print!("\r{}{}", text, " ".repeat(appended_spaces as usize));
        self.prev_print_length = line_length;
        if self.total == self.compressed {
            println!();
        }
    }

    fn compress_zst(path: &Path) {
        let mut output_file = Compressor::output_file(path, ZSTANDARD_EXTENSION);
        if let Err(err) =
            zstd::stream::copy_encode(&mut Compressor::input_file(path), &mut output_file, 22)
        {
            Logger::encoding_error(path, "zstandard");
            println!("{err}");
        }
    }

    fn compress_brotli(path: &Path) {
        let mut output_file = Compressor::output_file(path, BROTLI_EXTENSION);
        let mut reader = BufReader::new(Compressor::input_file(path));
        let mut writer = CompressorWriter::new(&mut output_file, 4096, 11, 22);
        if let Err(err) = copy(&mut reader, &mut writer) {
            Logger::encoding_error(path, "zstandard");
            println!("{err}");
        }
        if let Err(err) = writer.flush() {
            Logger::encoding_error(path, "zstandard");
            println!("{err}");
        }
    }

    fn compress_gzip(path: &Path) {
        let output_file = Compressor::output_file(path, GZIP_EXTENSION);
        let mut input = Compressor::input_file(path);
        let mut reader = BufReader::new(&mut input);
        let mut encoder = GzEncoder::new(output_file, Compression::best());
        if let Err(copy_err) = copy(&mut reader, &mut encoder) {
            Logger::encoding_error(path, "gzip");
            println!("{copy_err}");
            return;
        }
        if let Err(flush_error) = encoder.finish() {
            Logger::encoding_error(path, "gzip");
            println!("{flush_error}");
        }
    }

    fn compress_deflate(path: &Path) {
        let mut input = Compressor::input_file(path);
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer).expect("done");
        let mut encoder = DeflateEncoder::new(
            Compressor::output_file(path, DEFLATE_EXTENSION),
            Compression::best(),
        );
        if let Err(write_err) = encoder.write_all(&buffer) {
            Logger::encoding_error(path, "deflate");
            println!("{write_err}");
            return;
        }
        if let Err(flush_err) = encoder.finish() {
            Logger::encoding_error(path, "deflate");
            println!("{flush_err}");
        }
    }

    fn input_file(path: &Path) -> File {
        let file_result = File::open(path);
        if let Err(open_error) = &file_result {
            eprintln!("{open_error}");
            Logger::error("Failed to open the file:");
            Logger::log_file_path(&path.to_string_lossy());
        }
        file_result.unwrap()
    }

    fn output_file(path: &Path, extension: &str) -> File {
        let path_string = format!("{}.{extension}", path.to_string_lossy());
        let new_path = Path::new(&path_string);
        if new_path.exists()
            && let Err(remove_error) = remove_file(new_path)
        {
            eprintln!("{remove_error}");
            Logger::error("Failed to delete the following stale file:");
            Logger::log_file_path(&path_string);
            panic!();
        }
        let create_result = File::create(new_path);
        if let Err(create_error) = create_result {
            eprintln!("{create_error}");
            Logger::error("Failed to create compressed file:");
            Logger::log_file_path(&path_string);
            panic!();
        }
        create_result.unwrap()
    }

    fn validate_path(&self) -> PathBuf {
        let path = Path::new(&self.target);
        if !path.is_absolute() || !path.exists() {
            Logger::exit_with_info("Please specify an absolute path to a file or directory");
        }
        Logger::info(format!("Compressing {}", self.target).as_str());
        path.to_path_buf()
    }
}
