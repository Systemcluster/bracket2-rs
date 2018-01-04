
use std::collections::HashMap;
use std::vec::Vec;
use std::option::Option;
use std::boxed::Box;
use std::string::String;
use std::default::Default;
use std::result::Result;
use unicode::UnicodeSegmentation;
use regex::Regex;
use log;

use result::*;
pub type ParserResult = Result<ParserSuccess, ParserError>;
pub type ProgramResult = Result<Program, ParserError>;

#[derive(Debug)]
enum Value {
	Sub(usize),
	SubLink(usize),
	SubName(String),
	I64(i64),
	F64(f64),
	String(String)
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
		// wrap code in single sub for easier parsing
		let code = format!("[[\n{}\n]]", code);
		let subs = {
			let mut parser = Parser::default();
			parser.code = code.graphemes(true).collect();
			parser.parse(0, 0, 0, None).map_err(|mut e|{
				// adjust parse index for added wrapper
				e.index -= 3; e.line -= 1; e
			})?;
			parser.subs
		};
		Ok(Program { subs: subs })
	}
	pub fn run() {
		unimplemented!();
	}
	pub fn tree(&self, stride: usize) -> String {
		self.subtree(0, 1, stride)
	}

	// generate a tree-visualization of the program
	fn subtree(&self, index: usize, depth: usize, stride: usize) -> String {
		let mut s = String::new();
		let sub = &self.subs[index];
		s.push_str("- sub");
		if sub.name.len() > 0 {
			s.push_str(" (");
			s.push_str(&sub.name);
			s.push_str(")");
		}
		s.push('\n');
		let do_match = |child: &Value| -> String {
			let mut s = String::new();
			match child {
				Value::Sub(i) => {
					s.push_str(&self.subtree(*i, depth+1, stride));
				}
				Value::SubName(n) => {
					s.push_str("-> (");
					s.push_str(n);
					s.push(')');
				}
				Value::SubLink(i) => {
					s.push_str("-> ");
					s.push_str(&format!("&{}", i));
				}
				Value::I64(i) => {
					s.push_str(&format!("_ {}", i));
				}
				Value::F64(f) => {
					s.push_str(&format!("_ {}", f));
				}
				_ => {
					unimplemented!();
				}
			}
			s
		};
		let do_indent = |s: &mut String| {
			for i in 0..depth {
				s.push('|');
				for i in 0..stride {
					s.push(' ');
				}
			}
		};
		if sub.subs.len() > 0 {
			do_indent(&mut s);
			s.push_str("subs:");
			for child in &sub.subs {
				s.push('\n');
				do_indent(&mut s);
				s.push_str(&do_match(&child));
			}
		}
		if let Some(modf) = &sub.modf {
			s.push('\n');
			do_indent(&mut s);
			s.push_str("mod:\n");
			do_indent(&mut s);
			s.push_str(&do_match(&modf));
		}
		s
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
	fn resolve_token(&mut self, token: &str) -> Value {
		lazy_static!{
			static ref SUBLINK: Regex = Regex::new(r"^&([0-9]*)$").unwrap();
			static ref INTEGER: Regex = Regex::new(r"^([0-9]+)$").unwrap();
			static ref FLOATP: Regex = Regex::new(r"^([0-9]+\.[0-9]+)$").unwrap();
		}
		if SUBLINK.is_match(token) {
			let cap = SUBLINK.captures(token).unwrap();
			if cap[1].len() > 0 {
				return Value::SubLink(0);
			}
			else {
				return Value::SubLink(cap[1].parse::<usize>().unwrap())
			}
		}
		if INTEGER.is_match(token) {
			let cap = INTEGER.captures(token).unwrap();
			return Value::I64(cap[1].parse::<i64>().unwrap())
		}
		if FLOATP.is_match(token) {
			let cap = FLOATP.captures(token).unwrap();
			return Value::F64(cap[1].parse::<f64>().unwrap())
		}
		Value::SubName(token.into())
	}

	fn parse(&mut self, mut index: usize, mut line: usize, mut lineindex: usize, parent: Option<usize>) 
	-> ParserResult {
		debug!("- starting sub at position {}", index);
		let mut current = Sub::new();
		current.parent = parent;
		self.subs.push(current);
		let current = self.subs.len() - 1;

		let mut state = State::None;
		let mut ename = String::new();

		while index < self.code.len() && state != State::End {
			match (&state.clone(), self.code[index]) {
				(_,  " ") if ename.len() == 0 => {}
				(_, "\t") if ename.len() == 0 => {} 
				(_, "\n") if ename.len() == 0 => {} 
				(_, "\r") if ename.len() == 0 => {}
				(State::None,    "[") => { state = State::SubOpen; }
				(State::None,     _ ) => { unreachable!(); }
				(State::SubOpen, "[") |
				(State::NameEnd, "[") => { state = State::Sub; }
				(State::SubName, "[") => {
					debug!("  name: {}", ename);
					self.subs[current].name = ename;
					state = State::Sub;
					ename = "".into();
				}
				(State::SubOpen, "]") |
				(State::SubName, "]") |
				(State::NameEnd, "]") => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub after sub open, found ']' at index {}", index)));
				}
				(State::SubOpen,  s ) => {
					state = State::SubName;
					ename = s.into();
				}
				(State::SubName,  s ) => { 
					ename.push_str(s); 
				}
				(State::NameEnd,  s ) => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub after sub name, found '{}' at index {}", s, index)));
				}
				(State::Sub,     "]") => {
					state = State::Mod;
				}
				(State::Sub,     "[") => {
					let child = self.parse(index, line, lineindex, Some(current))?;
					index = child.index - 1;
					self.subs[current].subs.push(Value::Sub(child.sub));
				}
				(State::Sub,      s ) => {
					// step back to parse tokens & literals
					state = State::SubVal;
					index = index - 1;
				}
				(State::SubVal,  " ") |
				(State::SubVal, "\t") |
				(State::SubVal, "\n") |
				(State::SubVal, "\r") |
				(State::SubVal,  "[") |
				(State::SubVal,  "]") => {
					// end token & literals parsing and step back
					debug!("  found token/literal '{}' at position {}", ename, index);
					state = State::Sub;
					self.subs[current].subs.push(self.resolve_token(&ename));
					ename = "".into();
					index = index - 1;
				}
				(State::SubVal,   s ) => {
					ename.push_str(s);
				}
				(State::Mod,     "[") => {
					state = State::ModEnd;
					let child = self.parse(index, line, lineindex, parent)?;
					index = child.index - 1;
					self.subs[current].modf = Some(Value::Sub(child.sub));
				}
				(State::Mod,     "]") => { state = State::End;}
				(State::Mod,      s ) => {
					state = State::ModName;
					ename = s.into();
				}
				(State::ModName,  " ") |
				(State::ModName, "\t") |
				(State::ModName, "\n") |
				(State::ModName, "\r") |
				(State::ModName, "]") => {
					// end mod name parsing and step back
					state = State::ModEnd;
					debug!("  mod: {} at position {}", ename, index);
					self.subs[current].modf = Some(self.resolve_token(&ename));
					ename = "".into();
					index = index - 1;
				}
				(State::ModName, "[") => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub end after mod name, found '[' at index {}", index)));
				}
				(State::ModName,  s ) => {
					ename.push_str(s);
				}
				(State::ModEnd,  "]") => { state = State::End; }
				(State::ModEnd,   s ) => {
					return Err(ParserError::new(index, line, lineindex, format!(
						"expected sub end after mod, found '{}' at index {}", s, index)));
				}
				(State::End,       _) => { unreachable!(); }
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
