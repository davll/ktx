# ktx-async

[![crate-badge]][crate-link] [![docs-badge]][docs-link] [![ci-status]][ci-link]

[crate-badge]: https://img.shields.io/crates/v/ktx-async.svg
[crate-link]: https://crates.io/crates/ktx-async
[docs-badge]: https://docs.rs/ktx-async/badge.svg
[docs-link]: https://docs.rs/ktx-async
[ci-status]: https://travis-ci.com/davll/ktx.svg?branch=master
[ci-link]: https://travis-ci.com/davll/ktx


Asynchronous reader for KTX texture format

Features:

- Asynchronous IO API
- Works with [tokio](https://github.com/tokio-rs/tokio)
- Supports KTX 1.1

TODO:

- Skip mipmap levels
- Custom buffer allocation (ex: OpenGL Pixel Buffer Object)
- Add `std::io::Read` support (?)
- KTX 2.0 (?) [spec](http://github.khronos.org/KTX-Specification/)

Example:

```rust
use ktx_async as ktx;
use tokio::fs::File;
use tokio::stream::StreamExt as _;

// In async code

// Open AsyncRead
let file = File::open("example.ktx").await.unwrap();

// Start decoding KTX
let decoder = ktx::Decoder::new(file);
let (info, mut stream) = decoder.read_async().await.unwrap();

// create and bind a texture object ...

// Get all the frames from the stream
while let Some((frame, buf)) = stream.next().await.map(|r| r.unwrap()) {
    unsafe {
        gl::TexImage2D(gl::TEXTURE_2D,
            frame.level as GLint,
            info.gl_internal_format as GLint,
            frame.pixel_width as GLsizei,
            frame.pixel_height as GLsizei,
            /*border*/ 0,
            info.gl_format as GLenum,
            info.gl_type as GLenum,
            buf.as_ptr() as *const GLvoid);
    }
}
```

## Development

Build:

```
cargo build
```

Run Test:

```
cargo test
```

Run Example:

```
cargo run --example basic
```

## License

This project is licensed under the [MIT License](LICENSE)

## References

- [Documentation](https://www.khronos.org/opengles/sdk/tools/KTX/)
- [Specification](https://www.khronos.org/opengles/sdk/tools/KTX/file_format_spec/)
- [PVRTexTool](https://www.imgtec.com/developers/powervr-sdk-tools/pvrtextool/)
- [Khronos's Library and Tools](https://github.com/KhronosGroup/KTX-Software)
