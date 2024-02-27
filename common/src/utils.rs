pub mod prelude {
    pub use crate::{Instruction, Value, Width, Register, Immediate, utils::{Error, Result}};
}
use crate::prelude::*;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("incompatible operands: {0:?}")]
    InvalidOperands(Instruction),

    #[error("number out of bounds: {0} doesn't fit {1:?}")]
    NumberOOB(u64, Width),

    #[error("reached EOL")]
    EOL,

    #[error("missing token in instruction \"{0}\"")]
    MissingToken(String),

    #[error("unexpected token \"{1}\" in instruction \"{0}\"")]
    UnexpectedToken(String, String),

    #[error("unknown instruction \"{0}\"")]
    UnknownInstruction(String),

    #[error("label not defined \"{0}\"")]
    LabelNotDefined(String),

    #[error("missing opcode")]
    NoOpcode,

    #[error("missing registers")]
    NoRegs,

    #[error("missing value byte {0}")]
    NoValue(u8),

    #[error("no such opcode \"{0:#04x}\"")]
    NoSuchOpcode(u8),

    #[error("{0}")]
    Misc(String),
}
pub type Result<T> = std::result::Result<T, Error>;
