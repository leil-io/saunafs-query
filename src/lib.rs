pub mod output;
pub mod parser;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    process::exit,
};

use output::print_result;
use parser::{inodes::Inodes, line_parser::Parser};

/// Struct to hold the start and end timestamps
/// The start and end timestamps are used to determine the range of logs to read
/// The start_is_set and end_is_set are used to determine if the start and end timestamps are set
/// by the user. If they are not set, the first and last timestamps found in the logs are used.
#[derive(Debug, Default)]
pub struct TimestampRange {
    /// The start timestamp
    pub start: chrono::NaiveDateTime,
    /// The end timestamp
    pub end: chrono::NaiveDateTime,
    /// Whether the start timestamp is set by the user
    pub start_is_set: bool,
    /// Whether the end timestamp is set by the user
    pub end_is_set: bool,
}

/// Struct to hold the results of the changelog analysis
#[derive(Debug, Default)]
pub struct ChangelogResults {
    /// HashMap to hold the count of each operation
    pub op_count: HashMap<String, u64>,
    /// Inodes struct to hold both currently active and historical inodes
    pub inodes: Inodes,
    /// Count of files created
    pub file_count: u64,
    /// Count of directories created
    pub dir_count: u64,
    /// Count of inodes created
    pub inode_created_count: u64,
}

/// Run the main logic of the program
///
/// # Arguments
/// * `args` - The list of files to read from
/// * `timeline` - The timeline struct to update. mut is needed because if start and end are not
/// set, they are set to the first and last timestamp found
pub fn run(mut args: Vec<String>, mut timeline: TimestampRange) -> std::io::Result<()> {
    let mut results = ChangelogResults::default();

    args.sort_by_key(|s| {
        s.split('.')
            .last()
            .unwrap_or("0")
            .parse::<i32>()
            .unwrap_or(0)
    });

    'outer: for f in args.iter().rev() {
        let file = File::open(f)?;
        let reader = BufReader::new(file);
        let mut count: i64 = 0;

        for line in reader.lines() {
            count += 1;
            let line = line?;
            let cont = match parse_line(&line, &mut results, &mut timeline) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error parsing line at {} in file {}: {}", count, f, e);
                    eprintln!("Line: {}", line);
                    eprintln!("Exiting...");
                    exit(2);
                }
            };
            if !cont {
                break 'outer;
            }
        }
    }

    let mut count_vec: Vec<(&String, &u64)> = results.op_count.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));

    results.inodes.drain_active();
    print_result(&timeline, count_vec, &results);

    Ok(())
}

/// Parse a specific line from the changelog
/// Returns true if parsing should continue, false if it should stop (because we may have passed
/// the end date)
///
/// # Arguments
/// * `line` - The line to parse
/// * `results` - The results struct to update.
/// * `timeline` - The timeline struct to potentially update and check.
///
/// # Errors
/// It may return an error if the line is not in the expected format
pub fn parse_line(
    line: &str,
    results: &mut ChangelogResults,
    timeline: &mut TimestampRange,
) -> Result<bool, &'static str> {
    let parse = Parser::new(line)?;
    if timeline.end_is_set && timeline.end < parse.timestamp {
        // Skip everything
        return Ok(false);
    }
    if timeline.start_is_set && timeline.start > parse.timestamp {
        // Skip just this line
        return Ok(true);
    }

    check_inode_operation(&parse, results)?;

    *results.op_count.entry(parse.operation).or_insert(0) += 1;

    if timeline.start > parse.timestamp || timeline.start.timestamp() == 0 {
        timeline.start = parse.timestamp;
    };
    if timeline.end < parse.timestamp {
        timeline.end = parse.timestamp;
    };

    Ok(true)
}

/// Check if the operation is an inode operation, and if so, update the inodes struct in
/// ChangelogResults.
///
/// # Arguments
/// * `parse` - The parser struct
/// * `results` - The results struct to update
///
/// # Errors
/// It may return an error if parsing the LENGTH for the inode length fails.
fn check_inode_operation(
    parse: &Parser,
    results: &mut ChangelogResults,
) -> Result<(), &'static str> {
    if let Some(inode) = parse.inode {
        match parse.operation.as_str() {
            "CREATE" => {
                results.inodes.append(inode, Some(parse.timestamp));
                parse.parse_line_for_dir_file(&mut results.dir_count, &mut results.file_count);
                results.inode_created_count += 1;
            }
            "UNLINK" => results.inodes.delete(inode, Some(parse.timestamp)),
            _ => (),
        }
    } else if parse.operation.as_str() == "LENGTH" {
        let (inode, length) = parse.parse_length()?;
        results.inodes.update_length(inode, length)
    };
    Ok(())
}
