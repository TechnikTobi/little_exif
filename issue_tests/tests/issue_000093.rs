// Copyright © 2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Simplified code

fn process_file(path: &Path, dir_broken: &Path) -> Result<bool, String> {
    panic::catch_unwind(|| {
        let data = read_file_vec(path)?;
        let ext = detect_ext_from_vec(&data).ok_or_else(|| "Failed to detect file type".to_string())?;
        let metadata = load_metadata_from_vec(&data, ext)?;

        let mut data_new = data.clone();
        match metadata.write_to_vec(&mut data_new, ext){
            Ok(()) => Ok(false),
            Err(e) => {
                eprintln!("File {} is broken: {}", path.display(), e);
                let extension = path.extension().unwrap().to_string_lossy();
                fs::copy(path, dir_broken.join(format!("{}.{extension}", random::<u128>())))
                    .map_err(|e| format!("Failed to copy broken file {}: {}", path.display(), e)).unwrap();
                Ok(true)
            }
        }
    }).unwrap_or_else(|e| {
        Err(format!("Panic occurred while processing {}: {:?}", path.display(), e))
    })
}
Project - same.zip

Usage

cargo run --release -- /path_to_gather_files /path_where_save_broken_files
Reading metadata and writing back the exact same metadata to the original file should always succeed, but this errors are reported:

failed to fill whole buffer
Broken files -Broken.zip
*/

/*
Solved:
The two image files that caused the problem both are missing their EOI marker 
(0xFFD9) at the end of the file. Modified the read_exact operations in these
cases to check the error kind they return - if it is UnexpectedEof, we just 
assume that we reached the EOI marker and continue as normal. Otherwise we 
return the error.
*/

use std::path::Path;
use std::fs;
use std::panic;

/*
fn
read_exif_data_fails
(
    path: &std::path::Path,
)
{
    let data = fs::read(path).unwrap();
    let ext = little_exif_0_6_21::filetype::FileExtension::JPEG;
    let metadata = little_exif_0_6_21::metadata::Metadata::new_from_vec(&data, ext).unwrap();

    let mut data_new = data.clone();
    match metadata.write_to_vec(&mut data_new, ext)
    {
        Ok(()) => (),
        Err(e) => {
            let extension = path.extension().unwrap().to_string_lossy();

            fs::copy(
                path,
                format!("resources/issue_000093/{}_copy.{}", 
                path.file_stem().unwrap().to_str().unwrap(), &extension)
            ).unwrap();

            panic!("{}", e.to_string());
        }
    };
}

#[test]
#[should_panic (expected = "failed to fill whole buffer")]
fn
read_exif_data_jpg_1_fails()
{
    let jpg_path_1 = Path::new("resources/issue_000006/309465781-9420afb8-2a57-4bae-a188-a719f6d62b1f.JPG");
    read_exif_data_fails(jpg_path_1);
}

#[test]
#[should_panic (expected = "failed to fill whole buffer")]
fn
read_exif_data_jpg_2_fails()
{
    let jpg_path_2 = Path::new("resources/issue_000093/24204211336558490774263267518266904513.JPG");
    read_exif_data_fails(jpg_path_2);
}
*/


fn
read_exif_data_current
(
    path: &std::path::Path,
)
{
    let data = fs::read(path).unwrap();
    let ext = little_exif::filetype::FileExtension::JPEG;
    let metadata = little_exif::metadata::Metadata::new_from_vec(&data, ext).unwrap();

    let mut data_new = data.clone();
    match metadata.write_to_vec(&mut data_new, ext)
    {
        Ok(()) => (),
        Err(e) => {
            let extension = path.extension().unwrap().to_string_lossy();

            fs::copy(
                path,
                format!("resources/issue_000093/{}_copy.{}", 
                path.file_stem().unwrap().to_str().unwrap(), &extension)
            ).unwrap();

            panic!("{}", e.to_string());
        }
    };
}

#[test]
fn
read_exif_data_jpg_1_current()
{
    let jpg_path_1 = Path::new("resources/issue_000006/309465781-9420afb8-2a57-4bae-a188-a719f6d62b1f.JPG");
    read_exif_data_current(jpg_path_1);
}

#[test]
fn
read_exif_data_jpg_2_current()
{
    let jpg_path_2 = Path::new("resources/issue_000093/24204211336558490774263267518266904513.JPG");
    read_exif_data_current(jpg_path_2);
}
