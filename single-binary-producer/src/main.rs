use axum::{
    extract::Path as AxumPath,
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
    routing::get,
    Router,
};
use clap::{
    error::{Error as ClapError, ErrorKind},
    Command as ClapCommand,
};
use include_directory::{include_directory, Dir};
use once_cell::sync::Lazy;
use std::{fmt::Display, net::SocketAddr, process, str::FromStr, time::Duration};
use tokio::{net as tokio_net, time as tokio_time};

const ADDR: Lazy<SocketAddr> = Lazy::new(|| SocketAddr::from_str("127.0.0.1:8080").unwrap());
const CACHE: i64 = 3600;
const CORS: Lazy<String> = Lazy::new(|| String::from("*"));
const INDEX_FILES: [&str; 4] = ["index.html", "index.htm", "index.xhtml", "index.shtml"];
const PROJECT_DIR: Dir = include_directory!("./_public_dir_");

pub fn unrecoverable_clap_error(message: impl Display) {
    let cmd = ClapCommand::new("static-file-http-server");

    let a: ClapError = ClapError::raw(ErrorKind::Io, message).with_cmd(&cmd);

    let _ = a.print();

    process::exit(10);
}

// Hypothetical helper type for setting a single header
struct SetHeader(String, String);
impl IntoResponseParts for SetHeader {
    type Error = (StatusCode, String);

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        match (self.0.parse::<HeaderName>(), self.1.parse::<HeaderValue>()) {
            (Ok(name), Ok(value)) => {
                res.headers_mut().insert(name, value);
            }
            (Err(_), _) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid header name {}", self.0),
                ));
            }
            (_, Err(_)) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid header value {}", self.1),
                ));
            }
        }

        Ok(res)
    }
}
// Its also recommended to implement `IntoResponse` so `SetHeader` can be used on its own as
// the response
impl IntoResponse for SetHeader {
    fn into_response(self) -> Response {
        // This gives an empty response with the header
        (self, ()).into_response()
    }
}
fn set_content_type(content_type: &impl ToString) -> SetHeader {
    SetHeader(String::from("Content-Type"), content_type.to_string())
}
fn set_cors_access(cors_access: &impl ToString) -> SetHeader {
    SetHeader(
        String::from("Access-Control-Allow-Origin"),
        cors_access.to_string(),
    )
}
fn set_cache_control(cache_control: &i64) -> Option<SetHeader> {
    if *cache_control >= 0 {
        return Some(SetHeader(
            String::from("Cache-Control"),
            format!("max-age={}", cache_control.to_string()),
        ));
    };

    None
}

// 1.HTTP status code 2.Content-Type header 3.CORS header 4.Cache-Control header 5.Body
type HandlerReturnType = (StatusCode, SetHeader, SetHeader, Option<SetHeader>, String);

async fn handler(path: String) -> HandlerReturnType {
    match PROJECT_DIR.get_file(&path) {
        Some(file) => (
            StatusCode::OK,
            set_content_type(&file.mimetype_as_string()),
            set_cors_access(&CORS.as_str()),
            set_cache_control(&CACHE),
            String::from(file.contents_utf8().unwrap_or_default()),
        ),
        None => {
            let body = match PROJECT_DIR.get_file("404.html") {
                Some(file) => {
                    if let Some(contents) = file.contents_utf8() {
                        contents
                    } else {
                        "File not found!"
                    }
                }
                None => "File not found!",
            };

            (
                StatusCode::NOT_FOUND,
                set_content_type(&"text/html"),
                set_cors_access(&CORS.as_str()),
                set_cache_control(&CACHE),
                String::from(body),
            )
        }
    }
}

fn root_serve() -> String {
    let file_exists = |file: &str| PROJECT_DIR.get_file(file).is_some();
    let mut out = None;
    for index_file in INDEX_FILES {
        if file_exists(index_file) {
            out = Some(index_file);
        }
    }
    if out.is_none() {
        out = Some(INDEX_FILES[1]);
    }

    String::from(out.unwrap())
}

async fn addr_is_available_or_exit(addr: &SocketAddr) {
    let listener = tokio_net::TcpListener::bind(addr).await;
    match listener {
        Ok(_) => {}
        Err(err) => {
            unrecoverable_clap_error(format!(
                "Unable to bind on: {}! Reason: {}.",
                addr.to_string(),
                err.to_string()
            ));
        }
    };
}

pub async fn run() {
    let root_serve = root_serve();
    let app = Router::new()
        .route("/", get(move || handler(root_serve)))
        .route(
            "/:filename",
            get(move |AxumPath(path): AxumPath<String>| handler(path)),
        );

    tokio::spawn(async {
        let mut i = 0;
        loop {
            tokio_time::sleep(Duration::from_secs(2)).await;

            let addr = ADDR.to_string();
            let url = format!("http://{}", &addr);
            if let Ok(resp) = reqwest::get(url).await {
                if resp.status() == 200 || resp.status() == 404 {
                    println!("Server has started. Listening on {}.", &addr);

                    break;
                }
            }

            if i > 60 {
                unrecoverable_clap_error("Timout exceeded! Server failed to start.");
            }

            i += 1;
        }
    });

    addr_is_available_or_exit(&ADDR).await;
    let server = axum::Server::bind(&ADDR)
        .serve(app.into_make_service())
        .await;
    if let Err(err) = server {
        unrecoverable_clap_error(err);
    }
}

#[tokio::main]
async fn main() {
    run().await;
}
