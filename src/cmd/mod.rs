/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

pub mod compat;
pub mod config;
pub mod list;
pub mod view;

use clap::{Clap, ValueHint};

fn parse_yes_no(s: &str) -> Result<bool, &'static str> {
	match s.to_lowercase().trim() {
		"true" | "yes" | "y" | "on" => Ok(true),
		"false" | "no" | "n" | "off" => Ok(false),
		_ => Err("expected true/yes/y or false/no/n"),
	}
}

#[derive(Clap, Debug)]
#[clap(
	author,
	about,
	version,
	after_help = r#"      _
     ( )
      H
      H
     _H_
  .-'-.-'-.
 /         \
|           |
|   .-------'._
|  / /  '.' '. \   Yeah well, I'm gonna go make my own configurator...
|  \ \ @   @ / /          With (lib)blackjack! And (lib)hookers!
|   '---------'
|    _______|
|  .'-+-+-+|
|  '.-+-+-+|
|    """""" |
'-.__   __.-'
     """

copyright (C) 2021, aspen
All rights reserved.
This software may only be used in situations where the use of libhooker is permitted."#
)]
pub enum CmdOpts {
	/// List tweaks, configurations, et cetera
	List(CmdList),
	/// Configure libhooker.
	Config {
		/// Change the tweak loader to only load selected tweaks.
		#[clap(long, group = "allowdeny")]
		allow: bool,
		/// Change the tweak loader to deny loading selected tweaks.
		#[clap(long, group = "allowdeny")]
		deny: bool,
		/// Whether to enable custom configuration or not for the target (true/on/yes/y/false/off/no/n)
		#[clap(long, parse(try_from_str = parse_yes_no), alias = "custom")]
		custom_config: Option<bool>,
		/// Whether to set tweaks as enabled or not for the target (true/on/yes/y/false/off/no/n)
		#[clap(long, parse(try_from_str = parse_yes_no), alias = "tweaks")]
		enable_tweaks: Option<bool>,
		/// Set these tweaks "on" in the configuration,
		/// denying them if "deny" mode is on,
		/// allowing them if "allow" mode is on.
		/// You can put "all" here to set all tweaks "on".
		#[clap(short, long, alias = "enable")]
		add: Vec<String>,
		/// Set these tweaks "off" in the configuration,
		/// allowing them if "deny" mode is on,
		/// denying them if "allow" mode is on.
		/// You can put "all" here to set all tweaks "off".
		#[clap(short, long, alias = "disable")]
		remove: Vec<String>,
		/// The target to configure. Either an app bundle,
		/// executable path, daemon/service name, "default",
		///or "springboard".
		#[clap(value_hint = ValueHint::ExecutablePath)]
		target: String,
	},
	/// View an existing configuration.
	View {
		/// The target to view configuration for.
		/// Either an app bundle, executable path, "default",
		/// daemon/service name, or "springboard".
		/// Leave blank to see all currently set configurations.
		#[clap(value_hint = ValueHint::ExecutablePath)]
		target: Option<String>,
	},
	/// Configure the compatibility mode for tweaks.
	Compat {
		/// Use the libhooker default compatibility mode for these tweaks.
		/// This SHOULD work on 99% of tweaks, especially newer ones.
		#[clap(short, long, aliases = &["lh", "new", "default"], required_unless_present = "substrate")]
		libhooker: Vec<String>,
		/// Use the Substrate compatibility mode for these tweaks.
		/// This may allow some poorly written / outdated tweaks to work.
		/// However, this will increase memory usage.
		#[clap(short, long, alias = "old", required_unless_present = "libhooker")]
		substrate: Vec<String>,
	},
}

#[derive(Clap, Debug)]
pub enum CmdList {
	/// List the installed tweaks that can be enabled/disabled.
	#[clap(alias = "tweak")]
	Tweaks,
	/// List the available app bundles.
	#[clap(alias = "app")]
	Apps,
	/// List the available daemons.
	#[clap(aliases = &["daemon", "service", "services"])]
	Daemons,
}
