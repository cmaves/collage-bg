use collage_bg::ColGen;
use std::path::Path;

#[test]
fn test_is_valid_image_pos() {
	let path = Path::new("test_data/img1.jpg");
	assert!(ColGen::is_valid_image(&path));
}

#[test]
fn test_is_valid_image_pos2() {
	let path = Path::new("test_data/img1.jpeg");
	assert!(ColGen::is_valid_image(&path));
}

#[test] 
fn test_is_valid_image_neg() {
	let path = Path::new("test_data/img1.txt");
	assert!(!ColGen::is_valid_image(&path));
}

#[test]
fn test_is_valid_image_neg2() {
	let path = Path::new("test_data/i_dont_exists.jpg");
	assert!(!ColGen::is_valid_image(&path));

}

