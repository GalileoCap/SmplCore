#[allow(unused_imports)]
use common::prelude::*;
use sasm_lib::compile;

use clap::Parser;

/// Assembler for SimpleASM
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input path
    in_file : String,

    /// Output file
    #[arg(short = 'o', default_value = "main.bin")]
    out_path : String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let code = read_file(&args.in_file)?;
    let bytes = compile(&code)?;
    write_file(&args.out_path, &bytes)
}

fn read_file(fpath : &str) -> Result<String> {
    std::fs::read_to_string(fpath)
        .map_err(|err| Error::Misc(err.to_string()))
}

fn write_file(fpath : &str, bytes : &[u8]) -> Result<()> {
    use std::io::Write;

    let mut fout = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(fpath)
        .map_err(|err| Error::Misc(err.to_string()))?;

    fout.write_all(bytes)
        .map_err(|err| Error::Misc(err.to_string()))?;
    Ok(())
}
