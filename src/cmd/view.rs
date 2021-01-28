/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use crate::libhooker::{LibhookerConfig, Target};
use color_eyre::eyre::Result;
use colorful::Colorful;
use std::fmt::Write;

pub fn view(mut config: LibhookerConfig, target: Option<Target>) -> Result<()> {
	assert!(std::path::PathBuf::from("/.procursus_strapped").is_file());
	let tweak_cfg = &mut config.tweak_configs;
	let targets: Vec<Target> = target.map(|x| vec![x]).unwrap_or_else(|| {
		let mut targets = Vec::new();
		if !tweak_cfg.default.should_skip() {
			targets.push(Target::Default);
		}
		targets.extend(
			tweak_cfg
				.paths
				.iter()
				.map(|(path, cfg)| (Target::Executable(path.clone()), cfg))
				.chain(
					tweak_cfg
						.bundles
						.iter()
						.map(|(bundle, cfg)| (Target::App(bundle.clone()), cfg)),
				)
				.filter(|(_, cfg)| !cfg.should_skip())
				.map(|(target, _)| target),
		);
		targets
	});

	for target in targets {
		let mut output = format!("configuration for {}\n", target.to_string().yellow());
		let cfg = target.get_config(tweak_cfg);
		writeln!(
			output,
			" {} is {}",
			"custom configuration".blue(),
			if cfg.custom_config {
				"on".green()
			} else {
				"off".red()
			}
		)?;
		writeln!(
			output,
			" {} is {}",
			"tweak loading".blue(),
			if cfg.enable_tweaks {
				"on".green()
			} else {
				"off".red()
			}
		)?;
		if cfg.custom_config {
			writeln!(output, " tweak loader is in {} mode", cfg.allow_or_deny)?;
			writeln!(output, " tweaks:")?;
			for (tweak, state) in cfg.tweak_configs.iter() {
				writeln!(
					output,
					"  {} is {}",
					tweak.as_str().yellow(),
					if cfg.allow_or_deny.check(*state) {
						"ENABLED".green()
					} else {
						"DISABLED".red()
					}
				)?;
			}
		}
		println!("{}", output);
	}

	Ok(())
}
