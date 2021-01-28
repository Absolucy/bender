/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use once_cell::sync::Lazy;
use std::{ffi::OsStr, path::PathBuf};

// This is a "lazy static" global, it's initialized on first use,
// then reused for subsequent uses.
pub static TWEAKS: Lazy<Vec<String>> = Lazy::new(|| {
	assert!(std::path::PathBuf::from("/.procursus_strapped").is_file());
	let mut tweaks = Vec::new();
	// Start to read the contents of the directory `/usr/lib/TweakInject`
	let dir_contents = std::fs::read_dir("/usr/lib/TweakInject").unwrap_or_else(|err| {
		panic!(
			"/usr/lib/TweakInject dir not found, are you not using libhooker?\n{}",
			err
		)
	});
	// Now, iterate through all the files in TweakInject
	for entry in dir_contents {
		if let Some(file_name) = entry
			.ok()
			// Ensure that this entry is a file
			.filter(|entry| entry.path().is_file())
			// Ensure that this entry is has a ".dylib" extension
			.filter(|entry| entry.path().extension().and_then(OsStr::to_str) == Some("dylib"))
			// Convert the path to just the file name
			.map(|entry| entry.file_name())
			// Convert the file name, which is an OsString, to a normal String
			.map(|file_name| file_name.to_string_lossy().to_string())
		{
			// Well, this is a dylib, so it's a tweak, push it to the vec!
			tweaks.push(file_name);
		}
	}
	tweaks
});

pub fn fix_tweak_name(input: &str) -> Option<String> {
	let normalized_input = input.trim();
	let base_path = PathBuf::from("/usr/lib/TweakInject");
	if base_path.join(input).is_file() {
		// This tweak name is already correct, just return it.
		Some(input.to_string())
	} else if base_path.join(input).with_extension("dylib").is_file() {
		// Tweak name is just missing the dylib extension, add it.
		Some([input, "dylib"].join("."))
	} else {
		// Hm, we might have to look a bit harder. Let's look at *ALL* tweak dylibs
		let normalized_input = normalized_input.to_lowercase();
		for entry in std::fs::read_dir(&base_path).ok()? {
			if let Ok(entry) = entry {
				let path = entry.path();
				// Make sure this is actually a .dylib file we're looking at, don't waste time on folders or .plist files or whatever.
				if !path.is_file()
					|| !path
						.extension()
						.and_then(OsStr::to_str)
						.map(|x| x.to_lowercase() == "dylib")
						.unwrap_or(false)
				{
					continue;
				}
				// First, we do a case-insensitive comparison on full file names.
				let file_name = match path.file_name().and_then(OsStr::to_str) {
					Some(s) => s,
					None => continue,
				};
				let normalized_file_name = file_name.trim().to_lowercase();
				// If the case-insensitive comparison matches, then we return the name of the actual file.
				if normalized_file_name == normalized_input {
					return Some(file_name.to_string());
				}
				// Next, we check the file stem, which is the file name without the extension.
				let file_stem = match path.file_stem().and_then(OsStr::to_str) {
					Some(s) => s,
					None => continue,
				};
				// Same deal, if the case-insensitive comparison matches, then we return the name of the actual file.
				let normalized_file_stem = file_stem.trim().to_lowercase();
				if normalized_file_stem == normalized_input {
					return Some(file_name.to_string());
				}
			}
		}
		// Welp, we couldn't find it.
		None
	}
}
