use actix_files::Files;
use actix_web_lab::extract::Query;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::header::{ContentType, TryIntoHeaderPair};
use actix_web::middleware::Compress;
use base64::{Engine as _, engine::general_purpose};
use handlebars::Handlebars;
use rand::RngCore;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

fn fruit() -> String {
    let fruits = [
        "🍊",
        "🍋",
        "🍌",
        "🍍",
        "🥭",
        "🍎",
        "🍏",
        "🍐",
        "🍑",
        "🍒",
        "🍓",
        "🥝",
        "🌽",
    ];
    let fruit = fruits.choose(&mut rand::thread_rng()).expect(fruits[0]).to_string();
    fruit
}

fn csp(nonce: String) -> impl TryIntoHeaderPair {
    (
        "Content-Security-Policy",
        format!("default-src 'none'; style-src 'self' 'nonce-{}'; img-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors *;", nonce),
    )
}

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let mut data = BTreeMap::new();
    data.insert("action", "About");
    let index = hb.render("index", &false).unwrap();
    data.insert("body", &index);
    let fruit = fruit();
    data.insert("fruit", &fruit);
    let nonce = nonce();
    data.insert("nonce", &nonce);
    let base = base();
    data.insert("base", &base);
    let body = hb.render("base", &data).unwrap();

    HttpResponse::Ok()
        .insert_header(csp(nonce))
        .content_type(ContentType::html())
        .body(body)
}

#[get("/new")]
async fn new(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let mut data = BTreeMap::new();
    data.insert("action", "New bingo");
    let new = hb.render("new", &false).unwrap();
    data.insert("body", &new);
    let fruit = fruit();
    data.insert("fruit", &fruit);
    let nonce = nonce();
    data.insert("nonce", &nonce);
    let base = base();
    data.insert("base", &base);
    let body = hb.render("base", &data).unwrap();

    HttpResponse::Ok()
        .insert_header(csp(nonce))
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
async fn edit(hb: web::Data<Handlebars<'_>>, bingo: Query<QBingoGrid>) -> impl Responder {
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
    let edit_data = BingoGrid {
        size: bingo.size,
        fields: fields,
        nonce: nonce.clone(),
        base: base.clone(),
    };
    let edit = hb.render("edit", &edit_data).unwrap();

    let fruit = fruit();
    let mut base_data = BTreeMap::new();
    base_data.insert("action", "Edit bingo");
    base_data.insert("body", &edit);
    base_data.insert("nonce", &nonce);
    base_data.insert("base", &base);
    
    base_data.insert("fruit", &fruit);
    let body = hb.render("base", &base_data).unwrap();

    HttpResponse::Ok()
        .insert_header(csp(nonce))
        .content_type(ContentType::html())
        .body(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut hbars = Handlebars::new();
    hbars
        .register_templates_directory(".html", "./templates/")
        .unwrap();
    let hbars_ref = web::Data::new(hbars);

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .app_data(hbars_ref.clone())
            .service(index)
            .service(new)
            .service(edit)
            .service(Files::new("/static", "static"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
