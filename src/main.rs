// Steganography application.

use log::info;
use log4rs;
use actix_files as fsx;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};
use chrono::Utc;
use futures_util::stream::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use std::fs;
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::File as StdFile;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sanitize_filename::sanitize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::settings::Settings;
use crate::steg::Steganography;

pub mod settings;
pub mod steg;

// Create a global variable for applications settings.
// This will be available in other files.
lazy_static! {
    static ref SETTINGS: Mutex<Settings> = {
        // Read YAML settings file.
        let mut file = futures::executor::block_on(File::open("settings.yml")).expect("Unable to open file");
        let mut contents = String::new();
        futures::executor::block_on(file.read_to_string(&mut contents)).expect("Unable to read file");

        // Deserialize YAML into Settings struct.
        let settings: Settings = serde_yaml::from_str(&contents).expect("Unable to parse YAML");
        Mutex::new(settings)
    };
}

#[get("/")]
async fn intro() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(include_str!("../static/index.html"))
}

#[post("/upload")]
async fn upload(mut payload: Multipart, steg: web::Data<Arc<Mutex<Steganography>>>,) -> impl Responder {
    // Get steg instance in scope.
    let steg = steg.clone();

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    // Json map of response to upload request
    // following analysis by Steganography methods.
    let mut response_data = HashMap::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        if let Some(filename) = content_disposition.get_filename() {
            let filepath = format!("{}/{}", settings.thumb_folder, sanitize(&filename));
            let filepath_clone = filepath.clone();

            // File::create is a blocking operation, use threadpool.
            let mut f = web::block(move || StdFile::create(filepath)).await.unwrap().unwrap();

            // Field in turn is stream of *Bytes* object.
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                // Filesystem operations are blocking, we have to use threadpool.
                f = web::block(move || {
                    let mut file = f;
                    file.write_all(&data).map(|_| file)
                }).await.unwrap().unwrap();
            }

            // Process the uploaded file with Steganography instance.
            let mut steg = steg.lock().unwrap();
        
            // Load a file for analysis.
            // This includes whether or not it is coded.
            steg.load_new_file(filepath_clone);

            // Construct image file analysis results for display to the user.
            response_data.insert("coded", "False".to_string());
            response_data.insert("password", "False".to_string());
            response_data.insert("capacity", steg.embed_capacity.to_string());
            response_data.insert("overhead", steg.overhead_per_file.to_string());
            if steg.pic_coded == true {
                response_data.insert("coded", "True".to_string());
                if steg.pic_has_pw == true {
                    response_data.insert("password", "True".to_string());
                }
            }
        }
    }

    HttpResponse::Ok().json(response_data)
}

#[post("/extract")]
async fn extract(
    form: web::Form<HashMap<String, String>>,
    steg: web::Data<Arc<Mutex<Steganography>>>,
) -> impl Responder {

    // User password received from UI.
    let password = form.get("password").cloned().unwrap_or_default(); 

    // Get access to steg instance.
    let mut steg = steg.lock().unwrap();

    // Initialise vector of extracted files.
    let mut response_data = HashMap::new();

    // Perform extraction of current uploaded file.
    // Check status of extaction
    match steg.extract_data(password.clone()) {
        // Extraction completed successfully.
        Ok(_) => {
            // Extraction completed successfully.
            // Get vector of extract files to display on UI.
            let saved_files = &steg.embedded_files;
            let mut files = Vec::new();
            for file in saved_files {
                let file_name = Path::new(&file.file_name)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let file_path = format!("secrets/{}", file_name);
                let file_type = &file.file_type;

                // Include coded status of extracted file.
                // Only need to bother checking possible MIME types.
                let mut file_coded = false;
                if file_type == "image/png" {
                    file_coded = file.file_coded;
                }

                files.push(HashMap::from([
                    ("name", file_name),
                    ("path", file_path),
                    ("type", file_type.to_string()),
                    ("coded", file_coded.to_string()),
                ]));
            }

            // Respond with extraction status to display on UI.
            response_data.insert("extracted", "True".to_string());
            let test_time_ms:f64 = steg.extract_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", test_time_ms);
            response_data.insert("time", duration_str);

            // Respond with names of extracted files.
            let files_json = serde_json::to_string(&files).unwrap();
            response_data.insert("files", files_json.clone());
        }
        // Extraction failed with error result.
        Err(_e) => {
            // Respond with failed extraction status to display on UI.
            response_data.insert("extracted", _e.to_string());
            let test_time_ms:f64 = steg.extract_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", test_time_ms);
            response_data.insert("time", duration_str);
        }
    }
    HttpResponse::Ok().json(response_data)
}

#[post("/embed")]
async fn embed(mut payload: Multipart, steg: web::Data<Arc<Mutex<Steganography>>>) -> impl Responder {

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();

    let mut password = String::new();
    let mut files = Vec::new();
    let temp_dir = temp_dir();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        if let Some(name) = content_disposition.get_name() {
            if name == "password" {
                while let Some(chunk) = field.try_next().await.unwrap() {
                    password.push_str(&String::from_utf8(chunk.to_vec()).unwrap());
                }
            } else if name == "files" {
                if let Some(filename) = content_disposition.get_filename() {
                    let sanitized_filename = sanitize(filename);
                    let file_path = temp_dir.join(&sanitized_filename);
                    let mut file = StdFile::create(&file_path).unwrap();
                    while let Some(chunk) = field.try_next().await.unwrap() {
                        file.write_all(&chunk).unwrap();
                    }
                    files.push(file_path.to_str().unwrap().to_string());
                }
            }
        }
    }

    // Convert Vec<String> to Vec<&str> for steg function.
    let files_ref: Vec<&str> = files.iter().map(|s| &**s).collect();

    // Get access to steg instance.
    let mut steg = steg.lock().unwrap();
    // Call the embed_files function with appropriate parameters.
    match steg.embed_files(!password.is_empty(), &password, &files_ref) {
        Ok(_) => {
            // Embedding succesful, so save to temporary file.
            // Create temporary file name from current time.
            let mut ts_string = Utc::now().to_string();
            ts_string = ts_string.chars().filter(|c| !c.is_whitespace()).collect();
 
            // Save in secrets folder from settings.
            // Should actually check the folder exists, and create if not.
            let mut wrt_path = PathBuf::new();       
            wrt_path.push(&settings.secret_folder);
            if !wrt_path.exists() {
                create_dir_all(&wrt_path).unwrap();
            }
            wrt_path.push(format!("{}.png",ts_string));
            let wrt_path_string = wrt_path.to_string_lossy().into_owned();
            let wrt_path_string_clone = wrt_path_string.clone();

            // Save the temporary output file.
            steg.save_image(wrt_path_string_clone.clone());

            // Embedding successful, respond with embedding status.
            let mut response_data = HashMap::new();
            response_data.insert("embedded", "True".to_string());
            let test_time_ms:f64 = steg.embed_duration.as_millis() as f64 / 1000.0 as f64;
            let duration_str = format!("{:.3} sec", test_time_ms);
            response_data.insert("time", duration_str);
            response_data.insert("thumbnail", wrt_path_string_clone.clone());
            response_data.insert("filename", wrt_path_string_clone.clone());

            // Respond with embedding status to display on UI.
            HttpResponse::Ok().json(response_data)
        }
        Err(e) => {
            // Embedding failed, respond with error.
            let mut response_data = HashMap::new();
            response_data.insert("embedded", "False".to_string());
            response_data.insert("error", e.to_string());

            // Respond with embedding status to display on UI.
            HttpResponse::InternalServerError().json(response_data)
        }
    }
}

async fn help(settings: web::Data<Settings>) -> impl Responder {
    // Help endpoint function
    // Read the help file.
    let help_file_content = fs::read_to_string("./static/peekaboo-help.html")
        .expect("Unable to read help file");

    // Replace the version placeholder with the actual version number from settings.
    // Repeat as necessary for other setting information required in help.
    let help_content = help_file_content.replace("{{version}}", &settings.program_ver);

    HttpResponse::Ok().content_type("text/html").body(help_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging configuration held in log4rs.yml .
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();
    // Do initial program version logging, mainly as a test.
    info!("Application started: {} v({})", settings.program_name, settings.program_ver);

    // Instantiate a steganography struct.
    // Call init method to initialise struct.
    let img_steg = Arc::new(Mutex::new(Steganography::init()));

    // Create and start web service.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(img_steg.clone()))
            .app_data(web::Data::new(settings.clone()))
            .service(fsx::Files::new("/secrets", "./secrets").show_files_listing())
            .service(intro)
            .service(upload)
            .service(extract)
            .service(embed)
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .route("/help", web::get().to(help))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
