use clap::error::ErrorKind;
use clap::Parser;
use parser::line_parser;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Write};
use std::fs::{self, File};
use std::thread::Thread;
use std::time::Duration;

#[derive(Parser)]
#[command(about = "Read from a file")]
struct Cli {
    /// Optional input file. Reads from stdin if not provided.
    file: Option<String>,

    /// Root directory of SaunaFS. If not provided, returns raw inode numbers
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

fn translate_inode_to_path(mountpoint: &Option<String>, inode: u64) -> Option<String> {
    match mountpoint {
        None => None,
        Some(path) => {
            let mut try_counter = 0;
            loop {
                let inode_str = inode.to_string();
                match fs::read_to_string(path.to_owned() + "/.saunafs_path_by_inode/" + &inode_str) {
                    Ok(path) => return Some(path),
                    Err(e) => {
                        match e.kind() {
                            std::io::ErrorKind::Other => {
                                if try_counter > 3 {
                                    eprintln!("Could not get path_by_inode after 4 tries: {e}");
                                    return None
                                }
                                // Try again in a few hundred ms
                                std::thread::sleep(Duration::from_millis(200));
                                try_counter += 1;
                                continue;
                            }
                            _ => {
                                eprintln!("Could not get path_by_inode: {e}");
                                return None
                            }
                        }
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
                    let inode_path = translate_inode_to_path(&mountpoint, inode);
                    let msg = match inode_path {
                        Some(path) => {
                            format!("Operation: {} inode: {inode} path: '{path}'\n", line.operation)
                        }
                        None => {
                            format!("Operation: {} inode: {inode}\n", line.operation)
                        }
                    };
                    writer.write_all(msg.as_bytes())?;
                    writer.flush()?;
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
