use clap::Parser;
use parser::line_parser;
use std::io::{self, stdin, stdout, BufRead, BufReader, BufWriter, Read, Write};
use std::fs::{self, File};
use std::time::Duration;
use std::fmt;

use futures::StreamExt;
use std::{env, str::from_utf8};

const KILOBYTE: usize = 1024;

#[derive(Parser)]
#[command(about = "Read from a file")]
struct Cli {
    /// Optional input file. Reads from stdin if not provided.
    file: Option<String>,

    /// Root directory of LeilFS. If not provided, returns raw inode numbers
    #[arg(short, long)]
    mountpoint: Option<String>
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if let Some(filename) = cli.file {
        let mut reader = BufReader::new(File::open(filename)?);
        let mut writer = BufWriter::new(stdout().lock());
        run(&mut reader, &mut writer, cli.mountpoint)?;
    } else {
        let mut reader = BufReader::new(stdin().lock());
        let mut writer = BufWriter::new(stdout().lock());
        run(&mut reader, &mut writer, cli.mountpoint)?;
    }
    Ok(())
}

fn read_file(path: String) -> io::Result<String>  {
    // We need to do a streaming read on saunafs_path_by_inode
    let mut f = File::open(path)?;
    let mut buf = vec![0u8; 4 * KILOBYTE];
    let mut out = Vec::new();

    loop {
        match f.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(String::from_utf8(out).expect("could not convert buffer to utf8"))
}

enum FileType {
    File,
    Dir,
    Symlink,
    Unknown,
}


impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            FileType::File => "file",
            FileType::Dir => "dir",
            FileType::Symlink => "symlink",
            FileType::Unknown => "unknown",
        };
        write!(f, "{s}")
    }
}

fn inode_filetype(mountpoint: &Option<String>, inode: u64) -> FileType {
    match mountpoint {
        None => FileType::Unknown,
        Some(path) => {
            let mut try_counter = 0;
            loop {
                let inode_str = inode.to_string();
                let path = path.to_owned() + "/.saunafs_file_by_inode/" + &inode_str;

                match fs::metadata(path) {
                    Ok(path) => {
                        if path.is_dir() {
                            return FileType::Dir
                        } else if path.is_file() {
                            return FileType::File
                        } else if path.is_symlink() {
                            return FileType::Symlink
                        } else {
                            return FileType::Unknown
                        }
                    },
                    Err(e) if e.kind() == io::ErrorKind::NotFound => return FileType::Unknown,
                    Err(e) => {
                        if try_counter > 3 {
                            eprintln!("Could not get file_by_inode after 4 tries: {e}");
                            return FileType::Unknown;
                        }
                        // Try again in a few hundred ms
                        eprintln!("could not get file_by_inode ({try_counter}/4): {e}");
                        std::thread::sleep(Duration::from_millis(200));
                        try_counter += 1;
                        continue;
                    }
                };
            }
        }
    }
}

fn translate_inode_to_path(mountpoint: &Option<String>, inode: u64) -> Option<String> {
    match mountpoint {
        None => None,
        Some(path) => {
            let mut try_counter = 0;
            loop {
                let inode_str = inode.to_string();
                let path = path.to_owned() + "/.saunafs_path_by_inode/" + &inode_str;

                match read_file(path) {
                    Ok(path) => return Some(path),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => return None,
                    Err(e) => {
                        if try_counter > 3 {
                            eprintln!("Could not get path_by_inode after 4 tries: {e}");
                            return None
                        }
                        // Try again in a few hundred ms
                        eprintln!("could not get path_by_inode ({try_counter}/4): {e}");
                        std::thread::sleep(Duration::from_millis(200));
                        try_counter += 1;
                        continue;
                    }
                };
            }
        }
    }
}

fn run<R: Sized + Read, W: Sized + Write>(
    reader: &mut BufReader<R>,
    writer: &mut BufWriter<W>,
    mountpoint: Option<String>
) -> std::io::Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        match line_parser::Parser::new(&line) {
            Ok(line) => {
                if let Some(inode) = line.inode {
                    let msg = operation_str(&line, inode, &mountpoint);
                    writer.write_all(msg.as_bytes())?;
                    writer.flush()?;
                    match send_to_nats(msg) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Could not OP to NATS server: {e}")
                    }
                }
            },
            Err(e) => {
                eprintln!("error parsing line: {e}");
                eprintln!("{line}");
                continue
            }
        }
    }
    writer.flush()?;
    Ok(())
}

fn operation_str(parser: &line_parser::Parser, inode: u64, mountpoint: &Option<String>) -> String {
    match parser.operation.as_str() {
        "MOVE" => {
            let move_op = match parser.parse_move() {
                Ok(op) => op,
                Err(e) => {
                    eprintln!("could not parse MOVE operation: {e}");
                    return "".to_string();
                }
            };
            let file_type = inode_filetype(mountpoint, inode);
            let source_dir = translate_inode_to_path(mountpoint, move_op.source_dir_inode);
            let target_dir = translate_inode_to_path(mountpoint, move_op.target_dir_inode);

            if source_dir.is_none() || target_dir.is_none() {
                let source_inode = move_op.source_dir_inode;
                let target_inode = move_op.target_dir_inode;
                eprintln!("Could not get paths for inode {source_inode} (source) and {target_inode} (target)");
                return format!("operation: {} inode: {inode}, type: {file_type}, {source_inode},\n", parser.operation);
            }

            let source_dir = source_dir.unwrap() + "/" + move_op.source_file_name;
            let target_dir = target_dir.unwrap() + "/" + move_op.target_file_name;
            format!("operation: {} inode: {inode}, type: {file_type}, sourcePath: `{source_dir}`, targetPath: `{target_dir}`\n", parser.operation)
        }
        "UNLINK" => {
            let (dir_inode, base_name) = match parser.parse_unlink() {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("could not parse UNLINK operation: {e}");
                    return "".to_string();
                }
            };

            match translate_inode_to_path(mountpoint, dir_inode) {
                Some(dir) => {
                    let path = dir + "/" + base_name;
                    format!("operation: {}, inode: {inode}, path: `{path}`\n", parser.operation)
                }
                None => format!("operation: {}, inode: {inode}\n", parser.operation)
            }
        }
        _ => {
            // Assuming all other operations other than CREATE are about files
            let mut file_type = FileType::File;

            // With CREATE, we can use is_dir to save time communicating with master
            if parser.operation.as_str() == "CREATE" && parser.is_dir() {
                file_type = FileType::Dir;
            }
            let inode_path = translate_inode_to_path(mountpoint, inode);

            match inode_path {
                Some(path) => {
                    format!("operation: {} inode: {inode}, type: {file_type}, path: `{path}`\n", parser.operation)
                }
                None => {
                    format!("operation: {} inode: {inode}, type: {file_type}\n", parser.operation)
                }
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn send_to_nats(str: String) -> Result<(), async_nats::Error> {
    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let client = async_nats::connect(nats_url).await?;

    let mut subscription =
        client.subscribe("opencloud.*").await?.take(3);

    client.publish("opencloud.leil", str.clone().into()).await?;

    while let Some(message) = subscription.next().await {
        println!(
            "{:?} received on {:?}",
            from_utf8(&message.payload),
            &message.subject
        );
    }

    Ok(())
}
