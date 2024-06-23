// Steganography application.

use log::info;
use log4rs;
use futures::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use sanitize_filename::sanitize;
use actix_files as fs;
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

            // *******************************************************************************
            // Temporary code to generate test data for UI development.
            // *******************************************************************************
            // steg.load_new_file(String::from("./images/volleyballs.png"));
            // let embed_files = vec!["/home/mike/Desktop/TestPics/kittenseyes.png", "/home/mike/Desktop/TestPics/littlecat.jpeg"];
            // if let Err(err) = steg.embed_files(false, &String::from(""), &embed_files) {
            //     eprintln!("Error: {}", err);
            // }
            // else {
            //     steg.save_image("/home/mike/Desktop/TestPics/twocatpics_npw.png".to_string())
            // }
            // *******************************************************************************
            // steg.load_new_file(String::from("./images/volleyballs.png"));
            // let embed_files = vec!["/home/mike/Desktop/TestPics/kittenseyes.png",
            //                         "/home/mike/Desktop/TestPics/littlecat.jpeg",
            //                         "/home/mike/Desktop/TestPics/lotsofkittens.jpeg"];
            // if let Err(err) = steg.embed_files(false, &String::from(""), &embed_files) {
            //     eprintln!("Error: {}", err);
            // }
            // else {
            //     steg.save_image("/home/mike/Desktop/TestPics/threecatpics_npw.png".to_string())
            // }
            // *******************************************************************************
            // steg.load_new_file(String::from("./images/volleyballs.png"));
            // let embed_files = vec!["/home/mike/Desktop/TestPics/kittenseyes.png",
            //                         "/home/mike/Desktop/TestPics/littlecat.jpeg",
            //                         "/home/mike/Desktop/TestPics/lotsofkittens.jpeg",
            //                         "/home/mike/Desktop/TestPics/upsidedownkitten.jpeg"];
            // if let Err(err) = steg.embed_files(true, &String::from("michaek"), &embed_files) {
            //     eprintln!("Error: {}", err);
            // }
            // else {
            //     steg.save_image("/home/mike/Desktop/TestPics/fourcatpics_pw.png".to_string())
            // }
            // *******************************************************************************
        
            // Load a file for analysis.
            // This includes whether or not it is coded.
            steg.load_new_file(filepath_clone);

            // Construct image file analysis results for display to the user.
            response_data.insert("coded", "False".to_string());
            response_data.insert("password", "False".to_string());
            response_data.insert("capacity", steg.embed_capacity.to_string());
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
                let file_type = file.file_type.clone();

                files.push(HashMap::from([
                    ("name", file_name.clone()),
                    ("path", file_path.clone()),
                    ("type", file_type.clone()),
                ]));
            }

            // Respond with extraction status to display on UI.
            response_data.insert("extracted", "True".to_string());
            let duration_str = format!("{:?}", steg.extract_duration);
            response_data.insert("time", duration_str);

            // Respond with names of extracted files.
            let files_json = serde_json::to_string(&files).unwrap();
            response_data.insert("files", files_json.clone());
        }
        // Extraction failed with error result.
        Err(_e) => {
            // Respond with failed extraction status to display on UI.
            response_data.insert("extracted", "False".to_string());
            let duration_str = format!("{:?}", steg.extract_duration);
            response_data.insert("time", duration_str);
        }
    }
    HttpResponse::Ok().json(response_data)
}

#[post("/embed")]
async fn embed(
    form: web::Form<HashMap<String, String>>,
    steg: web::Data<Arc<Mutex<Steganography>>>,
) -> impl Responder {

    // User password for embedding received from UI.
    let _password = form.get("password").cloned().unwrap_or_default(); 

    // Get access to steg instance.
    let steg = steg.lock().unwrap();

    // Initialise vector of files.
    let mut response_data = HashMap::new();

    // Respond with embedding status to display on UI.
    response_data.insert("embedded", "True".to_string());
    let duration_str = format!("{:?}", steg.embed_duration);
    response_data.insert("time", duration_str);

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
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(img_steg.clone()))
            .service(fs::Files::new("/secrets", "./secrets").show_files_listing())
            .service(intro)
            .service(upload)
            .service(extract)
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
