use std::fs::File;
use std::process;

use tokio::signal::unix::{signal, SignalKind};

pub fn spawn_signal_handler(dir: String) {
    // SIGUSR2: dump stack trace hint (use `sample <pid> 1` on macOS)
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            if let Ok(mut sig_usr2) = signal(SignalKind::user_defined2()) {
                while sig_usr2.recv().await.is_some() {
                    let pid = process::id();
                    eprintln!("\n=== SIGUSR2 received (pid={}) ===", pid);
                    eprintln!("To see where threads are stuck, run:");
                    eprintln!("  macOS:   sample {} 1", pid);
                    eprintln!("  Linux:   gdb -p {} -ex 'thread apply all bt' -ex quit", pid);
                    eprintln!("===============================\n");
                }
            }
        });
    });

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let mut sig_int = signal(SignalKind::interrupt()).expect("SIGINT");
            let mut sig_term = signal(SignalKind::terminate()).expect("SIGTERM");
            tokio::select! {
                _ = sig_int.recv() => {
                    info!("Receive SIGINT");
                }
                _ = sig_term.recv() => {
                    info!("Receive SIGTERM");
                }
            }
            flame::end("read file");
            std::fs::create_dir_all(&dir).unwrap();
            let graph = dir.to_owned() + "/flame-graph.html";
            info!("flame graph=> {}", graph);
            flame::dump_html(&mut File::create(graph).unwrap()).unwrap();
            std::process::exit(0);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_spawn_signal_handler() {
        use crate::common::random_dir;
        use crate::logger;
        logger::init_test_env_log();
        spawn_signal_handler(*random_dir())
    }
}
