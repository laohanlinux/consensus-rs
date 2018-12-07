use std::fs::File;

use ::actix::prelude::*;
use futures::prelude::*;
use tokio::prelude::*;
use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

pub fn spawn_signal_handler(dir: String) {
    let int_fut = Signal::new(SIGINT).flatten_stream();
    let term_fut = Signal::new(SIGTERM).flatten_stream();
    let s_stream = int_fut.select(term_fut);

    info!("Start signal handler");
    flame::start("read file");
    let code = System::run(move || {
        tokio::spawn(
            s_stream
                .into_future()
                .and_then(move |(item, _s)| {
                    info!("Receive a signal, code: {}", item.unwrap());
                    System::current().stop();
                    flame::end("read file");
                    ::std::fs::create_dir_all(&dir).unwrap();
                    let graph = dir.to_owned() + "/flame-graph.html";
                    info!("flame graph=> {}", graph);
                    flame::dump_html(&mut File::create(graph).unwrap()).unwrap();
                    future::ok(())
                })
                .map_err(|_err| ()),
        );
    });
    ::std::process::exit(code);
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
