
use std::env::args;
use std::path::PathBuf;
use collage_bg::ColGen;
use xcb::Connection;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("Hello, world!");

	let cd = PathBuf::from(args().nth(1).unwrap());
	eprintln!("{:?}", cd);
	let (conn, _) = Connection::connect(None).unwrap();
	let mut cg = ColGen::new(&cd, 300, 300, &conn).unwrap();
	cg.set_verbose(true);
	cg.update_roots();
	let mut count = 0;
	let sleep_dur = Duration::from_secs(50);
	loop {
		sleep(sleep_dur);
		if cg.check_update() { continue; }
		cg.replace_random(count % cg.lens());
		count += 1;
	}
}

