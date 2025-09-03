/*!
This module contains a layer over the common submodule for
handling paths with a `FlexPathVariant` variant.
*/

use super::{
    STARTS_WITH_PATH_SEPARATOR,
    FlexPathVariant
};
use lazy_regex::*;

static STARTS_WITH_WINDOWS_PATH_PREFIX: Lazy<Regex> = lazy_regex!(r#"(?x)
    ^ (
        ([\\/][\\/]\?\\([A-Za-z]\:)?)  | # extended-length prefix
        ([\\/][\\/])                   | # UNC prefix
        ([A-Za-z]\:)                     # drive prefix
    )
"#);

static STARTS_WITH_WINDOWS_PATH_PREFIX_OR_SLASH: Lazy<Regex> = lazy_regex!(r#"(?x)
    ^ (
        ([\\/][\\/]\?[\\/]([A-Za-z]\:)?)  | # extended-length prefix
        ([\\/][\\/])                      | # UNC prefix
        ([A-Za-z]\:)                      | # drive prefix
        [\/\\] ([^/\\] | $)                 # slash
    )
"#);

static UNC_OR_EXT_PREFIX: Lazy<Regex> = lazy_regex!(r#"(?x)
    ^[\\/][\\/](?:\?[\\/])?$
"#);

pub fn resolve(path1: &str, path2: &str, manipulation: FlexPathVariant) -> String {
    match manipulation {
        FlexPathVariant::Common => {
            crate::common::resolve(path1, path2)
        },
        FlexPathVariant::Windows => {
            let paths = [path1, path2].map(|p| p.to_owned());
            let prefixed: Vec<String> = paths.iter().filter(|path| STARTS_WITH_WINDOWS_PATH_PREFIX.is_match(path)).cloned().collect();
            if prefixed.is_empty() {
                return crate::common::resolve(path1, path2);
            }
            let prefix = STARTS_WITH_WINDOWS_PATH_PREFIX.find(prefixed.last().unwrap().as_ref()).map(|m| m.as_str().to_owned()).unwrap();
            let paths: Vec<String> = paths.iter().map(|path| STARTS_WITH_WINDOWS_PATH_PREFIX.replace(path.as_ref(), |_: &Captures| "/").into_owned()).collect();
            let r = crate::common::resolve(&paths[0], &paths[1]);
            if UNC_OR_EXT_PREFIX.is_match(&prefix.as_str()) {
                return prefix + &r[1..];
            }
            prefix + &r
        },
    }
}

pub fn resolve_n<'a, T: IntoIterator<Item = &'a str>>(paths: T, manipulation: FlexPathVariant) -> String {
    let paths = paths.into_iter().collect::<Vec<&'a str>>();
    if paths.is_empty() {
        return "".to_owned();
    }
    if paths.len() == 1 {
        return resolve(paths[0], "", manipulation);
    }
    let initial_path = resolve(paths[0], paths[1], manipulation);
    paths[2..].iter().fold(initial_path, |a, b| resolve(&a, b, manipulation))
}

pub fn resolve_one(path: &str, manipulation: FlexPathVariant) -> String {
    resolve_n([path], manipulation)
}

pub fn is_absolute(path: &str, manipulation: FlexPathVariant) -> bool {
    match manipulation {
        FlexPathVariant::Common => STARTS_WITH_PATH_SEPARATOR.is_match(path),
        FlexPathVariant::Windows => STARTS_WITH_WINDOWS_PATH_PREFIX_OR_SLASH.is_match(path),
    }
}

pub fn relative(from_path: &str, to_path: &str, manipulation: FlexPathVariant) -> String {
    match manipulation {
        FlexPathVariant::Common =>
            crate::common::relative(from_path, to_path),
        FlexPathVariant::Windows => {
            assert!(
                [from_path.to_owned(), to_path.to_owned()].iter().all(|path| is_absolute(path, manipulation)),
                "fairyvoid_path::argumented::relative() requires absolute paths as arguments"
            );
            let mut paths = [from_path, to_path].map(|s| s.to_owned());
            let prefixes: Vec<String> = paths.iter().map(|path| STARTS_WITH_WINDOWS_PATH_PREFIX_OR_SLASH.find(path.as_ref()).unwrap().as_str().into()).collect();
            let prefix = prefixes[0].clone();
            if prefix != prefixes[1] {
                return resolve_one(to_path, manipulation);
            }
            for path in &mut paths {
                *path = path[prefix.len()..].to_owned();
                if !STARTS_WITH_PATH_SEPARATOR.is_match(path.as_ref()) {
                    *path = "/".to_owned() + path.as_ref();
                }
            }
            crate::common::relative(paths[0].as_ref(), paths[1].as_ref())
        },
    }
}