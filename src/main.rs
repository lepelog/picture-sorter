use std::error::Error;
use std::path::{PathBuf, Path};
use std::clone::Clone;
use fltk::{app, dialog, button::Button, input::Input, group::Pack, prelude::*, window::Window};

extern crate exif;
pub mod exif_util;

#[derive(Clone)]
enum Message {
    DoSort,
    ChooseDir,
}

fn main() {
    let app_main = app::App::default();
    let mut wind = Window::default()
        .with_size(300, 300)
        .center_screen()
        .with_label("Picture Sort");
    
    let mut pack = Pack::default().with_size(120, 140).center_of(&wind);
    pack.set_spacing(10);
    let rename_name = Input::default()
        .with_size(200, 30)
        .with_label("Name:");
    let mut but_choose = Button::default()
        .with_size(0, 40)
        .with_label("Select Directory");
    let mut but_sort = Button::default()
        .with_size(0, 40)
        .with_label("Sort Pictures");
    
    but_sort.deactivate();
        
    wind.end();
    wind.show();
    let (sender, rec) = app::channel::<Message>();
    let choose_sender = sender.clone();
    but_choose.set_callback(move |_| choose_sender.send(Message::ChooseDir));
    but_sort.set_callback(move |_| sender.send(Message::DoSort));
        
    // App state
    let mut chosen_dir: Option<PathBuf> = None;
    while app_main.wait() {
        if let Some(msg) = rec.recv() {
            match msg {
                Message::DoSort => {
                    if let Err(err) = do_sort_files(chosen_dir.as_ref().unwrap(), &rename_name.value()) {
                        dialog::alert_default(&err.to_string());
                    } else {
                        dialog::message_default("successfully sorted pictures!!");
                    }
                },
                Message::ChooseDir => {
                    let mut file_dial = dialog::FileDialog::new(dialog::FileDialogType::BrowseDir);
                    file_dial.set_option(dialog::FileDialogOptions::NoOptions);
                    file_dial.show();
                    let chosen_filename = file_dial.filename().to_string_lossy().to_string();
                    if !chosen_filename.is_empty() {
                        chosen_dir = Some(file_dial.filename());
                        but_sort.activate();
                    }
                },
            }
        }
    }
    // app_main.run().unwrap();
}

fn do_sort_files<P: AsRef<Path>>(directory_path: P, prefix: &str) -> Result<(), Box<dyn Error>> {
    // get all pictures with their time
    let mut result = crate::exif_util::read_all_with_date_from_dir(directory_path).unwrap();
    // sort by date, ascending
    result.sort_by(|(_path, date_time1), (_, date_time2)| date_time1.cmp(date_time2));
    // println!("{:?}", result);
    
    let num_digits = get_digits_len(result.len());
    // build rename vec
    // (oldpath, newpath)
    let mut rename_table = Vec::with_capacity(result.len());
    for (i, (file_path, _time)) in result.into_iter().enumerate() {
        let mut new_path = file_path.clone();
        new_path.set_file_name(format!("{} {1:02$}.jpg", prefix, i, num_digits));
        rename_table.push((file_path, new_path));
    }
    // make sure that the new names do not exist
    for (_, new_path) in rename_table.iter() {
        if new_path.exists() {
            return Err(Box::from(format!("A file with the name {} already exists!", new_path.file_name().unwrap().to_string_lossy())));
        }
    }
    // actually rename all files
    for (old_path, new_path) in rename_table.iter() {
        std::fs::rename(old_path, new_path)?;
    }
    Ok(())
}

fn get_digits_len(mut number: usize) -> usize {
    let mut digits = 0;
    while number > 0 {
        number /= 10;
        digits += 1;
    }
    digits
}
