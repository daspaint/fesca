use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use log::{info, warn};

/// If env var TCP_ECHO_ADDR is set (e.g. "0.0.0.0:6000"), start a TCP echo server
/// in the background and return Some(addr). Otherwise do nothing and return None.
pub async fn maybe_start_tcp_echo() -> Option<String> {
    let addr = match std::env::var("TCP_ECHO_ADDR") {
        Ok(s) if !s.is_empty() => s,
        _ => return None, // disabled by default
    };

    // Spawn the server task
    let addr_clone = addr.clone();
    tokio::spawn(async move {
        if let Err(e) = run(addr_clone).await {
            warn!("TCP echo server error: {e}");
        }
    });

    Some(addr)
}

async fn run(addr: String) -> anyhow::Result<()> {
    info!("TCP_benchmark server listening on {addr}");
    let listener = TcpListener::bind(&addr).await?;
    loop {
        let (mut s, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut len_buf = [0u8; 4];
            loop {
                if s.read_exact(&mut len_buf).await.is_err() { break; }
                let len = u32::from_be_bytes(len_buf) as usize;
                let mut data = vec![0u8; len];
                if s.read_exact(&mut data).await.is_err() { break; }
                if s.write_all(&len_buf).await.is_err() { break; }
                if s.write_all(&data).await.is_err() { break; }
            }
        });
    }
}
