#[allow(unused_imports)]
use common::prelude::*;

pub struct Scanner<T> {
    vec: std::collections::VecDeque<T>,
}

pub enum ScannerAction<T> {
    /// Immediately advance the cursor and return
    Return(T),

    /// If the next iteration returns None, return T without advancing the cursor
    Request(T),

    /// If the next iteration returns None, return None without advancing the cursor
    Require,

    None,
}

impl<T> Scanner<T> {
    pub fn new(vec : Vec<T>) -> Self {
        Self { vec: vec.into() }
    }

    pub fn peek(&self) -> Option<&T> {
        self.vec.front()
    }

    pub fn pop(&mut self) -> Option<T> {
        self.vec.pop_front()
    }

    pub fn test(&mut self, cb : impl FnOnce(&T) -> bool) -> bool {
        self.peek().is_some_and(cb)
    }

    pub fn take(&mut self, cb : impl FnOnce(&T) -> bool) -> Option<T> {
        self.test(cb).then(|| self.vec.pop_front().unwrap())
    }

    pub fn take_while(&mut self, cb : impl Fn(&T) -> bool) -> Vec<T> {
        let mut res = Vec::new();
        while let Some(t) = self.take(&cb) {
            res.push(t)
        }
        res
    }

    pub fn transform<U>(&mut self, cb : impl FnOnce(&T) -> Option<U>) -> Option<U> {
        let res = cb(self.peek()?)?;
        self.vec.pop_front();
        Some(res)
    }

    pub fn scan<U>(&mut self, cb : impl Fn(&[T]) -> ScannerAction<U>) -> Result<Option<U>> {
        let mut sequence = Vec::new();
        let mut require = false;
        let mut request = None;

        loop {
            let Some(c) = self.vec.pop_front() else {
                break if require { Err(Error::EOL) } else { Ok(request) } 
            };

            sequence.push(c);
            match cb(&sequence) {
                ScannerAction::Return(res) => break Ok(Some(res)),

                ScannerAction::Request(res) => request = Some(res),

                ScannerAction::Require => require  = true,

                ScannerAction::None => {
                    self.vec.push_front(sequence.pop().unwrap()); // Put the char back
                    break if require { Err(Error::EOL) } else { Ok(request) }
                },
            }
        }
    }
}
