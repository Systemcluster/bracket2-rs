
use std::collections::HashMap;
use std::vec::Vec;
use std::option::Option;
use std::boxed::Box;
use std::string::String;
use std::default::Default;
use std::result::Result;
use std::error;
use std::fmt;
use unicode::UnicodeSegmentation;
use log;


#[derive(Debug)]
pub struct ParserSuccess {
	sub: usize,
	index: usize,
	line: usize,
	lineindex: usize,
}
#[derive(Debug)]
pub struct ParserError {
	description: String,
	index: usize,
	line: usize,
	lineindex: usize,
}
impl ParserError {
	fn new(index: usize, line: usize, lineindex: usize, description: String) -> ParserError {
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
pub type ParserResult = Result<ParserSuccess, ParserError>;
pub type ProgramResult = Result<Program, ParserError>;

#[derive(Debug)]
enum Value {
	Sub(usize),
	SubLink(usize),
	SubName(String),
	I64(i64)
}
#[derive(Debug, Default)]
struct Sub {
	name: String,
	subs: Vec<Value>,
	modf: Option<Value>,
	parent: Option<usize>
}
impl Sub {
	fn new() -> Sub { Sub::default() }
}

#[derive(Debug, Default)]
pub struct Program {
	subs: Vec<Sub>
}
impl Program {
	pub fn from_code(code: &str) -> ProgramResult {
		let code = format!("[![{}]]", code);
		let subs = {
			let mut parser = Parser::default();
			parser.code = code.graphemes(true).collect();
			parser.parse(0, 0, 0, None).map_err(|e|e)?;
			parser.subs
		};
		Ok(Program { subs: subs })
	}
	pub fn run() {
		unimplemented!();
	}
}


#[derive(PartialEq, Eq, Debug, Clone)]
enum State {None, SubOpen, SubName, NameEnd, Sub, SubVal, Mod, ModName, ModEnd, End}

#[derive(Debug, Default)]
pub struct Parser<'a> {
	subs: Vec<Sub>,
	code: Vec<&'a str>
}
impl<'a> Parser<'a> {
	// todo: resolve sub names

	fn parse(&mut self, mut index: usize, mut line: usize, mut lineindex: usize, parent: Option<usize>) -> ParserResult {
		debug!("- starting sub at position {}", index);
		let mut current = Sub::new();
		current.parent = parent;
		self.subs.push(current);
		let current = self.subs.len() - 1;

		let mut state = State::None;
		let mut ename = String::new();

		while index < self.code.len() && state != State::End {
			match (&state.clone(), self.code[index], ename.len()) {
				(_,  " ", 0) | 
				(_, "\t", 0) | 
				(_, "\n", 0) | 
				(_, "\r", 0) => {
					// ignore unnecessary whitespace 
				}
				(State::None,    "[", _) => { state = State::SubOpen; }
				(State::None,     _ , _) => { unreachable!(); }
				(State::SubOpen, "[", _) |
				(State::NameEnd, "[", _) => { state = State::Sub; }
				(State::SubName, "[", _) => {
					debug!("  name: {}", ename);
					self.subs[current].name = ename;
					state = State::Sub;
					ename = "".into();
				}
				(State::SubOpen, "]", _) |
				(State::SubName, "]", _) |
				(State::NameEnd, "]", _) => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub after sub open, found ']' at index {}", index)));
				}
				(State::SubOpen,  s , _) => {
					state = State::SubName;
					ename = s.into();
				}
				(State::SubName,  s , _) => { 
					ename.push_str(s); 
				}
				(State::NameEnd,  s , _) => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub after sub open, found '{}' at index {}", s, index)));
				}
				(State::Sub,     "]", _) => {
					state = State::Mod;
				}
				(State::Sub,     "[", _) => {
					let child = self.parse(index, line, lineindex, Some(current))?;
					index = child.index - 1;
					self.subs[current].subs.push(Value::Sub(child.sub));
				}
				(State::Sub,      s , _) => {
					// step back to parse tokens & literals
					state = State::SubVal;
					index = index - 1;
				}
				(State::SubVal,  " ", _) |
				(State::SubVal, "\t", _) |
				(State::SubVal, "\n", _) |
				(State::SubVal, "\r", _) |
				(State::SubVal,  "[", _) |
				(State::SubVal,  "]", _) => {
					// end token & literals parsing and step back
					debug!("  found token/literal '{}' at position {}", ename, index);
					state = State::Sub;
					let mut child = Sub::new();
					child.parent = Some(current);
					child.subs.push(Value::SubName(ename));
					ename = "".into();
					index = index - 1;
				}
				(State::SubVal,   s , _) => {
					ename.push_str(s);
				}
				(State::Mod,     "[", _) => {
					state = State::ModEnd;
					let child = self.parse(index, line, lineindex, parent)?;
					index = child.index - 1;
					self.subs[current].modf = Some(Value::Sub(child.sub));
				}
				(State::Mod,     "]", _) => { state = State::End;}
				(State::Mod,      s , _) => {
					state = State::ModName;
					ename = s.into();
				}
				(State::ModName,  " ", _) |
				(State::ModName, "\t", _) |
				(State::ModName, "\n", _) |
				(State::ModName, "\r", _) |
				(State::ModName, "]", _) => {
					// end mod name parsing and step back
					state = State::ModEnd;
					debug!("  mod: {} at position {}", ename, index);
					self.subs[current].modf = Some(Value::SubName(ename));
					ename = "".into();
					index = index - 1;
				}
				(State::ModName, "[", _) => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub end after mod name, found '[' at index {}", index)));
				}
				(State::ModName,  s , _) => {
					ename.push_str(s);
				}
				(State::ModEnd,  "]", _) => { state = State::End; }
				(State::ModEnd,   s , _) => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub end after mod, found '{}' at index {}", s, index)));
				}
				(State::End,       _, _) => { unreachable!(); }
			}
			
			lineindex += 1;
			if self.code[index] == "\n" {
				line += 1;
				lineindex = 0;
			}
			index += 1;
		}
		Ok(ParserSuccess{sub: current, index, line, lineindex})
	}
}
