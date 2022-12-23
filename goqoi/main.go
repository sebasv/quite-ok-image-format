package main

import (
	"errors"
	"fmt"
)

const QOI_HEADER_SIZE = 14
const QOI_FOOTER_SIZE = 8
const QOI_MAGIC = "qoif" // Go does not have constant arrays, but it does have constant strings

// initially I forgot to type these as bytes. Because Go silently upcasts this could have led to a bug
const QOI_OP_RUN byte = 0b11000000
const QOI_OP_INDEX byte = 0b00000000
const QOI_OP_DIFF byte = 0b01000000
const QOI_OP_LUMA byte = 0b10000000

const QOI_OP_RGB byte = 0b11111110
const QOI_OP_RGBA byte = 0b11111111

type Pixel struct {
	r byte
	g byte
	b byte
	a byte
}

func default_pixel() Pixel {
	return Pixel{r: 0, g: 0, b: 0, a: 255}
}

func make_pixel_from_slice(s []byte, previous_alpha byte) Pixel {
	if len(s) == 4 {
		return Pixel{s[0], s[1], s[2], s[3]}
	} else {
		return Pixel{s[0], s[1], s[2], previous_alpha}

	}
}

// operator overloading is not a thing in Go
func (lhs Pixel) add(rhs Pixel) Pixel {
	return Pixel{
		r: lhs.r + rhs.r,
		g: lhs.g + rhs.g,
		b: lhs.b + rhs.b,
		a: lhs.a + rhs.a,
	}
}

func (lhs Pixel) sub(rhs Pixel) Pixel {
	return Pixel{
		r: lhs.r - rhs.r,
		g: lhs.g - rhs.g,
		b: lhs.b - rhs.b,
		a: lhs.a - rhs.a,
	}
}

func (p *Pixel) hash() byte {
	return (p.r*3 + p.g*5 + p.b*7 + p.a*11) % 64
}

func decode(data []byte) ([]byte, uint32, uint32, bool, bool, error) {
	if len(data) < QOI_FOOTER_SIZE+QOI_HEADER_SIZE {
		return []byte{}, 0, 0, false, false, errors.New("blep")
	}
	var width, height, has_alpha, s_rgb = decode_header(data[:QOI_HEADER_SIZE])
	var bytes_per_pixel uint32
	if has_alpha {
		bytes_per_pixel = 4
	} else {
		bytes_per_pixel = 3
	}
	var n = int(width * height * bytes_per_pixel)
	var out = make([]byte, 0, n)
	var previous_pixel = default_pixel()
	var lookup = make([]Pixel, 64)
	lookup[previous_pixel.hash()] = previous_pixel
	var i int = QOI_HEADER_SIZE
	for i < len(data)-QOI_FOOTER_SIZE {
		var b = data[i]
		var run int = 1
		if b == QOI_OP_RGB || b == QOI_OP_RGBA {
			previous_pixel = make_pixel_from_slice(data[i+1:i+2+int(3&b)], previous_pixel.a)
			lookup[previous_pixel.hash()] = previous_pixel
			i += int(bytes_per_pixel)
		} else {
			var header = b & QOI_OP_RUN
			var payload = b & ^QOI_OP_RUN
			switch header {
			case QOI_OP_RUN:
				run = int(payload) + 1
				i += 1
			case QOI_OP_INDEX:
				previous_pixel = lookup[payload]
				i += 1
			case QOI_OP_DIFF:
				previous_pixel = previous_pixel.add(Pixel{(payload >> 4 & 3) - 2, (payload >> 2 & 3) - 2, (payload & 3) - 2, 0})
				lookup[previous_pixel.hash()] = previous_pixel
				i += 1
			case QOI_OP_LUMA:
				var d_g = payload - 32
				var d_r = data[i+1]>>4 + d_g - 8
				var d_b = data[i+1]&15 + d_g - 8
				previous_pixel = previous_pixel.add(Pixel{d_r, d_g, d_b, 0})
				lookup[previous_pixel.hash()] = previous_pixel
				i += 2
			}
		}
		for j := 0; j < run; j++ {
			out = append(out, previous_pixel.r)
			out = append(out, previous_pixel.g)
			out = append(out, previous_pixel.b)
			if has_alpha {
				out = append(out, previous_pixel.a)
			}
		}
	}
	return out, width, height, has_alpha, s_rgb, nil
}

func decode_header(data []byte) (uint32, uint32, bool, bool) {
	var width, height uint32
	width = bytes_to_int(data[4:8])
	height = bytes_to_int(data[8:12])
	var has_alpha = byte_to_bool(data[12])
	var s_rgb = byte_to_bool(data[13])
	return width, height, has_alpha, s_rgb
}

func encode(data []byte, width uint32, height uint32, has_alpha bool, s_rgb bool) []byte {
	var previous_pixel = default_pixel()
	var n = len(data)
	var out = encode_header(width, height, has_alpha, s_rgb)
	var lookup = make([]Pixel, 64)
	lookup[previous_pixel.hash()] = previous_pixel
	var stride int
	if has_alpha {
		stride = 4
	} else {
		stride = 3
	}
	var run byte = 0
	for i := 0; i < n; i += stride {
		var pixel = make_pixel_from_slice(data[i:i+stride], previous_pixel.a)
		// why yes, comparison is automatically derived
		if pixel == previous_pixel {
			run += 1
			if run == 62 || i == n-1 {
				out = append(out, QOI_OP_RUN|(run-1))
				run = 0
			}
		} else {
			if run != 0 {
				out = append(out, QOI_OP_RUN|(run-1))
				run = 0
			}
			var index = pixel.hash()
			if lookup[index] == pixel {
				out = append(out, QOI_OP_INDEX|index)
			} else {
				lookup[index] = pixel
				var diff = pixel.sub(previous_pixel)
				var diff_biased = diff.add(Pixel{2, 2, 2, 0})
				if diff_biased.r|diff_biased.g|diff_biased.b|3 == 3 && diff_biased.a == 0 {
					out = append(out, QOI_OP_DIFF|(diff_biased.r<<4)|diff_biased.g<<2|diff_biased.b)
				} else {
					var luma_diff = diff
					luma_diff.r -= luma_diff.g
					luma_diff.b -= luma_diff.g
					luma_diff = luma_diff.add(Pixel{8, 32, 8, 0})
					if luma_diff.g < 64 && luma_diff.r|luma_diff.b|15 == 15 {
						out = append(out, QOI_OP_LUMA|luma_diff.g, luma_diff.r<<4|luma_diff.b)
					} else {
						if diff.a != 0 {
							out = append(out, QOI_OP_RGBA)
							out = append(out, pixel.r)
							out = append(out, pixel.g)
							out = append(out, pixel.b)
							out = append(out, pixel.a)
						} else {
							out = append(out, QOI_OP_RGB)
							out = append(out, pixel.r)
							out = append(out, pixel.g)
							out = append(out, pixel.b)
						}
					}
				}
			}

		}
		previous_pixel = pixel
	}
	if run != 0 {
		out = append(out, QOI_OP_RUN|(run-1))
		run = 0
	}
	return append(out, 0, 0, 0, 0, 0, 0, 0, 1)
}

func encode_header(width uint32, height uint32, has_alpha bool, s_rgb bool) []byte {
	var header = make([]byte, 0, QOI_HEADER_SIZE+QOI_FOOTER_SIZE+width*height)
	return append(append(append(append(header, QOI_MAGIC...), int_to_bytes(width)...), int_to_bytes(height)...), bool_to_byte(has_alpha)+3, bool_to_byte(s_rgb))
}

func byte_to_bool(b byte) bool {
	return b > 0
}
func bool_to_byte(b bool) byte {
	// bool is not a bit apparently, so we need to convert manually instead of upcasting.
	// Also there is no ternary operator in Go
	if b {
		return 1
	} else {
		return 0
	}
}
func bytes_to_int(b []byte) uint32 {
	return uint32(b[0])<<24 | uint32(b[1])<<16 | uint32(b[2])<<8 | uint32(b[3])
}
func int_to_bytes(i uint32) []byte {
	// Go drops the most significant bits silently when casting int-likes, so this should work
	// you would think i wanted to return a fixed-size array [4]byte, but that is not supported by `append`
	// Not that it matters in Go since fixed-size arrays are also heaped
	return []byte{byte(i >> 24), byte(i >> 16), byte(i >> 8), byte(i)}
}
func main() {
	fmt.Println("Hello, World!")
}
