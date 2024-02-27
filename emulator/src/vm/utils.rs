#[allow(unused_imports)]
use common::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterValue(pub u64);

impl RegisterValue {
    pub fn set_byte(&mut self, idx : u8, value : u8) {
        self.0 &= !(0xFF << (idx * 8)); // Set this byte to 0
        self.0 |= (value as u64) << (idx * 8); // Set the value
    }

    pub fn set_word(&mut self, idx : u8, value : u16) {
        self.0 &= !(0xFFFF << (idx * 16)); // Set this word to 0
        self.0 |= (value as u64) << (idx * 16); // Set the value
    }

    pub fn get_byte(&self, idx : u8) -> u8 {
        (self.0 >> (idx * 8)) as u8
    }

    pub fn get_word(&self, idx : u8) -> u16 {
        (self.0 >> (idx * 16)) as u16
    }
}

impl From<u64> for RegisterValue {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<RegisterValue> for u64 {
    fn from(value: RegisterValue) -> Self {
        value.0
    }
}
