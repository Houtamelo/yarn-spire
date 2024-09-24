use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;

pub fn all_tokens(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_snake_case)]
		#![allow(non_camel_case_types)]
		#![allow(unused)]
		
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		declarative_type_state::delegated_enum! {
			ENUM_OUT: {
				#[derive(Debug, Copy, Clone)]
				#[derive(PartialEq)]
				#[derive(Serialize, Deserialize)]
				pub enum Instruction {
					Speech(SpeechLine),
					Command(CommandLine),
					Options(OptionsFork),
				}
			}
			
			DELEGATES: {}
		}
		
		#[derive(Debug, Copy, Clone)]
		#[derive(PartialEq)]
		#[derive(Serialize, Deserialize)]
		pub enum YarnYield {
			Instruction(Instruction),
			Finished,
		}
		
		impl From<Instruction> for YarnYield {
			fn from(value: Instruction) -> Self { YarnYield::Instruction(value) }
		}
	}
}