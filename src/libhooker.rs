/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use crate::{apps::fix_app_name, DAEMONS, TWEAKS};
use colorful::Colorful;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{collections::BTreeMap, ffi::OsStr, fmt::Display, path::PathBuf};

// Note: we use BTreeMap over HashMap, because BTreeMaps are sorted by their keys,
// and we don't want to randomly shift the order of everything whenever we save the config.

pub const COMPAT_LIBHOOKER: bool = true;
pub const COMPAT_SUBSTRATE: bool = false;

const fn default_true() -> bool {
	true
}

const fn default_false() -> bool {
	false
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
pub enum TweakMode {
	Allow = 0,
	Deny = 1,
}

impl TweakMode {
	pub fn check(&self, val: bool) -> bool {
		let mode = *self as u8 != 0;
		!((mode && val) || !(mode || val))
	}
}

impl Display for TweakMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			// fyi: this isn't some statement about how whitelist/blacklist is bad or whatever,
			// i'm just using the same terminology that libhooker does, as it uses the key "allowOrDeny" in the plist.
			TweakMode::Allow => write!(f, "{}", "allowlist".green()),
			TweakMode::Deny => write!(f, "{}", "denylist".red()),
		}
	}
}

impl Default for TweakMode {
	fn default() -> Self {
		Self::Deny
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibhookerConfig {
	#[serde(rename = "tweakconfigs", default)]
	pub tweak_configs: ConfigTypes,
	#[serde(rename = "webProcessTweaks", default = "default_true")]
	pub web_process_tweaks: bool,
	#[serde(
		rename = "memPrefs",
		default,
		skip_serializing_if = "BTreeMap::is_empty"
	)]
	pub memory_compat_prefs: BTreeMap<String, bool>,
}

impl Default for LibhookerConfig {
	fn default() -> Self {
		Self {
			tweak_configs: ConfigTypes::default(),
			web_process_tweaks: true,
			memory_compat_prefs: BTreeMap::default(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigTypes {
	#[serde(default)]
	pub paths: BTreeMap<PathBuf, TweakConfig>,
	#[serde(default)]
	pub bundles: BTreeMap<String, TweakConfig>,
	#[serde(default, skip_serializing_if = "TweakConfig::should_skip")]
	pub default: TweakConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TweakConfig {
	#[serde(rename = "customConfig", default = "default_false")]
	pub custom_config: bool,
	#[serde(rename = "enableTweaks", default = "default_true")]
	pub enable_tweaks: bool,
	#[serde(rename = "tweakConfigs")]
	pub tweak_configs: BTreeMap<String, bool>,
	#[serde(rename = "allowDeny", default)]
	pub allow_or_deny: TweakMode,
}

impl TweakConfig {
	/// Check to see whether a tweak will load or not,
	/// based on the tweak mode (allowOrDeny) and it's
	/// boolean value.
	pub fn will_tweak_load(&self, name: &str) -> bool {
		if !self.enable_tweaks {
			return false;
		}
		match self.tweak_configs.get(name) {
			Some(s) => self.allow_or_deny.check(*s),
			None => false,
		}
	}

	/// Check to see if this tweak configuration should be saved.
	/// If it has no entries, or if everything will load anyways,
	/// then we won't bother serializing this part of the plist.
	pub fn should_skip(&self) -> bool {
		self.tweak_configs.is_empty()
			|| self
				.tweak_configs
				.values()
				.all(|x| self.allow_or_deny.check(*x))
	}
}

impl Default for TweakConfig {
	fn default() -> Self {
		let tweak_configs: BTreeMap<String, bool> =
			TWEAKS.iter().map(|name| (name.clone(), false)).collect();
		Self {
			allow_or_deny: TweakMode::default(),
			custom_config: false,
			enable_tweaks: true,
			tweak_configs,
		}
	}
}

#[derive(Debug)]
pub enum Target {
	/// Configure the executable located at this path.
	Executable(PathBuf),
	/// Configure the app with this bundle ID.
	App(String),
	/// Configure the default libhooker configuration.
	Default,
}

impl Target {
	pub fn get_config<'a>(&self, base_config: &'a mut ConfigTypes) -> &'a mut TweakConfig {
		match self {
			Target::Executable(path) => base_config
				.paths
				.entry(path.clone())
				.or_insert_with(TweakConfig::default),
			Target::App(bundle) => base_config
				.bundles
				.entry(bundle.clone())
				.or_insert_with(TweakConfig::default),
			Target::Default => &mut base_config.default,
		}
	}

	pub fn apply_config(&self, base_config: &mut ConfigTypes, tweak_config: TweakConfig) {
		match self {
			Target::Executable(path) => {
				base_config.paths.insert(path.clone(), tweak_config);
			}
			Target::App(bundle) => {
				base_config.bundles.insert(bundle.clone(), tweak_config);
			}
			Target::Default => base_config.default = tweak_config,
		}
	}

	pub fn get_tweak_mode(&self, base_config: &ConfigTypes) -> TweakMode {
		match self {
			Target::Executable(path) => base_config
				.paths
				.get(path)
				.map(|x| x.allow_or_deny)
				.unwrap_or_default(),
			Target::App(bundle) => base_config
				.bundles
				.get(bundle)
				.map(|x| x.allow_or_deny)
				.unwrap_or_default(),
			Target::Default => base_config.default.allow_or_deny,
		}
	}
}

impl Display for Target {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Target::Executable(path) => {
				write!(
					f,
					"{}",
					path.file_name()
						.and_then(OsStr::to_str)
						.map(|x| x.to_string())
						.unwrap_or_else(|| path.display().to_string())
				)
			}
			Target::App(name) => {
				write!(f, "{}", crate::apps::get_friendly_app_name(&name))
			}
			Target::Default => write!(f, "libhooker defaults"),
		}
	}
}

impl From<String> for Target {
	fn from(target: String) -> Self {
		let target_path = PathBuf::from(&target);
		let target = target.trim();
		match target.to_lowercase().as_str() {
			"springboard" => {
				Self::Executable("/System/Library/CoreServices/SpringBoard.app/SpringBoard".into())
			}
			"default" => Self::Default,
			_ => {
				if target_path.is_file() && target_path.is_absolute() {
					Self::Executable(target_path)
				} else if let Some(bundle_id) = fix_app_name(target) {
					Self::App(bundle_id)
				} else {
					let normalized_target = target.trim().to_lowercase();
					let daemon = DAEMONS
						.iter()
						.filter_map(|path| {
							path.file_name().and_then(OsStr::to_str).map(|x| (path, x))
						})
						.find(|(path, name)| {
							let normalized_name = name.trim().to_lowercase();
							normalized_name == normalized_target || **path == target_path
						});

					match daemon {
						Some((path, _)) => Self::Executable(path.clone()),
						None => {
							eprintln!(
								r#"An {}, {}, was entered!
It should either be an {}, {}, {}, "{}", or "{}"!"#,
								"invalid target".red().bold(),
								target.red().underlined(),
								"absolute path to an executable".cyan(),
								"app name or bundle ID".magenta(),
								"daemon or service name".blue(),
								"springboard".yellow(),
								"default".green()
							);
							std::process::exit(1);
						}
					}
				}
			}
		}
	}
}
