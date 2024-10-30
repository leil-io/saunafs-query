use chrono::TimeDelta;

use crate::{ChangelogResults, TimestampRange};

/// Print the results of the changelog analysis
pub fn print_result(
    timeline: &TimestampRange,
    op_count: Vec<(&String, &u64)>,
    results: &ChangelogResults,
) {
    let written_amount = results.inodes.all.iter().map(|i| i.written).sum::<u64>();
    let all_op_count = op_count.iter().map(|op| op.1).sum::<u64>();
    println!("Start: {}", timeline.start);
    println!("End: {}", timeline.end);
    println!("Total operations: {}", all_op_count);
    println!(
        "Operations/s: {0:.2}",
        calculate_rate(&all_op_count, timeline)
    );
    println!("Estimated written bytes: {}", format_bytes(written_amount));
    println!(
        "Estimated written bytes/s: {}",
        format_bytes(calculate_rate(&written_amount, timeline) as u64)
    );
    println!("Total files created: {}", results.file_count);
    println!(
        "Files created/s: {0:.2}",
        calculate_rate(&results.file_count, timeline)
    );
    println!("Total directories created: {}", results.dir_count);
    println!(
        "Directories created/s: {0:.2}",
        calculate_rate(&results.dir_count, timeline)
    );
    println!("Total inodes created: {}", results.inode_created_count);
    println!(
        "Inodes created/s: {0:.2}",
        calculate_rate(&results.inode_created_count, timeline)
    );
    println!("---");
    println!("{0:>15}{1:>10} | Ops/s", "Operation", "Count");
    for v in op_count.iter() {
        println!(
            "{0:>15}{1:>10} | {2:.2}/s",
            v.0.clone() + ":",
            v.1,
            calculate_rate(v.1, timeline)
        );
    }
}

/// Calculate the rate of operations per second
fn calculate_rate(count: &u64, timeline: &TimestampRange) -> f64 {
    let duration = timeline.end - timeline.start;
    // To avoid division by zero, check if duration is zero
    // This can happen if all operations occurred at the same timestamp
    if duration > TimeDelta::zero() {
        *count as f64 / duration.num_seconds() as f64
    } else {
        *count as f64
    }
}

/// Format a byte amount into a human-readable string
fn format_bytes(bytes: u64) -> String {
    let units = ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, units[unit_index])
}
