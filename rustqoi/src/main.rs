#![feature(test)]
extern crate test;
use std::ops::{Add, Sub};

const QOI_HEADER_SIZE: usize = 14;
const QOI_FOOTER_SIZE: usize = 8;
const QOI_MAGIC: [u8; 4] = *b"qoif";
const QOI_OP_RUN: u8 = 0b11000000;
const QOI_OP_RUN_END: u8 = QOI_OP_RUN | 0b00111101;
const QOI_OP_INDEX: u8 = 0b00000000;
const QOI_OP_INDEX_END: u8 = QOI_OP_INDEX | 0b00111111;
const QOI_OP_DIFF: u8 = 0b01000000;
const QOI_OP_DIFF_END: u8 = QOI_OP_DIFF | 0b00111111;
const QOI_OP_LUMA: u8 = 0b10000000;
const QOI_OP_LUMA_END: u8 = QOI_OP_LUMA | 0b00111111;

const QOI_OP_RGB: u8 = 0b11111110;
const QOI_OP_RGBA: u8 = 0b11111111;

fn decode<'a>(
    data: &'a (impl AsRef<[u8]> + ?Sized),
) -> Result<(Vec<u8>, u32, u32, bool, bool), String> {
    if data.as_ref().len() < QOI_HEADER_SIZE + QOI_FOOTER_SIZE {
        return Err(String::from("bytestream too short"));
    }
    let (header, body) = data.as_ref().split_at(14);
    let (mut body, _footer) = body.split_at(body.len() - QOI_FOOTER_SIZE);
    let (width, height, channels, colorspace) = try_decode_header(&header)?;
    let bytes_per_pixel = if channels { 4 } else { 3 };
    let mut out = Vec::with_capacity(width as usize * height as usize * bytes_per_pixel);
    let mut runner = Runner::new();
    let mut previous_pixel = Pixel::default();
    loop {
        let mut run = 1;
        match body {
            [QOI_OP_RGB, r, g, b, tail @ ..] => {
                previous_pixel = previous_pixel.update_rgb(*r, *g, *b);
                runner.update(previous_pixel);
                body = tail;
            }
            [QOI_OP_RGBA, r, g, b, a, tail @ ..] => {
                previous_pixel = previous_pixel.update_rgba(*r, *g, *b, *a);
                runner.update(previous_pixel);
                body = tail;
            }
            [byte @ QOI_OP_RUN..=QOI_OP_RUN_END, tail @ ..] => {
                run = 1 + (byte & !QOI_OP_RUN);
                body = tail;
            }
            [byte @ QOI_OP_DIFF..=QOI_OP_DIFF_END, tail @ ..] => {
                previous_pixel = previous_pixel.decode_diff(byte & !QOI_OP_RUN);
                runner.update(previous_pixel);
                body = tail;
            }
            [byte @ QOI_OP_INDEX..=QOI_OP_INDEX_END, tail @ ..] => {
                previous_pixel = runner.memory[(byte & !QOI_OP_RUN) as usize];
                runner.update(previous_pixel);
                body = tail;
            }
            [byte_1 @ QOI_OP_LUMA..=QOI_OP_LUMA_END, byte_2, tail @ ..] => {
                previous_pixel = previous_pixel.decode_luma_diff(byte_1 & !QOI_OP_RUN, *byte_2);
                runner.update(previous_pixel);
                body = tail;
            }
            [] => break,
            _ => panic!("invalid pattern"),
        }

        for _ in 0..run {
            previous_pixel.copy_to_vec(&mut out)
        }
    }

    Ok((out, width, height, channels, colorspace))
}

fn try_decode_header<'a>(data: &'a [u8]) -> Result<(u32, u32, bool, bool), String> {
    if data[..4] != QOI_MAGIC {
        return Err(String::from("magic is missing in header"));
    }
    let width = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let height = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let channels = data[12] != 0;
    let colorspace = data[13] != 0;
    return Ok((width, height, channels, colorspace));
}

fn encode<'a>(
    data: &'a (impl AsRef<[u8]> + ?Sized),
    width: usize,
    height: usize,
    has_alpha: bool,
    s_rgb: bool,
) -> Result<Vec<u8>, String> {
    let n_pixels = width * height;

    let mut previous_pixel = Pixel::default();
    let mut run_length: u8 = 0;
    let mut runner = Runner::new();
    let mut out = initialize(width, height, has_alpha, s_rgb);
    let chunksize = if has_alpha { 4 } else { 3 };
    for (i, pixel) in data
        .as_ref()
        .chunks_exact(chunksize)
        .map(Pixel::from)
        .enumerate()
    {
        if pixel == previous_pixel {
            run_length += 1;
            if run_length == 62 || i == n_pixels - 1 {
                out.push(QOI_OP_RUN | (run_length - 1));
                run_length = 0;
            }
        } else {
            if run_length != 0 {
                out.push(QOI_OP_RUN | (run_length - 1));
                run_length = 0;
            }
            if let Some(ix) = runner.match_or_update(&pixel) {
                out.push(QOI_OP_INDEX | ix);
            } else {
                let raw_diff = pixel - previous_pixel;

                if let Some(diff) = raw_diff.diff_offset() {
                    out.push(diff);
                } else if let Some(luma_diff) = raw_diff.luma_diff_offset() {
                    out.push(luma_diff.0);
                    out.push(luma_diff.1);
                } else {
                    if raw_diff.a == 0 {
                        out.push(QOI_OP_RGB);
                        out.push(pixel.r);
                        out.push(pixel.g);
                        out.push(pixel.b);
                    } else {
                        out.push(QOI_OP_RGBA);
                        out.push(pixel.r);
                        out.push(pixel.g);
                        out.push(pixel.b);
                        out.push(pixel.a);
                    }
                }
            }
            previous_pixel = pixel;
        }
    }
    Ok(finalize(out))
}

fn finalize(vec: Vec<u8>) -> Vec<u8> {
    [vec, [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8].to_vec()].concat()
}
fn initialize(width: usize, height: usize, has_alpha: bool, s_rgb: bool) -> Vec<u8> {
    let mut init = Vec::with_capacity(width * height);
    init.append(&mut encode_header(width, height, has_alpha, s_rgb).to_vec());
    init
}

fn encode_header(
    width: usize,
    height: usize,
    has_alpha: bool,
    s_rgb: bool,
) -> [u8; QOI_HEADER_SIZE] {
    let mut header = [0; QOI_HEADER_SIZE];
    header[..4].copy_from_slice(&QOI_MAGIC);
    header[4..8].copy_from_slice(&mut (width as u32).to_be_bytes());
    header[8..12].copy_from_slice(&mut (height as u32).to_be_bytes());
    header[12] = has_alpha as u8 + 3;
    header[13] = s_rgb.into();
    header
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Sub for Pixel {
    type Output = Pixel;
    fn sub(self, rhs: Self) -> Self::Output {
        Pixel {
            r: self.r.wrapping_sub(rhs.r),
            g: self.g.wrapping_sub(rhs.g),
            b: self.b.wrapping_sub(rhs.b),
            a: self.a.wrapping_sub(rhs.a),
        }
    }
}
impl Add for Pixel {
    type Output = Pixel;
    fn add(self, rhs: Self) -> Self::Output {
        Pixel {
            r: self.r.wrapping_add(rhs.r),
            g: self.g.wrapping_add(rhs.g),
            b: self.b.wrapping_add(rhs.b),
            a: self.a.wrapping_add(rhs.a),
        }
    }
}
impl Add<u8> for Pixel {
    type Output = Pixel;
    fn add(self, rhs: u8) -> Self::Output {
        Pixel {
            r: self.r.wrapping_add(rhs),
            g: self.g.wrapping_add(rhs),
            b: self.b.wrapping_add(rhs),
            a: self.a,
        }
    }
}

const LUMA_DIFF_OFFSET: Pixel = Pixel {
    r: 8,
    g: 32,
    b: 8,
    a: 0,
};

const DIFF_OFFSET: Pixel = Pixel {
    r: 2,
    g: 2,
    b: 2,
    a: 0,
};

impl Pixel {
    #[inline(always)]
    fn copy_to_vec(&self, vec: &mut Vec<u8>) {
        vec.push(self.r);
        vec.push(self.g);
        vec.push(self.b);
        vec.push(self.a);
    }

    #[inline(always)]
    fn decode_diff(self, data: u8) -> Pixel {
        self - DIFF_OFFSET + Pixel::from_diff(data)
    }

    #[inline(always)]
    fn decode_luma_diff(self, data_1: u8, data_2: u8) -> Pixel {
        const LAST_FOUR: u8 = 0b00001111;
        let dg = (data_1).wrapping_sub(32);
        let dg_8 = dg.wrapping_sub(8);
        Pixel {
            r: self.r.wrapping_add(dg_8).wrapping_add(data_2 >> 4),
            g: self.g.wrapping_add(dg),
            b: self.b.wrapping_add(dg_8).wrapping_add(data_2 & LAST_FOUR),
            a: self.a,
        }
    }

    fn luma_diff_offset(&self) -> Option<(u8, u8)> {
        let new = {
            let mut t = *self + LUMA_DIFF_OFFSET;
            t.r = t.r.wrapping_sub(self.g);
            t.b = t.b.wrapping_sub(self.g);
            t
        };
        if new.g | 63 == 63 && new.r | new.b | 15 == 15 {
            Some((QOI_OP_LUMA | new.g, new.r << 4 | new.b))
        } else {
            None
        }
    }
    fn diff_offset(&self) -> Option<u8> {
        let new = *self + DIFF_OFFSET;
        if new.r | new.g | new.b | 3 == 3 && new.a == 0 {
            Some(QOI_OP_DIFF | new.r << 4 | new.g << 2 | new.b)
        } else {
            None
        }
    }

    #[inline(always)]
    fn from_diff(data: u8) -> Pixel {
        const LAST_TWO: u8 = 0b00000011;
        Pixel {
            r: (data >> 4) & LAST_TWO,
            g: (data >> 2) & LAST_TWO,
            b: data & LAST_TWO,
            a: 0,
        }
    }
    fn zero() -> Pixel {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    fn update_rgb(&self, unwrap_1: u8, unwrap_2: u8, unwrap_3: u8) -> Pixel {
        Pixel {
            r: unwrap_1,
            g: unwrap_2,
            b: unwrap_3,
            a: self.a,
        }
    }

    fn update_rgba(&self, unwrap_1: u8, unwrap_2: u8, unwrap_3: u8, unwrap_4: u8) -> Pixel {
        Pixel {
            r: unwrap_1,
            g: unwrap_2,
            b: unwrap_3,
            a: unwrap_4,
        }
    }
}
impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}
impl From<&[u8]> for Pixel {
    fn from(data: &[u8]) -> Self {
        Pixel {
            r: data[0],
            g: data[1],
            b: data[2],
            a: if data.len() == 4 { data[3] } else { 0 },
        }
    }
}
struct Runner {
    memory: [Pixel; 64],
}

impl Runner {
    fn new() -> Self {
        Runner {
            memory: [Pixel::zero(); 64],
        }
    }

    #[inline]
    fn hash(pixel: &Pixel) -> u8 {
        pixel.r.wrapping_mul(3).wrapping_add(
            pixel.g.wrapping_mul(5).wrapping_add(
                pixel
                    .b
                    .wrapping_mul(7)
                    .wrapping_add(pixel.a.wrapping_mul(11)),
            ),
        ) % 64
        // ((3 * pixel.r + 5 * pixel.g + 7 * pixel.b + 11 * pixel.a) % 64).into()
    }

    #[inline]
    fn match_or_update(&mut self, pixel: &Pixel) -> Option<u8> {
        let hash = Runner::hash(pixel);
        if pixel == &self.memory[hash as usize] {
            Some(hash)
        } else {
            self.memory[hash as usize] = *pixel;
            None
        }
    }

    fn update(&mut self, pixel: Pixel) {
        let hash = Runner::hash(&pixel);
        self.memory[hash as usize] = pixel;
    }
}

fn main() {
    println!("Hello, world!");
    let encoded = encode(&[], 0, 0, false, false);
    println!("{:?}", encoded);
    let decoded = decode(&encoded.unwrap());
    println!("{:?}", decoded);

    println!("{}", QOI_OP_DIFF | 0b00010101);
    println!("{}", QOI_OP_DIFF | 0b00111111);
    println!("{}", QOI_OP_RUN | 0b00000000);
    println!(
        "{}",
        QOI_OP_INDEX
            | Runner::hash(&Pixel {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            })
    );
    println!("{:b}", 193u8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::io::Reader as ImageReader;
    #[test]
    fn test_encode_2x2() {
        let black: [u8; 4] = [0, 0, 0, 255];
        let white: [u8; 4] = [255, 255, 255, 255];
        let image = [[white, black], [black, white]].concat().concat();

        let expected = vec![
            113, 111, 105, 102, 0, 0, 0, 2, 0, 0, 0, 2, 4, 1, 85, 127, 192, 38, 0, 0, 0, 0, 0, 0,
            0, 1,
        ];

        let encoded = encode(&image, 2, 2, true, true);
        assert_eq!(encoded, Ok(expected));
    }
    #[test]
    fn test_encode_162_run() {
        let black: [u8; 4] = [0, 0, 0, 255];
        let len = 162;
        let image = black.repeat(len);

        let expected = vec![
            113,
            111,
            105,
            102,
            0,
            0,
            0,
            len as u8,
            0,
            0,
            0,
            1,
            4,
            1,
            QOI_OP_RUN | 61,
            QOI_OP_RUN | 61,
            QOI_OP_RUN | 37,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
        ];

        let encoded = encode(&image, len, 1, true, true);
        assert_eq!(encoded, Ok(expected));
    }
    #[test]
    fn test_decode_162_run() {
        let black: [u8; 4] = [0, 0, 0, 255];
        let len = 162;
        let image = black.repeat(len);

        let encoded = vec![
            113,
            111,
            105,
            102,
            0,
            0,
            0,
            len as u8,
            0,
            0,
            0,
            1,
            4,
            1,
            QOI_OP_RUN | 61,
            QOI_OP_RUN | 61,
            QOI_OP_RUN | 37,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
        ];
        let decoded = decode(&encoded);
        assert_eq!(decoded, Ok((image, len as u32, 1, true, true)));
    }

    #[test]
    fn test_encode_empty() {
        let empty_image_encoded = vec![
            113, 111, 105, 102, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 1,
        ];
        let encoded = encode(&[], 0, 0, true, true);
        assert_eq!(encoded, Ok(empty_image_encoded));
    }

    #[test]
    fn test_decode_empty() {
        let data = vec![
            113, 111, 105, 102, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 1,
        ];

        let decoded = decode(&data);
        assert_eq!(decoded, Ok((vec![], 0, 0, true, true)));
    }

    #[test]
    fn test_decode_2x2() {
        let data = vec![
            113, 111, 105, 102, 0, 0, 0, 2, 0, 0, 0, 2, 4, 1, 85, 127, 192, 38, 0, 0, 0, 0, 0, 0,
            0, 1,
        ];

        let black: [u8; 4] = [0, 0, 0, 255];
        let white: [u8; 4] = [255, 255, 255, 255];
        let expected = [[white, black], [black, white]].concat().concat();

        let decoded = decode(&data);
        assert_eq!(decoded, Ok((expected, 2, 2, true, true)));
    }

    #[test]
    fn test_encode_decode_empty() {
        let encoded = encode(&[], 0, 0, true, true);
        let decoded = decode(&encoded.unwrap());
        assert_eq!(decoded.unwrap().0, vec![]);
    }

    #[test]
    fn test_encode_decode_2x2() {
        let encoded = encode(&[], 0, 0, true, true);
        let decoded = decode(&encoded.unwrap());
        assert_eq!(decoded.unwrap().0, vec![]);
    }

    #[test]
    fn test_encode_decode_2x2_luma() {
        let black: [u8; 4] = [0, 0, 0, 255];
        let white: [u8; 4] = [255, 255, 255, 255];
        let grey: [u8; 4] = [9, 10, 11, 255];
        let image = [[white, black], [grey, white]].concat().concat();

        let encoded = encode(&image, 2, 2, true, true);
        let decoded = decode(&encoded.unwrap());
        assert_eq!(decoded.unwrap().0, image);
    }

    #[test]
    fn test_encode_decode_2x2_rgb() {
        let black: [u8; 4] = [0, 0, 0, 255];
        let red: [u8; 4] = [155, 0, 0, 255];
        let white: [u8; 4] = [255, 255, 255, 255];
        let grey: [u8; 4] = [10, 10, 10, 255];
        let image = [[red, black], [grey, white]].concat().concat();

        let encoded = encode(&image, 2, 2, true, true);
        let decoded = decode(&encoded.unwrap());
        assert_eq!(decoded.unwrap().0, image);
    }
    #[test]
    fn test_encode_decode_2x2_run() {
        let black: [u8; 4] = [0, 0, 0, 255];
        let white: [u8; 4] = [255, 255, 255, 255];
        let image = [[black, black], [white, white]].concat().concat();

        let encoded = encode(&image, 2, 2, true, true);
        let decoded = decode(&encoded.unwrap());
        assert_eq!(decoded.unwrap().0, image);
    }
    #[test]
    fn test_encode_decode_2x2_alpha() {
        let black: [u8; 4] = [0, 0, 0, 255];
        let white: [u8; 4] = [255, 255, 255, 255];
        let transparent: [u8; 4] = [0, 0, 0, 0];
        let image = [[white, black], [transparent, white]].concat().concat();

        let encoded = encode(&image, 2, 2, true, true);
        let decoded = decode(&encoded.unwrap());
        assert_eq!(decoded.unwrap().0, image);
    }

    #[test]
    fn test_encode_decode_go() {
        let img = ImageReader::open("../go.jpg")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();
        let data = img.to_vec();
        let width = img.width();
        let height = img.height();
        let has_alpha = true;
        let s_rgb = true;
        let encoded = encode(&data, width as usize, height as usize, has_alpha, s_rgb);
        let decoded = decode(&encoded.unwrap()).unwrap();
        assert_eq!(
            (decoded.1, decoded.2, decoded.3, decoded.4),
            (width, height, has_alpha, s_rgb)
        );
        assert_eq!(decoded.0.len(), data.len());
        println!(
            "{:?}",
            decoded
                .0
                .chunks_exact(4)
                .zip(data.chunks_exact(4))
                .enumerate()
                .filter(|(_, (l, r))| l != r)
                .next()
        );
        assert!(decoded.0.iter().eq(data.iter()), "data not the same");
    }
}

#[cfg(test)]
mod benches {
    use super::*;
    use image::io::Reader as ImageReader;
    use test::Bencher;

    #[bench]
    fn bench_encode_decode_go(b: &mut Bencher) {
        let img = ImageReader::open("../go.jpg")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();
        let data = img.to_vec();
        let width = img.width();
        let height = img.height();
        let has_alpha = true;
        let s_rgb = true;
        b.iter(|| {
            let encoded = encode(&data, width as usize, height as usize, has_alpha, s_rgb);
            let _decoded = decode(&encoded.unwrap()).unwrap();
        });
    }
    #[bench]
    fn bench_encode_go(b: &mut Bencher) {
        let img = ImageReader::open("../go.jpg")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();
        let data = img.to_vec();
        let width = img.width();
        let height = img.height();
        let has_alpha = true;
        let s_rgb = true;
        b.iter(|| {
            let _encoded = encode(&data, width as usize, height as usize, has_alpha, s_rgb);
        });
    }
    #[bench]
    fn bench_decode_go(b: &mut Bencher) {
        let img = ImageReader::open("../go.jpg")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();
        let data = img.to_vec();
        let width = img.width();
        let height = img.height();
        let has_alpha = true;
        let s_rgb = true;
        let encoded = encode(&data, width as usize, height as usize, has_alpha, s_rgb).unwrap();
        b.iter(|| {
            let _decoded = decode(&encoded).unwrap();
        });
    }
}
