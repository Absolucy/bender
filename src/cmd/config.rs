/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use crate::libhooker::{LibhookerConfig, Target, TweakMode};
use color_eyre::eyre::Result;
use colorful::Colorful;
use std::collections::BTreeMap;

pub fn configure(
	mut config: LibhookerConfig,
	target: Target,
	custom_config: Option<bool>,
	enable_tweaks: Option<bool>,
	mode: TweakMode,
	changes: BTreeMap<String, bool>,
) -> Result<()> {
	assert!(std::path::PathBuf::from("/.procursus_strapped").is_file());
	let tweak_config = target.get_config(&mut config.tweak_configs);
	tweak_config.allow_or_deny = mode;
	tweak_config.custom_config = custom_config.unwrap_or(tweak_config.custom_config);
	tweak_config.enable_tweaks = enable_tweaks.unwrap_or(tweak_config.enable_tweaks);
	println!(
		"{} is {}",
		"custom configuration".blue(),
		if tweak_config.custom_config {
			"on".green()
		} else {
			"off".red()
		}
	);
	println!(
		"{} is {}",
		"tweak loading".blue(),
		if tweak_config.custom_config {
			"on".green()
		} else {
			"off".red()
		}
	);
	tweak_config.tweak_configs.extend(changes);

	plist::to_file_binary(
		"/var/mobile/Library/Preferences/org.coolstar.libhooker.plist",
		&config,
	)?;

	Ok(())
}
