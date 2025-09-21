fn main() {
    tonic_build::configure()
        .compile(
            &["../computing_node/proto/bench_echo.proto"],
            &["../computing_node/proto"],
        )
        .unwrap();
}
