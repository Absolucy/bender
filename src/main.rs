/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

#[macro_use]
extern crate objc;

pub mod apps;
pub mod cmd;
pub mod daemons;
pub mod libhooker;
pub mod tweaks;

use crate::{
	cmd::CmdOpts,
	libhooker::{LibhookerConfig, Target, TweakMode},
};
use clap::Clap;
use color_eyre::eyre::Result;
use colorful::Colorful;
use std::collections::BTreeMap;

pub use crate::{apps::APPS, daemons::DAEMONS, tweaks::TWEAKS};

fn main() -> Result<()> {
	color_eyre::install()?;

	assert!(std::path::PathBuf::from("/.procursus_strapped").is_file());

	let opts = CmdOpts::parse();
	let libhooker_config: LibhookerConfig =
		plist::from_file("/var/mobile/Library/Preferences/org.coolstar.libhooker.plist")
			.unwrap_or_default();
	match opts {
		CmdOpts::List(opt) => cmd::list::list(opt, libhooker_config)?,
		CmdOpts::Config {
			allow,
			deny,
			custom_config,
			enable_tweaks,
			add,
			remove,
			target,
		} => {
			let target = Target::from(target);
			let mode = if allow {
				TweakMode::Allow
			} else if deny {
				TweakMode::Deny
			} else {
				target.get_tweak_mode(&libhooker_config.tweak_configs)
			};
			println!(
				"Configuring {} in {} mode",
				target.to_string().yellow(),
				mode
			);
			let mut changes = BTreeMap::<String, bool>::new();
			if add.contains(&"all".into()) {
				TWEAKS.iter().for_each(|tweak| {
					changes.insert(tweak.clone(), true);
				});
			} else {
				add.into_iter()
					.map(|tweak| {
						tweaks::fix_tweak_name(&tweak).unwrap_or_else(|| {
							eprintln!(
								"Tweak '{}' not found!\nUse `{}` to see a list of available tweaks!",
								tweak.red(),
								"bender list tweaks".green()
							);
							std::process::exit(1);
						})
					})
					.for_each(|tweak| {
						println!(
							"{} {}",
							match mode {
								TweakMode::Allow => "ALLOWING".green(),
								TweakMode::Deny => "DENYING".red(),
							},
							tweak.strip_suffix(".dylib").unwrap_or(&tweak)
						);
						changes.insert(tweak, true);
					});
			}

			if remove.contains(&"all".into()) {
				TWEAKS.iter().for_each(|tweak| {
					changes.insert(tweak.clone(), false);
				});
			} else {
				remove
					.into_iter()
					.map(|tweak| {
						tweaks::fix_tweak_name(&tweak).unwrap_or_else(|| {
							eprintln!(
								"Tweak '{}' not found!\nUse `{}` to see a list of available tweaks!",
								tweak.red(),
								"bender list tweaks".green()
							);
							std::process::exit(1);
						})
					})
					.for_each(|tweak| {
						println!(
							"{} {}",
							match mode {
								TweakMode::Allow => "DENYING".red(),
								TweakMode::Deny => "ALLOWING".green(),
							},
							tweak.strip_suffix(".dylib").unwrap_or(&tweak)
						);
						changes.insert(tweak, false);
					});
			}

			cmd::config::configure(
				libhooker_config,
				target,
				custom_config,
				enable_tweaks,
				mode,
				changes,
			)?;
		}
		CmdOpts::View { target } => {
			let target = target.map(Target::from);
			cmd::view::view(libhooker_config, target)?;
		}
	}
	Ok(())
}
