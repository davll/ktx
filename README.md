# ktx

[![crate-badge]][crate-link] [![docs-badge]][docs-link]

[crate-badge]: https://img.shields.io/crates/v/ktx.svg
[crate-link]: https://crates.io/crates/ktx
[docs-badge]: https://docs.rs/ktx/badge.svg
[docs-link]: https://docs.rs/ktx

KTX texture format reader written in Rust

Features:

- Simple API
- Supports KTX 1.1
- Asynchronous (with [tokio][tokio])

[tokio][https://github.com/tokio-rs/tokio]

TODO:

- Add `std::io::Read` support
- Custom buffer allocation

Example:

```rust
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

### Build

```
cargo build
```

### Run Test

```
cargo test
```

### Run Example

```
cargo run --example basic
```

## License

This project is licensed under the [MIT License](LICENSE)

## File Structure

```
Byte[12] identifier
UInt32 endianness
UInt32 glType
UInt32 glTypeSize
UInt32 glFormat
Uint32 glInternalFormat
Uint32 glBaseInternalFormat
UInt32 pixelWidth
UInt32 pixelHeight
UInt32 pixelDepth
UInt32 numberOfArrayElements
UInt32 numberOfFaces
UInt32 numberOfMipmapLevels
UInt32 bytesOfKeyValueData

for each keyValuePair that fits in bytesOfKeyValueData
    UInt32   keyAndValueByteSize
    Byte     keyAndValue[keyAndValueByteSize]
    Byte     valuePadding[3 - ((keyAndValueByteSize + 3) % 4)]
end

for each mipmap_level in numberOfMipmapLevels*
    UInt32 imageSize;
    for each array_element in numberOfArrayElements*
        for each face in numberOfFaces
            for each z_slice in pixelDepth*
                for each row or row_of_blocks in pixelHeight*
                    for each pixel or block_of_pixels in pixelWidth
                        Byte data[format-specific-number-of-bytes]**
                    end
                end
            end
            Byte cubePadding[0-3]
        end
    end
    Byte mipPadding[3 - ((imageSize + 3) % 4)]
end
```

## References

- [Documentation](https://www.khronos.org/opengles/sdk/tools/KTX/)
- [Specification](https://www.khronos.org/opengles/sdk/tools/KTX/file_format_spec/)
- [PVRTexTool](https://www.imgtec.com/developers/powervr-sdk-tools/pvrtextool/)
- [Khronos's Library and Tools](https://github.com/KhronosGroup/KTX-Software)
