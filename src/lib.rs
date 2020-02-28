use bg_setter::{BgError,XBgSetter};
use rand::prelude::{thread_rng,ThreadRng,Rng,SliceRandom};
use image::{FilterType,ImageResult,RgbImage};
use regex::RegexSet;
use std::collections::HashSet;
use std::fs::{DirEntry,read_dir};
use std::io;
use std::path::{Path,PathBuf};
use xcb::ConnError;

#[macro_use]extern crate lazy_static; 

lazy_static! {
	static ref I_RE: RegexSet = RegexSet::new(&[r".*\.jpe?g",r".*\.png"]).unwrap();
}
pub struct ColGen<'a> {
	paths: Vec<PathBuf>,
	rng: ThreadRng,
	width: u32,
	height: u32,
	bg: XBgSetter<'a>,
	roots: Vec<(HashSet<usize>, Vec<usize>)>,
	verbose: bool
}

#[derive(Debug)]
pub enum CgError {
	IO(io::Error),
	BG(BgError)
}


impl From<std::io::Error> for CgError {
	fn from(err: std::io::Error) -> Self {
		CgError::IO(err)
	}
}
impl From<BgError> for CgError {
	fn from(err: BgError) -> Self {
		CgError::BG(err)
	}
}
impl<'a> ColGen<'a> {
	pub fn new(dir_path: &Path, width: u32, height: u32, conn: &'a xcb::Connection) -> Result<Self, CgError> {
		let paths = Self::read_files(dir_path)?;
		let rng = thread_rng();
		// TODO: Validate image count
		let bg = XBgSetter::new(conn)?;
		Ok(ColGen { paths, rng, width, height, 
			bg, roots: Vec::new(), verbose: false })
	}
	pub fn update_roots(&mut self) {
		self.roots.clear();
		for screen in self.bg.get_displays(0).into_iter().enumerate() {
			let (width, height) = screen.1;
			let num = self.num_img(width, height);
			let mut hs = HashSet::with_capacity(num);
			let mut locs = Vec::with_capacity(num);
			for n in 0..self.num_img(width, height) {
				let i = loop  {
					let r: usize = self.rng.gen();
					let r = r % self.paths.len();
					if hs.insert(r) {
						locs.push(r);
						break r;
					}
				};
				let image = self.read_image(&self.paths[i]).unwrap();
				let (x, y) = self.screen_offset(width, height, n);
				self.bg.replace(0, screen.0, x, y, &image);
			}
			self.roots.push((hs, locs));

		}
	}
	pub fn check_update(&mut self) -> bool {
		if self.bg.check_resized_refresh() {
			self.update_roots();
			true
		} else { false } 
	}
	pub fn lens(&self) -> usize { self.roots.len() }
	fn read_files(dir_path: &Path) -> io::Result<Vec<PathBuf>> {
		let files = read_dir(dir_path)?;
		let files = files.filter_map(io::Result::ok).map(|x|{x.path()})
			.filter(ColGen::is_valid_image);
		Ok(files.collect())
	}
	fn read_image<P: AsRef<Path>>(&self, path: &P) -> ImageResult<RgbImage> {
		let image = image::open(path)?;
		let image = image.resize_to_fill(self.width, self.height, FilterType::Lanczos3);
		Ok(image.to_rgb())
		
	}
	pub fn is_valid_image<P: AsRef<Path>>(path: &P) -> bool {
		let path = path.as_ref();
		if let Some(s) = path.to_str() {
			I_RE.is_match(s) && path.is_file() 
		} else {
			false
		}
	}
	pub fn set_verbose(&mut self, verbose: bool) { self.verbose = verbose; self.bg.set_verbose(verbose); }
	fn num_img(&self, width: u16, height: u16) -> usize {
		((width as u32 / self.width) * (height as u32 / self.height)) as usize
		
	}
	pub fn replace_random(&mut self, screen: usize) -> Result<(), ()> {
		let (width, height) = *(self.bg.get_displays(0).get(screen).ok_or(())?);
		let tbr = self.rng.gen::<usize>() % self.num_img(width, height);
		let root = &mut self.roots[screen];
		let i = loop  {
			let r: usize = self.rng.gen();
			let r = r % self.paths.len();
			if root.0.insert(r) {
				root.0.remove(&root.1[tbr]);
				root.1[tbr] = r;
				break r;
			}
		};
		if self.verbose {println!("Replacing {} on screen {}", tbr, screen); }
		let image = self.read_image(&self.paths[i]).unwrap();
		let (x, y) = self.screen_offset(width, height, tbr);
		self.bg.fade(0, screen, x, y, &image, 10.0);
		Ok(())

	}
	fn screen_offset(&self, width: u16, height: u16, index: usize) -> (u16, u16) {
		let width = width as u32;
		let height = height as u32;
		let index = index as u32;
		let width_off = width  % self.width / 2;
		let height_off = height % self.height / 2;
		let stride = width / self.width;
		let row = index / stride;
		let column = index % stride;
		let (x, y) = (width_off + self.width * column, height_off + self.height * row);
		(x as u16, y as u16)
	}
}


