use napi::bindgen_prelude::AsyncTask;
use napi_derive::napi;

use crate::async_compress::AsyncCompress;

mod async_compress;
mod unwind_panic;

/// ## Compress
///
/// Given an absolute path to a file or directory, recursively compresses
/// the target path in place using Brotli, Gzip, Zstandard, and deflate at
/// the highest settings
/// ```typescript
/// import { compress } from "@ui-perf/build-compression";
///
/// await compress(/path/to/production/build);
/// ```
#[napi]
pub fn compress(path: String) -> AsyncTask<AsyncCompress> {
    AsyncTask::new(AsyncCompress { path })
}
