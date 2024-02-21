use std::collections::HashMap;
use std::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use crate::compass::YarnCompass;
use crate::prelude::{NodeID, RuntimeError, VariableStorage, YarnInstruction};
use crate::{PlayerDecision, YieldCounter};

/// # Error that can happen when rewinding a dialogue.
/// 
/// See each variant's documentation for more information.
#[derive(Debug, Clone, Error)]
pub enum RewindError<TCmd: Clone + PartialEq + Debug> {
	/// During rewinding, we compare the newly generated instructions with the output_history,
	/// if any doesn't match, this error is returned.
	#[error("The output of the coroutine did not match the expected output.\n\
	Expected: {expected:?}\n\
	Got: {got:?}\n\
	At yield: {yield_counter:?}")]
	OutputMismatch {
		yield_counter: YieldCounter,
		expected: YarnInstruction<TCmd>,
		got: YarnInstruction<TCmd>,
	},
	/// The coroutine ended early, this error is returned when
	/// the coroutine ends before yielding the same number of times as the output_history.
	#[error("The coroutine ended early, expected {expected_yields} yields, ended at {ended_at}.\n\
	Maybe error: {maybe_error:?}")]
	EndedEarly {
		expected_yields: YieldCounter,
		ended_at: YieldCounter,
		maybe_error: Option<RuntimeError>,
	},
	/// Rewinding works by re-running the node from the beginning, providing identical inputs,
	/// as such, the process is susceptible to the same runtime errors that can happen during the normal execution.
	#[error("A runtime error happened during the rewind process: {0:?}")]
	RuntimeError(RuntimeError),
	/// The Node has already finished, it cannot be rewinded.
	#[error("Cannot rewind a Node that has already finished.")]
	CannotRewindFinishedNode,
}

/// Attempts to rewind a dialogue to a previous state, by `steps` amount of yields.
/// 
/// See [Node::rewind_by](crate::prelude::BoxedYarnNode::rewind_by) for more information.
pub fn attempt_rewind<
	'a,
	TStorage: VariableStorage,
	TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
	ID: NodeID<Storage = TStorage, Command = TCmd>
>(output_history: Vec<YarnInstruction<TCmd>>,
  player_decisions: HashMap<YieldCounter, PlayerDecision>,
  storage_at_start: Box<TStorage>,
  steps: usize) 
	-> Result<YarnCompass<'a, TStorage, TCmd, ID>, RewindError<TCmd>>
{
	let old_output_len = output_history.len();
	if steps >= old_output_len { // rewind to the start of the node
		return Ok(ID::play(storage_at_start));
	}

	let rewinded_len = old_output_len - steps;
	
	let output_history = {
		let mut temp = output_history;
		temp.truncate(rewinded_len);
		temp
	};

	let player_decisions = {
		let mut temp = player_decisions;
		for yield_index in rewinded_len..old_output_len {
			temp.remove(&yield_index);
		}
		temp
	};
	
	let first_compass = ID::play(storage_at_start);

	return 
		output_history
			.into_iter()
			.enumerate()
			.map(|(index, instruction)| (index + 1, instruction))
			.try_fold(first_compass, |compass, (yield_index, expected)| {
				
				let player_decision =
					player_decisions.get(&yield_index).cloned();

				match compass {
					YarnCompass::Speech(speech_compass) => {
						if let Some(player_decision) = player_decision {
							return Err(RewindError::RuntimeError(
								RuntimeError::UnexpectedPlayerDecision {
									got: player_decision,
								}));
						}

						return if let YarnInstruction::Speech(expected_speech) = &expected
							&& expected_speech == &speech_compass.speech {
							Ok(speech_compass.next())
						} else {
							Err(RewindError::OutputMismatch {
								yield_counter: yield_index,
								expected,
								got: YarnInstruction::Speech(speech_compass.speech),
							})
						};
					},
					YarnCompass::Command(command_compass) => {
						if let Some(player_decision) = player_decision {
							return Err(RewindError::RuntimeError(
								RuntimeError::UnexpectedPlayerDecision {
									got: player_decision,
								}));
						}

						return if let YarnInstruction::Command(expected_command) = &expected
							&& expected_command == &command_compass.command {
							Ok(command_compass.next())
						} else {
							Err(RewindError::OutputMismatch {
								yield_counter: yield_index,
								expected,
								got: YarnInstruction::Command(command_compass.command),
							})
						};
					},
					YarnCompass::Choices(choices_compass) => {
						let Some(player_decision) = player_decision
							else {
								return Err(RewindError::RuntimeError(
									RuntimeError::ExpectedPlayerDecision {
										options_provided: choices_compass.options.clone(),
									}));
							};

						return if let YarnInstruction::Choices(expected_choices) = &expected
							&& expected_choices == &choices_compass.options {
							Ok(choices_compass.next(player_decision))
						} else {
							Err(RewindError::OutputMismatch {
								yield_counter: yield_index,
								expected,
								got: YarnInstruction::Choices(choices_compass.options),
							})
						};
					},
					YarnCompass::NodeEnd(_, maybe_error) => {
						return Err(RewindError::EndedEarly {
							expected_yields: old_output_len,
							ended_at: yield_index,
							maybe_error
						});
					}
				}
			});
}