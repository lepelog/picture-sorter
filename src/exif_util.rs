use exif::Value::Ascii;
use std::path::{PathBuf, Path};
use std::fs::read_dir;
use std::error::Error;
use exif::{Tag, In};
use chrono::NaiveDateTime;

// read all jp(e)g files in a directory and returns a vec of their paths and their time
// pictures without a time attacked to them are ignored
pub fn read_all_with_date_from_dir<P: AsRef<Path>>(path: P) -> Result<Vec<(PathBuf, NaiveDateTime)>, Box<dyn Error>> {
    let mut result = Vec::new();
    for file_maybe in read_dir(path)? {
        let filename = file_maybe?;
        if let Some(extension) = filename.path().extension() {
            let lower_ext = extension.to_string_lossy().to_lowercase();
            if !(lower_ext == "jpeg" || lower_ext == "jpg") {
                continue;
            }
        } else {
            continue;
        }
        if let Ok(mabe_date_time) = maybe_read_date(filename.path()) {
            if let Some(date_time) = mabe_date_time {
                result.push((filename.path().clone(), date_time));
            }
        }
    }
    return Ok(result);
}

// reads a file and returns the exif date time, if present
fn maybe_read_date<P: AsRef<Path>>(path: P) -> Result<Option<NaiveDateTime>, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;
    if let Some(date_time_field) = exif.get_field(Tag::DateTime, In::PRIMARY) {
        match &date_time_field.value {
            Ascii(ref value) => {
                let text = String::from_utf8(value[0].clone())?;
                let parsed = NaiveDateTime::parse_from_str(&text, "%Y:%m:%d %H:%M:%S")?;
                return Ok(Some(parsed));
            },
            _ => panic!("not a string!"),
        }
    }
    return Ok(None);
}