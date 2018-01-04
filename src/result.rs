use std::error;
use std::fmt;
use std::string::String;

#[derive(Debug)]
pub struct ParserSuccess {
	pub sub: usize,
	pub index: usize,
	pub line: usize,
	pub lineindex: usize,
}
#[derive(Debug)]
pub struct ParserError {
	pub description: String,
	pub index: usize,
	pub line: usize,
	pub lineindex: usize,
}
impl ParserError {
	pub fn new(index: usize, line: usize, lineindex: usize, description: String) -> ParserError {
		ParserError {index, line, lineindex, description}
	}
}
impl fmt::Display for ParserError {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result { 
		write!(formatter, "ParserError on line {}:{}: {}", self.line, self.lineindex, self.description) 
	}
}
impl error::Error for ParserError {
	fn description(&self) -> &str { &self.description }
}
