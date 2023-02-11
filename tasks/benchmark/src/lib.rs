use std::{path::PathBuf, str::FromStr};

fn err_to_string<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}

/// # Errors
/// # Panics
pub fn get_code(lib: &str) -> Result<(String, String), String> {
    let url = url::Url::from_str(lib).map_err(err_to_string)?;

    let segments = url.path_segments().ok_or_else(|| "lib url has no segments".to_string())?;

    let filename = segments.last().ok_or_else(|| "lib url has no segments".to_string())?;

    let mut file = PathBuf::from_str("target").map_err(err_to_string)?;
    file.push(filename);

    if let Ok(code) = std::fs::read_to_string(&file) {
        println!("[{filename}] - using [{}]", file.display());
        Ok((filename.to_string(), code))
    } else {
        println!("[{filename}] - Downloading [{lib}] to [{}]", file.display());
        match ureq::get(lib).call() {
            Ok(response) => {
                let mut reader = response.into_reader();

                std::fs::remove_file(&file).unwrap();
                let mut writer = std::fs::File::create(&file).map_err(err_to_string)?;
                std::io::copy(&mut reader, &mut writer).unwrap();

                std::fs::read_to_string(&file)
                    .map_err(err_to_string)
                    .map(|code| (filename.to_string(), code))
            }
            Err(e) => Err(format!("{e:?}")),
        }
    }
}
