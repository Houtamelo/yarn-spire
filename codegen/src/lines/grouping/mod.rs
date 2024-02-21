use crate::lines::grouping::scope::{read_next_scope, YarnScope};
use crate::lines::raw::RawLine;
use anyhow::Result;

pub mod choices;
pub mod if_branch;
pub mod scope;
pub mod quoted;

pub fn raw_lines_into_scopes(raw_lines: Vec<RawLine>) -> Result<Vec<YarnScope>> {
	let mut stream_buffer =
		raw_lines
			.into_iter()
			.peekable();

	let mut scopes = Vec::new();
	while stream_buffer.peek().is_some() {
		let next_scope_option =
			read_next_scope(-1, &mut stream_buffer)?;

		if let Some(next_scope) = next_scope_option {
			scopes.push(next_scope);
		}
	}

	return Ok(scopes);
}