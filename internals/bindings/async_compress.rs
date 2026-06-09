use napi::{Env, Error, Result as NapiResult, Status, Task};
use napi_derive::napi;
use ui_build_compression::compress;

use crate::unwind_panic::unwind_panic;

pub struct AsyncCompress {
    pub path: String,
}

#[napi]
impl Task for AsyncCompress {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output, Error<Status>> {
        unwind_panic(|| compress(&self.path))
    }

    fn resolve(&mut self, _: Env, output: ()) -> NapiResult<Self::JsValue> {
        Ok(output)
    }
}
