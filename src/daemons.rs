/*
	bender - a CLI interface for configuring libhooker on iOS
	copyright (C) 2021, aspen <aspenuwu@protonmail.com>
	All rights reserved.

	LGBTQ+ rights are human rights. If you disagree, kindly fuck off and don't use my code :)
	🏳️‍🌈🏳️‍⚧️
*/

use libc::{c_char, close, fcntl, pipe, read, F_SETFL, O_NONBLOCK};
use once_cell::sync::Lazy;
use std::{collections::HashMap, ffi::CStr, path::PathBuf, process::Command};
use xpc_connection::{message_to_xpc_object, xpc_object_to_message, Message as XpcMessage};
use xpc_connection_sys::{_os_alloc_once_table, xpc_global_data, xpc_object_t};

extern "C" {
	fn xpc_pipe_routine(pipe: xpc_object_t, request: xpc_object_t, reply: *mut xpc_object_t)
		-> i32;
	fn xpc_strerror(err: i32) -> *const c_char;
}

// This is weird Darwin shit. I don't know what it does.
fn xpc_bootstrap_pipe() -> xpc_object_t {
	let xpc_gd: &xpc_global_data =
		unsafe { &*(_os_alloc_once_table.get_unchecked(1).ptr as *const xpc_global_data) };
	xpc_gd.xpc_bootstrap_pipe
}

/// Get a service's full path to it's executable from it's label.
pub fn lookup_service(name: &str) -> Option<PathBuf> {
	const ROUTINE_DUMP_PROCESS: XpcMessage = XpcMessage::Uint64(0x2c4);
	const PROGRAM_PREFIX: &str = "program = ";

	// Create an IPC pipe.
	let mut fds: [i32; 2] = [0, 0];
	if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
		return None;
	}
	unsafe { fcntl(fds[0], F_SETFL, O_NONBLOCK) };

	// Create our XPC dictionary, basically the arguments we pass.
	let mut dict = HashMap::<String, XpcMessage>::new();
	dict.insert("handle\0".into(), XpcMessage::Uint64(0));
	dict.insert("name\0".into(), XpcMessage::String(format!("{}\0", name)));
	dict.insert("routine\0".into(), ROUTINE_DUMP_PROCESS);
	dict.insert("subsystem\0".into(), XpcMessage::Uint64(2));
	dict.insert("type\0".into(), XpcMessage::Uint64(1));
	dict.insert("fd\0".into(), XpcMessage::Fd(fds[1]));

	// Convert the dictionary to actual xpc_object structs
	let dict = message_to_xpc_object(XpcMessage::Dictionary(dict));
	// We also create a blank dictionary, which xpc will later use to output an error to if needed.
	let mut out_dict = message_to_xpc_object(XpcMessage::Dictionary(HashMap::default()));

	// Now, we actually send the xpc command.
	// Our reply will be in `out_dict`.
	unsafe {
		xpc_pipe_routine(
			xpc_bootstrap_pipe() as *mut _,
			dict as *mut _,
			&mut out_dict as *mut _,
		)
	};
	// Convert the output to a dictionary. If it's not a dictionary, then something is wrong
	// and we should just panic.
	let out_dict = match xpc_object_to_message(out_dict) {
		XpcMessage::Dictionary(d) => d,
		_ => panic!("failed to get xpc dict",),
	};
	// If our `out_dict` isn't empty, that probably means there's an error value in it.
	if !out_dict.is_empty() {
		// Extract the error value, a 64-bit signed integer, from the dictionary
		let err = match out_dict
			.get("error")
			.expect("failed to check for xpc error")
		{
			XpcMessage::Int64(i) => *i,
			_ => panic!("xpc error was not int64 even though it should be"),
		};
		// Now, we panic, after converting the error code to a human-readable string.
		panic!(
			"xpc error: {}",
			unsafe { CStr::from_ptr(xpc_strerror(err as i32)) }
				.to_str()
				.unwrap()
				.to_owned()
		);
	}
	// Close the output pipe that we gave to xpc, we're done with it for now.
	unsafe { close(fds[1]) };

	// We're going to read the data from our input, first we need to set up buffers
	let mut output = Vec::<u8>::new();
	let mut buffer = vec![0u8; 1024];
	// Then, we loop on read(), filling the output with recieved bytes,
	// until the number of 'bytes reader' is below 1.
	loop {
		let bytes = unsafe { read(fds[0], buffer.as_mut_ptr() as *mut _, 1024) };
		// is_positive returns false for 0 AND negative numbers.
		if !bytes.is_positive() {
			break;
		}
		// Add the contents of our buffer to the ouytput
		output.extend_from_slice(&buffer[0..bytes as usize]);
	}
	// Close our half of the pipe, now we're fully done.
	unsafe { close(fds[0]) };

	// Convert the output to a UTF-8 String, it's some weird plist-like format.
	let output = String::from_utf8(output).expect("xpc did not return valid utf-8 string");
	output
		// Now, we split up our output by lines
		.lines()
		// Our goal is to find something like "program = /usr/bin/stupidd".
		// So, we trim all whitespace at the beginning and end of the string,
		// and then if the trimmed string begins with "program = ",
		// we use that string with the "program = " part removed.
		.find_map(|line| line.trim().strip_prefix(PROGRAM_PREFIX))
		// Convert the string to a PathBuf
		.map(PathBuf::from)
		// And ensure that this path is, in fact, a valid, existing file.
		.filter(|path| path.is_file())
}

// This is a "lazy static" global, it's initialized on first use,
// then reused for subsequent uses.
pub static DAEMONS: Lazy<Vec<PathBuf>> = Lazy::new(|| {
	assert!(std::path::PathBuf::from("/.procursus_strapped").is_file());
	// Here we call `launchctl list`, read stdout, and
	// then try to convert it to a UTF-8 String.
	let output = String::from_utf8(
		Command::new("launchctl")
			.arg("list")
			.output()
			.expect("failed to run `launchctl list`!")
			.stdout,
	)
	.expect("launchctl gave invalid utf8... what?");

	let mut out = output
		// Split the output by newline
		.lines()
		// Skip the first line of launchctl (it just says "PID STATUS LABEL")
		.skip(1)
		// Split each line by whitespace, and only keep the third entry, the label.
		.map(|x| x.split_ascii_whitespace().collect::<Vec<&str>>()[2])
		// Don't include jailbreakd, amfidebilitate, or UIKit shit in the daemon list.
		.filter(|name| {
			*name != "jailbreakd"
				&& *name != "amfidebilitate"
				&& !name.starts_with("UIKitApplication:")
		})
		// Now, we call `lookup_service` to get the full path of the daemon
		.filter_map(|name| lookup_service(name))
		// And finally, convert this all into one big vec.
		.collect::<Vec<PathBuf>>();

	// Pre-sort the paths.
	out.sort();

	out
});
