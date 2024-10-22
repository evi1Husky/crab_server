use std::{
    env,
    io::{prelude::*, BufReader},
    process::{Command, Stdio},
};

pub fn run_command() {
    let args: Vec<String> = env::args().collect();
    let mut arg: String = args
        .into_iter()
        .filter(|x| x.starts_with("--run="))
        .collect();
    if arg.is_empty() {
        return;
    }
    arg.drain(0..6);
    let cmd: String = arg.split(' ').take(1).collect();
    let cmd_args = arg.split(' ').skip(1);

    println!("Running: {arg}");

    let mut child = Command::new(cmd)
        .args(cmd_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute command");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines() {
        match line {
            Ok(line) => println!("{}", line),
            Err(err) => println!("Error: {}", err),
        }
    }
}
