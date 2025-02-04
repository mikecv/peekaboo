// Steganography data structure and methods.
//
// Steganography in this application is embedding files in lossess images,
// specifically in PNG format images.
// Supported formats are rgb and rgba colour formats, although only
// the rgb colour bytes are used to encode data into.
//
// A pic coded image contains a particular byte string embedded in the image.
// Here 'contains' implies embedded in the image colour bytes.
// The format of pic coded files is as follows:
//
// Pic coded signature : specific, but arbitray number of bytes.
// Password enabled : 1 byte, 'Y' or 'N'.
// If password enabled : 32 byte hash of password.
// Number of files embedded : (num_files_chars) digit integer, leading zeros.
// For each file section the following applies:
//
// File name length : (len_filename_chars) digit integer, leading zeros.
// File name : file name string in file name length bytes.
// File length in bytes : (file_len_chars) digit integer, leading zeros.
// File contents : file bytes in file length bytes.

pub mod image_read;
pub mod image_write;

extern crate image;
extern crate ring;

use log::{error, info, warn, debug};
use image::{DynamicImage, GenericImageView};
use ring::digest;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::{Instant, Duration};

use crate::settings::Settings;
use crate::SETTINGS;

// Define program code here so not exposed in settings file.
const PROG_CODE : &str = "PICCODER";

// Error result enum.
#[derive(Debug)]
pub enum SteganographyError {
    IncorrectPassword,
}

// Display of Steganography specific errors.
impl fmt::Display for SteganographyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SteganographyError::IncorrectPassword => write!(f, "Incorrect password provided"),
        }
    }
}

impl std::error::Error for SteganographyError {}

// Struct to hold details about files embedded in an image.
// This struct will be included in the main Steganography struct.
pub struct EmbeddedFile {
    pub file_name: String,
    pub file_type: String,
    pub file_extracted: bool,
    pub file_coded: bool,
    pub file_analysed: bool,
}

// Struct of parameters for embedd file and
// for file to be embedded.
pub struct Steganography {
    pub settings: Settings,
    pub img_to_proc: bool,
    pub img_proc_running: bool,
    pub image_file: String,
    pub image: Option<DynamicImage>,
    pub pic_coded: bool,
    pub user_permit: bool,
    pub pic_has_pw: bool,
    pub pic_code_name_len: u8,
    pub pic_width: u32,
    pub pic_height: u32,
    pub pic_col_planes: u8,
    pub row: u32,
    pub col: u32,
    pub plane: usize,
    pub bit: u8,
    pub bytes_read: u32,
    pub code_bytes: Vec<u8>,
    pub overhead_per_file: u16,
    pub embed_capacity: u64,
    pub load_duration: Duration,
    pub extract_duration: Duration,
    pub embed_duration: Duration,
    pub embedded_files: Vec<EmbeddedFile>,
    pub retry_extract: bool,
}

// Initialise all struct variables.
// This method called at the start.
impl Steganography {
    pub fn init() -> Self {
        info!("Initialising Steganography struct.");

        // Lock the global SETTINGS to obtain access to the Settings object.
        let settings = SETTINGS.lock().unwrap().clone();

        Steganography {
            settings,
            img_to_proc: false,
            img_proc_running: false,
            image_file: String::from(""),
            image: None,
            pic_coded: false,
            user_permit: false,
            pic_has_pw: false,
            pic_code_name_len: 0,
            pic_width: 0,
            pic_height: 0,
            pic_col_planes: 0,
            row: 0,
            col: 0,
            plane: 0,
            bit: 0,
            bytes_read: 0,
            code_bytes: Vec::with_capacity(0),
            overhead_per_file: 0,
            embed_capacity: 0,
            load_duration: Duration::new(0, 0),
            extract_duration: Duration::new(0, 0),
            embed_duration: Duration::new(0, 0),
            embedded_files: Vec::new(),
            retry_extract: false,
        }
    }
}

// Initialise struct for image loaded properties.
impl Steganography {
    pub fn init_image_params(&mut self) {
        info!("Initialising load image file parameters.");
        self.image_file = String::from("");
        self.image = None;
        self.img_to_proc = false;
        self.pic_coded = false;
        self.user_permit = false;
        self.pic_has_pw = false;
        self.pic_code_name_len = 0;
        self.pic_width = 0;
        self.pic_height = 0;
        self.pic_col_planes = 0;
        self.embed_capacity = 0;
        self.retry_extract = false;
    }
}

// Initialise struct for reading and writing
// embedded files.
impl Steganography {
    pub fn init_embed_params(&mut self) {
        info!("Initialising embedded file parameters.");
        self.row = 0;
        self.col = 0;
        self.plane = 0;
        self.bit = 0;
        self.bytes_read = 0;
    }
}

// Method to load a brand new image for analysis.
impl Steganography {
    pub fn load_new_file(&mut self, in_file:String) {
        // Initialise timer for function.
        let load_start = Instant::now();

        // Do image intialisatioins to clean up after any
        // successful or failed image loading.
        // That is, parameters for loaded and imbedded image.
        self.init_image_params();
        self.init_embed_params();

        // Several checks along the way so status
        // to keep progress along the way.
        let mut cont_ckh: bool = true;

        // Create path to image.
        let mut img_path = PathBuf::new();
        img_path.push(in_file.clone());
        let img_path_string = img_path.to_string_lossy().into_owned();
        self.image_file = img_path_string;

        let img_result = image::open(&img_path);
        // Handle exceptions, specific file not found, and generic.
        let _img = match img_result {
            Ok(_img) => {
                // Set flag to indicate we have an image to process.
                self.img_to_proc = true;
                self.image = Some(_img.clone());
                _img
            }
            Err(err) => {
                // Set flag indicating that there was an issue opening the file.
                // So we don't have to continue after this.
                cont_ckh = false;
                match err {
                    // File not found error.
                    image::ImageError::IoError(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {

                        warn!("Warning file not found: {}", in_file.clone());
                        // Return a placeholder image.
                        image::DynamicImage::new_rgb8(1, 1)
                    }
                    // Generic exception.
                    _ => {
                        error!("Error openning image file: {}", in_file.clone());
                        // Return a placeholder image.
                        image::DynamicImage::new_rgb8(1, 1)
                    }
                }
            }
        };

        // If we have an image file open, then read the parameters.
        // Need to check if 3 colour planes as well.
        if cont_ckh == true {
            if let Some(image) = &self.image {
                // Get image width and height
                (self.pic_width, self.pic_height) = image.dimensions();
                info!("Image loaded with width: {}, height: {}", self.pic_width, self.pic_height);

                // Need to check if colour format is acceptable.
                // Need 3 colour planes.
                let cols = image.color();
                match cols {
                    // Even though only writing to rgb planes for now,
                    // Need to keep track if there is a transparency layer.
                    image::ColorType::Rgb8 => {
                        // Store number of colour planes
                        self.pic_col_planes = 3;
                        info!("Image loaded with colour planes: {}", self.pic_col_planes);
                    }
                    image::ColorType::Rgba8 => {
                        // Store number of colour planes
                        self.pic_col_planes = 4;
                        info!("Image loaded with colour planes: {}", self.pic_col_planes);
                    }
                    _ => {
                        // Unsupported image colour type
                        info!("Image not a supported rgb colour type.");
                    }
                }
            }
            else {
                error!("Image is of None type");
            }
        }

        // Calculate the available space for storage.
        // Basically how many bits get used when embeddng files
        // in an image.
        // Here capacity is in bytes.
        if cont_ckh == true {
            let _embed_bytes: u32 = self.pic_width * self.pic_height * self.pic_col_planes as u32;
            self.embed_capacity = _embed_bytes as u64;

            info!("Absolute host file capacity (bytes): {}", self.embed_capacity);

            // There is a fixed amount of capacity that must be reserved.
            // Allocation for pic code preamble.
            // Allocation (pw_protected_chars) byte for is password protected
            // Allocation (pw_chars) bytes for password
            // Allocation (num_files_chars) bytes for number of files
            self.embed_capacity = self.embed_capacity - PROG_CODE.len() as u64 - 36;
            self.embed_capacity -= PROG_CODE.len() as u64;
            self.embed_capacity -= self.settings.pw_protected_chars as u64;
            self.embed_capacity -= self.settings.pw_chars as u64;
            self.embed_capacity -= self.settings.num_files_chars as u64;
            info!("Embedding capacity (bytes): {}", self.embed_capacity);

            // There is also an overhead per file to cover the file name and size etc.
            // Need to account for this when embedding.
            // File name length (len_filename_chars) leading zeros. Assume worse case.
            // File name : file name string in file name length bytes.
            // File length in bytes : leading zeros.
            self.overhead_per_file = self.settings.len_filename_chars as u16;
            self.overhead_per_file += u16::pow(10, self.settings.len_filename_chars as u32);
            self.overhead_per_file += self.settings.file_len_chars as u16;
        }

        // Check if the file is already pic coded.
        self.check_for_code();
        if self.pic_coded == true {
            info!("Image file contains preamble code.");

            // Now that we know that the image is pic coded,
            // we can see if there is a password encoded in the image.
            // Password yes (Y) or no (N) is in the next 1 byte.
            self.check_for_password();

            // If password protected can't go further, until the user
            // gives a valid password.
            if self.pic_has_pw == false {
                // If embedded image is not password protected
                // we can continue.
                info!("Files embedded WITHOUT password.")
            }
            else {
                info!("Files embedded WITH password.")
            }
        }

        // Determine delta time for function.
        self.load_duration = load_start.elapsed();
        info!("Time for upload: {:?}", self.load_duration)
    }
}

// Method to check if image has been previously encoded,
// that is, it contains the preamble code.
impl Steganography {
    pub fn check_for_code(&mut self) {
        // First check if file is even large enough to hold a code.
        // Can do this by checking emdedding capacity.
        if self.embed_capacity < 1 {
            warn!("Capacity less than minimum for coding (bytes): {}", self.embed_capacity);
            self.pic_coded = false;
            return;
        }

        // File large enough to hold preamble code.
        // Extract data from image and match with code.
        // Read number of bytes for the pic code.
        let bytes_to_read:u32 = PROG_CODE.len().try_into().unwrap();
        self.read_data_from_image(bytes_to_read);
        if self.bytes_read != bytes_to_read {
            error!("Expected bytes: {}, bytes read: {}", bytes_to_read, self.bytes_read);
            info!("Image file is not pic coded.");  
            self.pic_coded = false;
            return;
        }
        else {
            // Compare the byte array read with the pic coded array (string).
            let string_result = String::from_utf8((&*self.code_bytes).to_vec());
            match string_result {
                Ok(string) => {
                    // String read so need to see if it matches the code.
                    if string == PROG_CODE {
                        self.pic_coded = true;
                        info!("Image is pic coded.");
                    }
                    else {
                        self.pic_coded = false;
                        info!("Image is not pic coded.");
                    }
                }
                _ => {
                    self.pic_coded = false;
                    info!("Image is not pic coded.");
                }
            }
        }
    }
}

// Method to check if image has a password.
impl Steganography {
    pub fn check_for_password(&mut self) {

        // Read number of bytes for whether or not there is a password.
        let bytes_to_read:u32 = self.settings.pw_protected_chars as u32;
        self.read_data_from_image(bytes_to_read);
        if self.bytes_read != bytes_to_read {
            error!("Expected bytes: {}, bytes read: {}", bytes_to_read, self.bytes_read);
            info!("Image does not include a password.");  
            self.user_permit = false;
            return;
        }
        else {
            // Check for Y(es) or N(o) re password.
            let string_result = String::from_utf8((&*self.code_bytes).to_vec());
            match string_result {
                Ok(string) => {
                    // String read so need to see if it is Y or N.
                    if string == "Y" {
                        self.pic_has_pw = true;
                        info!("Image includes a password.");
                    }
                }
                _ => {
                    self.pic_has_pw = false;
                    info!("Image does not include a password.");
                }
            }
        }
    }
}

// Method to extract data from file.
// Password string required, empty string if no
// password required.
impl Steganography {
    pub fn extract_data(&mut self, pw:String) -> Result<(), SteganographyError> {
        // Initialise timer for function.
        let extract_start = Instant::now();

        // Initialise embedded files prior to starting.
        self.embedded_files = Vec::new();

        // If retrying extraction then have to reset read position.
        // Here retries occur if wrong password entered.
        if self.retry_extract == true {
            // Reset to start of file
            self.init_embed_params();

            // Need to point to password location.
            // Offset by previous file lengths.
            let pw_offset:u32 = PROG_CODE.len() as u32 + self.settings.pw_protected_chars as u32;
            self.read_data_from_image(pw_offset);
        }

        // If password required then check it.
        if self.pic_has_pw == true {
            // Password required, so check password provided.
            self.check_valid_password(pw);
            if self.user_permit == true {
                info!("Correct password provided.");

                self.retry_extract = false;

            }
            else {
                // Determine delta time for function, albeit failed.
                self.extract_duration = extract_start.elapsed();
                info!("Time for file(s) extraction: {:?}", self.extract_duration);
                info!("Correct password NOT provided.");

                self.retry_extract = true;

                return Err(SteganographyError::IncorrectPassword);
            }
        }
        // Either password not required or correct password entered.
        // Either way we can proceed with extracting data.
        self.get_embedded_data();

        // Determine delta time for function.
        self.extract_duration = extract_start.elapsed();
        info!("Time for file(s) extraction: {:?}", self.extract_duration);

        Ok(())
    }
}

// Method to check user's password entry.
impl Steganography {
    pub fn check_valid_password(&mut self, password: String) {
        // Before checking the password we have to get the
        // hashed password stored in the image.
        // The password is a SHA-256 so always 32 bytes long.
        let bytes_to_read:u32 = self.settings.pw_chars as u32;
        self.read_data_from_image(bytes_to_read);
        if self.bytes_read != bytes_to_read {
            error!("Expected bytes: {}, bytes read: {}", bytes_to_read, self.bytes_read);
            self.user_permit = false;
            return;
        }
        else {
            // Check password against hash of user entry.
            match digest::digest(&digest::SHA256, password.as_bytes()).as_ref() == &self.code_bytes[..] {
                true => {
                    self.user_permit = true;
                    info!("User entered password matches.");
                }
                false => {
                    self.user_permit = false;
                    info!("User entered password does not match.");
                }
            }
        }
    }
}

// Method to get embedded data from the image.
impl Steganography {
    pub fn get_embedded_data(&mut self) {

        // First get the number of files embedded.
        let bytes_to_read:u32 = self.settings.len_filename_chars as u32;
        self.read_data_from_image(bytes_to_read);
        if self.bytes_read != bytes_to_read {
            error!("Expected bytes: {}, bytes read: {}", bytes_to_read, self.bytes_read);
            return;
        }
        else {
            let _string_result = String::from_utf8((&*self.code_bytes).to_vec());
            match _string_result {
                Ok(string) => {
                    let num_files:u8 = string.parse().unwrap();
                    info!("Number of embedded files: {}", num_files);

                    // Let's process each embedded file, one by one.
                    for _idx in 1..= num_files {

                        // First get the length of the file name.
                        let bytes_to_read:u32 = self.settings.num_files_chars as u32;
                        self.read_data_from_image(bytes_to_read);
                        if self.bytes_read != bytes_to_read {
                            error!("Expected bytes: {}, bytes read: {}", bytes_to_read, self.bytes_read);
                            return;
                        }
                        else {
                            let _string_result = String::from_utf8((&*self.code_bytes).to_vec());
                            match _string_result {
                                Ok(string) => {
                                    let file_name_len:u8 = string.parse().unwrap();
                                    info!("File name length: {}", file_name_len);

                                    // Now that we have the length of the file name we can extract it.
                                    let bytes_to_read:u32 = file_name_len as u32;
                                    self.read_data_from_image(bytes_to_read);
                                    if self.bytes_read != bytes_to_read {
                                        error!("Expected bytes: {}, bytes read: {}", bytes_to_read, self.bytes_read);
                                        return;
                                    }
                                    else {
                                        let _string_result = String::from_utf8((&*self.code_bytes).to_vec());
                                        match _string_result {
                                            Ok(string) => {
                                                let file_name:String = string;
                                                info!("Embedded file name: {}", file_name);

                                                // Now we need to get the length of the file.
                                                let bytes_to_read:u32 = self.settings.file_len_chars as u32;
                                                self.read_data_from_image(bytes_to_read);
                                                if self.bytes_read != bytes_to_read {
                                                    error!("Expected bytes: {}, bytes read: {}", bytes_to_read, self.bytes_read);
                                                    return;
                                                }
                                                else {
                                                    let _string_result = String::from_utf8((&*self.code_bytes).to_vec());
                                                    match _string_result {
                                                        Ok(string) => {
                                                            let file_len:u32 = string.parse().unwrap();
                                                            info!("File length: {}", file_len);

                                                            // Now we have all the file details, we can
                                                            // read the data from the image and construct
                                                            // the file.
                                                            let _ = self.extract_file(file_len, file_name);
                                                        }
                                                        _ => {
                                                            warn!("Invalid file length.");
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                warn!("Invalid file name.");
                                            }
                                        }
                                    }                       
                                }
                                _ => {
                                    warn!("Invalid file name length.");
                                }
                            }
                        }
                    }
                    
                    // Go through extracted files and check if embedded.
                    // First, collect the file names of embedded PNGs for analysis.
                    let png_file_names: Vec<String> = self.embedded_files
                        .iter()
                        .filter(|file| file.file_type == "image/png")
                        .map(|file| file.file_name.clone())
                        .collect();

                    // Create a HashMap to store the coded status of each file by its file name.
                    // This is so we can update the class veriable for embedded_files.
                    let mut file_coded_map: HashMap<String, bool> = HashMap::new();

                    // Perform analysis on collected file names.
                    for file_name in png_file_names {

                        // Clone the file name before passing to avoid moving it
                        self.load_new_file(file_name.clone());

                        // Update hash map of embeded status of files.
                        let coded_status = self.pic_coded;
                        file_coded_map.insert(file_name, coded_status);
                    }

                    // Now that the analysis is done, mutably iterate over embedded_files to set file_coded
                    // status in the class variable returned to main.
                    for file in &mut self.embedded_files {
                        // Check if the file was in the analysis result and set the file_coded attribute.
                        if let Some(&coded_status) = file_coded_map.get(&file.file_name) {
                            file.file_coded = coded_status; // Set the coded status for the file.
                            debug!("Updated file_coded status for: {:?} to {:?}", file.file_name, file.file_coded);
                        }
                    }
                }
                _ => {
                    warn!("Invalid number of files length.");
                }
            }
        }
    }
}

// Method to extract a file from the image,
// and save it to file.
impl Steganography {
    pub fn extract_file(&mut self, file_size:u32, file_name:String) -> io::Result<()> {
        info!("Extracting file of size: {}.", file_size);

        // Now the file data in the image needs to be written to a
        // file.
        // Will do this by reading chunks of data from the image at a time,
        // and appending chunks to the file.
        // When the file is complete save the file.

        // Check if folder for storing embedded files exists.
        // If it doesn't exist, create it.
        fs::create_dir_all(&self.settings.secret_folder)?;

        // We need to remove the path from the filename,
        // as we are not interested in the original path.
        let path = Path::new(&file_name);
        let raw_filename = path.file_name().unwrap();

        // Get file path for the file to be written.
        // All files will be written to a specific folder.
        let mut wrt_path = PathBuf::new();       
        wrt_path.push(&self.settings.secret_folder);
        wrt_path.push(raw_filename);
        let mut wrt_path_string = wrt_path.to_string_lossy().into_owned();

        // Check if we are going to overwrite an existing file.
        // If so we will add a suffix to the end of the file name
        // to make it unique.
        let mut suffix = 1;
        let original_filename = wrt_path_string.clone();
        while Path::new(&wrt_path_string).exists() {
            // Construct next suffix.
            let extension = match original_filename.rfind('.') {
                Some(idx) => &original_filename[idx..],
                None => "",
            };
            // Construct base file path.
            let base_filename = if let Some(idx) = original_filename.rfind('.') {
                &original_filename[..idx]
            } else {
                &original_filename
            };
            // Construct complete file name.
            wrt_path_string = format!("{}-{:03}{}", base_filename, suffix, extension);
            // Increment suffix if this file name exists.
            suffix += 1;
        }

        // Open the file for writing.
        info!("Opening file for writing: {}", wrt_path_string.clone());
        let mut file = File::create(&wrt_path_string)?;

        // Keep track of bytes left to write.
        let mut bytes_remaining:u32 = file_size;

        // Chunk of bytes to read each tiem.
        // Except maybe the last time when likely to be less.
        let mut bytes_to_read = self.settings.byte_chunk;

        // Keep reading data from image until file read in full.
        while bytes_remaining > 0 {
            // Check if we have read a full or part chunk.
            if bytes_remaining < self.settings.byte_chunk {
                bytes_to_read = bytes_remaining;
            }

            // Read a chunk of bytes from the image.
            self.read_data_from_image(bytes_to_read);
            if self.bytes_read != bytes_to_read {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Incorrect number of bytes read: {}", self.bytes_read),
                ));
            } else {
                // Write bytes read to the file.
                file.write_all(&self.code_bytes)?;

                // Update the number of bytes remaining to read.
                bytes_remaining = bytes_remaining - self.bytes_read as u32;
            }
        }

        // File writing completed, so save and close the file.
        // No need to manually close as the file will be closed when it goes out of scope.
        debug!("Data written to file successfully.");

        // From file extention fix the file type.
        // Use in front end UI for displaying file thumbnails.
        let file_extension = Path::new(&wrt_path_string)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let mime_type = get_mime_type(file_extension);
        debug!("Data file of mime type: {:?}" , mime_type);

        // Push the filename onto the vector array so that we have a list of all
        // files written.
        let file_details = EmbeddedFile {
            file_name : String::from(&wrt_path_string),
            file_type: String::from(mime_type),
            file_extracted : true,
            file_coded : false,
            file_analysed : true,
        };
        self.embedded_files.push(file_details);
        Ok(())
    }
}

// Helper function to map file extensions to MIME types.
// Used by front end when displaying thumbnails of extracted images.
fn get_mime_type(extension: &str) -> &str {
    match extension {
        "txt" => "text/plain",
        "html" => "text/html",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "pdf" => "application/pdf",
        "gz" | "tar.gz" => "application/x-tar",
        // Add other extensions and their MIME types as needed.
        _ => "application/octet-stream",
    }
}

// Method to embed one or more files into a loaded image.
impl Steganography {
    pub fn embed_files(&mut self, pw:bool, pw_str:&str, files_to_embed:&[&str]) -> io::Result<()> {
        // Initialise timer for function.
        let embed_start = Instant::now();

        // Don't need to initialise image parameters as we require
        // a loaded image to embed files into.
        if self.img_to_proc == true {
            // We have an image to embed into so all good.
            // It doesn't matter if the image is already pic coded as we
            // will just overwrite the previous embedding.
            // We should also alaready know the embedding width, height,
            // and embedding capacity of the image.

            // First check to see if there is space for the file(s) requested.
            let mut bytes_to_embed = 0;
            for file in files_to_embed {
                // Need to get sum of file lengths to embed.
                let metadata = fs::metadata(file)?;
                let file_size = metadata.len();
                bytes_to_embed = bytes_to_embed + file_size;
                info!("File: {} Size: {} bytes", file, file_size);
            }
            // Need to compare bytes to embed with image capacity.
            // Ignoring size of file names as not likely to be significant.
            if bytes_to_embed > self.embed_capacity {
                // Exceeded embedding capacity so can't imbed.
                warn!("Exceeded image emdedding: {}", self.embed_capacity)
            }
            else {
                // Within the embedding capacity of the image, so proceed.
                info!("Total data to embed: {} bytes", bytes_to_embed);

                // First step is to write the preamble to the file.
                self.embed_preamble();

                // Next we need to embed a password if required.
                self.embed_password(pw, pw_str);

                // Next need to embed the number of files we are embedding.
                let num_files:u16 = files_to_embed.len() as u16;
                self.embed_num_of_files(num_files);

                // Next need to embed files themselves, one at a time.
                for file in files_to_embed {
                    // Need to embed the file.
                    // This also means embeddng the name of the file,
                    // and the length of the file.
                    if let Err(err) = self.embed_file(file) {
                        eprintln!("Error: {}", err); 
                    }
                    else {
                        info!("Successfully embedded file: {}", file);
                    }
                }
            }

            // Determine delta time for function.
            self.embed_duration = embed_start.elapsed();
            info!("Time to embed file(s): {:?}", self.embed_duration);

            Ok(())
        }
        else {
            info!("No files to process.");

            // Determine delta time for function.
            self.embed_duration = embed_start.elapsed();
            info!("Time to NOT embed any file: {:?}", self.embed_duration);
 
            Ok(())
        }
    }
}

// Method to add the preable code to the image.
impl Steganography {
    pub fn embed_preamble(&mut self) {
        info!("Embedding preamble into image.");

        // Initialise embedding parameters.
        // Reset before preabmle, NEVER after the preamble
        // else will overwrite early data.
        self.init_embed_params();

        // Send preamble as bytes vector for embedding.
        // All writes to the image is done in chunks.
        let preamble_string = PROG_CODE;
        let preamble_bytes = preamble_string.as_bytes();
        for chunk in preamble_bytes.chunks(self.settings.byte_chunk.try_into().unwrap()) {
            let bytes_written:u32 = self.write_data_to_image(chunk);
            if bytes_written != chunk.len() as u32{
                error!("Incorrect number of bytes written: {}", bytes_written)
            }
        }
    }
}

// Method to embed password (if required) to the image.
impl Steganography {
    pub fn embed_password(&mut self, _pw:bool, _pw_str:&str) {
        info!("Embedding whether passworded or not.");

        // Send pasword as applicable as bytes vector for embedding.
        // All writes to the image is done in chunks.
        if _pw == false {
            let have_pw_str = String::from("N");
            let have_pw_bytes = have_pw_str.as_bytes();
            for chunk in have_pw_bytes.chunks(self.settings.byte_chunk.try_into().unwrap()) {
                let bytes_written:u32 = self.write_data_to_image(chunk);
                if bytes_written != chunk.len() as u32{
                    error!("Incorrect number of bytes written: {}", bytes_written)
                }
            }
        }
        else {
            // We have a password to embed.
            // First we need to get the hash of the password to embed.
            info!("Embedding passworded.");
            // First the tag that there is a password.
            let have_pw_str = String::from("Y");
            let have_pw_bytes = have_pw_str.as_bytes();
            // Next we have the hashed password.
            let digest = digest::digest(&digest::SHA256, _pw_str.as_bytes());
            let hashed_password = digest.as_ref();
            let password_bytes = hashed_password;
            // Concatenate the two.
            let pw_bytes:Vec<u8> = [have_pw_bytes, password_bytes].concat();
            // Embed into image.
            for chunk in pw_bytes.chunks(self.settings.byte_chunk.try_into().unwrap()) {
                let bytes_written:u32 = self.write_data_to_image(chunk);
                if bytes_written != chunk.len() as u32{
                    error!("Incorrect number of bytes written: {}", bytes_written)
                }
            }
        }
    }
}

// Method to embed the number of files being embedded.
impl Steganography {
    pub fn embed_num_of_files(&mut self, num_files:u16) {
        info!("Embedding number of files: {}", num_files);

        // Get the number of files as a string with leading 0s.
        let _num_files:String = format!("{:0>width$}", num_files, width=self.settings.num_files_chars as usize);
        let num_file_bytes = _num_files.as_bytes();

        // Embed into image.
        for chunk in num_file_bytes.chunks(self.settings.byte_chunk.try_into().unwrap()) {
            let bytes_written:u32 = self.write_data_to_image(chunk);
            if bytes_written != chunk.len() as u32{
                error!("Incorrect number of bytes written: {}", bytes_written)
            }
        }
    }
}

// Method to embed the contents of a file into the image.
impl Steganography {
    pub fn embed_file(&mut self, file_path:&str) -> io::Result<()> {
        info!("Embedding file: {}", file_path);

        // Need to get the filename to give the file,
        // and the length of this filename, as both are embedded.
        // File name, and filename length.
        let _file_path = Path::new(file_path);
        // Extract file name.
        let _file_name = _file_path.file_name().unwrap();
        let _file_name_bytes = _file_name.as_encoded_bytes();
        // Determine filename length.
        // And format to 3 digits, with leading 0s.
        let _file_name_len = _file_name.len() as u8;
        let _file_name_len_str:String = format!("{:0>width$}", _file_name_len, width=self.settings.len_filename_chars as usize);
        let _file_name_len_bytes = _file_name_len_str.as_bytes();
        // Determine file length in bytes.
        // Format to 10 digits, with leading 0s.
        let _metadata = fs::metadata(file_path)?;
        let _file_size = _metadata.len();
        let _file_size_str:String = format!("{:0>width$}", _file_size, width=self.settings.file_len_chars as usize);
        let _file_size_bytes = _file_size_str.as_bytes();

        // Concatenate file details for embedding.
        let file_detail_bytes:Vec<u8> = [_file_name_len_bytes, _file_name_bytes, _file_size_bytes].concat();
        // Embed into image.
        for chunk in file_detail_bytes.chunks(self.settings.byte_chunk.try_into().unwrap()) {
            let bytes_written:u32 = self.write_data_to_image(chunk);
            if bytes_written != chunk.len() as u32{
                error!("Incorrect number of bytes written: {}", bytes_written)
            }
        }

        // Now the file needs to be written to the image.
        // Will do this by reading chunks of data from the file at a time,
        // and writing the data to the image, until the file is done.

        // Open the file for reading.
        let mut file = File::open(file_path)?;

        // Define a buffer to use for the chunks of read data.
        let mut buffer = vec![0u8; self.settings.byte_chunk as usize];

        // Loop until there are no bytes in the file to write.
        loop {
            // Read a chunk of data from the file.
            let bytes_read = file.read(&mut buffer)?;

            // If no bytes were read, we've reached the end of the file.
            if bytes_read == 0 {
                break;
            }

            // Write the chunk of data to the image.
            let bytes_written = self.write_data_to_image(&buffer[..bytes_read]);

            // Check that the correct number of bytes were written.
            if bytes_written != bytes_read as u32 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Incorrect number of bytes written: {}", bytes_written),
                ));
            }
        }
        // Return ok result.
        info!("File data written to image successfully.");
        Ok(())
    }
}

// Method to save image with name.
// Will overwrite the existing image if no file specified.
impl Steganography {
    pub fn save_image(&mut self, mut save_file:String) {

        // Check if file path string provided.
        // If not then overwrite the loaded image file instead.
        if save_file.len() == 0 {
            save_file = self.image_file.clone();
            info!("Overwritting original image.")
        }
        // Create path to image file .
        let mut img_path = PathBuf::new();
        img_path.push(&save_file.clone());
        let img_path_string = img_path.to_string_lossy().into_owned();
        info!("Writing to image: {}", img_path_string);

        // Save the image with embedded data to file.
        if let Some(image) = &self.image {
            image.save(img_path_string).expect("Failed to save image");
        } else {
            panic!("Failed to save image file.");
        }
    }
}
