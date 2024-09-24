use crate::shared_internal::*;

/// - A type that provides type-safe information about a variable that can be read/written inside a coroutine.
/// - You must implement this trait for every single variable that needs to be accessed/mutated inside a coroutine.
/// - This may seem like boilerplate, but this is what allows the compiler to **ensure type safety inside Nodes**. 
///   It also **ensures** that you can't make typos when writing variable names or perform operations that don't make sense for a given variable type,
/// such as comparing a `String` with a number.
///
/// # Example
///
/// Assuming you already have a type that implements [VariableStorage](crate::traits::VariableStorage):
/// ```rs
/// use yarn_spire::shared_internal::*;
///
/// #[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// pub struct CustomVariableStorage { 
/// 	pub player_name: String,
/// }
///
///
/// // You can implement [YarnVar](crate::traits::YarnVar) for a custom variable like this:
///
/// #[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// pub struct PlayerName; // This is just a marker type, it doesn't have to be the actual variable type, but it can.
///
/// impl YarnVar for PlayerName {
///     type Return = String;
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
/// let mut storage = CustomVariableStorage { player_name: "John" };
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
/// storage.set_var::<PlayerName>("Jane");
/// ```
///
/// Although using YarnVar requires additional "boilerplate" code, 
/// it saves time and effort in the long run by preventing runtime errors.
/// **This is the philosophy behind the design of this library.**
pub trait YarnVar {
	type Return;

	fn get(storage: &Storage) -> Self::Return;
	fn set(storage: &mut Storage, value: Self::Return);
}