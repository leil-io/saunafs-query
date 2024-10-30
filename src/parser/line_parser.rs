use chrono::NaiveDateTime;

/// Struct to hold the parsed line information
pub struct Parser<'a> {
    /// The timestamp of the log line
    pub timestamp: NaiveDateTime,
    /// The changelog number
    pub _id: u64,
    /// The operation performed
    pub operation: String,
    /// The inode number, if any
    pub inode: Option<u64>,
    /// The original line
    pub line: &'a str,
}

impl<'a> Parser<'a> {
    /// Create a new parser from a log line.
    /// The line is parsed and the timestamp, id, operation and inode (if possible) are extracted.
    ///
    /// # Arguments
    /// * `line` - The log line to parse
    pub fn new(line: &'a str) -> Result<Self, &'static str> {
        let timestamp = parse_timestamp(line)?;
        let _id = parse_id(line)?;
        let operation = parse_operation(line)?;
        let inode = parse_inode(line, &operation);
        Ok(Self {
            timestamp,
            _id,
            operation,
            inode,
            line,
        })
    }

    /// Parse the LENGTH operation and return the inode and length.
    pub fn parse_length(&self) -> Result<(u64, u64), &'static str> {
        let start = self
            .line
            .find('(')
            .ok_or("Could not find '(' in length operation line")?;
        let end = self
            .line
            .find(')')
            .ok_or("Could not find ')' in length operation line")?;
        let numbers_comma_slice = &self.line[start + 1..end];

        let comma = numbers_comma_slice
            .find(',')
            .ok_or("Could not find ',' in length operation line")?;
        let inode: u64 = numbers_comma_slice[..comma]
            .parse()
            .map_err(|_| "Failed to parse inode into u64 in lenght operation.")?;
        let length: u64 = numbers_comma_slice[comma + 1..]
            .parse()
            .map_err(|_| "Failed to parse length into u64.")?;

        Ok((inode, length))
    }

    /// Parse the line for a directory or file and update the counters.
    /// Only works for CREATE operations.
    pub fn parse_line_for_dir_file(&self, dirs: &mut u64, files: &mut u64) {
        if self.line.contains(",d,") {
            *dirs += 1;
        } else if self.line.contains(",f,") {
            *files += 1;
        }
    }
}

/// Parse the timestamp from a log line.
///
/// # Arguments
/// * `line` - The log line to parse
fn parse_timestamp(line: &str) -> Result<chrono::NaiveDateTime, &'static str> {
    let parts: Vec<&str> = line.split(": ").collect();
    if parts.len() != 2 {
        return Err("Line format incorrect, expected ':' separator");
    }

    let fields: Vec<&str> = parts[1].split('|').collect();
    if fields.is_empty() {
        return Err("No fields found after splitting by '|'");
    }

    match fields[0].parse::<i64>() {
        Ok(timestamp) => Ok(NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .expect("Failed to convert i64 to NaiveDateTime")),
        Err(_) => Err("Failed to parse timestamp as i64"),
    }
}

/// Parse the changelog number from a log line.
///
/// # Arguments
/// * `line` - The log line to parse
fn parse_id(line: &str) -> Result<u64, &'static str> {
    let parts: Vec<&str> = line.split(": ").collect();
    if parts.len() != 2 {
        return Err("Line format incorrect, expected ':' separator");
    }

    match parts[0].parse::<u64>() {
        Ok(id) => Ok(id),
        Err(_) => Err("Failed to parse ID as u64"),
    }
}

/// Parse the inode number from a log line, if possible.
/// Does not return an inode for WRITE or TRUNC operations, since this function only uses the `):`
/// at the end of the line to find the inode. The `WRITE` and `TRUNC` operations use temporary
/// inodes at the end.
fn parse_inode(line: &str, operation: &str) -> Option<u64> {
    match operation {
        "WRITE" | "TRUNC" => return None,
        _ => (),
    }
    let parts: Vec<&str> = line.split("):").collect();
    if parts.len() != 2 {
        return None;
    }

    let inode_str = parts[1].trim();
    match inode_str.parse::<u64>() {
        Ok(inode) => Some(inode),
        Err(_) => None,
    }
}

/// Parse the operation from a log line.
///
/// # Arguments
/// * `line` - The log line to parse
fn parse_operation(line: &str) -> Result<String, &'static str> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() != 2 {
        return Err("Line format incorrect, expected '|' separator");
    }

    let operation: Vec<&str> = parts[1].split('(').collect();
    if operation.is_empty() {
        return Err("No operation found after splitting by '('");
    }

    Ok(operation[0].to_string())
}

#[test]
fn test_parse_inodes() {
    let create = "5: 1710181938|CREATE(1,configuration.h,f,420,1000,1000,0):2";
    let session = "4: 1710181842|SESSION():1";
    let write = "33: 1710182099|WRITE(3,0,1,3033285594):15";
    let trunc = "72: 1710183175|TRUNC(4,0,0):16";
    let unlink = "59: 1710183125|UNLINK(1,configuration.h):3";

    assert_eq!(Some(2), parse_inode(create, "CREATE"));
    assert_eq!(Some(1), parse_inode(session, "SESSION"));
    assert_eq!(None, parse_inode(write, "WRITE"));
    assert_eq!(None, parse_inode(trunc, "TRUNC"));
    assert_eq!(Some(3), parse_inode(unlink, "UNLINK"));
}
