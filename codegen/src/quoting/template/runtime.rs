use genco::quote;
use genco::lang::rust::Tokens;
use crate::config::YarnConfig;

pub fn all_tokens(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::collections::HashMap;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
		pub struct OutputHistory {
			start_yield: YieldCounter,
			lines: Vec<Instruction>,
		}
		
		#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
		pub struct Runtime {
			current_node: NodeTitle,
			current_line: Instruction,
			current_storage: $(&cfg.storage_direct),
			yield_counter: YieldCounter,
			output_history: OutputHistory,
			player_decisions: HashMap<YieldCounter, PlayerDecision>,
			snapshots: HashMap<YieldCounter, $(&cfg.storage_direct)>,
		}
		
		impl Runtime {
			pub fn current_node(&self) -> &NodeTitle { &self.current_node }
			pub fn current_line(&self) -> &Instruction { &self.current_line }
			pub fn yield_counter(&self) -> YieldCounter { self.yield_counter }
			pub fn output_history(&self) -> &OutputHistory { &self.output_history }
			pub fn player_decisions(&self) -> &HashMap<YieldCounter, PlayerDecision> { &self.player_decisions }
			pub fn storage(&self) -> &$(&cfg.storage_direct) { &self.current_storage }
			pub fn storage_mut(&mut self) -> &mut $(&cfg.storage_direct) { &mut self.current_storage }
			pub fn snapshots(&self) -> &HashMap<YieldCounter, $(&cfg.storage_direct)> { &self.snapshots }
		}
		
		impl Runtime {
			pub fn take_snapshot(&mut self) {
				if !self.snapshots.contains_key(&self.yield_counter) {
					self.snapshots.insert(self.yield_counter, self.current_storage.clone());
				}
			}
			
			fn memorize_decision(&mut self, yield_counter: YieldCounter, player_decision: PlayerDecision) {
				self.player_decisions.insert(yield_counter, player_decision);
			}
			
			fn push_yield(&mut self, yarn_yield: &YarnYield) {
				match yarn_yield {
					YarnYield::Instruction(instruction) => {
						self.output_history.lines.push(instruction.clone());
						self.yield_counter += 1;
					},
					YarnYield::Finished => {
						self.current_storage.increment_visited(self.current_node);
					},
				}
			}
		}
	}
}