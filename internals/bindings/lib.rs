use napi::bindgen_prelude::AsyncTask;
use napi_derive::napi;

use crate::async_compress::AsyncCompress;

mod async_compress;
mod unwind_panic;

#[napi]
pub fn compress(path: String) -> AsyncTask<AsyncCompress> {
    AsyncTask::new(AsyncCompress { path })
}
