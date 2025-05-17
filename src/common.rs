/*!
Work with generic file paths. This module only considers an _absolute path_ to be a path
that starts with a path separator.
*/

use super::STARTS_WITH_PATH_SEPARATOR;
use lazy_regex::*;

static PATH_SEPARATOR: Lazy<Regex> = lazy_regex!(r"[/\\]");

pub fn relative(from_path: &str, to_path: &str) -> String {
    assert!(
        [from_path.to_owned(), to_path.to_owned()].iter().all(|path| STARTS_WITH_PATH_SEPARATOR.is_match(path)),
        "hydroperx_path::relative() requires absolute paths as arguments"
    );

    let mut r = Vec::<String>::new();

    let mut from_parts: Vec<String> = PATH_SEPARATOR.split(resolve_one(from_path).as_ref()).map(|s| s.to_owned()).collect();
    let mut to_parts: Vec<String> = PATH_SEPARATOR.split(resolve_one(to_path).as_ref()).map(|s| s.to_owned()).collect();

    // given each path is absolute, each one can contain an empty
    // initial second part. in that case, remove the empty string,
    // since the path `"/"` is previously split into a `vec!["", ""]`.
    let from_parts = remove_empty(&mut from_parts);
    let to_parts = remove_empty(&mut to_parts);

    fn remove_empty(parts: &mut Vec<String>) -> &mut Vec<String> {
        if parts[1].is_empty() {
            parts.remove(1);
        }
        parts
    }

    let mut common_indices = Vec::<usize>::new();

    for i in 0..usize::min(from_parts.len(), to_parts.len()) {
        if from_parts[i] != to_parts[i] {
            break;
        }
        common_indices.push(i);
    }
    for i in common_indices.iter().rev() {
        let j = common_indices[*i];
        from_parts.remove(j);
        to_parts.remove(j);
    }
    r.append(&mut Vec::from_iter((0..from_parts.len()).map(|_| "..".to_owned())));
    r.append(&mut to_parts.clone());

    let r = r.join("/");
    let r = r.trim_start().to_owned();
    if r.ends_with('/') { r[..r.len() - 1].to_owned() } else { r }
}

#[allow(unused)]
pub fn resolve_n<'a, T: IntoIterator<Item = &'a str>>(paths: T) -> String {
    let paths = paths.into_iter().collect::<Vec<&'a str>>();
    if paths.is_empty() {
        return "".to_owned();
    }
    if paths.len() == 1 {
        return resolve_one(paths[0]);
    }
    let initial_path = resolve(paths[0], paths[1]);
    paths[2..].iter().fold(initial_path, |a, b| resolve(&a, b))
}

pub fn resolve(path1: &str, path2: &str) -> String {
    if STARTS_WITH_PATH_SEPARATOR.is_match(path2) {
        return resolve_one(path2);
    }
    let starts_with_slash = STARTS_WITH_PATH_SEPARATOR.is_match(path1);
    let mut r: String;
    let path1_resolved = resolve_one_without_starting_sep(path1);
    if path2.is_empty() {
        r = path1_resolved;
    }
    else {
        let paths_combination = path1_resolved + "/" + path2;
        r = resolve_one_without_starting_sep(paths_combination.as_ref());
    }
    if starts_with_slash {
        r = "/".to_owned() + &r;
    }
    r
}

pub fn resolve_one(path: &str) -> String {
    let starts_with_slash = STARTS_WITH_PATH_SEPARATOR.is_match(path);
    let r = resolve_one_without_starting_sep(path);
    if starts_with_slash { "/".to_owned() + &r } else { r }
}

fn resolve_one_without_starting_sep(path: &str) -> String {
    let mut r = Vec::<String>::new();
    for p in PATH_SEPARATOR.split(path) {
        if p == "." {
            continue;
        } else if p == ".." {
            if !r.is_empty() {
                r.remove(r.len() - 1);
            }
        } else if !p.is_empty() {
            r.push(p.to_owned());
        }
    }
    r.join("/")
}