pub fn char_index_to_byte_index(str_ref: &str, char_index: usize) -> usize {
	return str_ref
		.chars()
		.take(char_index)
		.fold(0, |byte_index, ch| 
			byte_index + ch.len_utf8());
}

#[test]
fn test_char_index_to_byte_index() {
	assert_eq!(char_index_to_byte_index("hello", 3), 3);
	assert_eq!(char_index_to_byte_index("utf16: \u{1F600} \u{1F601} \u{1F602}", 9), 12);
	assert_eq!(char_index_to_byte_index("utf16: \u{1F600} \u{1F601} \u{1F602}", 10), 16);
	assert_eq!(char_index_to_byte_index("utf16: \u{1F600} \u{1F601} \u{1F602}", 11), 17);
}

pub fn next_byte_index(line: &str, mut current_index: usize) -> usize {
	let line_len = line.len();
	if line_len <= current_index {
		return current_index;
	}

	while current_index < line_len
		&& !line.is_char_boundary(current_index) {
		current_index += 1;
	}

	return current_index;
}

pub fn indent_level(line: &impl AsRef<str>) -> isize {
	return line
		.as_ref()
		.chars()
		.scan(0, |sum, ch|
			match ch {
				' ' => { 
					*sum += 1;
					Some(*sum)
				},
				'\t' => {
					*sum += 4;
					Some(*sum)
				},
				_ => None,
			})
		.last()
		.unwrap_or(0);
}