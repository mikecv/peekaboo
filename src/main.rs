// Steganography application.

use log::info;
use log4rs;
use futures::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use sanitize_filename::sanitize;
use actix_multipart::Multipart;

use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder};

use crate::settings::Settings;
use crate::steg::Steganography;

pub mod settings;
pub mod steg;

// Create a global variable for applications settings.
// This will be available in other files.
lazy_static! {
    static ref SETTINGS: Mutex<Settings> = {
        // Read YAML settings file.
        let mut file = File::open("settings.yml").expect("Unable to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read file");

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
    // following analysis be Steganography methods.
    let mut response_data = HashMap::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        if let Some(filename) = content_disposition.get_filename() {
            let filepath = format!("{}/{}", settings.thumb_folder, sanitize(&filename));
            let filepath_clone = filepath.clone();

            // File::create is a blocking operation, use threadpool.
            let mut f = web::block(move || File::create(filepath)).await.unwrap().unwrap();

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
            response_data.insert("coded", "False");
            response_data.insert("password", "False");
            if steg.pic_coded == true {
                response_data.insert("coded", "True");
                if steg.pic_has_pw == true {
                    response_data.insert("password", "True");
                }
            }
        }
    }

    HttpResponse::Ok().json(response_data)
}

#[post("/extract")]
async fn extract(steg: web::Data<Arc<Mutex<Steganography>>>) -> impl Responder {
    // Get steg instance in scope.
    let mut steg = steg.lock().unwrap();

    // Define the type of response_data explicitly.
    let mut response_data = HashMap::new();

    // Perform extraction of embedded files.
    steg.extract_data(String::from(""));

    // Go through steg.embedded_files struct and
    // extrat the 'file_name' element.
    let saved_files = &steg.embedded_files;
    let mut files_string = String::from("");
    for file in saved_files {
        files_string.push_str(&file.file_name);
    }

    // Construct a response based on the extraction result.
    response_data.insert("extracted", "True");
    let duration_str = format!("{:?}", steg.extract_duration);
    response_data.insert("time", &duration_str);

    // ADD THE EXTRACTED FILE IMAGES HERE.
    
    HttpResponse::Ok().json(response_data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging configuration held in log4rs.yml .
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Get application settings in scope.
    let settings: Settings = SETTINGS.lock().unwrap().clone();
    // Do initial program version logging, mainly as a test.
    info!("Application started: {} v({})", settings.program_name, settings.program_ver);

    // Instatiate a steganography struct.
    // Call init method to initialise struct.
    let img_steg = Arc::new(Mutex::new(Steganography::init()));

    // Create and start web service.
    HttpServer::new(move || {  // Use the `move` keyword to capture `img_steg`
        App::new()
            .app_data(web::Data::new(img_steg.clone()))
            .service(intro)
            .service(upload)
            .service(extract)
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
