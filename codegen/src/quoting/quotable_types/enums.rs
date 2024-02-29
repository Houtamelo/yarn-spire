use crate::quoting::quotable_types::line_ids::InstructionKind;

pub const SUFFIX_ANY: &str = "_Line_Any";
pub const SUFFIX_SPEECH: &str = "_Line_Speech";
pub const SUFFIX_COMMAND: &str = "_Line_Command";
pub const SUFFIX_OPTIONS_FORK: &str = "_VirtualLine_OptionsFork";
pub const SUFFIX_OPTION_LINE: &str = "_Line_Option";

pub struct LineEnum<'a> {
	pub node_title: &'a str,
	pub raw_id: &'a str,
	pub instruction_kind: InstructionKind,
}

pub fn line_variant(line_id: &str) -> String {
	format!("L_{}", line_id)
}

pub fn enum_type_any(node_title: &str) -> String {
	format!("{}_Line_Any", node_title)
}

pub fn enum_type_speech(node_title: &str) -> String {
	format!("{}_Line_Speech", node_title)
}

pub fn enum_type_command(node_title: &str) -> String {
	format!("{}_Line_Command", node_title)
}

pub fn enum_type_options_fork(node_title: &str) -> String {
	format!("{}_VirtualLine_OptionsFork", node_title)
}

impl<'a> LineEnum<'a> {
	pub fn any_qualified(&self) -> String {
		format!("{}::{}", enum_type_any(self.node_title), self.variant_name())
	}

	pub fn typed_qualified(&self) -> String {
		return match self.instruction_kind {
			InstructionKind::Speech =>
				format!("{}::{}", enum_type_speech(self.node_title), self.variant_name()),
			InstructionKind::Command =>
				format!("{}::{}", enum_type_command(self.node_title), self.variant_name()),
			InstructionKind::OptionsFork =>
				format!("{}::{}", enum_type_options_fork(self.node_title), self.variant_name()),
		};
	}

	pub fn variant_name(&self) -> String {
		return line_variant(self.raw_id);
	}
}

pub struct OptionLineEnum<'a> {
	pub node_title: &'a str,
	pub raw_id: &'a str,
}

pub fn enum_type_option_line(node_title: &str) -> String {
	format!("{}_Line_Option", node_title)
}

impl<'a> OptionLineEnum<'a> {
	pub fn qualified(&self) -> String {
		format!("{}::{}", enum_type_option_line(self.node_title), self.variant_name())
	}

	pub fn variant_name(&self) -> String {
		return line_variant(self.raw_id);
	}
}
