//! KTX Texture Format Loader
//!
//! https://www.khronos.org/opengles/sdk/tools/KTX/file_format_spec/

/*
File Structure:

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
*/
// # imageSize
//
// For most textures `imageSize` is the number of bytes of
// pixel data in the current LOD level.
//
// This includes all array layers, all z slices, all faces,
// all rows (or rows of blocks) and all pixels (or blocks) in
// each row for the mipmap level. It does not include any
// bytes in mipPadding.
//
// The exception is non-array cubemap textures
// (any texture where numberOfFaces is 6 and
// numberOfArrayElements is 0).
//
// For these textures imageSize is the number of bytes in
// each face of the texture for the current LOD level,
// not including bytes in cubePadding or mipPadding.
//
// # cubePadding
//
// For non-array cubemap textures (any texture where
// numberOfFaces is 6 and numberOfArrayElements is 0)
// cubePadding contains between 0 and 3 bytes of value 0x00
// to ensure that the data in each face begins at a file offset
// that is a multiple of 4.
//
// In all other cases cubePadding is empty (0 bytes long).
//
// This is empty in the non-array cubemap case as well.
// The requirement of GL_UNPACK_ALIGNMENT = 4 means the
// size of uncompressed textures will always be a multiple of
// 4 bytes. All known compressed formats, that are usable for
// cubemaps, have block sizes that are a multiple of 4 bytes.
//
// The field is still shown in case a compressed format emerges
// with a block size that is not a multiple of 4 bytes.
//
// # mipPadding
//
// Between 0 and 3 bytes of value 0x00 to make sure that all
// imageSize fields are at a file offset that is a multiple of 4.
//
// This is empty for all known texture formats for the reasons
// given in cubePadding and is retained for the same reason.
//

#![recursion_limit = "256"]
#![deny(unsafe_code)]

extern crate async_stream;
extern crate byteorder;
extern crate error_chain;
extern crate futures_core;
extern crate num_derive;
extern crate num_traits;
extern crate tokio;

use error_chain::{bail, error_chain};
use futures_core::stream::Stream;
use tokio::io::{AsyncRead, AsyncReadExt as _};

/// KTX decoder
pub struct Decoder<R> {
    read: R,
}

impl<R> Decoder<R> {
    pub fn new(read: R) -> Self {
        Decoder { read }
    }
}

impl<R> Decoder<R>
where
    R: AsyncRead + Unpin,
{
    /// Read the header and the following frames asynchronously
    pub async fn read_async(
        self,
    ) -> Result<(
        HeaderInfo,
        impl Stream<Item = Result<(FrameInfo, Vec<u8>)>> + Unpin,
    )> {
        let mut read = self.read;

        // Read the header
        let info = read_header_async(&mut read).await?;

        // Create the stream of the frames
        let stream = new_async_stream(read, &info);

        Ok((info, stream))
    }
}

fn new_async_stream(
    read: impl AsyncRead + Unpin,
    info: &HeaderInfo,
) -> impl Stream<Item = Result<(FrameInfo, Vec<u8>)>> + Unpin {
    use async_stream::try_stream;
    use byteorder::{ByteOrder as _, NativeEndian as NE};
    use std::cmp::max;

    // Prepare parameters for the stream
    let pixel_width = info.pixel_width;
    let pixel_height = info.pixel_height;
    let pixel_depth = info.pixel_depth;
    let nlayers = max(1, info.number_of_array_elements);
    let nfaces = max(1, info.number_of_faces);
    let nlevels = info.number_of_mipmap_levels;

    // Check if it is a non-array cubemap
    let is_cubemap = info.number_of_faces == 6 && info.number_of_array_elements == 0;

    Box::pin(try_stream! {
        let mut read = read;
        for level in 0..nlevels {
            let image_size = {
                let mut buf = [0_u8; 4];
                read.read_exact(&mut buf).await?;
                NE::read_u32(&buf)
            };

            // FIXME: what if image_size is not 4-byte aligned?
            assert!(image_size % 4 == 0);

            // dimensions of the current mipmap level
            let pixel_width = max(1, pixel_width >> level);
            let pixel_height = max(1, pixel_height >> level);
            let pixel_depth = max(1, pixel_depth >> level);

            // Compute buffer size
            let face_size = if is_cubemap {
                image_size
            } else {
                assert!(image_size % nlayers == 0);
                let layer_size = image_size / nlayers;
                assert!(layer_size % 4 == 0);
                assert!(layer_size % nfaces == 0);
                layer_size / nfaces
            };
            assert!(face_size % 4 == 0);
            let buf_size = face_size as usize;

            // Read pixels
            for layer in 0..nlayers {
                for face in 0..nfaces {
                    let mut buf = vec![0_u8; buf_size];
                    read.read_exact(&mut buf).await?;
                    let frame_info = FrameInfo {
                        level,
                        layer,
                        face,
                        pixel_width,
                        pixel_height,
                        pixel_depth,
                    };
                    yield (frame_info, buf);
                }
            }
        }
    })
}

/// KTX Frame Info
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// mip-map level
    pub level: u32,
    /// layer in texture array
    pub layer: u32,
    /// face in cubemap (+X, -X, +Y, -Y, +Z, -Z).
    /// 0 if not cubemap.
    pub face: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_depth: u32,
}

/// KTX Header Info
#[derive(Debug, Clone)]
pub struct HeaderInfo {
    /// For compressed textures, glType must equal 0.
    /// For uncompressed textures, glType specifies the type
    /// parameter passed to glTex{,Sub}Image*D, usually one of
    /// the values from table 8.2 of the OpenGL 4.4 specification
    /// [OPENGL44] (UNSIGNED_BYTE, UNSIGNED_SHORT_5_6_5, etc.)
    pub gl_type: u32,
    /// glTypeSize specifies the data type size that should be used
    /// when endianness conversion is required for the texture data
    /// stored in the file. If glType is not 0, this should be the
    /// size in bytes corresponding to glType. For texture data which
    /// does not depend on platform endianness, including compressed
    /// texture data, glTypeSize must equal 1.
    pub gl_type_size: u32,
    /// For compressed textures, glFormat must equal 0.
    /// For uncompressed textures, glFormat specifies the format
    /// parameter passed to glTex{,Sub}Image*D, usually one of
    /// the values from table 8.3 of the OpenGL 4.4 specification
    /// [OPENGL44] (RGB, RGBA, BGRA, etc.)
    pub gl_format: u32,
    /// For compressed textures, glInternalFormat must equal the
    /// compressed internal format, usually one of the values from
    /// table 8.14 of the OpenGL 4.4 specification [OPENGL44].
    /// For uncompressed textures, glInternalFormat specifies the
    /// internalformat parameter passed to glTexStorage*D or
    /// glTexImage*D, usually one of the sized internal formats
    /// from tables 8.12 & 8.13 of the OpenGL 4.4 specification
    /// [OPENGL44].
    /// The sized format should be chosen to match the bit depth of
    /// the data provided. glInternalFormat is used when
    /// loading both compressed and uncompressed textures,
    /// exceptwhen loading into a context that does not support
    /// sized formats, such as an unextended OpenGL ES 2.0 context
    /// where the internalformat parameter is required to have the
    /// same value as the format parameter.
    pub gl_internal_format: u32,
    /// For both compressed and uncompressed textures,
    /// glBaseInternalFormat specifies the base internal
    /// format of the texture, usually one of the values
    /// from table 8.11 of the OpenGL 4.4 specification [OPENGL44]
    /// (RGB, RGBA, ALPHA, etc.). For uncompressed textures,
    /// this value will be the same as glFormat and is used as
    /// the internalformat parameter when loading into a context
    /// that does not support sized formats, such as an unextended
    /// OpenGL ES 2.0 context.
    pub gl_base_internal_format: u32,
    /// The size of the texture image for level 0, in pixels.
    /// No rounding to block sizes should be applied for block
    /// compressed textures.
    ///
    /// For 1D textures pixelHeight and pixelDepth must be 0.
    /// For 2D and cube textures pixelDepth must be 0.
    pub pixel_width: u32,
    /// See `pixel_width`
    pub pixel_height: u32,
    /// See `pixel_width`
    pub pixel_depth: u32,
    /// numberOfArrayElements specifies the number of array elements.
    /// If the texture is not an array texture, numberOfArrayElements must equal 0.
    pub number_of_array_elements: u32,
    /// numberOfFaces specifies the number of cubemap faces.
    /// For cubemaps and cubemap arrays this should be 6.
    /// For non cubemaps this should be 1.
    /// Cube map faces are stored in the order: +X, -X, +Y, -Y, +Z, -Z.
    pub number_of_faces: u32,
    /// numberOfMipmapLevels must equal 1 for non-mipmapped textures.
    /// For mipmapped textures, it equals the number of mipmaps.
    /// Mipmaps are stored in order from largest size to smallest size.
    /// The first mipmap level is always level 0.
    /// A KTX file does not need to contain a complete mipmap pyramid.
    /// If numberOfMipmapLevels equals 0, it indicates that a full mipmap
    /// pyramid should be generated from level 0 at load time (this is
    /// usually not allowed for compressed formats).
    pub number_of_mipmap_levels: u32,
    /// keyAndValue contains 2 separate sections.
    /// First it contains a key encoded in UTF-8 without
    /// a byte order mark (BOM). The key must be terminated by a
    /// NUL character (a single 0x00 byte). Keys that begin with
    /// the 3 ascii characters 'KTX' or 'ktx' are reserved and must
    /// not be used except as described by this spec (this version
    /// of the KTX spec defines a single key). Immediately following
    /// the NUL character that terminates the key is the Value data.
    ///
    /// The Value data may consist of any arbitrary data bytes.
    /// Any byte value is allowed. It is encouraged that the value
    /// be a NUL terminated UTF-8 string but this is not required.
    /// UTF-8 strings must not contain BOMs. If the Value data is
    /// binary, it is a sequence of bytes rather than of words.
    /// It is up to the vendor defining the key to specify how
    /// those bytes are to be interpreted (including the endianness
    /// of any encoded numbers). If the Value data is a string of
    /// bytes then the NUL termination should be included in the
    /// keyAndValueByteSize byte count (but programs that read KTX
    /// files must not rely on this).
    pub key_value_data: KeyValueData,
}

impl HeaderInfo {
    pub fn mipmap_size(&self, level: u32) -> (u32, u32, u32) {
        use std::cmp::max;
        let w = max(1, self.pixel_width >> level);
        let h = max(1, self.pixel_height >> level);
        let d = max(1, self.pixel_depth >> level);
        (w, h, d)
    }
}

async fn read_header_async(mut reader: impl AsyncRead + Unpin) -> Result<HeaderInfo> {
    use byteorder::{ByteOrder as _, NativeEndian as NE};

    let buf = {
        let mut v = [0_u8; 64];
        reader.read_exact(&mut v).await?;
        v
    };

    // Check magic
    {
        let magic: &[u8] = &buf[0..12];
        if magic != MAGIC {
            let mut m = [0_u8; 12];
            m.copy_from_slice(magic);
            bail!(ErrorKind::InvalidFormat(m));
        }
    }

    let endianness = NE::read_u32(&buf[12..16]);
    let gl_type = NE::read_u32(&buf[16..20]);
    let gl_type_size = NE::read_u32(&buf[20..24]);
    let gl_format = NE::read_u32(&buf[24..28]);
    let gl_internal_format = NE::read_u32(&buf[28..32]);
    let gl_base_internal_format = NE::read_u32(&buf[32..36]);
    let pixel_width = NE::read_u32(&buf[36..40]);
    let pixel_height = NE::read_u32(&buf[40..44]);
    let pixel_depth = NE::read_u32(&buf[44..48]);
    let number_of_array_elements = NE::read_u32(&buf[48..52]);
    let number_of_faces = NE::read_u32(&buf[52..56]);
    let number_of_mipmap_levels = NE::read_u32(&buf[56..60]);
    let bytes_of_key_value_data = NE::read_u32(&buf[60..64]);

    if number_of_mipmap_levels == 0 {
        bail!(ErrorKind::InvalidNumberOfMipmapLevels(
            number_of_mipmap_levels
        ));
    }

    if (endianness == ENDIANNESS) && (bytes_of_key_value_data % 4 == 0) {
        let mut kvbuf = vec![0; bytes_of_key_value_data as usize];
        reader.read_exact(&mut kvbuf).await?;
        let info = HeaderInfo {
            gl_type,
            gl_type_size,
            gl_format,
            gl_internal_format,
            gl_base_internal_format,
            pixel_width,
            pixel_height,
            pixel_depth,
            number_of_array_elements,
            number_of_faces,
            number_of_mipmap_levels,
            key_value_data: KeyValueData { raw: kvbuf },
        };
        Ok(info)
    } else {
        bail!(ErrorKind::MismatchedEndianness(ENDIANNESS, endianness));
    }
}

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }
    foreign_links {
        Io(::std::io::Error);
    }
    errors {
        InvalidFormat(magic: [u8;12]) {
        }
        MismatchedEndianness(expect: u32, actual: u32) {
        }
        InvalidNumberOfMipmapLevels(v: u32) {
        }
    }
}

const MAGIC: [u8; 12] = [
    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x31, 0x31, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
];
const ENDIANNESS: u32 = 0x0403_0201;

#[derive(Clone)]
pub struct KeyValueData {
    raw: Vec<u8>,
}

pub struct Entries<'a>(&'a [u8]);

impl KeyValueData {
    pub fn iter(&self) -> Entries {
        Entries(&self.raw)
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = (&'a str, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        use byteorder::{ByteOrder, NativeEndian};
        use std::str::from_utf8;

        if self.0.is_empty() {
            return None;
        }
        let (len_bytes, resting) = self.0.split_at(4);
        let len = NativeEndian::read_u32(len_bytes);
        let (kv, nextbuf) = resting.split_at(force_align(len) as usize);
        let (kv, _padding) = kv.split_at(len as usize);
        self.0 = nextbuf;
        let nul_idx = kv
            .iter()
            .enumerate()
            .filter(|(_, x)| **x == 0)
            .map(|(i, _)| i)
            .nth(0)
            .unwrap();
        let (key, value) = kv.split_at(nul_idx);
        let value = value.split_at(1).1;
        let key = from_utf8(key).unwrap();
        Some((key, value))
    }
}

impl std::fmt::Debug for KeyValueData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "KeyValueData[")?;
        for (key, value) in self.iter() {
            write!(f, "({:?}, bytes(len={})), ", key, value.len())?;
        }
        write!(f, "]")
    }
}

#[inline]
fn force_align(x: u32) -> u32 {
    (x + 0x3) & 0xFFFF_FFFC
}
