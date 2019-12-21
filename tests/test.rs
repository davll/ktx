extern crate ktx_async as ktx;

use futures_core::stream::Stream;
use futures_util::stream::StreamExt as _;
use ktx::Decoder;
use lazy_static::lazy_static;
use tokio;
use tokio::fs::File;
use tokio::io::BufReader;

const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_RGB: u32 = 0x1907;
const GL_RGBA: u32 = 0x1908;
const GL_RGB8: u32 = 0x8051;
const GL_RGBA8: u32 = 0x8058;
const GL_COMPRESSED_SRGB_ALPHA_PVRTC_4BPPV1_EXT: u32 = 0x8A57;
const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT: u32 = 0x83F3;
const GL_COMPRESSED_RGB8_ETC2: u32 = 0x9274;
const GL_ETC1_RGB8_OES: u32 = 0x8D64;

#[allow(non_upper_case_globals)]
const GL_COMPRESSED_RGBA_ASTC_8x8_KHR: u32 = 0x93B7;

#[tokio::test]
async fn test_rgb_reference() {
    let path = "data/khr/rgb-reference.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, GL_UNSIGNED_BYTE);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, GL_RGB);
    assert_eq!(info.gl_internal_format, GL_RGB8);
    assert_eq!(info.gl_base_internal_format, GL_RGB);
    assert_eq!(info.pixel_width, 128);
    assert_eq!(info.pixel_height, 128);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 0);
    assert_eq!(info.number_of_faces, 1);
    assert_eq!(info.number_of_mipmap_levels, 1);

    let (frame, buf) = stream.next().await.map(|r| r.unwrap()).unwrap();
    let expected_image_size = ((128 * 3 + 3) / 4) * 4 * 128;
    assert_eq!(frame.level, 0);
    assert_eq!(frame.layer, 0);
    assert_eq!(frame.face, 0);
    assert_eq!(frame.pixel_width, 128);
    assert_eq!(frame.pixel_height, 128);
    assert_eq!(frame.pixel_depth, 1);
    assert_eq!(buf.len(), expected_image_size);
}

#[tokio::test]
async fn test_rgb_mipmap_reference() {
    let path = "data/khr/rgb-mipmap-reference.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, GL_UNSIGNED_BYTE);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, GL_RGB);
    assert_eq!(info.gl_internal_format, GL_RGB8);
    assert_eq!(info.gl_base_internal_format, GL_RGB);
    assert_eq!(info.pixel_width, 64);
    assert_eq!(info.pixel_height, 64);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 0);
    assert_eq!(info.number_of_faces, 1);
    assert_eq!(info.number_of_mipmap_levels, 7);

    let mut level = 0;
    while let Some((frame, buf)) = stream.next().await.map(|r| r.unwrap()) {
        let width = info.pixel_width >> level;
        let height = info.pixel_height >> level;
        let expected_image_size = ((width * 3 + 3) / 4) * 4 * height;
        assert_eq!(frame.level, level);
        assert_eq!(frame.layer, 0);
        assert_eq!(frame.face, 0);
        assert_eq!(frame.pixel_width, width);
        assert_eq!(frame.pixel_height, height);
        assert_eq!(frame.pixel_depth, 1);
        assert_eq!(buf.len(), expected_image_size as usize);
        level += 1;
    }
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_rgba_reference() {
    let path = "data/khr/rgba-reference.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, GL_UNSIGNED_BYTE);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, GL_RGBA);
    assert_eq!(info.gl_internal_format, GL_RGBA8);
    assert_eq!(info.gl_base_internal_format, GL_RGBA);
    assert_eq!(info.pixel_width, 128);
    assert_eq!(info.pixel_height, 128);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 0);
    assert_eq!(info.number_of_faces, 1);
    assert_eq!(info.number_of_mipmap_levels, 1);

    let (frame, buf) = stream.next().await.map(|r| r.unwrap()).unwrap();
    let expected_image_size = ((128 * 4 + 3) / 4) * 4 * 128;
    assert_eq!(frame.level, 0);
    assert_eq!(frame.layer, 0);
    assert_eq!(frame.face, 0);
    assert_eq!(frame.pixel_width, 128);
    assert_eq!(frame.pixel_height, 128);
    assert_eq!(frame.pixel_depth, 1);
    assert_eq!(buf.len(), expected_image_size);
}

#[tokio::test]
async fn test_etc1() {
    let path = "data/khr/etc1.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, 0);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, 0);
    assert_eq!(info.gl_internal_format, GL_ETC1_RGB8_OES);
    assert_eq!(info.gl_base_internal_format, GL_RGB);
    assert_eq!(info.pixel_width, 128);
    assert_eq!(info.pixel_height, 128);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 0);
    assert_eq!(info.number_of_faces, 1);
    assert_eq!(info.number_of_mipmap_levels, 1);

    let (frame, buf) = stream.next().await.map(|r| r.unwrap()).unwrap();
    let expected_image_size = etc1_block_image_size(128, 128) as usize;
    assert_eq!(frame.level, 0);
    assert_eq!(frame.layer, 0);
    assert_eq!(frame.face, 0);
    assert_eq!(frame.pixel_width, 128);
    assert_eq!(frame.pixel_height, 128);
    assert_eq!(frame.pixel_depth, 1);
    assert_eq!(buf.len(), expected_image_size);
}

#[tokio::test]
async fn test_cubemap_etc2() {
    let path = "data/khr/cubemap_yokohama_etc2_unorm.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, 0);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, 0);
    assert_eq!(info.gl_internal_format, GL_COMPRESSED_RGB8_ETC2);
    assert_eq!(info.gl_base_internal_format, GL_RGB);
    assert_eq!(info.pixel_width, 512);
    assert_eq!(info.pixel_height, 512);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 0);
    assert_eq!(info.number_of_faces, 6);
    assert_eq!(info.number_of_mipmap_levels, 1);

    let mut face = 0;
    while let Some((frame, buf)) = stream.next().await.map(|r| r.unwrap()) {
        let expected_image_size = etc2_block_image_size(512, 512) as usize;
        assert_eq!(frame.level, 0);
        assert_eq!(frame.layer, 0);
        assert_eq!(frame.face, face);
        assert_eq!(frame.pixel_width, 512);
        assert_eq!(frame.pixel_height, 512);
        assert_eq!(frame.pixel_depth, 1);
        assert_eq!(buf.len(), expected_image_size);
        face += 1;
    }
}

#[tokio::test]
async fn test_cubemap_mipmap_reference() {
    let path = "data/khr/cubemap_yokohama_astc_8x8_unorm.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, 0);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, 0);
    assert_eq!(info.gl_internal_format, GL_COMPRESSED_RGBA_ASTC_8x8_KHR);
    assert_eq!(info.gl_base_internal_format, GL_RGBA);
    assert_eq!(info.pixel_width, 512);
    assert_eq!(info.pixel_height, 512);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 0);
    assert_eq!(info.number_of_faces, 6);
    assert_eq!(info.number_of_mipmap_levels, 10);

    for level in 0..10 {
        let width = 512 >> level;
        let height = 512 >> level;
        for face in 0..6 {
            let (frame, buf) = stream.next().await.map(|r| r.unwrap()).unwrap();
            assert_eq!(frame.level, level);
            assert_eq!(frame.layer, 0);
            assert_eq!(frame.face, face);
            assert_eq!(frame.pixel_width, width);
            assert_eq!(frame.pixel_height, height);
            assert_eq!(frame.pixel_depth, 1);
            let _ = buf;
        }
    }
}

#[tokio::test]
async fn test_array_pvrtc() {
    let path = "data/pvr/array-pvrtc-mipmap.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, 0);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, 0);
    assert_eq!(
        info.gl_internal_format,
        GL_COMPRESSED_SRGB_ALPHA_PVRTC_4BPPV1_EXT
    );
    assert_eq!(info.gl_base_internal_format, GL_RGBA);
    assert_eq!(info.pixel_width, 256);
    assert_eq!(info.pixel_height, 256);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 7);
    assert_eq!(info.number_of_faces, 1);
    assert_eq!(info.number_of_mipmap_levels, 9);

    for level in 0..9 {
        let width = 256 >> level;
        let height = 256 >> level;
        for layer in 0..7 {
            let (frame, buf) = stream.next().await.map(|r| r.unwrap()).unwrap();
            let expected_image_size = pvrtc4bppv1_block_image_size(width, height) as usize;
            assert_eq!(frame.level, level);
            assert_eq!(frame.layer, layer);
            assert_eq!(frame.face, 0);
            assert_eq!(frame.pixel_width, width);
            assert_eq!(frame.pixel_height, height);
            assert_eq!(frame.pixel_depth, 1);
            assert_eq!(buf.len(), expected_image_size);
        }
    }
}

#[tokio::test]
async fn test_array_bc3_unorm() {
    let path = "data/khr/texturearray_bc3_unorm.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let reader = BufReader::new(file);
    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, 0);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, 0);
    assert_eq!(info.gl_internal_format, GL_COMPRESSED_RGBA_S3TC_DXT5_EXT);
    assert_eq!(info.gl_base_internal_format, GL_RGBA);
    assert_eq!(info.pixel_width, 256);
    assert_eq!(info.pixel_height, 256);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 7);
    assert_eq!(info.number_of_faces, 1);
    assert_eq!(info.number_of_mipmap_levels, 1);

    let mut lyr = 0;
    while let Some((frame, buf)) = stream.next().await.map(|r| r.unwrap()) {
        let expected_image_size = bc3_block_image_size(256, 256) as usize;
        assert_eq!(frame.level, 0);
        assert_eq!(frame.layer, lyr);
        assert_eq!(frame.face, 0);
        assert_eq!(frame.pixel_width, 256);
        assert_eq!(frame.pixel_height, 256);
        assert_eq!(frame.pixel_depth, 1);
        assert_eq!(buf.len(), expected_image_size);
        lyr += 1;
    }
}

lazy_static! {
    static ref PROJECT_DIR: std::path::PathBuf = {
        use std::env::var_os;
        var_os("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap()
    };
}

fn pvrtc4bppv1_block_image_size(w: u32, h: u32) -> u32 {
    use std::cmp::max;

    (max(w, 8) * max(h, 8) * 4 + 7) / 8
}

fn bc3_block_image_size(w: u32, h: u32) -> u32 {
    let bw = (w + 3) / 4;
    let bh = (h + 3) / 4;
    (16 * bw * bh)
}

fn etc2_block_image_size(w: u32, h: u32) -> u32 {
    let bw = (w + 3) / 4;
    let bh = (h + 3) / 4;
    (8 * bw * bh)
}

fn etc1_block_image_size(w: u32, h: u32) -> u32 {
    let bw = (w + 3) / 4;
    let bh = (h + 3) / 4;
    (8 * bw * bh)
}

pub struct StreamRead<S> {
    stream: S,
    buf: Vec<u8>,
    pos: usize,
}

impl<S> StreamRead<S> {
    pub fn new(stream: S) -> Self {
        StreamRead {
            stream,
            buf: vec![],
            pos: 0,
        }
    }
}

impl<S> tokio::io::AsyncRead for StreamRead<S>
where
    S: Stream<Item = std::io::Result<Vec<u8>>> + Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut futures_core::task::Context<'_>,
        buf: &mut [u8],
    ) -> futures_core::task::Poll<std::io::Result<usize>> {
        use futures_core::task::Poll;
        use std::cmp::min;
        use std::pin::Pin;

        let this = self.get_mut();
        let curr_buf = &mut this.buf;
        let curr_pos = &mut this.pos;
        let stream = Pin::new(&mut this.stream);

        // Edge case
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        // Fetch Data
        if curr_buf.is_empty() || *curr_pos == curr_buf.len() {
            match stream.poll_next(cx) {
                Poll::Ready(Some(result)) => match result {
                    Ok(buf) => {
                        *curr_buf = buf;
                        *curr_pos = 0;
                    }
                    Err(e) => return Poll::Ready(Err(e)),
                },
                Poll::Ready(None) => return Poll::Ready(Ok(0)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let max_count = curr_buf.len() - *curr_pos;
        let count = min(buf.len(), max_count);
        let start = *curr_pos;
        let end = start + count;
        *curr_pos += count;
        buf[0..count].copy_from_slice(&curr_buf[start..end]);
        Poll::Ready(Ok(count))
    }
}

#[tokio::test]
async fn test_rgb_reference_from_stream() {
    use futures_util::StreamExt as _;
    use tokio::io::AsyncReadExt as _;

    let path = "data/khr/rgb-reference.ktx";
    let file = File::open(PROJECT_DIR.join(path)).await.unwrap();
    let mut reader = BufReader::new(file);

    let mut buf = vec![];
    reader.read_to_end(&mut buf).await.unwrap();

    let stream = futures_util::stream::iter(buf.chunks(128).map(|x| Ok(Vec::from(x))));
    let reader = StreamRead::new(stream);

    let decoder = Decoder::new(reader);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    //println!("info = {:?}", &info);
    assert_eq!(info.gl_type, GL_UNSIGNED_BYTE);
    assert_eq!(info.gl_type_size, 1);
    assert_eq!(info.gl_format, GL_RGB);
    assert_eq!(info.gl_internal_format, GL_RGB8);
    assert_eq!(info.gl_base_internal_format, GL_RGB);
    assert_eq!(info.pixel_width, 128);
    assert_eq!(info.pixel_height, 128);
    assert_eq!(info.pixel_depth, 0);
    assert_eq!(info.number_of_array_elements, 0);
    assert_eq!(info.number_of_faces, 1);
    assert_eq!(info.number_of_mipmap_levels, 1);

    let (frame, buf) = stream.next().await.map(|r| r.unwrap()).unwrap();
    let expected_image_size = ((128 * 3 + 3) / 4) * 4 * 128;
    assert_eq!(frame.level, 0);
    assert_eq!(frame.layer, 0);
    assert_eq!(frame.face, 0);
    assert_eq!(frame.pixel_width, 128);
    assert_eq!(frame.pixel_height, 128);
    assert_eq!(frame.pixel_depth, 1);
    assert_eq!(buf.len(), expected_image_size);
}
