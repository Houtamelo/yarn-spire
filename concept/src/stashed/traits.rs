use std::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use crate::compass::YarnCompass;

/// The original YarnSpinner's [tracking setting](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header).
/// 
/// 
/// Currently not used in this library, but you can access it if you want to mimic YarnSpinner's behavior,
/// to do so, access [NodeID::TRACKING](crate::traits::NodeID::TRACKING).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackingSetting {
	Always,
	Never,
}

/// - A type that provides type-safe information about a variable that can be read/written inside a coroutine.
/// - You must implement this trait for every single variable that needs to be accessed/mutated inside a coroutine.
/// - This may seem like boilerplate, but this is what allows the compiler to **ensure type safety inside Nodes**. 
/// It also **ensures** that you can't make typos when writing variable names or perform operations that don't make sense for a given variable type,
/// such as comparing a `String` with a number.
/// 
/// # Example
/// 
/// Assuming you already have a type that implements [VariableStorage](crate::traits::VariableStorage):
/// ```
/// use yarn_spinner_aot::prelude::*;
///
/// #[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// pub struct CustomVariableStorage { 
/// 	pub player_name: String,
/// }
///
/// impl VariableStorage for CustomVariableStorage { }
///
/// // You can implement [YarnVar](crate::traits::YarnVar) for a custom variable like this:
///
/// #[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// pub struct PlayerName; // This is just a marker type, it doesn't have to be the actual variable type, but it can.
///
/// impl YarnVar for PlayerName {
///     type Return = String;
///     type TVariableStorage = CustomVariableStorage;
///
///     fn get(storage: &Self::VariableStorage) -> Self::Return {
///         return storage.player_name.clone();
///     }
///
///     fn set(storage: &mut Self::VariableStorage, value: Self::Return) {
///         storage.player_name = value;
///     }
/// }
///
/// // Then, inside coroutines, this is how they get used:
/// let mut storage = CustomVariableStorage { player_name: "John".to_string() };
///
/// // get
/// // Notice that we don't pass the variable `key` or `name` as an argument (which is the case in the original YarnSpinner's get),
/// // instead, we pass the type that indicates everything we need to know about the variable at compile time.
/// if storage.get_var::<PlayerName>() == "John" { 
///     // play some dialogue
/// }
///
/// // set
/// // Notice that we don't pass the variable `key` or `name` as an argument (which is the case in the original YarnSpinner's set),
/// // instead, we pass the type that indicates everything we need to know about the variable at compile time.
/// storage.set_var::<PlayerName>("Jane".to_string());
/// ```
/// 
/// Although using YarnVar requires additional "boilerplate" code, 
/// it saves time and effort in the long run by preventing runtime errors.
/// **This is the philosophy behind the design of this library.**
pub trait YarnVar: Sized + Clone + Serialize + DeserializeOwned {
	type Return;
	type VariableStorage: VariableStorage;

	fn get(storage: &Self::VariableStorage) -> Self::Return;
	fn set(storage: &mut Self::VariableStorage, value: Self::Return);
}

/// A [VariableStorage] works similarly to a [VariableStorage](https://docs.yarnspinner.dev/using-yarnspinner-with-unity/components/variable-storage)
/// in the Unity version of YarnSpinner. 
/// 
/// A marker trait that tells YarnSpinner what is the type of the storage that will be used in a given scene.
///
/// You don't have to (and probably shouldn't) override the methods of this trait. 
/// 
/// For more information, see [YarnVar](crate::traits::YarnVar). 
pub trait VariableStorage: Sized + Clone + Serialize + DeserializeOwned + PartialEq  {
	/// Provided a given variable marker type, returns a copy of its value contained in the storage. 
	/// This method is used inside coroutines to fetch variable values, using the regular syntax: `$variable_name`
	fn get_var<T: YarnVar<VariableStorage = Self>>(&self) -> T::Return {
		return T::get(self);
	}
	
	/// Provided a given variable marker type, sets the value of the variable contained in the storage.
	/// This method is used inside coroutines to set variable values, using the `set command`: `<<set $variable_name = value>>`
	fn set_var<T: YarnVar<VariableStorage = Self>>(&mut self, value: T::Return) {
		T::set(self, value);
	}

	/// Returns `true` if the node has been visited at least once.
	///
	/// - This is used to implement the original 
	/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
	/// - This will always return `false` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
	///
	/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
	fn visited<T: NodeID<Storage = Self>>(&self) -> bool {
		return T::visited(self);
	}

	/// Returns the number of times the node has been visited.
	///
	/// - This is used to implement the original 
	/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
	/// - This will always return `0` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
	///
	/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
	fn visited_count<T: NodeID<Storage = Self>>(&self) -> usize {
		return T::visited_count(self);
	}
}

/// A marker trait that tells YarnSpinner exactly which Node to play.
/// 
/// 
/// The proc-macro [yarn_file!](yarn_spinner_aot_proc_macros::yarn_file) will generate a NodeID for each node in the Yarn file.
/// 
/// ___
/// 
/// # Playing a Node
/// 
/// - Assuming you invoked the macro [yarn_file!](yarn_spinner_aot_proc_macros::yarn_file):
/// `yarn_file!("/node_rainy_day.yarn", CustomVariableStorage, CustomCommand)`
/// - Also assuming that the file contains a node titled: "Rainy Day"
/// 
/// 
/// On that same file, the proc-macro will generate a type like this:
///
/// ```rs
/// #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// pub struct RainyDay;
/// 
/// impl NodeID for RainyDay {
///    type VariableStorage = CustomVariableStorage; // provided in macro invocation
///    type Command = CustomCommand; // provided in macro invocation
/// 
///    const METADATA: &'static str = "cloudy, city"; // read from the file
///    const TRACKING: Option<TrackingSetting> = Some(TrackingSetting::Never); // read from the file
/// 
///     fn play<'a>(storage_to_backup: &Box<Self::VariableStorage>)
/// 			   -> BoxedYarnNode<'a, Self::VariableStorage, Self::Command, Self> {
///         let coroutine = ScopedCoroutine::new(|| {
///             // ... coroutine code, generated by reading the lines in the file 
/// 		});
/// 
///         return BoxedYarnNode::new(RainyDay, coroutine, storage_to_backup.clone());
/// 	}
/// }
///
/// // Then, at runtime, if you want to play that Node, you can just:
/// 
/// let storage = ...;
/// 
/// let compass = RainyDay::play(storage);
/// ```
///
/// The compass is a struct containing the yield of the coroutine and the information necessary to resume it,
/// to continue playing the node, you can just match on the compass and call the `next` method.
/// 
/// 
/// See more on [YarnCompass](crate::compass::YarnCompass).
pub trait NodeID: Debug + Clone + Copy + PartialEq + Serialize + DeserializeOwned {
	type Storage: VariableStorage;
	type Command: Clone + PartialEq + Debug + Serialize + DeserializeOwned;
	
	/// The list of comma-separated tags that come after `tags:` above the Node declaration.
	///
	/// Check the [original documentation](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#tags-in-nodes) for more information.
	const TAGS: &'static [&'static str];
	
	/// See [TrackingSetting](crate::traits::TrackingSetting).
	/// 
	/// Unlike the original YarnSpinner, this package doesn't have access to your entire project (as there is no concept of project here).
	/// 
	/// So, our tracking is implemented differently:
	/// - The setting `TRACKING` defaults to `Always`, if it is not declared.
	/// - The trait [NodeID](crate::prelude::NodeID) has the function `visited_count`
	/// that always return `0`, but is overridable.
	/// - If `TRACKING` == `Always`, then the proc-macro will override the implementation of `visited_count`
	/// for the generated NodeID struct, in the following way:
	/// 
	/// Assuming your node is called: `"RomanticEvening"`
	/// ```rs
	/// use yarn_spinner_aot::prelude::*;
	/// struct RomanticEvening;
	/// 
	/// impl NodeID for RomanticEvening {
	///     //.. other impls ..//
	/// 
	///     fn visited_count(_storage: &Self::VariableStorage) -> usize {
	///        return _storage.get_var::<Visited_RomanticEvening>(); 
	///     }
	/// }
	/// ```
	/// 
	/// ### Note that there's a new type being used here: `Visited_RomanticEvening`.
	///
	/// ### The proc-macro **will not** declare that type. 
	///
	/// Since it accesses your variable storage, it is your responsibility to define it.
	///
	/// Here's an example of how it can look like: 
	/// ```rs
	/// use yarn_spinner_aot::prelude::*;
	/// 
	/// #[allow(non_camel_case_types)]
	/// #[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
	/// pub struct VisitedCount_RomanticEvening;
	/// 
	/// impl YarnVar for VisitedCount_RomanticEvening {
	///    type Return = usize;
	///    type VariableStorage = CustomVariableStorage;
	/// 
	///    fn get(storage: &Self::VariableStorage) -> Self::Return {
	/// 	   return storage.visited_count_romantic_evening; // assuming this field exists
	///    }
	/// 
	///    fn set(storage: &mut Self::VariableStorage, value: Self::Return) { 	/// 
	///        storage.visited_count_romantic_evening = value; // assuming this field exists
	///    }
	/// }
	/// ```
	/// 
	/// # We can't do magic
	/// The tracking setting is updated when the player finishes a node, meaning the coroutine returned its final value, 
	/// which is always: [NodeFinished](crate::prelude::YarnYield::NodeFinished).
	/// 
	/// This also means that, if the node ends early, the tracking setting will not be updated.
	/// 
	/// - The original YarnSpinner is made on top of C#, which allows for much more flexible memory management(or the lack of).
	/// 
	/// On the other hand, this crate keeps ownership tight and explicit, which means there is 
	/// no reasonable and ergonomic way for us to know when you finished playing a node.
	///
	/// ### This means that: 
	/// - If you drop the coroutine or the compass before it finishes, the tracking setting will not be updated.
	/// In those cases, you should update the tracking setting manually.
	/// - Note that, unlike the original YarnSpinner, we don't provide a default implementation of the command `jump`. 
	/// If you want to jump between nodes, you have to do it manually, and when you do, make sure to update the tracking setting.
	/// 
	/// # If you do not care about tracking the number of visits
	/// You will have to add the `"tracking: Never"`
	/// metadata to your nodes, otherwise the compiler will complain about the missing type.
	const TRACKING: Option<TrackingSetting>;

	/// Any other line above the Node declaration, that doesn't fit into either `title`, `tags` or `tracking`.
	///
	/// This means you can write your own custom metadata, and YarnSpinner will place it here. 
	const CUSTOMS: &'static [&'static str];

	/// Starts playing the node defined by the implementer of this trait.
	/// 
	/// 
	/// See trait-level information for more.
	/// 
	/// # Example
	/// ```rs
	/// use yarn_spinner_aot::prelude::*;
	/// 
	/// let storage: Box<VariableStorage> = ...;
	/// 
	/// let mut compass: YarnCompass<_> = NodeID::play(storage);
	/// 
	/// loop {
	///     match compass {
	///         YarnCompass::Speech(speech_compass) => { 
	///             /.. process speech ../
	///             compass = speech_compass.next();
	/// 	    },
	///  	    YarnCompass::Command(command_compass) => { 
	///             /.. process command ../
	///             compass = command_compass.next();
	///         }
	/// 	    YarnCompass::Choices(choices_compass) => { 
	///             let player_decision = /.. process choices ../;
	/// 			compass = choices_compass.next(player_decision);
	///         }
	///     }
	/// }
	/// 
	/// ```
	fn play<'a>(original_storage: Box<Self::Storage>)
	            -> YarnCompass<'a, Self::Storage, Self::Command, Self>;
	
	/// Returns `true` if the node has been visited at least once.
	/// 
	/// - This is used to implement the original 
	/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
	/// - This will always return `false` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
	///
	/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
	fn visited(_storage: &Self::Storage) -> bool {
		return Self::visited_count(_storage) > 0;
	}
	
	/// Returns the number of times the node has been visited.
	///
	/// - This is used to implement the original 
	/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
	/// - This will always return `0` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
	/// 
	/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
	fn visited_count(_storage: &Self::Storage) -> usize {
		return 0;
	}
}

