package main

import (
	"bytes"
	"image"
	_ "image/jpeg" // https://stackoverflow.com/a/39577526/5501815 // this is where I start hating Go
	"os"
	"testing"
)

// Go does not enforce formatting to the same level that Rust does, which is a missed opportunity.
// Eg I'm a bit sloppy with the camelCase/Snake_case and its all fine
// Also I made a ridiculously long line and it wasn't reformatted
func TestEncodeEmpty(t *testing.T) {

	var empty_image_encoded = []byte{
		113, 111, 105, 102, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 1,
	}
	var encoded = encode([]byte{}, 0, 0, true, true)
	if !bytes.Equal(encoded, empty_image_encoded) {
		t.Errorf("expected %d, got %d", empty_image_encoded, encoded)
	}
}

func TestEncode2x2(t *testing.T) {
	var black = []byte{0, 0, 0, 255}
	var white = []byte{255, 255, 255, 255}
	var image = append(append(white, black...), append(black, white...)...)

	var expected = []byte{
		113, 111, 105, 102, 0, 0, 0, 2, 0, 0, 0, 2, 4, 1, 85, 127, 192, 38, 0, 0, 0, 0, 0, 0,
		0, 1,
	}

	var encoded = encode(image, 2, 2, true, true)
	if !bytes.Equal(encoded, expected) {
		t.Errorf("expected \n%d, got \n%d", expected, encoded)

	}
}

func TestDecodeEmpty(t *testing.T) {
	var data = []byte{
		113, 111, 105, 102, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 1,
	}
	var expected = []byte{}
	var decoded, _, _, _, _, _ = decode(data)
	if !bytes.Equal(decoded, expected) {
		t.Errorf("expected %d, got %d", expected, decoded)
	}
}

func TestDecode_2x2(t *testing.T) {
	var data = []byte{
		113, 111, 105, 102, 0, 0, 0, 2, 0, 0, 0, 2, 4, 1, 85, 127, 192, 38, 0, 0, 0, 0, 0, 0,
		0, 1,
	}

	var black = []byte{0, 0, 0, 255}
	var white = []byte{255, 255, 255, 255}
	var image = append(append(white, black...), append(black, white...)...)

	var decoded, _, _, _, _, _ = decode(data)
	if !bytes.Equal(decoded, image) {
		t.Errorf("expected %d, got %d", image, decoded)

	}
}

func TestEncodeDecode_2x2_luma(t *testing.T) {

	var black = []byte{0, 0, 0, 255}
	var white = []byte{255, 255, 255, 255}
	var grey = []byte{9, 10, 11, 255}
	var image = append(append(white, black...), append(grey, white...)...)

	var encoded = encode(image, 2, 2, true, true)
	var decoded, _, _, _, _, _ = decode(encoded)
	if !bytes.Equal(decoded, image) {
		t.Errorf("expected %d, got %d", image, decoded)

	}
}

func TestEncodeDecode_2x2_rgb(t *testing.T) {

	var black = []byte{0, 0, 0, 255}
	var red = []byte{155, 0, 0, 255}
	var white = []byte{255, 255, 255, 255}
	var grey = []byte{10, 10, 10, 255}
	var image = append(append(red, black...), append(grey, white...)...)

	var encoded = encode(image, 2, 2, true, true)
	var decoded, _, _, _, _, _ = decode(encoded)
	if !bytes.Equal(decoded, image) {
		t.Errorf("expected %d, got %d", image, decoded)

	}
}
func TestEncodeDecode_2x2_run(t *testing.T) {

	var black = []byte{0, 0, 0, 255}
	var white = []byte{255, 255, 255, 255}
	var image = append(append(black, black...), append(white, white...)...)

	var encoded = encode(image, 2, 2, true, true)
	var decoded, _, _, _, _, _ = decode(encoded)
	if !bytes.Equal(decoded, image) {
		t.Errorf("expected %d, got %d", image, decoded)

	}
}
func TestEncodeDecode_2x2_alpha(t *testing.T) {

	var black = []byte{0, 0, 0, 255}
	var white = []byte{255, 255, 255, 255}
	var transparent = []byte{0, 0, 0, 0}
	var image = append(append(white, black...), append(transparent, white...)...)

	var encoded = encode(image, 2, 2, true, true)
	var decoded, _, _, _, _, _ = decode(encoded)
	if !bytes.Equal(decoded, image) {
		t.Errorf("expected %d, got %d", image, decoded)

	}
}

func TestEncode_decode_empty(t *testing.T) {
	var image = []byte{}
	var encoded = encode(image, 0, 0, true, true)
	var decoded, _, _, _, _, _ = decode(encoded)
	if !bytes.Equal(decoded, image) {
		t.Errorf("expected %d, got %d", image, decoded)

	}
}

func TestEncode_decode_2x2(t *testing.T) {
	var black = []byte{0, 0, 0, 255}
	var white = []byte{255, 255, 255, 255}
	var image = append(append(white, black...), append(black, white...)...)

	var encoded = encode(image, 0, 0, true, true)
	var decoded, _, _, _, _, _ = decode(encoded)
	if !bytes.Equal(decoded, image) {
		t.Errorf("expected %d, got %d", image, decoded)

	}
}

func getImage(filePath string) ([]byte, uint32, uint32, bool, bool, error) {

	f, err := os.Open(filePath)
	if err != nil {
		return nil, 0, 0, false, false, err
	}
	defer f.Close()
	img, _, err := image.Decode(f)
	if err != nil {
		return nil, 0, 0, false, false, err
	}
	// img = img.ColorModel().Convert(color.RGBAModel)
	var b = img.Bounds()
	var width = b.Dx()
	var height = b.Dy()
	var has_alpha = true
	var s_rgb = true
	var data = make([]byte, 0, width*height*4)
	for y := b.Min.Y; y < b.Max.Y; y++ {
		for x := b.Min.X; x < b.Max.X; x++ {
			var r, g, b, a = img.At(x, y).RGBA()
			data = append(data, byte(r), byte(g), byte(b), byte(a))
		}
	}

	return data, uint32(width), uint32(height), has_alpha, s_rgb, err
}
func BenchmarkEncodeDecodeGo(b *testing.B) {
	var data, width, height, has_alpha, s_rgb, err = getImage("../go.jpg")
	if err != nil {
		b.Errorf("failed to open go file, %s", err)
	}

	for i := 0; i < b.N; i++ {
		var encoded = encode(data, width, height, has_alpha, s_rgb)
		_, _, _, _, _, _ = decode(encoded)
		if err != nil {
			b.Error("failed to decode")
		}
	}
}
func BenchmarkEncodeGo(b *testing.B) {
	var data, width, height, has_alpha, s_rgb, err = getImage("../go.jpg")
	if err != nil {
		b.Errorf("failed to open go file, %s", err)
	}

	for i := 0; i < b.N; i++ {
		_ = encode(data, width, height, has_alpha, s_rgb)
		if err != nil {
			b.Error("failed to decode")
		}
	}
}
func BenchmarkDecodeGo(b *testing.B) {
	var data, width, height, has_alpha, s_rgb, err = getImage("../go.jpg")
	if err != nil {
		b.Errorf("failed to open go file, %s", err)
	}
	var encoded = encode(data, width, height, has_alpha, s_rgb)

	for i := 0; i < b.N; i++ {
		_, _, _, _, _, _ = decode(encoded)
		if err != nil {
			b.Error("failed to decode")
		}
	}
}
func TestEncodeDecodeGo(t *testing.T) {
	var data, width, height, has_alpha, s_rgb, err = getImage("../go.jpg")
	if err != nil {
		t.Errorf("failed to open go file, %s", err)
	}

	var encoded = encode(data, width, height, has_alpha, s_rgb)
	var decoded, d_width, d_height, d_has_alpha, d_s_rgb, err2 = decode(encoded)
	if err2 != nil {
		t.Error("failed to decode")
	}
	if !bytes.Equal(decoded, data) {
		t.Error("conversion failed")
	}
	if d_width != width || d_height != height || d_has_alpha != has_alpha || d_s_rgb != s_rgb {
		t.Error("image metadata got mangled during encode-decode roundtrip")
	}
}
func BenchmarkEncodeDecodeRust(b *testing.B) {
	var data, width, height, has_alpha, s_rgb, err = getImage("../rust.png")
	if err != nil {
		b.Errorf("failed to open go file, %s", err)
	}

	for i := 0; i < b.N; i++ {
		var encoded = encode(data, width, height, has_alpha, s_rgb)
		_, _, _, _, _, _ = decode(encoded)
		if err != nil {
			b.Error("failed to decode")
		}
	}
}
func TestEncodeDecodeRust(t *testing.T) {
	var data, width, height, has_alpha, s_rgb, err = getImage("../rust.png")
	if err != nil {
		t.Errorf("failed to open go file, %s", err)
	}

	var encoded = encode(data, width, height, has_alpha, s_rgb)
	var decoded, d_width, d_height, d_has_alpha, d_s_rgb, err2 = decode(encoded)
	if err2 != nil {
		t.Error("failed to decode")
	}
	if !bytes.Equal(decoded, data) {
		t.Error("conversion failed")
	}
	if d_width != width || d_height != height || d_has_alpha != has_alpha || d_s_rgb != s_rgb {
		t.Error("image metadata got mangled during encode-decode roundtrip")
	}
}
