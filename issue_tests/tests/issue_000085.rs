// Copyright © 2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
*/

/*
Solved:
*/

// use std::path::Path;
// use std::fs;

// fn
// read_exif_data_current
// (
//     path: &std::path::Path,
// )
// {
//     let data = fs::read(path).unwrap();
//     // let ext = little_exif::filetype::FileExtension::JPEG;
//     let ext = little_exif::filetype::FileExtension::TIFF;
//     let metadata = little_exif::metadata::Metadata::new_from_vec(&data, ext).unwrap();

//     let mut data_new = data.clone();
//     metadata.write_to_vec(&mut data_new, ext).unwrap();
//     // little_exif::metadata::Metadata::clear_metadata(&mut data_new, ext).unwrap();

//     let extension = path.extension().unwrap().to_string_lossy();

//     let copy_path = format!(
//         "resources/issue_000085/{}_copy.{}", 
//         path.file_stem().unwrap().to_str().unwrap(), 
//         &extension
//     );
//     fs::copy(path, copy_path.clone()).unwrap();
//     fs::write(&copy_path, &data_new).unwrap();
// }

// #[test]
// fn
// read_exif_data_jpg_1_current()
// {
//     let jpg_path_1 = Path::new("resources/issue_000085/321382351771927671550184244404557053076_img_base.jpg");
//     read_exif_data_current(jpg_path_1);
// }

// #[test]
// fn
// read_exif_data_jpg_2_current()
// {
//     let jpg_path_2 = Path::new("resources/issue_000085/275238915753691881415960246141657713218_img_base.jpg");
//     read_exif_data_current(jpg_path_2);
// }

// #[test]
// fn
// read_exif_data_jpg_3_current()
// {
//     let jpg_path_3 = Path::new("resources/issue_000085/307118273949694625498793115961147426280_img_base.jpg");
//     read_exif_data_current(jpg_path_3);
// }

// #[test]
// fn
// read_exif_data_jpg_4_current()
// {
//     let jpg_path_4 = Path::new("resources/issue_000085/334982160166384331531360669298487872463_img_base.jpg");
//     read_exif_data_current(jpg_path_4);
// }

// #[test]
// fn
// read_exif_data_jpg_5_current()
// {
//     let jpg_path_5 = Path::new("resources/issue_000085/259624375424747111241219798258747248513_img_base.jpg");
//     read_exif_data_current(jpg_path_5);
// }

// #[test]
// fn
// read_exif_data_tiff_current()
// {
//     let tiff_path = Path::new("resources/issue_000085/337925378596053285636251667967174304145_img_base.tif");
//     read_exif_data_current(tiff_path);
// }