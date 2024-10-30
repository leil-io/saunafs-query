use std::process::{exit, Command};

use chrono::NaiveDateTime;
use clap::Parser;
use saunafs_query::{run, TimestampRange};

/// CLI parser
/// Uses the `clap` library to parse command line arguments
/// and macros to generate the parser code;
#[derive(Parser)]
#[command(version, about = "Query .sfs journal files", long_about = None)]
struct Cli {
    /// When to start reading from the logs
    #[arg(long, value_name = "STRING")]
    start: Option<String>,
    /// When to stop reading from the logs
    #[arg(long, value_name = "STRING")]
    stop: Option<String>,
    /// Metadata files to read from
    files: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let mut timeline = TimestampRange::default();

    set_timeline_date(cli.start, &mut timeline, true);
    set_timeline_date(cli.stop, &mut timeline, false);

    run(cli.files, timeline)?;

    Ok(())
}

/// Set the start or stop date for the parser to determine the range of logs to read
///
/// # Arguments
///
/// * `date_str` - The date string to parse into a timestamp
/// * `timeline` - The timeline struct to update
/// * `is_start` - Whether to update the start or end date
fn set_timeline_date(date_str: Option<String>, timeline: &mut TimestampRange, is_start: bool) {
    if let Some(date) = date_str {
        match get_unix_timestamp(&date) {
            Ok(time) => match NaiveDateTime::from_timestamp_opt(time, 0) {
                Some(datetime) => {
                    if is_start {
                        timeline.start = datetime;
                        timeline.start_is_set = true;
                    } else {
                        timeline.end = datetime;
                        timeline.end_is_set = true;
                    }
                }
                None => eprintln!("Timestamp out of range for date: {}", date),
            },
            Err(e) => {
                eprintln!(
                    "Failed to parse date in --{} option: {}",
                    if is_start { "start" } else { "stop" },
                    e
                );
                exit(3);
            }
        }
    }
}

/// Get the unix timestamp for a given date string. Uses GNU `date` command
///
/// # Arguments
///
/// * `date_str` - The date string to parse into a timestamp
fn get_unix_timestamp(date_str: &str) -> Result<i64, String> {
    let output = Command::new("date")
        .args(["-u", "-d", date_str, "+%s"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let timestamp = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
                Ok(timestamp.trim().parse::<i64>().unwrap())
            } else {
                let error_message =
                    String::from_utf8(output.stderr).expect("PANIC: Can't read stderr");
                Err(error_message.to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
