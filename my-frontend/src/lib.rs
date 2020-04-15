#[macro_use]
extern crate lazy_static;

include!(concat!(env!("OUT_DIR"), "/wasm_blobs.rs"));

pub fn get_static_asset(s: &str) -> Option<&'static [u8]> {
    WASM.get(s).map(|p| *p)
}
