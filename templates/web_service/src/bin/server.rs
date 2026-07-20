// web service server
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::Json;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use web_service::server_api::*;
use web_service::shared::{Timestamp, TypeError, hostname, write_string_to_named_tempfile};
use web_service::tracer;
use rcgen::{CertifiedKey, generate_simple_self_signed};
use rust_embed::Embed;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use tempfile::NamedTempFile;
use tracing::{error, info, warn};

// command-line arguments
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
struct ProgramArgs {
    /// Secure port for web UI to use
    #[arg(short = 'p', long = "port", default_value_t = 8443)]
    port: u16,
    /// Security certificates in PEM bundle format
    #[arg(short = 'c', long = "certificates")]
    certificates: Option<String>,
    /// Security key in PEM format
    #[arg(short = 'k', long = "key")]
    key: Option<String>,
    /// Delete log files when closing
    #[arg(short = 'd', long = "delete-logs", default_value_t = false)]
    delete_logs_on_exit: bool,
}

// embed web static files into 'Web' struct
#[derive(Embed)]
#[folder = "web"]
struct Web;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();
        match Web::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

// handlers for embedded static files
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();
    StaticFile(path)
}

async fn index_handler() -> impl IntoResponse {
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

// *** Server APIs ***
// Helper functions to extract and convert requests into API structs
/// attempt to downcast 'err' into a 'T' and if that fails recursively, try and downcast the err's source
fn find_error_source<'a, T>(err: &'a (dyn std::error::Error + 'static)) -> Option<&'a T>
where
    T: std::error::Error + 'static,
{
    if let Some(err) = err.downcast_ref::<T>() {
        Some(err)
    } else if let Some(source) = err.source() {
        find_error_source(source)
    } else {
        None
    }
}

/// attempt to extract the inner 'serde_path_to_error::Error<serde_json::Error>'
/// so we can provide a more specific error message
fn serde_json_error_response<E>(error: E) -> ApiError
where
    E: std::error::Error + 'static,
{
    let error_message: String;
    if let Some(error) = find_error_source::<serde_path_to_error::Error<serde_json::Error>>(&error)
    {
        let serde_json_err = error.inner();
        error_message = format!(
            "Invalid JSON at line {} column {}: {}",
            serde_json_err.line(),
            serde_json_err.column(),
            error
        );
    } else {
        error_message = format!("Unknown error: {error}");
    }
    error!("{}", error_message);
    ApiError::BadRequestError(error_message)
}

/// Extract the request's JSON body into the appropriate struct type
/// Send a BAD_REQUEST response and error message if the extraction fails.
fn extract_json_request<RequestType>(
    result: Result<Json<RequestType>, JsonRejection>,
) -> Result<RequestType, ApiError> {
    match result {
        Ok(Json(payload)) => {
            let request: RequestType = payload;
            Ok(request)
        }
        Err(error) => match error {
            JsonRejection::JsonDataError(error) => Err(serde_json_error_response(error)),
            JsonRejection::JsonSyntaxError(error) => Err(serde_json_error_response(error)),
            JsonRejection::MissingJsonContentType(_) => {
                let error_message = "Missing 'Content-Type; application/json' header".to_string();
                error!("{}", error_message);
                Err(ApiError::BadRequestError(error_message))
            }
            JsonRejection::BytesRejection(_) => {
                let error_message = "Failed to buffer request body".to_string();
                error!("{}", error_message);
                Err(ApiError::BadRequestError(error_message))
            }
            _ => {
                let error_message = "Unknown error".to_string();
                error!("{}", error_message);
                Err(ApiError::TypeError(
                    TypeError::ConversionError(error_message),
                ))
            }
        },
    }
}

// HTTP route handlers
/// Server Status APIs
#[tokio::test]
async fn test_server_status_get_status() {
    let _tracer = logger::tracer::test_log(logger::function_name!());
    shared::start_after_random_delay(RANDOM_DELAY).await;
    let request = StatusRequest {
        request_id: RequestId::new(),
        timestamp: Timestamp::now(),
    };
    let response_json = get_status(Ok(Json(request.clone()))).await.unwrap();
    assert_eq!(response_json.0.request, request);
}

async fn get_status(
    json_request: Result<Json<StatusRequest>, JsonRejection>,
) -> Result<Json<StatusResponse>, ApiError> {
    let request: StatusRequest = extract_json_request::<StatusRequest>(json_request)?;
    info!("Received request: {:?}", request);
    // Unpack and validate request
    let StatusRequest {
        request_id: _request_id,
        timestamp: _timestamp,
    } = request.clone();
    let response = StatusResponse {
        request,
        timestamp: Timestamp::now(),
    };
    info!("Sending response: {:?}", response);
    Ok(Json(response))
}


/// Ctrl-C handler
async fn shutdown_signal(tls_server_handle: axum_server::Handle<SocketAddr>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Keyboard interrupt (Ctrl-C) received.  Shutting down . . .")
        },
        _ = terminate => {
            info!("Interrupt signal received.  Shutting down . . .")
        },
    }

    // shutdown the TLS server
    tls_server_handle.graceful_shutdown(Some(std::time::Duration::from_secs(10))); // Docker waits 10 secs to force shutdown
}


struct TlsConfig {
    pub rust_tls_config: RustlsConfig,
    // keep the paths for the certificate and key files for use by other system components
    #[allow(dead_code)]
    pub certs_pathbuf: PathBuf,
    #[allow(dead_code)]
    pub key_pathbuf: PathBuf,
    // if we generate a self-signed certificate and key, we need to keep the tempfile
    // or it will be deleted before other system components can read it
    pub _maybe_certs_tempfile: Option<NamedTempFile>,
    pub _maybe_key_tempfile: Option<NamedTempFile>,
}

/// Setup TLS configuration
async fn get_tls_config(args: &ProgramArgs) -> Result<TlsConfig, ApiError> {
    // Setup default crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring as rustls crypto provider");

    // Get key and certs for TLS
    if args.certificates.is_some() && args.key.is_some() {
        let certs_pathbuf = PathBuf::from(args.certificates.as_ref().unwrap())
            .canonicalize()
            .expect("Error getting certificates full path");
        let key_pathbuf = PathBuf::from(args.key.as_ref().unwrap())
            .canonicalize()
            .expect("Error getting key full path");
        match RustlsConfig::from_pem_chain_file(certs_pathbuf.clone(), key_pathbuf.clone()).await {
            Ok(tls_config) => Ok(TlsConfig {
                rust_tls_config: tls_config,
                certs_pathbuf,
                key_pathbuf,
                _maybe_certs_tempfile: None,
                _maybe_key_tempfile: None,
            }),
            Err(error) => {
                let error_message = format!(
                    "Error getting security certificates and key for web_service_server TLS: {error}"
                );
                error!("{}", error_message);
                Err(ApiError::TypeError(
                    TypeError::ValidationError(error_message),
                ))
            }
        }
    } else if args.certificates.is_some() && args.key.is_none() {
        let cert = args.certificates.as_ref().unwrap();
        let error_message = format!(
            "TLS certificates {cert} was given, but no TLS key was given!  Both TLS certificates and key must be provided."
        );
        error!("{}", error_message);
        return Err(ApiError::TypeError(
            TypeError::ValidationError(error_message.to_string()),
        ));
    } else if args.certificates.is_none() && args.key.is_some() {
        let key = args.certificates.as_ref().unwrap();
        let error_message = format!(
            "TLS key {key} was given, but no TLS certificates were given!  Both TLS certificates and key must be provided."
        );
        error!("{}", error_message);
        return Err(ApiError::TypeError(
            TypeError::ValidationError(error_message.to_string()),
        ));
    } else {
        // both are none
        let warning_message = "No TLS certificates or key were provided.  Generating and using self-signed certificate and key for TLS configuration.";
        warn!("{}", warning_message);
        eprintln!("{warning_message}");
        let hostname = hostname();
        let subject_alt_names = vec![hostname, "localhost".to_string()];
        let CertifiedKey { cert, signing_key } = generate_simple_self_signed(subject_alt_names)?;
        // save the generated cert and key to named temp files so they can be used by other system
        // components if needed
        let cert_named_tempfile = write_string_to_named_tempfile("cert_", ".pem", &cert.pem()).await?;
        let key_named_tempfile = write_string_to_named_tempfile("key_", ".pem", &signing_key.serialize_pem()) .await?;
        let cert_path = cert_named_tempfile.path();
        let key_path = key_named_tempfile.path();

        match RustlsConfig::from_pem(
            cert.pem().into_bytes(),
            signing_key.serialize_pem().into_bytes(),
        )
        .await
        {
            Ok(tls_config) => Ok(TlsConfig {
                rust_tls_config: tls_config,
                certs_pathbuf: cert_path
                    .to_path_buf()
                    .canonicalize()
                    .expect("Error getting certificate full path for self-signed certificate"),
                key_pathbuf: key_path
                    .to_path_buf()
                    .canonicalize()
                    .expect("Error getting key full path for self-signed key"),
                _maybe_certs_tempfile: Some(cert_named_tempfile),
                _maybe_key_tempfile: Some(key_named_tempfile),
            }),
            Err(error) => {
                let error_message = format!(
                    "Error generating self-signed security certificates and key for web_service_server TLS: {error}"
                );
                error!("{}", error_message);
                Err(ApiError::TypeError(
                    TypeError::ValidationError(error_message),
                ))
            }
        }
    }
}


#[tokio::main]
async fn main() -> ExitCode {
    // parse command line args
    let args = ProgramArgs::parse();
    // start logger
    let _server_logger =
        match tracer::init_logger("/var/tmp", "server", args.delete_logs_on_exit) {
            Ok(server_logger) => {
                info!("web_service server logging started");
                server_logger
            }
            Err(error) => {
                eprintln!("Error starting web_service server logging: {error}");
                return ExitCode::FAILURE;
            }
        };
    info!("Args are: {:?}", args);

    // Get key and certs for TLS
    let tls_config = match get_tls_config(&args).await {
        Ok(the_tls_config) => the_tls_config,
        Err(error) => {
            eprintln!("Error getting web_service server TLS configuration: {error}");
            return ExitCode::FAILURE;
        }
    };

    // create a handle for our TLS server so the shutdown signal can all shutdown
    let tls_server_handle = axum_server::Handle::new();
    // save the future for easy shutting down of redirect server
    let _shutdown_future = shutdown_signal(tls_server_handle.clone());

    info!("Defining server routes");
    let app = Router::<()>::new()
        // API-defined routes
        .route(STATUS_PATH, put(get_status))
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/{*file}", get(static_handler));
    // start server
    info!("Starting web_service server on port {}", args.port);
    println!("Starting web_service server on port {}", args.port);
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    axum_server::bind_rustls(addr, tls_config.rust_tls_config)
        .handle(tls_server_handle)
        .serve(app.into_make_service())
        .await
        .unwrap();

    ExitCode::SUCCESS
}
