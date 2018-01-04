// Bracket2
// Christian Sdunek, 2018

#![feature(inclusive_range_syntax)] // range syntax additions
#![feature(target_feature, cfg_target_feature)] // target feature branching
#![feature(match_default_bindings, match_beginning_vert)] // simplify matching
#![feature(underscore_lifetimes, in_band_lifetimes, nll, nested_method_call)] // simplify lifetimes
#![feature(universal_impl_trait, conservative_impl_trait, dyn_trait)] // impl trait
#![feature(copy_closures, clone_closures)] // closures enhancement
#![feature(try_trait, termination_trait, catch_expr)] // error handling
#![feature(use_nested_groups, crate_in_paths, crate_visibility_modifier, non_modrs_mods)] // module handling
#![feature(decl_macro, proc_macro)] // macro improvements
#![feature(arbitrary_self_types)] // additional self method arguments
#![feature(generators, generator_trait)] // generators/coroutines
#![feature(fn_traits, unboxed_closures)] // function-like type traits
#![feature(never_type)] // new types
#![feature(const_fn)] // const functions
#![feature(const_generics)] // const generics

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate unicode_segmentation as unicode;
extern crate regex;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate simplelog;

mod program;
mod result;

fn main() -> Result<(), result::ParserError> {
	simplelog::TermLogger::init(log::LogLevelFilter::Debug, simplelog::Config::default()).ok();

	let code: String = r#"
[f[
  [[ [[ test &0 ] [[1]sub] ] ] [[f][[ [[[[&2][[0]cmp]]]neg] ]con]] ]
]]
[[2]f]"#.into();
	let program = program::Program::from_code(&code)?;
	
	println!("{}", &program.tree(1));

	Ok(())
}

