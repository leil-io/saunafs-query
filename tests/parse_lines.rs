/// Integration test for parsing lines from a changelog file

#[test]
fn test_written_parse() {
    let test_str = include_str!("./file_changes.sfs").trim();

    let (_, results) = test_utils::new_results(test_str);

    assert_eq!(results.op_count.iter().map(|op| op.1).sum::<u64>(), 82);
    println!("{:?}", results.inodes);
    assert_eq!(results.inodes.all.len(), 3);
    assert_eq!(
        31923,
        results.inodes.all.iter().map(|i| i.written).sum::<u64>()
    )
}

#[cfg(test)]
pub mod test_utils {
    use saunafs_query::parse_line;
    use saunafs_query::ChangelogResults;
    use saunafs_query::TimestampRange;

    pub fn new_results(test_str: &str) -> (TimestampRange, ChangelogResults) {
        let mut timestamp = TimestampRange {
            end: chrono::offset::Local::now().naive_utc(),
            ..Default::default()
        };
        let mut results = ChangelogResults::default();
        test_str.split('\n').for_each(|line| {
            parse_line(line, &mut results, &mut timestamp).unwrap();
        });

        results.inodes.drain_active();
        (timestamp, results)
    }
}

#[test]
fn test_directory_file_inode_counts() {
    let test_str = include_str!("./files_dirs.sfs").trim();
    let (_, results) = test_utils::new_results(test_str);
    assert_eq!(results.file_count, 5);
    assert_eq!(results.dir_count, 6);
}
