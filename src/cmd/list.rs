/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use crate::{
	cmd::CmdList,
	libhooker::{LibhookerConfig, COMPAT_LIBHOOKER},
	APPS, DAEMONS, TWEAKS,
};
use color_eyre::eyre::Result;
use colorful::Colorful;
use std::ffi::OsStr;

pub fn list(what: CmdList, cfg: LibhookerConfig) -> Result<()> {
	assert!(std::path::PathBuf::from("/.procursus_strapped").is_file());
	let default = &cfg.tweak_configs.default;
	match what {
		CmdList::Apps => {
			for app in APPS.iter() {
				println!("{} [{}]", app.name, app.identifier.as_str().dark_gray());
			}
		}
		CmdList::Tweaks => {
			for tweak_name in TWEAKS.iter() {
				let readable_tweak_name = tweak_name
					.strip_suffix(".dylib")
					.map(|x| x.to_string())
					.unwrap_or_else(|| tweak_name.clone());
				println!(
					"{}: {} by default{}",
					readable_tweak_name,
					if default.will_tweak_load(&tweak_name) {
						"ENABLED".light_green()
					} else {
						"DISABLED".red()
					},
					if cfg
						.memory_compat_prefs
						.get(tweak_name)
						.cloned()
						.unwrap_or(COMPAT_LIBHOOKER)
						== COMPAT_LIBHOOKER
					{
						String::new()
					} else {
						format!(", with {}", "substrate compatibility mode".red())
					}
				)
			}
		}
		CmdList::Daemons => {
			let mut daemons = DAEMONS
				.iter()
				.filter_map(|d| d.file_name().and_then(OsStr::to_str))
				.collect::<Vec<&str>>();
			daemons.sort_unstable();
			for daemon in daemons {
				println!("{}", daemon);
			}
		}
	}
	Ok(())
}
