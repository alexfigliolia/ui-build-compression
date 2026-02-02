use futures::executor;

use crate::compress::compressor::Compressor;

mod compress;
mod concurrency;
mod logger;

/// Recursively compresses all files in the specified directory using gzip, brotli, deflate, and zstandard.
///
/// Produces compressed file variants along side the original files in the directory. Compression takes place inside of a pool of threads to maximize concurrency on the executing machine.
///
/// #### Arguments
/// * `directory` - An absolute path to a directory to compress
///
/// # Examples
///
/// ```
/// // Code in documentation blocks is compiled and run as a test by rustdoc.
/// use ui_build_compression::compress;
///
/// compress("/path/to/my/directory");
/// ```
pub fn compress(directory: &str) {
    executor::block_on(Compressor::new(directory.to_string()).compress());
}
