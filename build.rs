const PROTOS: &[&str] = &[ "src/sample.proto" ];

fn main() {
    prost_build::compile_protos(PROTOS, &["src"]).unwrap();
}
