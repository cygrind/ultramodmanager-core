use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(message: impl ToString) -> Box<RuntimeError> {
        let message = message.to_string();

        Box::new(RuntimeError { message })
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: impl ToString) -> ParseError {
        let message = message.to_string();

        ParseError { message }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ParseError {}