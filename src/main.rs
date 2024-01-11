use actix_files::Files;
use actix_web_lab::extract::Query;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::header::{ContentType, TryIntoHeaderPair};
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
    nonce: String,
    base: String,
}

// BingoGrid but works with query parameters
#[derive(Deserialize, Serialize, Debug, Clone)]
struct QBingoGrid {
    size: u8,
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

fn pp() -> impl TryIntoHeaderPair {
    (
        "Permissions-Policy",
        "accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=(), conversion-measurement=(), focus-without-user-activation=(), hid=(), idle-detection=(), interest-cohort=(), serial=(), sync-script=(), trust-token-redemption=(), unload=(), window-placement=(), vertical-scroll=()",
    )
}

fn csp(nonce: &str) -> impl TryIntoHeaderPair {
    (
        "Content-Security-Policy",
        format!("default-src 'none'; style-src 'self' 'nonce-{}'; img-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors *; sandbox allow-same-origin;", nonce),
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
        .insert_header(pp())
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
        .insert_header(pp())
        .insert_header(csp(&nonce))
        .content_type(ContentType::html())
        .body(body)
}

fn default_names(n_names: usize) -> Vec<String> {
    let mut names = Vec::<String>::new();
    for _n in 0..n_names {
        names.push(String::from(""))
    }
    names
}

fn default_ranges(n_ranges: usize) -> Vec<u8> {
    let mut ranges = Vec::<u8>::new();
    for _n in 0..n_ranges {
        ranges.push(0)
    }
    ranges
}

#[get("/edit")]
async fn edit(tera: web::Data<Tera>, bingo: Query<QBingoGrid>) -> impl Responder {
    dbg!(&bingo);

    let n_fields = usize::from(bingo.size.pow(2));
    dbg!(&n_fields);

    let names = bingo.names.clone().unwrap_or_else(|| {
        default_names(n_fields)
    });
    dbg!(&names);

    let ranges = bingo.ranges.clone().unwrap_or_else(|| {
        default_ranges(n_fields)
    });
    dbg!(&ranges);

    let mut fields = Vec::<BingoField>::new();
    for n in 0..n_fields {
        dbg!(n);
        fields.push(BingoField {
            name: names[n].clone(),
            range: ranges[n],
        });
    }
    dbg!(&fields);

    let nonce = nonce();
    let base = base();
    let edit_context = BingoGrid {
        size: bingo.size,
        fields: fields,
        nonce: nonce.clone(),
        base: base.clone(),
    };
    let edit = tera.render("edit.html", &Context::from_serialize(&edit_context).unwrap()).unwrap();

    let fruit = fruit().to_string();
    let mut base_context = Context::new();
    base_context.insert("action", "Edit bingo");
    base_context.insert("body", &edit);
    base_context.insert("nonce", &nonce);
    base_context.insert("base", &base);
    
    base_context.insert("fruit", &fruit);
    let body = tera.render("base.html", &base_context).unwrap();

    HttpResponse::Ok()
        .insert_header(pp())
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
            .service(Files::new("/", &static_dir))
    })
    .bind(("127.0.0.1", port()))?
    .run()
    .await
}
