use std::fs::copy;
use std::fs::read;
use std::fs::remove_file;
use std::path::Path;
use std::time::Instant;
use std::time::Duration;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn 
measure_time_for_path
(
	path: &Path
)
-> (Duration, usize)
{
	let now = Instant::now();
	let mut data_len_sum = 0; // just so that this can't be optimized away

	for i in 0..1
	{
		data_len_sum += Metadata::new_from_path(path).unwrap().data().len();
	}

	let elapsed = now.elapsed();
	return (elapsed, data_len_sum);
}

fn main() 
{
	let fast1 = Path::new("./rsrc/image.jpg");
	let slow1 = Path::new("./rsrc/A0386910sw.JPG");
	let slow2 = Path::new("./rsrc/A0462208sw.JPG");

	let fast1_result = measure_time_for_path(&fast1);
	let slow1_result = measure_time_for_path(&slow1);
	let slow2_result = measure_time_for_path(&slow2);

	println!("\nResults:");
	println!("Fast1 - Elapsed: {:.2?}", fast1_result.0);
	println!("Slow1 - Elapsed: {:.2?}", slow1_result.0);
	println!("Slow2 - Elapsed: {:.2?}", slow2_result.0);
}
