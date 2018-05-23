extern crate prost_build;

use std::process::Command;
use std::path::Path;

fn main (){
//    prost_build::compile_protos(
//        &["src/blockchain/dpos/block.proto"],
//        &["src/"]).unwrap();

//    if Path::new("src/blockchain/dposblock.rs").is_file() {
//        return ;
//    }

    let output = Command::new("pb-rs")
        .arg("src/blockchain/block-dpos.proto")
        .output()
        .expect("Failed to execute pb-rs command");
    assert!(output.status.success());
}