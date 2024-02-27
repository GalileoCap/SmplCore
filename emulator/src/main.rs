mod vm;

#[allow(unused_imports)]
use common::prelude::*;

use clap::Parser;
use vm::VM;

/// Assembler for SimpleASM
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the rom file
    rom_path : String,

    /// Number of instructions to execute
    #[arg(long)]
    reps : usize,

    /// Size of the RAM available during execution
    #[arg(long, default_value_t = 0x8000)]
    ram_size : usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rom = read_bytes(&args.rom_path)?;
    let mut vm = VM::new(rom, args.ram_size);

    for _ in 0..args.reps {
        vm.execute_next()?;
    }
    println!("Finished with: {:?}", vm.regs());

    Ok(())
}

fn read_bytes(fpath : &str) -> Result<Vec<u8>> {
    std::fs::read(fpath)
        .map_err(|err| Error::Misc(err.to_string()))
}
