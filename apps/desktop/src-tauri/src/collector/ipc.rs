use tokio::net::windows::named_pipe::ServerOptions;
use tokio::io::AsyncReadExt;
use zero_core::collector::EventSender;
use zero_core::models::{Observation, BrowserStatePayload};
use chrono::Utc;
use windows::Win32::System::SystemInformation::GetTickCount64;

pub const PIPE_NAME: &str = r"\\.\pipe\zero-productivity-browser";

pub async fn start_ipc_server(sender: EventSender) {
    loop {
        let server = match ServerOptions::new()
            .first_pipe_instance(true)
            .create(PIPE_NAME) 
        {
            Ok(s) => s,
            Err(_) => {
                match ServerOptions::new().create(PIPE_NAME) {
                    Ok(s) => s,
                    Err(_) => {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }
        };

        match server.connect().await {
            Ok(_) => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    handle_client(server, sender).await;
                });
            }
            Err(e) => {
                eprintln!("Failed to connect to named pipe: {}", e);
            }
        }
    }
}

async fn handle_client(mut server: tokio::net::windows::named_pipe::NamedPipeServer, sender: EventSender) {
    let mut buffer = [0u8; 8192];
    loop {
        match server.read(&mut buffer).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                // The native host sends newline-delimited JSON or just raw JSON depending on how it's written.
                // We'll assume the native host sends exactly one JSON payload per write for now.
                if let Ok(payload) = serde_json::from_slice::<BrowserStatePayload>(&buffer[..n]) {
                    let now = Utc::now();
                    let monotonic = unsafe { GetTickCount64() };
                    let obs = Observation::new_browser_state(now, monotonic, payload, "win".into());
                    let _ = sender.send(obs);
                }
            }
            Err(_) => break,
        }
    }
}
