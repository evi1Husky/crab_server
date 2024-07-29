use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    env,
    io::{self, Write},
    net::{Ipv6Addr, SocketAddrV6, TcpListener, TcpStream},
    path::Path,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

pub fn watch(path: &Path) -> io::Result<()> {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = Arc::clone(&pair);

    let tmin = 1000;
    let mut t1 = Instant::now();
    let mut avg_ev_time = 0;
    let sleep_time = Arc::new(Mutex::new(0));
    let time_to_sleep = Arc::clone(&sleep_time);

    let handle = move |ev| match ev {
        Ok(_) => {
            let t2 = Instant::now();
            let delta_t = t2.duration_since(t1).as_millis();
            if delta_t >= tmin {
                let (mutex, cvar) = &*pair_clone;
                *mutex.lock().unwrap() = false;
                cvar.notify_all();
                *sleep_time.lock().unwrap() = avg_ev_time;
                avg_ev_time = 0;
            }
            if delta_t < tmin {
                avg_ev_time += delta_t;
            }
            t1 = t2;
        }
        Err(err) => eprintln!("{err}"),
    };

    let mut watcher = RecommendedWatcher::new(handle, Config::default()).unwrap();

    watcher.watch(path, RecursiveMode::Recursive).unwrap();

    let ip_addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
    let sock_addr = SocketAddrV6::new(ip_addr, 8087, 0, 0);
    let socket = TcpListener::bind(sock_addr)?;

    let delay = get_delay_from_args().unwrap_or(0);

    for stream in socket.incoming() {
        match stream {
            Ok(request) => {
                let pair_clone = Arc::clone(&pair);
                let (mutex, cvar) = &*pair_clone;
                let time_to_sleep = Arc::clone(&time_to_sleep);
                *mutex.lock().unwrap() = false;
                cvar.notify_all();
                thread::spawn(move || -> io::Result<()> {
                    let (mutex, cvar) = &*pair_clone;
                    let lock = mutex.lock().unwrap();
                    let _cvar = cvar.wait(lock).unwrap();
                    let sleep = *time_to_sleep.lock().unwrap();
                    thread::sleep(Duration::from_millis(sleep as u64 + delay));
                    respond(request)
                });
            }
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        }
    }

    fn respond(mut request: TcpStream) -> io::Result<()> {
        const RESPONSE: &str = "
            HTTP/1.1 200 OK\r\n
            Access-Control-Allow-Origin: *\r\n
            Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n
            Cache-Control: no-cache\r\n
            Content-Type: text/plain; charset=UTF-8\r\n
            Content-Length: 6\r\n\r\n
            reload";
        request.write_all(RESPONSE.as_bytes())?;
        Ok(())
    }

    fn get_delay_from_args() -> Option<u64> {
        let args: Vec<String> = env::args().collect();
        let mut arg: String = args
            .into_iter()
            .filter(|x| x.starts_with("--reloadDelay="))
            .collect();
        if arg.is_empty() {
            return None;
        }
        arg.drain(0..=13);
        match arg.parse() {
            Ok(x) => Some(x),
            Err(_) => {
                eprintln!("Error: invalid reload delay value");
                None
            }
        }
    }

    Ok(())
}
