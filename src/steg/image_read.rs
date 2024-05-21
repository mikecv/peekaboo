// Image pixel read methods.

use crate::steg::Steganography;

use image::{GenericImageView, Pixel};

// Method to read a certain number of bytes from an image.
impl Steganography {
    pub fn read_data_from_image(&mut self, bytes_to_read:u32) {

        // Initial loop counters.
        let mut bytes_read:u32 = 0;
        let mut row_cnt:u32 = self.row;
        let mut col_cnt:u32 = self.col;
        let mut col_plane:usize = self.plane;
        let mut _bits_read:u8 = self.bit;
        let mut _col_part:u8 = 0;
        let mut _code_data:u8 = 0;
        let mut _byte_bit:u8 = 0;
        let mut _mask:u8 = 0;

        // Initialise byte vector for read data.
        self.code_bytes = Vec::with_capacity(bytes_to_read as usize);

        // Initialise a colour bit mask.
        // This is so we can read an individual
        // bit in a pixel colour byte.
        _mask = 1 << _bits_read;

        // Loop while there are still bytes to read.
        while bytes_read < bytes_to_read {
            _code_data = 0;

            // Extract 1 byte of data from image.
            // one bit at a time.
            for _idx in 1..9 {
                // Get the pixel colour for the pixel we are at.
                if let Some(image) = &self.image {
                    if self.pic_col_planes == 3 { 
                        _col_part = image.get_pixel(col_cnt, row_cnt).to_rgb()[col_plane];
                    } else {
                        _col_part = image.get_pixel(col_cnt, row_cnt).to_rgba()[col_plane];
                    }
                }

                // Update the code data bit with the bit from the pixel.
                _byte_bit = _col_part & _mask;
                _byte_bit = _byte_bit >> _bits_read;
                _code_data = _code_data << 1;
                _code_data = _code_data | _byte_bit;

                // Next time around we need to point to the next pixel in the row.
                col_cnt = col_cnt + 1;
                // Until we get to the end of the row.
                // Then move to the start of the next row.
                if col_cnt == self.pic_width {
                    col_cnt = 0;
                    row_cnt = row_cnt + 1;
                    // If we have reached the end of the image then go
                    // back to the top and go to the next bit.
                    if row_cnt == self.pic_height {
                        row_cnt = 0;
                        col_plane = col_plane + 1;
                        // If we have processed the last plane (colour)
                        // ee go back to the next bit of the first plane,
                        if col_plane == 3 {
                            col_plane = 0;
                            _bits_read = _bits_read + 1;
                            _mask = _mask << 1;
                        }
                    }
                }
            }
            // Push the completed byte into the byte vector.
            self.code_bytes.push(_code_data);

            // Increment bytes read.
            bytes_read = bytes_read + 1;
        }

        // Save the state of the reading.
        // This allows us to carry on reading from where we
        // left off on the next chunk of reading.
        self.row = row_cnt;
        self.col = col_cnt;
        self.plane = col_plane;
        self.bit = _bits_read;
        self.bytes_read = bytes_read;
    }
}
