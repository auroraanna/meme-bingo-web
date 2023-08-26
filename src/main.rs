use actix_files::Files;
use actix_web_lab::extract::Query;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::header::ContentType;
use actix_web::middleware::Compress;
use handlebars::Handlebars;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BingoField {
    name: String,
    range: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BingoGrid {
    size: u8,
    fields: Vec<BingoField>,
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

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let mut data = BTreeMap::new();
    data.insert("action", "About");
    let index = hb.render("index", &false).unwrap();
    data.insert("body", &index);
    let fruit = fruit();
    data.insert("fruit", &fruit);
    let body = hb.render("base", &data).unwrap();

    HttpResponse::Ok()
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
    let body = hb.render("base", &data).unwrap();

    HttpResponse::Ok()
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

    let edit = hb.render("edit", &BingoGrid {
        size: bingo.size,
        fields: fields,
    }).unwrap();

    let fruit = fruit();
    let mut base_data = BTreeMap::new();
    base_data.insert("action", "Edit bingo");
    base_data.insert("body", &edit);
    
    base_data.insert("fruit", &fruit);
    let body = hb.render("base", &base_data).unwrap();

    HttpResponse::Ok()
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
            .service(Files::new("/style", "style").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
