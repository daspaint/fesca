use anyhow::Result;
use clap::{Parser, Subcommand, Args};
use hdrhistogram::Histogram;
use std::time::Instant;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tonic::transport::Channel;

// Reuse the Echo proto generated inside the computing_node crate
// (your computing_node/bench_echo_service.rs has `pub mod bench { tonic::include_proto!("bench"); }`)
use computing_node::bench_echo_service::bench::{self, echo_client::EchoClient, Payload};

#[derive(Parser, Debug)]
#[command(name="fesca-bench", about="FESCA networking & serde microbench")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// gRPC echo: end-to-end RTT of one round-trip, with payload & concurrency
    Grpc(CommArgs),

    /// TCP echo: bare-bones transport (enable TCP_ECHO_ADDR on computing_node)
    Tcp(TcpArgs),

    /// Protobuf (prost) encode/decode only (no networking)
    Serde(SerdeArgs),

    /// Sweep payload sizes (grpc or tcp) in one run
    Sweep(SweepArgs),
}

#[derive(Args, Debug, Clone)]
struct CommArgs {
    /// gRPC target; must include scheme, e.g. http://host:50051
    #[arg(long, default_value="http://127.0.0.1:50051")]
    target: String,
    /// message size in bytes
    #[arg(long, default_value_t=10_000)]
    size: usize,
    /// total requests
    #[arg(long, default_value_t=500)]
    iters: usize,
    /// parallel tasks
    #[arg(long, default_value_t=16)]
    conc: usize,
    /// warmups
    #[arg(long, default_value_t=50)]
    warmup: usize,
}

#[derive(Args, Debug, Clone)]
struct TcpArgs {
    /// TCP target host:port (e.g., host:6000). Start computing_node with TCP_ECHO_ADDR.
    #[arg(long, default_value="127.0.0.1:6000")]
    target: String,
    #[arg(long, default_value_t=10_000)]
    size: usize,
    #[arg(long, default_value_t=500)]
    iters: usize,
    #[arg(long, default_value_t=16)]
    conc: usize,
    #[arg(long, default_value_t=50)]
    warmup: usize,
}

#[derive(Args, Debug, Clone)]
struct SerdeArgs {
    /// Payload size in bytes
    #[arg(long, default_value_t=10_000)]
    size: usize,
    /// Iterations
    #[arg(long, default_value_t=10_000)]
    iters: usize,
}

#[derive(Args, Debug, Clone)]
struct SweepArgs {
    /// Mode: grpc or tcp
    #[arg(long, value_parser = ["grpc","tcp"], default_value="grpc")]
    mode: String,
    /// Target (grpc: http://host:50051, tcp: host:6000)
    #[arg(long)]
    target: String,
    /// Comma-separated sizes (bytes), e.g. 1000,10000,100000,1000000
    #[arg(long, default_value="1000,10000,100000,1000000")]
    sizes: String,
    /// Parallel tasks
    #[arg(long, default_value_t=16)]
    conc: usize,
    /// Total requests per size
    #[arg(long, default_value_t=500)]
    iters: usize,
    /// Warmups
    #[arg(long, default_value_t=50)]
    warmup: usize,
}

#[tokio::main(flavor="multi_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Grpc(a) => bench_grpc(a).await,
        Cmd::Tcp(a) => bench_tcp(a).await,
        Cmd::Serde(a) => bench_serde(a),
        Cmd::Sweep(a) => sweep(a).await,
    }
}

// ---------- gRPC echo ----------
async fn make_client(target: &str) -> Result<EchoClient<Channel>> {
    let ch = Channel::from_shared(target.to_string())?
        .tcp_nodelay(true)
        .connect()
        .await?;
    Ok(EchoClient::new(ch))
}

async fn bench_grpc(args: CommArgs) -> Result<()> {
    // warmup
    {
        let mut c = make_client(&args.target).await?;
        for _ in 0..args.warmup {
            let _ = rpc_once(&mut c, args.size).await?;
        }
    }
    // run
    let mut hist = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3)?;
    let per = args.iters / args.conc;
    let start = Instant::now();

    let mut tasks = Vec::new();
    for _ in 0..args.conc {
        let t = args.target.clone();
        let sz = args.size;
        tasks.push(tokio::spawn(async move {
            let mut c = make_client(&t).await.unwrap();
            let mut local = Vec::with_capacity(per);
            for _ in 0..per {
                let us = rpc_once(&mut c, sz).await.unwrap();
                local.push(us as u64);
            }
            local
        }));
    }
    let mut n = 0usize;
    for h in tasks {
        for v in h.await.unwrap() {
            let _ = hist.record(v);
            n += 1;
        }
    }
    let rps = (n as f64) / start.elapsed().as_secs_f64();
    print_hist("gRPC", args.size, &args.target, &hist, rps);
    Ok(())
}

async fn rpc_once(cli: &mut EchoClient<Channel>, bytes: usize) -> Result<u128> {
    let msg = Payload { data: vec![0u8; bytes].into() };
    let t0 = Instant::now();
    let _ = cli.ping(tonic::Request::new(msg)).await?;
    Ok(t0.elapsed().as_micros())
}

// ---------- TCP echo ----------
async fn bench_tcp(args: TcpArgs) -> Result<()> {
    // warmup
    {
        let mut s = TcpStream::connect(&args.target).await?;
        for _ in 0..args.warmup {
            let _ = tcp_once(&mut s, args.size).await?;
        }
    }
    // run
    let mut hist = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3)?;
    let per = args.iters / args.conc;
    let start = Instant::now();

    let mut tasks = Vec::new();
    for _ in 0..args.conc {
        let t = args.target.clone();
        let sz = args.size;
        tasks.push(tokio::spawn(async move {
            let mut s = TcpStream::connect(&t).await.unwrap();
            let mut local = Vec::with_capacity(per);
            for _ in 0..per {
                let us = tcp_once(&mut s, sz).await.unwrap();
                local.push(us as u64);
            }
            local
        }));
    }
    let mut n = 0usize;
    for h in tasks {
        for v in h.await.unwrap() {
            let _ = hist.record(v);
            n += 1;
        }
    }
    let rps = (n as f64) / start.elapsed().as_secs_f64();
    print_hist("TCP", args.size, &args.target, &hist, rps);
    Ok(())
}

async fn tcp_once(s: &mut TcpStream, bytes: usize) -> Result<u128> {
    let buf = vec![0u8; bytes];
    let len = (bytes as u32).to_be_bytes();
    let t0 = Instant::now();
    s.write_all(&len).await?;
    s.write_all(&buf).await?;
    let mut len_buf = [0u8; 4];
    s.read_exact(&mut len_buf).await?;
    let resp_len = u32::from_be_bytes(len_buf) as usize;
    let mut resp = vec![0u8; resp_len];
    s.read_exact(&mut resp).await?;
    Ok(t0.elapsed().as_micros())
}

// ---------- Protobuf serde ----------
fn bench_serde(args: SerdeArgs) -> Result<()> {
    use prost::Message;
    let iters = args.iters.max(1);
    let msg = Payload { data: vec![0u8; args.size].into() };
    let mut buf = Vec::with_capacity(args.size + 32);

    let t0 = Instant::now();
    for _ in 0..iters {
        buf.clear();
        msg.encode(&mut buf)?;
    }
    let enc = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let mut sum = 0usize;
    for _ in 0..iters {
        let decoded = Payload::decode(buf.as_slice())?;
        sum += decoded.data.len();
    }
    let dec = t1.elapsed().as_micros();

    println!(
        "Protobuf {}B -> enc {:.3} µs/op, dec {:.3} µs/op (iters={})",
        args.size,
        enc as f64 / iters as f64,
        dec as f64 / iters as f64,
        iters
    );
    eprintln!("_keep={sum}");
    Ok(())
}

// ---------- Sweep ----------
async fn sweep(a: SweepArgs) -> Result<()> {
    let sizes: Vec<usize> = a.sizes.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    for sz in sizes {
        match a.mode.as_str() {
            "grpc" => {
                bench_grpc(CommArgs{
                    target: a.target.clone(), size: sz,
                    iters: a.iters, conc: a.conc, warmup: a.warmup
                }).await?;
            }
            "tcp" => {
                bench_tcp(TcpArgs{
                    target: a.target.clone(), size: sz,
                    iters: a.iters, conc: a.conc, warmup: a.warmup
                }).await?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn print_hist(tag: &str, sz: usize, target: &str, hist: &Histogram<u64>, rps: f64) {
    println!(
        "{tag} {sz}B @ {target} -> p50={:.3} ms p95={:.3} ms p99={:.3} ms max={:.3} ms | {:.0} req/s",
        hist.value_at_percentile(50.0) as f64 / 1000.0,
        hist.value_at_percentile(95.0) as f64 / 1000.0,
        hist.value_at_percentile(99.0) as f64 / 1000.0,
        hist.max() as f64 / 1000.0,
        rps
    );
}
