pub mod prelude {
    pub use crate::{Instruction, Value, Width, Register, Immediate, utils::{Error, Result}};
}
use crate::prelude::*;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("incompatible operands: {0:?}")]
    InvalidOperands(Instruction),
}
pub type Result<T> = std::result::Result<T, Error>;
