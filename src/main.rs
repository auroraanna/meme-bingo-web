use actix_files::Files;
use actix_web_lab::extract::Query;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::header::{ContentType, TryIntoHeaderPair, HeaderName, HeaderValue, PERMISSIONS_POLICY, CONTENT_SECURITY_POLICY};
use actix_web::middleware::Compress;
use base64::{Engine as _, engine::general_purpose};
use tera::{Tera, Context};
use rand::RngCore;
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BingoField {
    name: String,
    range: u8,
}

// For inserting data into the edit template
#[derive(Deserialize, Serialize, Debug, Clone)]
struct BingoGrid {
    size: u8,
    fields: Vec<BingoField>,
}

impl BingoGrid {
    fn new_from_size(size: u8) -> Result<Self, &'static str> {
        if !(size <= 15) {
            return Err("Bingo size must not be larger than 15.");
        }

        let mut fields = Vec::<BingoField>::new();
        for _ in  0..(size.pow(2)) {
            fields.push(BingoField {
                name: String::from(""),
                range: 0,
            });
        }

        Ok(BingoGrid { size, fields })
    }

    fn new_from_fields(fields: Vec<BingoField>) -> Result<Self, &'static str> {
        let length = fields.len();
        if !(usize::from(15_u8.pow(2)) >= length) {
            return Err("Length of fields must not exceed 255.");
        }
        let size_float = (length as f32).sqrt();
        if !(size_float.floor() == size_float) {
            return Err("Taking the square root of the length of fields must yield a whole number.");
        }
        let size = size_float as u8;

        Ok(BingoGrid { size, fields })
    }

    fn new_from_names_and_ranges(names: Vec<String>, ranges: Vec<u8>) -> Result<Self, &'static str> {
        if !(names.len() == ranges.len()) {
            return Err("names and ranges must be of the same length.");
        }

        let mut fields = Vec::<BingoField>::new();
        for n in 0..names.len() {
            fields.push(BingoField {
                name: names[n].clone(),
                range: ranges[n],
            });
        }

        Ok(match Self::new_from_fields(fields) {
            Ok(v) => v,
            Err(e) => {
                return Err(e);
            },
        })
    }
}

// BingoGrid but works with query parameters
#[derive(Deserialize, Serialize, Debug, Clone)]
struct QBingoGrid {
    size: Option<u8>,
    #[serde(rename = "name")]
    names: Option<Vec<String>>,
    #[serde(rename = "range")]
    ranges: Option<Vec<u8>>,
}

fn base() -> String {
    let name = "MEME_BINGO_BASE";
    match env::var(name) {
        Ok(v) => return v,
        Err(e) => panic!("${} is not set ({})", name, e),
    }
}

fn nonce() -> String {
    let mut rng = OsRng::default();
    let mut nonce = [0u8; 16];
    rng.fill_bytes(&mut nonce);
    general_purpose::STANDARD.encode(&nonce)
}

fn fruit() -> char {
    let fruits = [
        '🍊',
        '🍋',
        '🍌',
        '🍍',
        '🥭',
        '🍎',
        '🍏',
        '🍐',
        '🍑',
        '🍒',
        '🍓',
        '🥝',
        '🌽',
    ];
    let mut rng = rand::thread_rng();
    let random_index: usize = rng.gen_range(0..fruits.len());
    fruits[random_index]
}

const PP: (HeaderName, HeaderValue) = (
    PERMISSIONS_POLICY,
    HeaderValue::from_static("accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=(), conversion-measurement=(), focus-without-user-activation=(), hid=(), idle-detection=(), interest-cohort=(), serial=(), sync-script=(), trust-token-redemption=(), unload=(), window-placement=(), vertical-scroll=()"),
 );

fn csp(nonce: &str) -> impl TryIntoHeaderPair {
    (
        CONTENT_SECURITY_POLICY,
        format!("default-src 'none'; style-src 'self' 'nonce-{}'; img-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors *; sandbox allow-same-origin allow-forms;", nonce),
    )
}

#[get("/")]
async fn index(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("action", "About");
    let index = tera.render("index.html", &Context::new()).unwrap();
    context.insert("body", &index);
    let fruit = fruit().to_string();
    context.insert("fruit", &fruit);
    let nonce = nonce();
    context.insert("nonce", &nonce);
    let base = base();
    context.insert("base", &base);
    let body = tera.render("base.html", &context).unwrap();

    HttpResponse::Ok()
        .insert_header(PP)
        .insert_header(csp(&nonce))
        .content_type(ContentType::html())
        .body(body)
}

#[get("/new")]
async fn new(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("action", "New bingo");
    let new = tera.render("new.html", &Context::new()).unwrap();
    context.insert("body", &new);
    let fruit = fruit().to_string();
    context.insert("fruit", &fruit);
    let nonce = nonce();
    context.insert("nonce", &nonce);
    let base = base();
    context.insert("base", &base);
    let body = tera.render("base.html", &context).unwrap();

    HttpResponse::Ok()
        .insert_header(PP)
        .insert_header(csp(&nonce))
        .content_type(ContentType::html())
        .body(body)
}

// For convenience
fn bad_request(reason: &'static str) -> HttpResponse {
    HttpResponse::BadRequest()
        .reason(&reason)
        .body(reason.to_string())
}

#[get("/edit")]
async fn edit(tera: web::Data<Tera>, qbingo: Query<QBingoGrid>) -> impl Responder {
    let bingo: BingoGrid = match qbingo.size {
        Some(v) => match BingoGrid::new_from_size(v) {
            Ok(v) => v,
            Err(e) => {
                return bad_request(e);
            },
        },
        None => match BingoGrid::new_from_names_and_ranges(
            match &qbingo.names {
                Some(v) => v.to_vec(),
                None => {
                    return bad_request("No size or name query parameters.");
                },
            },
            match &qbingo.ranges {
                Some(v) => v.to_vec(),
                None => {
                    return bad_request("No size or range query parameters.");
                },
            }
        ) {
            Ok(v) => v,
            Err(e) => {
                return bad_request(e);
            },
        },
    };

    let nonce = nonce();
    let base = base();

    let mut edit_context = Context::new();
    edit_context.insert("bingo", &bingo);
    edit_context.insert("nonce", &nonce);
    edit_context.insert("base", &base);
    let edit = tera.render("edit.html", &edit_context).unwrap();

    let mut base_context = Context::new();
    base_context.insert("action", "Edit bingo");
    base_context.insert("body", &edit);
    base_context.insert("nonce", &nonce);
    base_context.insert("base", &base);
    base_context.insert("fruit", &(fruit().to_string()));
    let body = tera.render("base.html", &base_context).unwrap();

    HttpResponse::Ok()
        .insert_header(PP)
        .insert_header(csp(&nonce))
        .content_type(ContentType::html())
        .body(body)
}

#[get("/samples")]
async fn samples(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("action", "Sample bingos");
    let samples = tera.render("samples.html", &Context::new()).unwrap();
    context.insert("body", &samples);
    context.insert("fruit", &fruit().to_string());
    let nonce = nonce();
    context.insert("nonce", &nonce);
    context.insert("base", &base());
    let body = tera.render("base.html", &context).unwrap();

    HttpResponse::Ok()
        .insert_header(PP)
        .insert_header(csp(&nonce))
        .content_type(ContentType::html())
        .body(body)
}

fn port() -> u16 {
    let name = "MEME_BINGO_PORT";
    let port = env::var(name);
    match port {
        Ok(v) => match v.parse::<u16>() {
            Ok(parsed_v) => return  parsed_v,
            Err(_) => panic!("Error parsing environment variable as u16"),
        },
        Err(e) => panic!("${} is not set ({})", name, e),
    }
}

fn templ_dir() -> String {
    let fallback_templ_dir = String::from("./templates");
    let name = "MEME_BINGO_TEMPLATES";
    let templ_dir = env::var(name);
    match templ_dir {
        Ok(v) => match v.parse::<String>() {
            Ok(parsed_v) => return  parsed_v,
            Err(_) => panic!("Error parsing environment variable as String"),
        },
        Err(_) => {
            eprintln!("${} is not set, using {}", name, fallback_templ_dir);
            return fallback_templ_dir;
        }
    }
}

fn static_dir() -> String {
    let fallback_static_dir = String::from("./static");
    let name = "MEME_BINGO_STATIC";
    let static_dir = env::var(name);
    match static_dir {
        Ok(v) => match v.parse::<String>() {
            Ok(parsed_v) => return  parsed_v,
            Err(_) => panic!("Error parsing environment variable as String"),
        },
        Err(_) => {
            eprintln!("${} is not set, using {}", name, fallback_static_dir);
            return fallback_static_dir;
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let static_dir = static_dir();

    let tera = Tera::new(&(templ_dir() + "/*.html")).unwrap();
    let tera_ref = web::Data::new(tera);

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .app_data(tera_ref.clone())
            .service(index)
            .service(new)
            .service(edit)
            .service(samples)
            .service(Files::new("/", &static_dir))
    })
    .bind(("127.0.0.1", port()))?
    .run()
    .await
}
