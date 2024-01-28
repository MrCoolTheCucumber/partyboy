#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(not(feature = "web"))]
pub fn now() -> u64 {
    use std::time::UNIX_EPOCH;
    UNIX_EPOCH.elapsed().unwrap().as_millis() as u64
}

#[wasm_bindgen(inline_js = r#"
    export function performance_now() {
        return performance.now();
    }
"#)]
#[cfg(feature = "web")]
extern "C" {
    fn performance_now() -> f64;
}

#[cfg(feature = "web")]
pub fn now() -> u64 {
    performance_now() as u64
}
