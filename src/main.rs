use std::{
    env,
    io::{self},
    net::{Ipv6Addr, SocketAddrV6, TcpListener},
    thread,
};

use rand::Rng;

mod mime_types;
mod not_found;
mod process;
mod task_runner;
mod thread_pool;
mod watcher;

fn main() -> io::Result<()> {
    let ip_addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
    let sock_addr = SocketAddrV6::new(ip_addr, 8080, 0, 0);
    let socket = TcpListener::bind(sock_addr)?;

    println!("Listening on: http://[::1]:8080");

    let threads = thread::available_parallelism()?.get();
    let thread_pool = thread_pool::ThreadPool::new(threads);

    thread_pool.push(task_runner::run_command);

    let exe_path = env::current_dir()?;

    thread_pool.push(move || {
        watcher::watch(exe_path.as_path())
            .inspect_err(|x| {
                eprintln!("{x}");
            })
            .ok();
    });

    let fav_icon = match rand::thread_rng().gen_range(0..=4) {
        0 => "🦀",
        1 => "🦊",
        2 => "✨",
        3 => "💫",
        4 => "🌟",
        _ => "🦀",
    };

    for stream in socket.incoming() {
        match stream {
            Ok(request) => {
                thread_pool.push(|| {
                    process::process(request, fav_icon)
                        .inspect_err(|x| {
                            eprintln!("{x}");
                        })
                        .ok();
                });
            }
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        }
    }

    Ok(())
}
