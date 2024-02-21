#![allow(non_camel_case_types)]

use std::fmt::Debug;
use corosensei::ScopedCoroutine;
use corosensei::stack::DefaultStack;
use houtamelo_utils::own;
use houtamelo_utils::prelude::CountOrMore;
use serde::{Deserialize, Serialize};
use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
struct VarTest {}

impl YarnVar for VarTest {
	type Return = i32;
	type VariableStorage = StorageTestTest;

	fn get(storage: &StorageTestTest) -> Self::Return {
		return storage.var;
	}
	
	fn set(storage: &mut StorageTestTest, value: Self::Return) {
		storage.var = value;
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct StorageTestTest {
	pub var: i32,
}

impl VariableStorage for StorageTestTest { }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum CmdTest {
	
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
struct Scene_Mansion_Meeting;


impl NodeID for Scene_Mansion_Meeting {
	type Storage = StorageTestTest;
	type Command = CmdTest;
	
	const METADATA: Option<&'static str> = Some("This is a test scene");
	const TRACKING: Option<TrackingSetting> = None;
	
	fn play<'a>(original_storage: Box<Self::Storage>)
	            -> YarnCompass<'a, Self::Storage, Self::Command, Self> {

		let coroutine: ScopedCoroutine<
			'a,
			(Box<StorageTestTest>, Option<PlayerDecision>),
			(Box<StorageTestTest>, YarnInstruction<CmdTest>),
			(Box<StorageTestTest>, Result<(), RuntimeError<CmdTest>>),
			DefaultStack
		> = ScopedCoroutine::new(|yielder, (mut storage, mut _player_decision)| {
			storage = yielder.suspend(
				(storage, YarnInstruction::Speech(Speech {
					speaker: Some(own!("Houtamelo")),
					text: own!("This is the first line"),
					metadata: None,
				}))).0;

			storage = yielder.suspend(
				(storage, YarnInstruction::Speech(
					Speech {
						speaker: Some(own!("Houtamelo")),
						text: own!("This is the second line"),
						metadata: None,
					}))).0;

			let options = CountOrMore::new(
				[
					ChoiceOption {
						metadata: None,
						text: own!("Option A"),
					}
				], vec![
					ChoiceOption {
						metadata: None,
						text: own!("Option B"),
					}
				]
			);

			// we expect a choice input here
			(storage, _player_decision) = yielder.suspend((storage, YarnInstruction::Choices(options)));

			match _player_decision {
				Some(0) => {
					storage = yielder.suspend(
						(storage, YarnInstruction::Speech(
							Speech {
								speaker: Some(own!("Houtamelo")),
								text: own!("You chose A!"),
								metadata: None,
							}))).0;
					storage = yielder.suspend(
						(storage, YarnInstruction::Speech(
							Speech {
								speaker: Some(own!("Houtamelo")),
								text: own!("This branch is inside A"),
								metadata: None,
							}))).0;
				},
				Some(1) => {
					storage = yielder.suspend(
						(storage, YarnInstruction::Speech(
							Speech {
								speaker: Some(own!("Houtamelo")),
								text: own!("You chose B!"),
								metadata: None,
							}))
					).0;
					storage = yielder.suspend(
						(storage, YarnInstruction::Speech(
							Speech {
								speaker: Some(own!("Houtamelo")),
								text: own!("This branch is inside B"),
								metadata: None,
							}))).0;
				},
				_ => {
					let options_provided =
						CountOrMore::new(
							[
								ChoiceOption {
									metadata: None,
									text: own!("Option A"),
								}
							], vec![
								ChoiceOption {
									metadata: None,
									text: own!("Option B"),
								}
							]
						);

					return match _player_decision {
						Some(invalid) => (storage,
							Err(RuntimeError::InvalidPlayerDecision {
								options_provided,
								got: Some(invalid),
							})),
						None => (storage,
							Err(RuntimeError::ExpectedPlayerDecision {
								options_provided,
							})
						),
					};
				}
			}

			return (storage, Ok(()));
		});

		let yarn_scene = BoxedYarnNode::new(Scene_Mansion_Meeting, &original_storage, coroutine);
		return yarn_scene.next((original_storage, None));
	}
}

#[test]
fn test() {
	let storage = StorageTestTest { var: 1 };
	let result = storage.get_var::<VarTest>();
	assert_eq!(result, 1);
}