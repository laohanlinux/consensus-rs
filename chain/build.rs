extern crate prost_build;

fn main (){
    prost_build::compile_protos(
        &["src/blockchain/dpos/block.proto"],
        &["src/"]).unwrap();

    
}