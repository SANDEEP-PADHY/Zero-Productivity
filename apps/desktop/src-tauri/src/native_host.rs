use std::io::{self, Read};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::io::AsyncWriteExt;
use crate::collector::ipc::PIPE_NAME;

pub async fn run_native_host() {
    let mut pipe_client = match ClientOptions::new().open(PIPE_NAME) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Native host could not connect to desktop pipe: {}", e);
            return;
        }
    };

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    
    loop {
        let mut len_buf = [0u8; 4];
        if handle.read_exact(&mut len_buf).is_err() {
            break;
        }
        let len = u32::from_ne_bytes(len_buf) as usize;
        
        if len > 1024 * 1024 { // 1MB max
            break;
        }

        let mut msg_buf = vec![0u8; len];
        if handle.read_exact(&mut msg_buf).is_err() {
            break;
        }

        if pipe_client.write_all(&msg_buf).await.is_err() {
            break;
        }
    }
}
