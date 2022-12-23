package main

import (
	"bytes"
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
