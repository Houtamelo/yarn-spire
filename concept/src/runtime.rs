#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::shared_internal::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputHistory {
	start_yield: YieldCounter,
	lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Runtime {
	current_node: NodeTitle,
	current_line: Line,
	current_storage: Storage,
	yield_counter: YieldCounter,
	output_history: OutputHistory,
	player_decisions: HashMap<YieldCounter, PlayerDecision>,
	snapshots: HashMap<YieldCounter, Storage>,
}

impl Runtime {
	pub fn current_node(&self) -> &NodeTitle { &self.current_node }
	pub fn current_line(&self) -> &Line { &self.current_line }
	pub fn yield_counter(&self) -> YieldCounter { self.yield_counter }
	pub fn output_history(&self) -> &OutputHistory { &self.output_history }
	pub fn player_decisions(&self) -> &HashMap<YieldCounter, PlayerDecision> { &self.player_decisions }
	pub fn storage(&self) -> &Storage { &self.current_storage }
	pub fn storage_mut(&mut self) -> &mut Storage { &mut self.current_storage }
	pub fn snapshots(&self) -> &HashMap<YieldCounter, Storage> { &self.snapshots }
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
	
	fn push_yield(&mut self, r#yield: &YieldResult) {
		match r#yield {
			YieldResult::Line(instruction) => {
				self.output_history.lines.push(instruction.clone());
				self.yield_counter += 1;
			}
			YieldResult::Finished => {
				self.current_storage.increment_visited(self.current_node);
			}
		}
	}
}
