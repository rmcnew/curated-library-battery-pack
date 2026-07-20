# curated-library-battery-pack

providing generally useful libraries and tools needed for Rust software development

## What is this?

"Il meglio è l'inimico del bene." \--Voltaire
"Perfect is the enemy of good."

Compared to the standard libraries of [C++](https://cppreference.com/cpp/standard_library), [Go](https://pkg.go.dev/std), [Python](https://docs.python.org/3/library/index.html), or [Java](https://docs.oracle.com/en/java/javase/25/docs/api/index.html), the [Rust Standard Library](https://doc.rust-lang.org/std/) limited.  Many common programming tasks require third-party crates because the Rust Standard Library lacks the needed functionality.  Rust developers must research many third-party crates, perform a careful analysis of candidate crates, select whatever candidate crate seems to be the best, and [hope that the crate they selected will be maintained](https://youtu.be/nOSxuaDgl3s?t=2954&si=Tn1RTB3ferbu3gCv) for the lifetime of the software they are building.  Despite all of the advantages the Rust programming language offers, its limited standard library and sprawling crate ecosystem can be an impediment to the adoption of Rust in enterprise software.  Switching to a different programming language like Rust has a high cost compared to continuing to use existing enterprise-friendly programming languages like Go or Java.  The uncertainty of the Rust crate ecosystem and lack of enterprise support can be deciding factors in not moving to Rust.

What is to be done?  It is difficult to get consensus on the best way to address this issue.  Many developers and organizations are content with the current status while others [seek a large battle-tested standard library](https://github.com/rust-lang/rfcs/pull/3810) as seen in other programming languages.  But why should the Rust Standard Library be special?  It is [not dependency free](https://github.com/CAD97/blog/discussions/2) and is built from the same crates ecosystem that all Rust software uses.  [Battery Packs](https://battery-pack-rs.github.io/battery-pack/) offer an interesting middle ground to make the Rust ecosystem more friendly.  Rust community members and enterprises can craft and share their own "extended standard libraries" to fill in functionality gaps or create domain-specific libraries of crates.

Enterprise software developers want stability, long-term support, and ready-to-use tools.  Battery Pack-powered extended standard libraries can help Rust community members and enterprises to efficiently navigate the Rust crate ecosystem and turbo-charge the Rust development experience.

The `curated-library-battery-pack` is an experiment to use a Battery Pack as an "extended standard library" for Rust software development.  Please provide feedback, bug reports, and feature requests as GitHub Issues.

## Usage

First, you will need to install the [Battery Pack CLI](https://battery-pack-rs.github.io/battery-pack/getting-started.html):
```sh
cargo install cargo-bp
```

Now you can add crates from the Curated Library Battery Pack to your Rust project or use the Web Service template:
```sh
cargo bp add curated-library-battery-pack
```

## Web Service template

The `web_service` template can be used as a starting point for creating a simple HTTPS web service that features both a web browser client and a command line client.

Create a new project using this template:
```sh
cargo bp new curated-library --template web_service
```

You will be prompted to provide a name for your project and a brief description of what it does.  The project template will be instantiated and your project is ready for development.

Build the project:
```sh
cd YOUR_PROJECT_NAME
cargo build
```

In the build output directory, you will see two binaries named `client` and `server`:
```
ls target/debug

build/  client*  client.d  deps/  examples/  incremental/  libtest2.d  libtest2.rlib  server*  server.d
```

### Server

`server` is the web server binary.  It embeds all of the web browser client files found in the `web` directory.  Running `server --help` gives the command line options:
````
Usage: server [OPTIONS]

Options:
  -p, --port <PORT>                  Secure port for web UI to use [default: 8443]
  -c, --certificates <CERTIFICATES>  Security certificates in PEM bundle format
  -k, --key <KEY>                    Security key in PEM format
  -d, --delete-logs                  Delete log files when closing
  -h, --help                         Print help
  -V, --version                      Print version
````

If you run the server without providing certificates and a private key, `server` generates self-signed certificates for use in non-production environments.
```
./target/debug/server

No TLS certificates or key were provided.  Generating and using self-signed certificate and key for TLS configuration.
Starting web_service server on port 8443
````

### Web Browser Client

Navigate your web browser to `https://localhost:8443` and bypass your browser's "Your connection is not private" warning.  (Your web browser does not recognize the self-signed certificate.)

You will find a very simple web page that calls the only API the web service stub implments:  A STATUS API to see if the web service is up.

Clicking the "Status Request" button will send a status request to the web service and update the web page.


### Command Line Client

`client` is a command-line client for the web service.  Running `client --help` gives the command line options:
````
Usage: client [OPTIONS] <COMMAND>

Commands:
  status
  help    Print this message or the help of the given subcommand(s)

Options:
  -s, --web_service-server-url <WEB_SERVICE_SERVER_URL>
          Url for the web_service server
  -b, --web_service-server-certificates <WEB_SERVICE_SERVER_CERTIFICATES>
          Use the provided custom certificates in PEM bundle format to verify server identity
  -a, --web_service-client-identity <WEB_SERVICE_CLIENT_IDENTITY>
          Use the provided client identity (private key and certificate) in PEM format
  -y, --allow-self-signed-cert <WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT>
          Allow web_service server to use self-signed certificate? DANGER! web_service server identity will not be assured [possible values: true, false]
  -w, --do-not-verify-server-hostname <WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME>
          Do not verify web_service server hostname DANGER! web_service server identity will not be assured [possible values: true, false]
  -d, --delete-logs
          Delete log files when closing?
  -h, --help
          Print help
  -V, --version
          Print version
````

The command line client can query the server, but expects a secure connection.  If you are using self-signed certificates, the client complains:
```
./target/debug/client -s https://localhost:8443 status

Error: Error sending request to web_service server: error sending request for url (https://localhost:8443/api/v1/status)
	Caused by: client error (Connect)
	Caused by: invalid peer certificate: UnknownIssuer
```

You can use `--allow-self-signed-cert true` to allow self-signed certificates:
```
./target/debug/client --allow-self-signed-cert true -s https://localhost:8443 status

'--allow-self-signed-cert' requested.  web_service server identity will NOT be verified.
web_service server is up and handling requests
```


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
