# See https://app.element.io/#/room/#wgpu:matrix.org/$QSLVRMvxaKKgt9iyQ64rp-8z7wf7nJqMZRbUovagA90
# For WebGPU you need to compile with RUSTFLAGS="--cfg=web_sys_unstable_apis" env
RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve
