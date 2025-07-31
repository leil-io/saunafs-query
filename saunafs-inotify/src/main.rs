use clap::Parser;
use parser::line_parser;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Write};
use std::fs::File;

#[derive(Parser)]
#[command(about = "Read from a file")]
struct Cli {
    /// Optional input file. Reads from stdin if not provided.
    file: Option<String>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if let Some(filename) = cli.file {
        let mut reader = BufReader::new(File::open(filename)?);
        let mut writer = BufWriter::new(stdout().lock());
        run(&mut reader, &mut writer)?;
    } else {
        let mut reader = BufReader::new(stdin().lock());
        let mut writer = BufWriter::new(stdout().lock());
        run(&mut reader, &mut writer)?;
    }
    Ok(())
}

fn run<R: Sized + Read, W: Sized + Write>(
    reader: &mut BufReader<R>,
    writer: &mut BufWriter<W>,
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
                    writer.write_all(
                        format!("Operation {} on inode {}\n", line.operation, inode).as_bytes(),
                    )?;
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
