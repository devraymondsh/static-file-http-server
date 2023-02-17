use crate::cli;
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
use std::{
    ffi::OsStr,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tokio::{fs, net as tokio_net, time as tokio_time};

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
fn set_content_type(content_type: impl ToString) -> SetHeader {
    SetHeader(String::from("Content-Type"), content_type.to_string())
}
fn set_cors_access(cors_access: impl ToString) -> SetHeader {
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

async fn handler(path: String, args: &cli::Args) -> HandlerReturnType {
    let public_dir = String::from(args.path.to_str().unwrap());
    let path = if public_dir.ends_with("/") {
        format!("{}{}", public_dir, path)
    } else {
        format!("{}/{}", public_dir, path)
    };

    match fs::read_to_string(&path).await {
        Ok(contents) => {
            let content_type = match Path::new(&path).extension().and_then(OsStr::to_str) {
                Some(ext) => new_mime_guess::from_ext(ext)
                    .first_or_text_plain()
                    .to_string(),
                None => String::from("text/plain"),
            };

            (
                StatusCode::OK,
                set_content_type(content_type),
                set_cors_access(&args.cors),
                set_cache_control(&args.cache),
                String::from(contents),
            )
        }
        Err(_) => {
            let body = match PathBuf::from_str(
                format!("{}/404.html", args.path.to_str().unwrap()).as_str(),
            ) {
                Ok(path) => {
                    if path.exists() {
                        fs::read_to_string(path).await.unwrap()
                    } else {
                        String::from("File not found!")
                    }
                }
                Err(_) => String::from("File not found!"),
            };

            (
                StatusCode::NOT_FOUND,
                set_content_type("text/html"),
                set_cors_access(&args.cors),
                set_cache_control(&args.cache),
                body,
            )
        }
    }
}

const INDEX_FILES: [&str; 4] = ["index.html", "index.htm", "index.xhtml", "index.shtml"];
fn root_serve(parent: &str) -> String {
    let parent = if !parent.ends_with("/") {
        format!("{}/", parent)
    } else {
        String::from(parent)
    };

    let file_exists = |file: &str| match PathBuf::from_str(format!("{}{}", parent, file).as_str()) {
        Ok(pathbuf) => pathbuf.exists(),
        Err(_) => false,
    };
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

async fn addr_is_available_or_exit(addr: SocketAddr) {
    let listener = tokio_net::TcpListener::bind(addr).await;
    match listener {
        Ok(_) => {}
        Err(err) => {
            cli::unrecoverable_clap_error_with_cmd(format!(
                "Unable to bind on: {}! Reason: {}.",
                addr.to_string(),
                err.to_string()
            ));
        }
    };
}

pub async fn run(args: &'static cli::Args) {
    let root_serve = root_serve(args.path.to_str().unwrap());
    let app = Router::new()
        .route("/", get(move || handler(root_serve, &args)))
        .route(
            "/:filename",
            get(move |AxumPath(path): AxumPath<String>| handler(path, &args)),
        );

    tokio::spawn(async {
        let mut i = 0;
        loop {
            tokio_time::sleep(Duration::from_secs(2)).await;

            let addr = args.addr.to_string();
            let url = format!("http://{}", &addr);
            if let Ok(resp) = reqwest::get(url).await {
                if resp.status() == 200 || resp.status() == 404 {
                    if args.open {
                        open::that(&addr).unwrap_or_else(|err| {
                            println!("Failed to open the browser. Reason: {}", err.to_string());
                        });
                    }

                    println!("Server has started. Listening on {}.", &addr);
                    break;
                }
            }

            if i > 60 {
                cli::unrecoverable_clap_error_with_cmd("Timout exceeded! Server failed to start.");
            }

            i += 1;
        }
    });

    addr_is_available_or_exit(args.addr).await;
    let server = axum::Server::bind(&args.addr)
        .serve(app.into_make_service())
        .await;
    if let Err(err) = server {
        cli::unrecoverable_clap_error_with_cmd(err);
    }
}
