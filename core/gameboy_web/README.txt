Re export the gameboy lib crate (with the web feature flag) as a cdylib crate type. This is so we can build it with wasm-pack separetely.

Build instructions (for usage with webpack):

wasm-pack build --target bundler
