# File structure
## Header
- 2 bytes (LE): width - 1
- 2 bytes (LE): height - 1
- Variable: frame contents

## Frame
- 1 bit: frame type (0 = P-frame, 1 = I-frame)
- 1 bit: initial color (0 = black, 1 = white)
- Variable: RLE Packets (color alternating)

## RLE Packet
[Same as Pokemon gen 1](https://www.youtube.com/watch?v=aF1Yw_wu2cM)

Let `x` = length of pixel stripe + 1
- Variable: MSB of `x` - 2
- Variable: `x` - (MSB of `x`)

### Decoding
1. Read until `0` bit.
2. Read (length of result in step 2) bits
3. Length is (result in step 1) + (result in step 2) + 1
