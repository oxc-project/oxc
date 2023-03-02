use std::{path::PathBuf, str::FromStr};

use oxc_common::PaddedStringView;

fn err_to_string<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}

/// # Errors
/// # Panics
pub fn get_code(lib: &str) -> Result<(String, PaddedStringView), String> {
    let url = url::Url::from_str(lib).map_err(err_to_string)?;

    let segments = url.path_segments().ok_or_else(|| "lib url has no segments".to_string())?;

    let filename = segments.last().ok_or_else(|| "lib url has no segments".to_string())?;

    let mut file = PathBuf::from_str("target").map_err(err_to_string)?;
    file.push(filename);

    if let Ok(code) = PaddedStringView::read_from_file(&file) {
        println!("[{filename}] - using [{}]", file.display());
        Ok((filename.to_string(), code))
    } else {
        println!("[{filename}] - Downloading [{lib}] to [{}]", file.display());
        match ureq::get(lib).call() {
            Ok(response) => {
                let mut reader = response.into_reader();

                let _drop = std::fs::remove_file(&file);
                let mut writer = std::fs::File::create(&file).map_err(err_to_string)?;
                let _drop = std::io::copy(&mut reader, &mut writer);

                PaddedStringView::read_from_file(&file)
                    .map_err(err_to_string)
                    .map(|code| (filename.to_string(), code))
            }
            Err(e) => Err(format!("{e:?}")),
        }
    }
}
