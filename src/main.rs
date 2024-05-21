// Steganography application.

use log::info;
use log4rs;
use lazy_static::lazy_static;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Mutex;
use actix_files as fs;
use actix_web::{get, App, HttpServer, Responder, HttpResponse};

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging configuration held in log4rs.yml .
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Get application metadata to include in initial logging.
    let settings: Settings = SETTINGS.lock().unwrap().clone();
    info!("Application started: {} v({})", settings.program_name, settings.program_ver);

    // Instatiate a steganography struct.
    // Call init method to initialise struct.
    let _img_steg = Steganography::init();

    // Create and start web service.
    HttpServer::new(|| {
        App::new()
            .service(intro)
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
