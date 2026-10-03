use std::io::Write;
use std::io::{self};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use clap::Parser;
use fmlib::FM;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use rustyline::Result;

/// FM-index based mass query search utility.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
	/// Path to source file
	path: std::path::PathBuf,
}

fn progress<T>(message: &'static str, op: impl FnOnce() -> T) -> T {
	let running = Arc::new(AtomicBool::new(true));
	let flag = Arc::clone(&running);

	thread::spawn(move || {
		let frames = ['/', '-', '\\', '|'];
		let mut i = 0;

		while flag.load(Ordering::Relaxed) {
			print!("\r{} [{}]", message, frames[i % frames.len()]);
			io::stdout().flush().unwrap();
			i += 1;
			thread::sleep(Duration::from_millis(100));
		}
	});

	let now = Instant::now();
	let value = op();
	let elapsed = now.elapsed();

	running.store(false, Ordering::Relaxed);

	let time = ((elapsed.as_millis() as f64) / 1000.0).to_string();
	println!(
		"\r{} [✓ {}s]{}",
		message,
		time,
		" ".repeat(12 - 12.min(time.len()))
	);

	value
}

pub fn main() -> Result<()> {
	let args = Args::parse();

	let src = progress("Reading source", || {
		std::fs::read_to_string(args.path).expect("failed to read source text")
	});

	let fm = progress("Building index", || FM::build(&src));

	let mut rl = DefaultEditor::new()?;
	loop {
		let readline = rl.readline(">> ");
		match readline {
			Ok(mut line) => {
				if line.is_empty() {
					continue;
				}

				let limit = line
					.strip_prefix(':')
					.filter(|x| !x.starts_with(':'))
					.and_then(|x| x.split_once(' '))
					.and_then(|(limit, rem)| limit.parse::<usize>().ok().map(|n| (n, rem)));

				let limit = if let Some((limit, rem)) = limit {
					line = rem.to_owned();
					Some(limit)
				} else if line.starts_with("::") {
					line = line[1..].to_owned();
					None
				} else {
					None
				};

				let results = progress("Querying", || fm.search_for(&line, limit));

				if results.is_empty() {
					println!("No results.");
				} else {
					println!("Found {} results", results.len(),);

					let width = (*results.iter().max().unwrap() as f64).log10().ceil() as usize;

					for idx in results {
						let from = idx.saturating_sub(15);
						let to = (idx + line.len() + 15).min(src.len());

						println!(
							"{:0width$}: \x1b[90m{}\x1b[0m{}\x1b[90m{}\x1b[0m",
							idx,
							fm.text(from..idx),
							fm.text(idx..idx + line.len()),
							fm.text(idx + line.len()..to)
						);
					}
				}
			}
			Err(ReadlineError::Interrupted) => {}
			Err(_) => break,
		}
	}

	Ok(())
}
