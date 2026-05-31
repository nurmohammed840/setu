use std::{fs, io, path::Path};

/// Note: It don't copy symlink
pub fn copy_dir(src: &Path, dst: &Path, depth: u8, filter: fn(&Path) -> bool) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let to = dst.join(entry.file_name());

        if file_type.is_dir() {
            if depth == 0 {
                continue;
            }
            copy_dir(&entry.path(), &to, depth - 1, filter)?;
        } else if file_type.is_file() {
            let path = entry.path();
            if !filter(&path) {
                continue;
            }
            fs::copy(path, to)?;
        }
    }
    Ok(())
}
