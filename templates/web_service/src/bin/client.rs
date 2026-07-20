// command line client
use clap::{Args, Parser, Subcommand};
use web_service::server_api::*;
use web_service::shared::*;
use web_service::tracer;
use reqwest::tls::{Certificate, Identity};
use tracing::{error, info, warn};
use url::Url;

/// Errors in types and their uses
#[derive(thiserror::Error)]
#[non_exhaustive]
pub enum ClientError {
    /// Wrap a TypeError
    #[error("{0}")]
    TypeError(TypeError),
    /// Error connecting to server
    #[error("Server Connection error: {0}")]
    ServerConnectionError(String),
}

impl std::fmt::Debug for ClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let error_string = match self {
            Self::TypeError(message) => {
                format!("{message}")
            }
            Self::ServerConnectionError(message) => message.to_string(),
        };
        write!(formatter, "{error_string}")
    }
}

impl From<TypeError> for ClientError {
    fn from(error: TypeError) -> Self {
        Self::TypeError(error)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(error: std::io::Error) -> Self {
        Self::TypeError(TypeError::from(error))
    }
}

impl From<url::ParseError> for ClientError {
    fn from(error: url::ParseError) -> Self {
        Self::TypeError(TypeError::from(error))
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_builder() || error.is_body() || error.is_decode() {
            Self::TypeError(TypeError::from(error))
        } else {
            let error_root_cause = error_root_cause(&error);
            let error_message = format!("Error sending request to web_service server: {error_root_cause}");
            Self::ServerConnectionError(error_message)
        }
    }
}

/// environment variable key for web_service server URL
const WEB_SERVICE_SERVER_URL: &str = "WEB_SERVICE_SERVER_URL";

/// Get the web_service server URL in priority order:
/// 1) command line arg
/// 2) environment variable
fn get_web_service_server_url(args: &ProgramArgs) -> Result<Url, ClientError> {
    match args.web_service_server_url.clone() {
        Some(web_service_server_url_string) => {
            let web_service_server_url = Url::parse(&web_service_server_url_string)?;
            info!(
                "Using web_service server URL from command args: {}",
                web_service_server_url.as_str()
            );
            Ok(web_service_server_url)
        }
        None => {
            info!("No web_service server URL provided in command args; checking environment variable");
            match std::env::var(WEB_SERVICE_SERVER_URL) {
                Ok(web_service_server_url_string) => {
                    let web_service_server_url = Url::parse(&web_service_server_url_string)?;
                    info!(
                        "Using web_service server URL from environment variable: {}",
                        web_service_server_url.as_str()
                    );
                    Ok(web_service_server_url)
                }
                Err(error) => {
                    let error_message = format!(
                        "No web_service server URL provided! Use --web_service-server-url argument or set WEB_SERVICE_SERVER_URL environment variable.  Details: {error}"
                    );
                    error!("{}", error_message);
                    Err(ClientError::TypeError(
                        TypeError::ValidationError(error_message),
                    ))
                }
            }
        }
    }
}

/// environment variable key for web_service server certificate
const WEB_SERVICE_SERVER_CERTIFICATES: &str = "WEB_SERVICE_SERVER_CERTIFICATES";

/// Get the web_service server certificate in priority order:
/// 1) command line arg
/// 2) environment variable
async fn get_web_service_server_certificates(
    args: &ProgramArgs,
) -> Result<Vec<Certificate>, ClientError> {
    match args.web_service_server_certificates.clone() {
        Some(web_service_server_certificates_string) => {
            info!(
                "Getting web_service server certificates from '--web_service-server-certificates' file: {}",
                web_service_server_certificates_string
            );
            let contents = tokio::fs::read(web_service_server_certificates_string).await?;
            let web_service_server_certificates = Certificate::from_pem_bundle(&contents)?;
            Ok(web_service_server_certificates)
        }
        None => {
            info!(
                "No web_service server certificates provided in command args; checking {} environment variable",
                WEB_SERVICE_SERVER_CERTIFICATES
            );
            match std::env::var(WEB_SERVICE_SERVER_CERTIFICATES) {
                Ok(web_service_server_certificates_string) => {
                    info!(
                        "Getting web_service server certificates from {} environment variable file: {}",
                        WEB_SERVICE_SERVER_CERTIFICATES, web_service_server_certificates_string
                    );
                    let contents = tokio::fs::read(web_service_server_certificates_string).await?;
                    let web_service_server_certificates = Certificate::from_pem_bundle(&contents)?;
                    Ok(web_service_server_certificates)
                }
                Err(_error) => {
                    let warning_message = "No web_service server certificates provided! Using default web certificates. Use --web_service-server-certificates argument or set WEB_SERVICE_SERVER_CERTIFICATES environment variable if additional certificates are needed.";
                    warn!("{}", warning_message);
                    Ok(Vec::new())
                }
            }
        }
    }
}

/// environment variable key for web_service client identity
const WEB_SERVICE_CLIENT_IDENTITY: &str = "WEB_SERVICE_CLIENT_IDENTITY";

/// Get the web_service client identity in priority order:
/// 1) command line arg
/// 2) environment variable
async fn get_web_service_client_identity(
    args: &ProgramArgs,
) -> Result<Option<Identity>, ClientError> {
    match args.web_service_client_identity.clone() {
        Some(web_service_client_identity_string) => {
            info!(
                "Getting web_service client identity from '--web_service-client-identity' file: {}",
                web_service_client_identity_string
            );
            let identity_contents = tokio::fs::read(web_service_client_identity_string).await?;
            let web_service_client_identity = Identity::from_pem(&identity_contents)?;
            Ok(Some(web_service_client_identity))
        }
        None => {
            info!(
                "No web_service client identity provided in command args; checking {} environment variable",
                WEB_SERVICE_CLIENT_IDENTITY
            );
            match std::env::var(WEB_SERVICE_CLIENT_IDENTITY) {
                Ok(web_service_client_identity_string) => {
                    info!(
                        "Getting web_service client identity from {} environment variable file: {}",
                        WEB_SERVICE_CLIENT_IDENTITY, web_service_client_identity_string
                    );
                    let identity_contents = tokio::fs::read(web_service_client_identity_string).await?;
                    let web_service_client_identity = Identity::from_pem(&identity_contents)?;
                    Ok(Some(web_service_client_identity))
                }
                Err(_error) => {
                    let warning_message = "No web_service client identity provided! Use --web_service-client-identity argument or set WEB_SERVICE_CLIENT_IDENTITY environment variable if client identity is required.";
                    warn!("{}", warning_message);
                    Ok(None)
                }
            }
        }
    }
}

/// environment variable key for Allow Server Self-Signed certificate
const WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT: &str = "WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT";

/// Get the whether the web_service server can use a self-signed certificate in priority order:
/// 1) command line arg
/// 2) environment variable
fn get_web_service_server_allow_self_signed_cert(
    args: &ProgramArgs,
) -> Result<bool, ClientError> {
    match args.web_service_server_allow_self_signed_cert {
        Some(web_service_server_allow_self_signed_cert) => {
            if web_service_server_allow_self_signed_cert {
                let warning_message = "'--allow-self-signed-cert' requested.  web_service server identity will NOT be verified.";
                warn!("{}", warning_message);
                eprintln!("{warning_message}");
            }
            Ok(web_service_server_allow_self_signed_cert)
        }
        None => {
            info!(
                "'--allow-self-signed-cert' not given in command args; checking {} environment variable",
                WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT
            );
            match std::env::var(WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT) {
                Ok(web_service_server_allow_self_signed_cert_string) => {
                    match web_service_server_allow_self_signed_cert_string.parse::<bool>() {
                        Ok(web_service_server_allow_self_signed_cert) => {
                            if web_service_server_allow_self_signed_cert {
                                let warning_message = format!(
                                    "Environment variable {WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT} set to {web_service_server_allow_self_signed_cert}.  web_service server identity will NOT be verified."
                                );
                                warn!("{}", warning_message);
                                println!("{warning_message}");
                            } else {
                                let message = format!(
                                    "Environment variable {WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT} set to {web_service_server_allow_self_signed_cert}.  web_service server identity WILL be verified."
                                );
                                info!("{}", message);
                            }
                            Ok(web_service_server_allow_self_signed_cert)
                        }
                        Err(error) => {
                            // could not parse environment variable as bool
                            let error_message = format!(
                                "Error parsing environment variable {WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT} value {web_service_server_allow_self_signed_cert_string} as boolean:  {error}.  Please set {WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT} as 'true' or 'false'."
                            );
                            error!("{}", error_message);
                            Err(ClientError::TypeError(
                                TypeError::ValidationError(error_message),
                            ))
                        }
                    }
                }
                Err(_error) => {
                    let message = format!(
                        "'--allow-self-signed-cert' not requested and {WEB_SERVICE_SERVER_ALLOW_SELF_SIGNED_CERT} environment variable not set to true.  web_service server identity WILL be verified."
                    );
                    info!("{}", message);
                    Ok(false)
                }
            }
        }
    }
}

/// environment variable key for Do Not Veriy Server Hostname
const WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME: &str =
    "WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME";

/// Get the whether the web_service server hostname will be verified against the certificate it presents:
/// 1) command line arg
/// 2) environment variable
fn get_web_service_server_do_not_verify_server_hostname(
    args: &ProgramArgs,
) -> Result<bool, ClientError> {
    match args.web_service_server_do_not_verify_server_hostname {
        Some(web_service_server_do_not_verify_server_hostname) => {
            if web_service_server_do_not_verify_server_hostname {
                let warning_message = "'--do-not-verify-server-hostname' requested.  web_service server hostname will NOT be verified.";
                warn!("{}", warning_message);
                eprintln!("{warning_message}");
            }
            Ok(web_service_server_do_not_verify_server_hostname)
        }
        None => {
            info!(
                "'--do-not-verify-server-hostname' not given in command args; checking {} environment variable",
                WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME
            );
            match std::env::var(WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME) {
                Ok(web_service_server_do_not_verify_server_hostname_string) => {
                    match web_service_server_do_not_verify_server_hostname_string.parse::<bool>() {
                        Ok(web_service_server_do_not_verify_server_hostname) => {
                            if web_service_server_do_not_verify_server_hostname {
                                let warning_message = format!(
                                    "Environment variable {WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME} set to {web_service_server_do_not_verify_server_hostname}.  web_service server hostname will NOT be verified."
                                );
                                warn!("{}", warning_message);
                                println!("{warning_message}");
                            } else {
                                let message = format!(
                                    "Environment variable {WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME} set to {web_service_server_do_not_verify_server_hostname}.  web_service server hostname WILL be verified."
                                );
                                info!("{}", message);
                            }
                            Ok(web_service_server_do_not_verify_server_hostname)
                        }
                        Err(error) => {
                            // could not parse environment variable as bool
                            let error_message = format!(
                                "Error parsing environment variable {WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME} value {web_service_server_do_not_verify_server_hostname_string} as boolean:  {error}.  Please set {WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME} as 'true' or 'false'."
                            );
                            error!("{}", error_message);
                            Err(ClientError::TypeError(
                                TypeError::ValidationError(error_message),
                            ))
                        }
                    }
                }
                Err(_error) => {
                    let message = format!(
                        "'--do-not-verify-server-hostname' not requested and {WEB_SERVICE_SERVER_DO_NOT_VERIFY_SERVER_HOSTNAME} environment variable not set to true.  web_service server hostname WILL be verified."
                    );
                    info!("{}", message);
                    Ok(false)
                }
            }
        }
    }
}


/// web_service CLI client command-line arguments
#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about=None)]
#[command(propagate_version = true)]
struct ProgramArgs {
    /// Command to use
    #[command(subcommand)]
    command: Command,
    /// Url for the web_service server
    #[arg(short = 's', long = "web_service-server-url")]
    web_service_server_url: Option<String>,
    /// Use the provided custom certificates in PEM bundle format to verify server identity
    #[arg(short = 'b', long = "web_service-server-certificates")]
    web_service_server_certificates: Option<String>,
    /// Use the provided client identity (private key and certificate) in PEM format
    #[arg(short = 'a', long = "web_service-client-identity")]
    web_service_client_identity: Option<String>,
    /// Allow web_service server to use self-signed certificate?
    /// DANGER! web_service server identity will not be assured
    #[arg(short = 'y', long = "allow-self-signed-cert")]
    web_service_server_allow_self_signed_cert: Option<bool>,
    /// Do not verify web_service server hostname
    /// DANGER! web_service server identity will not be assured
    #[arg(short = 'w', long = "do-not-verify-server-hostname")]
    web_service_server_do_not_verify_server_hostname: Option<bool>,
    /// Delete log files when closing?
    #[arg(short = 'd', long = "delete-logs", default_value_t = true)]
    delete_logs_on_exit: bool,
}

/// Sub-commands for the CLI client
#[derive(Clone, Debug, Subcommand)]
enum Command {
    Status(StatusArgs),
    // Add other commands here
}

/// Traits to convert args to a request
#[allow(dead_code)]
trait ToRequest {
    type Req: serde::ser::Serialize;
    fn to_request(&self) -> Result<Self::Req, ClientError>;
}

#[allow(dead_code)]
trait ToRequestAsync {
    type Req: serde::ser::Serialize;
    async fn to_request(&self) -> Result<Self::Req, ClientError>;
}

// Command argument types
#[derive(Clone, Debug, Args)]
struct StatusArgs {
    // The status command does not have any args
}

impl ToRequest for StatusArgs {
    type Req = StatusRequest;
    fn to_request(&self) -> Result<Self::Req, ClientError> {
        let request = Self::Req {
            // command args would be parsed from the command line
            // or whatever here
            request_id: RequestId::new(),
            timestamp: Timestamp::now(),
        };
        Ok(request)
    }
}

/// Convert args to the proper request type and then
/// send the request to the web_service server.  Convert
/// the response to a CLI-friendly printable version
/// or a human-friendly error message
/// This server_request function is for simple command line arguments
/// that do not require async function calls.  For example, parsing
/// a command line parameter or similar.
async fn server_request<Response>(
    client: &reqwest::Client,
    web_service_server_url: &url::Url,
    args: &impl ToRequest,
    server_path: &str,
) -> Result<(), ClientError>
where
    Response: CommandLineDisplay + serde::de::DeserializeOwned,
{
    let request = args.to_request()?;
    let request_url = web_service_server_url.join(server_path)?;
    match client.put(request_url.as_str()).json(&request).send().await {
        Ok(response) => { // successful comms with server
            match response.status().is_success() {
                true => {  // HTTP response 200 (Ok)
                    // Get response as JSON and deserialize to Response type
                    match response.json::<Response>().await {
                        // Success deserializing to expected Response type
                        Ok(payload) => {
                            payload.print();
                            Ok(())
                        }
                        // Error deserializing to expected Response type
                        Err(error) => {
                            error!("Error with response: {error}");
                            Err(error.into())
                        }
                    }
                }
                false => {  // HTTP non-Ok response; get response as text
                    match response.text().await {
                        Ok(text) => {
                            warn!("Non-Ok response from server: {text}");
                            Ok(())
                        }
                        Err(error) => {
                            error!("Error getting response as text: {error}");
                            Err(error.into())
                        }
                    }
                }
            }
        }
        Err(error) => { // error communicating with server
            error!("Error reaching server: {error}");
            Err(error.into())
        }
    }
}

/// Convert args to the proper request type using an async function
/// Send the request to the web_service server and the convert and print
/// the result or error message.  
/// This server_request function is for more complex client-side handling
/// that involves async function calls.  For example reading the contents 
/// of a file or waiting for a resource to be ready.
#[allow(dead_code)]
async fn server_request_async_to_request<Response>(
    client: &reqwest::Client,
    web_service_server_url: &url::Url,
    args: &impl ToRequestAsync,
    server_path: &str,
) -> Result<(), ClientError>
where
    Response: CommandLineDisplay + serde::de::DeserializeOwned,
{
    let request = args.to_request().await?;
    let request_url = web_service_server_url.join(server_path)?;
    match client.put(request_url.as_str()).json(&request).send().await {
        Ok(response) => { // successful comms with server
            match response.status().is_success() {
                true => {  // HTTP response 200 (Ok)
                    // Get response as JSON and deserialize to Response type
                    match response.json::<Response>().await {
                        // Success deserializing to expected Response type
                        Ok(payload) => {
                            payload.print();
                            Ok(())
                        }
                        // Error deserializing to expected Response type
                        Err(error) => {
                            error!("Error with response: {error}");
                            Err(error.into())
                        }
                    }
                }
                false => {  // HTTP non-Ok response; get response as text
                    match response.text().await {
                        Ok(text) => {
                            warn!("Non-Ok response from server: {text}");
                            Ok(())
                        }
                        Err(error) => {
                            error!("Error getting response as text: {error}");
                            Err(error.into())
                        }
                    }
                }
            }
        }
        Err(error) => { // error communicating with server
            error!("Error reaching server: {error}");
            Err(error.into())
        }
    }
}

/// Trait for formatting web_service server responses for CLI printing
trait CommandLineDisplay {
    // print output for a web_service server response
    fn print(&self);
}

impl CommandLineDisplay for StatusResponse {
    fn print(&self) {
        println!("web_service server is up and handling requests");
    }
}

// Implement CommandLineDisplay trait for other commands here

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // parse command line args
    let args = ProgramArgs::parse();
    // start logger
    let _client_logger = tracer::init_logger("/var/tmp", "client", args.delete_logs_on_exit)?;
    info!("Args are: {:?}", args);

    // Setup default crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring as rustls crypto provider");

    // get the needed parameters from command line args and environment variables
    let web_service_server_url = get_web_service_server_url(&args)?;
    let web_service_server_certificates = get_web_service_server_certificates(&args).await?;
    let maybe_web_service_client_identity = get_web_service_client_identity(&args).await?;
    let web_service_server_allow_self_signed_cert = get_web_service_server_allow_self_signed_cert(&args)?;
    let web_service_server_do_not_verify_server_hostname =
        get_web_service_server_do_not_verify_server_hostname(&args)?;

    // start building HTTP / HTTPS client configuration
    let mut client_builder = reqwest::Client::builder();

    // add built-in certificates and Web PKI CAs
    client_builder = client_builder.use_rustls_tls();

    if !web_service_server_certificates.is_empty() {
        for cert in web_service_server_certificates {
            client_builder = client_builder.add_root_certificate(cert);
        }
    }
    if maybe_web_service_client_identity.is_some() {
        let web_service_client_identity = maybe_web_service_client_identity.unwrap();
        client_builder = client_builder.identity(web_service_client_identity);
    }
    if web_service_server_allow_self_signed_cert {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    if web_service_server_do_not_verify_server_hostname {
        client_builder = client_builder.danger_accept_invalid_hostnames(true);
    }
    // Finish initialization
    let client = client_builder.build()?;

    // Build request struct
    match &args.command {
        Command::Status(status_args) => {
            server_request::<StatusResponse>(
                &client, 
                &web_service_server_url, 
                status_args, 
                STATUS_PATH
            ).await?;
        }
        // Other commands go here
    }
    Ok(())
}
