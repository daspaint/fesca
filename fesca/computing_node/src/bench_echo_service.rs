/*
Service that echoes back any received bytes. Used for benchmarking network performance.
 */
use tonic::{Request, Response, Status};

pub mod bench {
    tonic::include_proto!("bench");
}
use bench::echo_server::{Echo, EchoServer};
use bench::Payload;

#[derive(Default, Debug)]
pub struct BenchEcho;

#[tonic::async_trait]
impl Echo for BenchEcho {
    async fn ping(&self, req: Request<Payload>) -> Result<Response<Payload>, Status> {
        Ok(Response::new(req.into_inner())) // echo back same bytes
    }
}

// Re-export helper so grpc_server can add it
pub fn make_echo_server() -> EchoServer<BenchEcho> {
    EchoServer::new(BenchEcho::default())
}
