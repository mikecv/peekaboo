// Steganography application.

use log::info;
use log4rs;
use lazy_static::lazy_static;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Mutex;
use serde_yaml;
use actix_web::{HttpServer, App, web, HttpResponse, Responder};
use tera::{Tera, Context};

use crate::settings::Settings;

pub mod settings;

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

async fn index(tera: web::Data<Tera>) -> impl Responder {
    let mut data = Context::new();
    let settings: Settings = SETTINGS.lock().unwrap().clone();
    data.insert("title", &settings.program_name);
    data.insert("name", &settings.program_devs);

    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging onfiguration held in log4rs.yml .
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Get application metadata to include in initial logging.
    info!("Application started, version: {}", env!("CARGO_PKG_VERSION"));

    // Load the starting template file.
    HttpServer::new(|| {
        let tera = Tera::new("templates/**/*").unwrap();
        App::new()
            .data(tera)
            .route("/", web::get().to(index))
    })

    // Start the web application.
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
