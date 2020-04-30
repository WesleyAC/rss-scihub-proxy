#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::fs;

use rocket::State;
use rocket::http::{RawStr, Status};
use rocket::response::content;
use rocket::config::Environment;

use minidom::{Element, NSChoice};

use serde::Deserialize;

#[get("/<id>")]
fn feed(config: State<Config>, id: &RawStr) -> Result<content::Xml<Vec<u8>>, Status> {
    if let Some(feed) = config.feeds.get(&id.to_string()) {
        let response = ureq::get(&feed.url)
            .timeout_connect(feed.timeout_connect.unwrap_or(5000))
            .timeout_read(feed.timeout_read.unwrap_or(5000))
            .timeout_write(feed.timeout_write.unwrap_or(5000))
            .call();

        let mut root: Element = response
            .into_string()
            .map_err(|_| Status::ServiceUnavailable)?
            .parse()
            .map_err(|_| Status::ServiceUnavailable)?;

        for child in root.children_mut() {
            if child.is("item", NSChoice::Any) {
                if let Some(link) = child.get_child_mut("link", NSChoice::Any) {
                    let texts = link.texts_mut();
                    for text in texts {
                        *text = format!("https://sci-hub.tw/{}", text);
                    }
                }
            }
        }

        let mut out = vec![];
        root.write_to(&mut out)
            .map_err(|_| Status::ServiceUnavailable)?;

        Ok(content::Xml(out))
    } else {
        Err(Status::NotFound)
    }
}

#[get("/")]
fn index(config: State<Config>) -> Result<content::Html<String>, Status> {
    if config.show_index.unwrap_or(true) {
        Ok(content::Html(format!(r#"
            <!doctype html>
            <html>
                <head>
                    <title>rss-scihub-proxy</title>
                </head>
                <body>
                    <h1>rss-scihub-proxy</h1>
                    <ul>
                        {}
                    </ul>
                </body>
            </html>
        "#, config.feeds.iter().map(|(k, v)| {
            format!("<li><a href='/{}'>{}</a></li>", k, v.name.as_ref().unwrap_or(&k.to_string()))
        }).collect::<Vec<String>>().join("\n"))))
    } else {
        Err(Status::Forbidden)
    }
}

#[derive(Deserialize)]
struct Config {
    feeds: HashMap<String, Feed>,
    #[serde(default)]
    server: ServerConfig,
    show_index: Option<bool>,
}

#[derive(Deserialize)]
struct Feed {
    url: String,
    name: Option<String>,
    timeout_connect: Option<u64>,
    timeout_write: Option<u64>,
    timeout_read: Option<u64>,
}

#[derive(Default, Deserialize)]
struct ServerConfig {
    address: Option<String>,
    port: Option<u16>,
    workers: Option<u16>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} config.toml", args[0]);
        std::process::exit(1);
    }
    let config: Config = toml::from_str(&fs::read_to_string(args[1].clone())?)?;
    let rocket_config = rocket::config::Config::build(Environment::Production)
        .address(config.server.address.clone().unwrap_or("0.0.0.0".to_string()))
        .port(config.server.port.unwrap_or(8080))
        .workers(config.server.workers.unwrap_or(4))
        .unwrap();

    rocket::custom(rocket_config).mount("/", routes![index, feed]).manage(config).launch();

    Ok(())
}
