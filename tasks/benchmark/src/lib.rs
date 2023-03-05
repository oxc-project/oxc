use std::{path::PathBuf, str::FromStr, time::Duration};

pub struct Code {
    pub url: &'static str,
    pub file_name: String,
    pub source_text: String,
    pub measurement_time: Duration,
}

impl Code {
    /// # Errors
    pub fn new(measurement_seconds: u64, url: &'static str) -> Result<Self, String> {
        let (file_name, source_text) = Self::get_source_text(url)?;
        Ok(Self {
            url,
            file_name,
            source_text,
            measurement_time: Duration::new(measurement_seconds, 0),
        })
    }

    /// # Errors
    /// # Panics
    pub fn get_source_text(lib: &str) -> Result<(String, String), String> {
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

                    let _drop = std::fs::remove_file(&file);
                    let mut writer = std::fs::File::create(&file).map_err(err_to_string)?;
                    let _drop = std::io::copy(&mut reader, &mut writer);

                    std::fs::read_to_string(&file)
                        .map_err(err_to_string)
                        .map(|code| (filename.to_string(), code))
                }
                Err(e) => Err(format!("{e:?}")),
            }
        }
    }
}

fn err_to_string<E: std::fmt::Debug>(e: E) -> String {
    format!("{e:?}")
}
