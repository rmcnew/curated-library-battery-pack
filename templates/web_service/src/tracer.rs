use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use tracing::Level;
use tracing::subscriber::DefaultGuard;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::Rotation;

pub struct Tracer {
    pub log_directory: PathBuf,
    pub log_path: PathBuf,
    pub delete_logs_on_exit: bool,
    _worker_guard: WorkerGuard,
    _default_guard: Option<DefaultGuard>, // this is needed as a thread-local handle for unit tests
}

// deleting log files on exit is handled by a custom drop function
impl Drop for Tracer {
    fn drop(&mut self) {
        if self.delete_logs_on_exit {
            // delete log files
            for maybe_entry in fs::read_dir(&self.log_directory).unwrap() {
                let entry = maybe_entry.unwrap();
                let entry_path = entry.path();
                // note that Path.starts_with treats filenames with extensions differently from str.starts_with
                // Path.starts_with returns false unless the Path's filename matches, so we use the str.starts_with
                // which gives the desired prefix matching behavior
                if entry_path.is_file()
                    && (entry_path == self.log_path
                        || entry_path
                            .to_str()
                            .unwrap()
                            .starts_with(self.log_path.to_str().unwrap()))
                {
                    fs::remove_file(entry_path).unwrap();
                }
            }
        }
    }
}

pub const MAX_LOG_ARCHIVES: usize = 3;
pub const APP_LOG_LEVEL: Level = Level::INFO;
pub fn init_logger(
    log_directory: &str,
    log_basename: &str,
    delete_logs_on_exit: bool,
) -> Result<Tracer, std::io::Error> {
    // We use the process ID as part of the log filename to try to avoid filename conflicts
    let pid = process::id();
    let log_filename = format!("{log_basename}_PID{pid}");
    let log_path = PathBuf::from(log_directory).join(format!("{log_filename}.log"));

    match tracing_appender::rolling::Builder::new()
        .rotation(Rotation::HOURLY)
        .filename_prefix(log_filename)
        .filename_suffix("log")
        .max_log_files(MAX_LOG_ARCHIVES)
        .build(log_directory)
    {
        Ok(file_appender) => {
            let (non_blocking, _worker_guard) = tracing_appender::non_blocking(file_appender);
            tracing_subscriber::fmt()
                .with_writer(non_blocking) // write log file to non-blocking, rolling appender
                .with_ansi(false) // do not add terminal colors
                //.with_span_events(FmtSpan::ACTIVE)  // show enter and exit of code blocks
                .with_file(true) // show the source code file where the log event came from
                .with_line_number(true) // show the line in the source code file
                .with_level(true) // show the log level
                .with_max_level(APP_LOG_LEVEL) // set the log level filter
                .with_thread_ids(true) // show the thread ID
                .init(); // install to global tracer

            Ok(Tracer {
                log_directory: Path::new(log_directory).to_path_buf(),
                log_path,
                delete_logs_on_exit,
                _worker_guard,
                _default_guard: None,
            })
        }
        Err(error) => {
            let error_message = format!("Error initializing logging: {error}");
            Err(std::io::Error::other(error_message))
        }
    }
}

pub const TEST_LOG_DIRECTORY: &str = "/var/tmp";
pub const TEST_LOG_LEVEL: Level = Level::INFO;
/// configure a tracing subscriber for running tests
pub fn test_log(log_filename: &str) -> Tracer {
    let log_path = Path::new(TEST_LOG_DIRECTORY).join(log_filename);

    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(Rotation::NEVER)
        .filename_prefix(log_filename)
        .filename_suffix("log")
        .max_log_files(1)
        .build(TEST_LOG_DIRECTORY)
        .unwrap_or_else(|_| {
            panic!("Error creating test log {log_filename} at {TEST_LOG_DIRECTORY}")
        });
    let (non_blocking, _worker_guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(non_blocking) // write log file to non-blocking, rolling appender
        .with_ansi(false) // do not add terminal colors
        //.with_span_events(FmtSpan::ACTIVE)  // show enter and exit of code blocks
        .with_file(true) // show the source code file where the log event came from
        .with_line_number(true) // show the line in the source code file
        .with_level(true) // show the log level
        .with_max_level(TEST_LOG_LEVEL) // set the log level filter
        .with_thread_ids(true) // show the thread ID
        .finish(); // build the tracer, but do not install globally
    let _default_guard = tracing::subscriber::set_default(subscriber);
    Tracer {
        log_directory: TEST_LOG_DIRECTORY.into(),
        log_path,
        delete_logs_on_exit: false, // keep test logs
        _worker_guard,
        _default_guard: Some(_default_guard),
    }
}
