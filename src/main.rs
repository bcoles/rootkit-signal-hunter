// Rootkit Signal Hunter
// bcoles 2025

use clap::Parser;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;

/// Detect rootkits which use signals to elevate process privileges
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Launch root shell
    #[arg(short = 's', long = "shell")]
    shell: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Start at signal
    #[arg(long, default_value_t = 0)]
    min: i32,

    /// Stop at signal
    #[arg(long, default_value_t = 64)]
    max: i32,

    /// Number of worker threads
    #[arg(short = 't', long = "threads", default_value_t = 16)]
    threads: usize,

    /// Process ID to send signals to
    #[arg(short = 'p', long = "pid", default_value = "$$")]
    pid: String,
}

fn main() {
    if let Ok(out) = Command::new("id").arg("-u").output()
        && out.status.success()
    {
        let uid_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if uid_str == "0" {
            eprintln!("Refusing to run as root (uid 0). Please run as non-root.");
            std::process::exit(1);
        }
    }

    let args = Cli::parse();

    if args.verbose {
        println!("Launch root shell: {}", args.shell);
        println!("Worker threads:    {}", args.threads);
        println!("Start at signal:   {}", args.min);
        println!("Stop at signal:    {}", args.max);
        println!("Target PID:        {}", args.pid);
    }

    if args.min > args.max {
        eprintln!(
            "Start signal ({}) is greater than stop signal ({}); nothing to iterate.",
            args.min, args.max
        );
        return;
    }

    println!(
        "Trying signals {} to {} (PID: {}) ...",
        args.min, args.max, args.pid
    );

    let signals_shared: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
    let processes_shared: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

    let (tx, rx) = mpsc::channel::<i32>();
    let rx = Arc::new(Mutex::new(rx));

    let mut handles = Vec::new();
    for _ in 0..args.threads {
        let rx_cloned = Arc::clone(&rx);
        let signals_cloned = Arc::clone(&signals_shared);
        let processes_cloned = Arc::clone(&processes_shared);
        let verbose = args.verbose;
        let pid = args.pid.clone();

        let handle = thread::spawn(move || {
            loop {
                let signal = {
                    let lock = rx_cloned.lock().unwrap();
                    match lock.recv() {
                        Ok(s) => s,
                        Err(_) => break,
                    }
                };

                if verbose {
                    println!("Trying signal {} ...", signal);
                }

                let mut child = match Command::new("sh")
                    .arg("-c")
                    .arg(format!("kill -{} {} ; id", signal, pid))
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => {
                        let pid = c.id();
                        let mut processes = processes_cloned.lock().unwrap();
                        processes.push(pid);
                        c
                    }
                    Err(e) => {
                        eprintln!("Failed to spawn command for signal {}: {}", signal, e);
                        continue;
                    }
                };

                let start = Instant::now();
                let timeout = std::time::Duration::from_secs(5);
                let mut res = String::new();

                loop {
                    match child.try_wait() {
                        Ok(Some(_status)) => {
                            let mut stdout_str = String::new();
                            if let Some(mut out) = child.stdout.take() {
                                let _ = out.read_to_string(&mut stdout_str);
                            }
                            res = stdout_str;

                            if verbose {
                                let mut stderr_str = String::new();
                                if let Some(mut err) = child.stderr.take() {
                                    let _ = err.read_to_string(&mut stderr_str);
                                    if !stderr_str.is_empty() {
                                        res = format!(
                                            "{}\nstderr: {}",
                                            res.trim_end(),
                                            stderr_str.trim_end()
                                        );
                                    }
                                }
                            }
                            break;
                        }
                        Ok(None) => {
                            if start.elapsed() > timeout {
                                let _ = child.kill();
                                let _ = child.wait();
                                if verbose {
                                    eprintln!(
                                        "Command for signal {} timed out after {} seconds.",
                                        signal,
                                        timeout.as_secs()
                                    );
                                }
                                res.clear();
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                        Err(e) => {
                            eprintln!("Error waiting for command for signal {}: {}", signal, e);
                            let _ = child.kill();
                            let _ = child.wait();
                            res.clear();
                            break;
                        }
                    }
                }

                if verbose {
                    print!("Signal {} output: {}", signal, res.trim_end());
                    println!();
                }

                if res.contains("uid=0") {
                    println!("Found escalate signal: {}", signal);
                    let mut guard = signals_cloned.lock().unwrap();
                    guard.push(signal);
                }
            }
        });

        handles.push(handle);
    }

    for signal in args.min..=args.max {
        if tx.send(signal).is_err() {
            break;
        }
    }

    drop(tx);

    for h in handles {
        let _ = h.join();
    }

    if let Ok(processes) = Arc::try_unwrap(processes_shared) {
        println!("Cleaning up processes ...");

        if let Ok(processes) = processes.into_inner() {
            for pid in processes {
                let _output = Command::new("kill").arg(pid.to_string()).output();
            }
        }
    }

    let signals = Arc::try_unwrap(signals_shared)
        .map(|m| m.into_inner().unwrap())
        .unwrap_or_default();

    if signals.is_empty() {
        println!("Done. No rootkits detected.");
        return;
    }

    println!(
        "Done. Found {} signals for privilege escalation: {:?}",
        signals.len(),
        signals
    );

    if args.shell {
        println!("Launching root shell ...");
        Command::new("sh")
            .arg("-c")
            .arg(format!("kill -{} {} ; sh", signals[0], args.pid))
            .status()
            .expect("Failed to launch root shell.");
    } else {
        println!("Use -s to launch a root shell.");
    }
}
