use std::fs;

pub fn get_dirs_rec(dir: &str) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    let gitignore_file = fs::read_to_string(".ignore").unwrap_or("".to_string());
    let upaths = fs::read_dir(dir);
    if upaths.is_err() {
        return ret;
    }
    let paths = upaths.unwrap();
    for upath in paths.flatten() {
        let path = upath.path();
        let mut ignore = false;
        let display_str = path.display().to_string();
        for g in gitignore_file.lines() {
            if !g.is_empty() {
                ignore = ignore || display_str.contains(g);
            }
        }
        if ignore {
            continue;
        }
        if path.is_dir() {
            for p in get_dirs_rec(&display_str) {
                ret.push(p);
            }
        } else {
            ret.push(display_str)
        }
    }
    ret
}

pub fn get_dirs() -> Vec<String> {
    get_dirs_rec("./")
}
