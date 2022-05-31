# WGPU/Rust Exploration 

## Building
Serving/building with [`trunk`](https://trunkrs.dev/)
```
cargo install --locked trunk 
trunk serve
``` 

## Editor
If running into problems with language server, make sure that the proper flag is being set for wgpu compilation
```
RUSTFLAGS=--cfg=web_sys_unstable_apis
```
