# static-file-http-server: a simple static server
### static-file-http-server is a simple, zero-configuration command-line static server written in Rust. It is powerful enough for production usage, but it's simple and requires not configuration. It can also produce a single binary file that hosts your files independently which is what we recommend for production.

<br />

## Installation:
### Using npm:
`npm install --global static-file-http-server`

### Using cargo:

`cargo install static-file-http-server`

<br />

## How to use:
`static-file-http-server [OPTIONS] <PATH>`

## Options:
| Option(short) | Option(long)    | Description                                                                                                                                                                                       | Default      |
|---------------|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
| -a            | --addr          | Address to bind, for example: 0.0.0.0:80. You may need administrator permissions for binding on port 80 based on your OS.                                                                         | 127.0.0.1:8085 |
| -r            | --cors          | Controling CORS via the 'Access-Control-Allow-Origin' header.                                                                                                                                     | *            |
| -c            | --cache         | Set cache time (in seconds) for cache-control max-age header, for eaxmple: -c10 for 10 seconds. Use -c-1 to disable caching.                                                                      | 3600         |
| -o            | --open          | Open the browser after starting the server.                                                                                                                                                       |              |
| -p            | --single-binary | Produce a single binary that serves files that get embedded in the binary for better performance. You need to install Rust and Cargo before running this feature. (Recommend  ed for production). |              |
| -h            | --help          | Print help.                                                                                                                                                                                       |              |
| -V            | --version       | Print version.                                                                                                                                                                                    |              |
