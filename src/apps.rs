/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use objc::runtime::{Class, Object};
use objc_foundation::{INSArray, INSString, NSArray, NSObject, NSString};
use objc_id::Id;
use once_cell::sync::Lazy;

// This is a "lazy static" global, it's initialized on first use,
// then reused for subsequent uses.
pub static APPS: Lazy<Vec<App>> = Lazy::new(|| {
	// This is a pain, so bare with me.
	// Also, the various `unsafe { }` blocks are due to the fact that the Objective-C interop macros,
	// such as msg_send! and sel!, call external functions over FFI. Rust considers FFI to be "unsafe"
	// because the compiler cannot verify the soundness of external functions. But don't worry,
	// this code is sound. It doesn't even take user input.

	// Simple type aliases to make the code more clear.
	pub type LSApplicationProxy = NSObject;
	pub type LSApplicationWorkspace = *mut Object;

	// First, we're going to get a reference to the Objective-C class named "LSApplicationWorkspace"
	let ls_app_workspace =
		unsafe { NSClassFromString(NSString::from_str("LSApplicationWorkspace")) };
	// We're going to call the method `defaultWorkspace` on this class,
	// which gets us an actual instance of LSApplicationWorkspace.
	let default_app_workspace: LSApplicationWorkspace =
		unsafe { msg_send![ls_app_workspace, defaultWorkspace] };
	// Now, we call `allInstalledApplications` on the object instance,
	// which gives us an NSArray of the information of all installed apps.
	let installed_apps: &NSArray<LSApplicationProxy> =
		unsafe { msg_send![default_app_workspace, allInstalledApplications] };
	// Convert this NSArray to a Vec, else we can't actually use it.
	let installed_apps: Vec<&LSApplicationProxy> = installed_apps.to_vec();
	// Begin to iterate through the vec of app information.
	installed_apps
		.into_iter()
		.map(|app| {
			// Get a Selector object for the `_boundApplicationIdentifier` selector
			let bound_appid_sel = sel!(_boundApplicationIdentifier);
			// Check to see if our app information responds the `_boundApplicationIdentifier` selector
			let can_use_bound: bool =
				unsafe { msg_send![app, respondsToSelector: bound_appid_sel] };
			// If the app info does respond to `_boundApplicationIdentifier`, we call that,
			// otherwise we just call `applicationIdentifier`.
			// Either way, we end up with a reference to an NSString.
			let identifier: &NSString = if can_use_bound {
				unsafe { msg_send![app, _boundApplicationIdentifier] }
			} else {
				unsafe { msg_send![app, applicationIdentifier] }
			};
			// Get the "localized name" of this app,
			// by calling `localizedName` on the app information object.
			let localized_name: &NSString = unsafe { msg_send![app, localizedName] };
			// Now, we finish up and convert the NSStrings to Rust Strings,
			// creating our own App struct with them.
			App {
				identifier: identifier.as_str().to_string(),
				name: localized_name.as_str().to_string(),
			}
		})
		.collect()
});

#[allow(improper_ctypes)]
extern "C" {
	// This is just an FFI "header" for the `NSClassFromString` function,
	// a part of Foundation.framework on macOS and iOS.
	fn NSClassFromString(name: Id<NSString>) -> *mut Class;
}

pub struct App {
	pub identifier: String,
	pub name: String,
}

pub fn fix_app_name(input: &str) -> Option<String> {
	let normalized_input = input.trim().to_lowercase();
	APPS.iter()
		.find(|app| {
			let id = app.identifier.trim().to_lowercase();
			let name = app.name.trim().to_lowercase();

			id == normalized_input
				|| name == normalized_input
				|| name.replace(' ', "") == normalized_input.replace(' ', "")
		})
		.map(|app| app.identifier.clone())
}

pub fn get_friendly_app_name(id: &str) -> String {
	APPS.iter()
		.find(|app| app.identifier == id)
		.map(|app| app.name.clone())
		.expect("some idiot used get_friendly_app_name on unchecked input")
}
