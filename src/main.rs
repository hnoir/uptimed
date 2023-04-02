use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use signal_hook::consts as sigconsts;
use signal_hook::iterator::Signals;

mod config;
mod proc;
mod serde_duration;

const CONFIG_NOTICE: &str = r#"
Configuration can be specified in arguments using the -c flag or created in the default path $HOME/.config/uptimed/config.ini

Here's an example configuration:
-----------------------------------------------------------
# Path to the file containing target URLs.
targets_path: "/path/to/targets"

# How much time between requests?
request_interval: 0s

# How much time between one complete scan and the next one?
scan_interval: 15m

# List of custom HTTP headers and values to use in every request.
custom_headers:
  - name: "X-MyHeader"
    value: "my-value"
  - name: "Authorization"
    value: "Bearer token"

"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut config_path = match args.iter().position(|arg| arg == "-c") {
        Some(index) => {
            let path = PathBuf::from(&args[index + 1]);
            if !path.exists() {
                eprintln!("Configuration file does not exist: {}", path.display());
                process::exit(1);
            }
            path
        }
        None => {
            let mut path = dirs::config_dir().unwrap_or_default();
            path.push("uptimed/config.yml");
            if !path.exists() {
                if let Err(e) = fs::create_dir_all(path.parent().unwrap()) {
                    eprintln!("Could not create config directory: {}", e);
                    process::exit(1);
                }
                println!(
                    "A configuration file is needed for uptimed to work.\n{}",
                    CONFIG_NOTICE
                );
                process::exit(0);
            }
            path
        }
    };

    if config_path.to_str().unwrap() == "" {
        config_path = PathBuf::from("~/.config/uptimed/config.yml");
    }

    let config = match config::load_configuration(&config_path.to_string_lossy()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = config.is_valid() {
        eprintln!("Invalid configuration: {}", e);
        std::process::exit(1);
    }

    let running = Arc::new(AtomicBool::new(true));

    // Create a signal handler thread to listen for SIGKILL signals
    let signal_running = running.clone();
    thread::spawn(move || {
        let mut signals = Signals::new(&[sigconsts::SIGTERM, sigconsts::SIGINT]).unwrap();
        for sig in signals.forever() {
            signal_running.store(false, Ordering::SeqCst);
            println!("Received signal: {:?}", sig);
            break;
        }
    });

    let mut last_run = Instant::now();
    while running.load(Ordering::SeqCst) {
        if Instant::now().duration_since(last_run) >= config.scan_interval {
            match proc::process_urls(&config).await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("An error occurred: {}", e);
                    process::exit(1);
                }
            }
            last_run = Instant::now();
        }
        // Sleep for a short time to avoid hogging the CPU
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}
