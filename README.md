# WGPU/Rust Exploration 

## Prequisites
Install rustup 
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install the rust wasm32 target for compiling to WebAssembly
```
rustup target add wasm32-unknown-unknown
```

Install [`trunk`](https://trunkrs.dev/) for building rust web applications
```
cargo install --locked trunk
```

## Building
To build and serve, use ./debug or ./release respectively. Then open localhost:8080. If you are not seeing
anything in the canvas make sure that you have Unsafe WebGPU enabled (chrome://flags -> UnsafeWebGPU) in chrome (don't enable this on your main browser!)

## Editor
If running into problems with language server, make sure that the proper flag is being set for wgpu compilation
```
RUSTFLAGS=--cfg=web_sys_unstable_apis
```


