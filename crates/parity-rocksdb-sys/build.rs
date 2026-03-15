extern crate cmake;

use std::env;
use std::path::PathBuf;

use cmake::Config;

fn main() {
    // Use system RocksDB when ROCKSDB_LIB_DIR is set, or auto-detect via brew on macOS
    let lib_dir = env::var("ROCKSDB_LIB_DIR").ok().or_else(|| {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("brew")
                .args(["--prefix", "rocksdb"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string() + "/lib")
        }
        #[cfg(not(target_os = "macos"))]
        None::<String>
    });
    if let Some(lib_dir) = lib_dir {
        let rocksdb_lib = PathBuf::from(&lib_dir);
        let rocksdb_lib = if rocksdb_lib.join("lib").exists() {
            rocksdb_lib.join("lib")
        } else {
            rocksdb_lib.clone()
        };
        if rocksdb_lib.join("librocksdb.a").exists() || rocksdb_lib.join("librocksdb.dylib").exists() {
            println!("cargo:rustc-link-search=native={}", rocksdb_lib.display());
            println!("cargo:rustc-link-lib=dylib=rocksdb");
            if let Ok(snappy_dir) = env::var("SNAPPY_LIB_DIR") {
                println!("cargo:rustc-link-search=native={}", snappy_dir);
            }
            // Homebrew rocksdb links against zstd, zlib, snappy - use dylib to pull deps
            let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
            if target_os.contains("macos") || target_os.contains("ios") {
                println!("cargo:rustc-link-lib=c++");
            }
            return;
        }
    }

    let mut cfg = Config::new("rocksdb");
    cfg.define("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
        .define("CMAKE_VERBOSE_MAKEFILE", "ON")
        .register_dep("SNAPPY")
        .define("WITH_SNAPPY", "ON")
        .define("FAIL_ON_WARNINGS", "OFF")
        .build_target("rocksdb");

    let snappy = env::var_os("DEP_SNAPPY_INCLUDE").expect("DEP_SNAPPY_INCLUDE is set in snappy.");

    // NOTE: the cfg! macro doesn't work when cross-compiling, it would return values for the host
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS is set by cargo.");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").expect("CARGO_CFG_TARGET_ENV is set by cargo.");

    if target_os.contains("ios") {
        cfg.define("CMAKE_OSX_SYSROOT", "");
        cfg.define("IOS_CROSS_COMPILE", "");
    }

    if target_os.contains("windows") {
        println!("cargo:rustc-link-lib=dylib={}", "rpcrt4");
        println!("cargo:rustc-link-lib=dylib={}", "shlwapi");
    }

    if target_env.contains("msvc") {
        cfg.env("SNAPPY_INCLUDE", snappy);

        let features = env::var("CARGO_CFG_TARGET_FEATURE")
            .expect("CARGO_CFG_TARGET_FEATURE is set by cargo.");

        if features.contains("crt-static") {
            cfg.define("WITH_MD_LIBRARY", "OFF");
        }
    } else {
        cfg.define("SNAPPY_INCLUDE_DIR", snappy)
            .define("SNAPPY_LIBRARIES", "/dev/null");
    }

    // Added to support old CPUs
    // see https://github.com/paritytech/parity-ethereum/issues/9684
    let portable = match &*env::var("PORTABLE").unwrap_or("ON".to_owned()) {
        "OFF" => false,
        _ => true,
    };
    if portable {
        cfg.define("PORTABLE", "ON");
    }

    let out = cfg.build();

    let mut build = out.join("build");

    if target_os.contains("windows") && target_env.contains("msvc") {
        let profile = match &*env::var("PROFILE").unwrap_or("debug".to_owned()) {
            "bench" | "release" => "Release",
            _ => "Debug",
        };
        build = build.join(profile);
    }

    println!("cargo:rustc-link-search=native={}", build.display());
    println!("cargo:rustc-link-lib=static=rocksdb");
    println!("cargo:rustc-link-lib=static=snappy");

    // https://github.com/alexcrichton/cc-rs/blob/ca70fd32c10f8cea805700e944f3a8d1f97d96d4/src/lib.rs#L891
    if target_os.contains("macos") || target_os.contains("ios") || target_os.contains("freebsd") || target_os.contains("openbsd") {
        println!("cargo:rustc-link-lib=c++");
    } else if !target_env.contains("msvc") && !target_os.contains("android") {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
