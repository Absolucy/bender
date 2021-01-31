/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use crate::libhooker::{LibhookerConfig, COMPAT_LIBHOOKER};
use color_eyre::eyre::Result;
use colorful::Colorful;
use std::collections::BTreeMap;

pub fn compat(mut config: LibhookerConfig, changes: BTreeMap<String, bool>) -> Result<()> {
	assert!(std::path::PathBuf::from("/.procursus_strapped").is_file());

	config.memory_compat_prefs.extend(changes);

	// If any entries are true (the default), filter them out of the map,
	// because saving defaults is a waste of effort and space.
	config.memory_compat_prefs = config
		.memory_compat_prefs
		.into_iter()
		.filter(|(_, v)| *v != COMPAT_LIBHOOKER)
		.collect();

	plist::to_file_binary(
		"/var/mobile/Library/Preferences/org.coolstar.libhooker.plist",
		&config,
	)?;

	println!(
		"Ensure to {} or {} your device to apply the changes!",
		"respring".yellow(),
		"userspace reboot".magenta()
	);

	Ok(())
}
