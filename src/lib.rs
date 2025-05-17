/*!
Work with file paths by text only.

In the Windows operating system, absolute paths may either start with a drive letter followed by
a colon, or an UNC path prefix (`\\`), or an extended drive letter prefix (`\\?\X:`).
Therefore, this crate provides a `FlexPath` that is based on a variant ([_FlexPathVariant_]),
which you don't need to always specify. This variant indicates whether to
interpret Windows absolute paths or not.

There are two _FlexPathVariant_ variants currently:

- _Common_
- _Windows_

The constant `FlexPathVariant::native()` is one of these variants
based on the target platform. For the Windows operating system, it
is always _Windows_. For other platforms, it's always _Common_.

# Example

```
use hydroperx_path::FlexPath;

assert_eq!("a", FlexPath::new_common("a/b").resolve("..").to_string());
assert_eq!("a", FlexPath::new_common("a/b/..").to_string());
assert_eq!("a/b/c/d/e", FlexPath::from_n_common(["a/b", "c/d", "e/f", ".."]).to_string());
assert_eq!("../../c/d", FlexPath::new_common("/a/b").relative("/c/d"));
```
*/

use lazy_regex::*;
use std::{path::{Path, PathBuf}, str::FromStr};

pub(crate) mod common;
pub(crate) mod flexible;

/// Indicates if special absolute paths are considered.
///
/// Currently, only two variants are defined, considering that there is
/// no known operating system with different path support other than Windows:
/// 
/// * `Common`
/// * `Windows`
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum FlexPathVariant {
    /// Indicates that the path is manipulated in a Unix common way, resulting into forward slashes.
    Common,
    /// Indicates that the path is manipulated compatibly with the Windows operating system.
    Windows,
}

impl FlexPathVariant {
    pub(crate) const NATIVE: Self = {
        #[cfg(target_os = "windows")] {
            Self::Windows
        }
        #[cfg(not(target_os = "windows"))] {
            Self::Common
        }
    };

    /// The variant that represents the build's target platform.
    pub const fn native() -> Self {
        Self::NATIVE
    }
}

/// The `FlexPath` structure represents an always-resolved textual file path based
/// on a [_FlexPathVariant_].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FlexPath(String, FlexPathVariant);

impl FlexPath {
    /// Constructs a `FlexPath` with a given `variant`. This method
    /// will resolve the specified path.
    pub fn new(path: &str, variant: FlexPathVariant) -> Self {
        Self(flexible::resolve_one(path, variant), variant)
    }

    /// Constructs a `FlexPath` whose variant is `Common`. This method
    /// will resolve the specified path.
    pub fn new_common(path: &str) -> Self {
        Self(flexible::resolve_one(path, FlexPathVariant::Common), FlexPathVariant::Common)
    }

    /// Constructs a `FlexPath` whose variant is chosen according to the target platform.
    /// This method will resolve the specified path.
    pub fn new_native(path: &str) -> Self {
        Self(flexible::resolve_one(path, FlexPathVariant::NATIVE), FlexPathVariant::NATIVE)
    }

    /// Constructs a `FlexPath` from multiple paths and a given `variant`.
    pub fn from_n<'a, T: IntoIterator<Item = &'a str>>(paths: T, variant: FlexPathVariant) -> Self {
        Self(flexible::resolve_n(paths, variant), variant)
    }

    /// Constructs a `FlexPath` from multiple paths and a `Common` variant.
    pub fn from_n_common<'a, T: IntoIterator<Item = &'a str>>(paths: T) -> Self {
        Self::from_n(paths, FlexPathVariant::Common)
    }

    /// Constructs a `FlexPath` from multiple paths and a variant based on
    /// the target platform.
    pub fn from_n_native<'a, T: IntoIterator<Item = &'a str>>(paths: T) -> Self {
        Self::from_n(paths, FlexPathVariant::NATIVE)
    }

    /// Returns the variant this `FlexPath` object is based on.
    pub fn variant(&self) -> FlexPathVariant {
        self.1
    }

    /// Indicates whether the `FlexPath` is absolute or not.
    pub fn is_absolute(&self) -> bool {
        flexible::is_absolute(&self.0, self.1)
    }

    /// Resolves `path2` relative to `path1`.
    ///
    /// Behavior:
    /// - Eliminates the segments `..` and `.`.
    /// - If `path2` is absolute, this function returns a resolution of solely `path2`.
    /// - All path separators that are backslashes (`\`) are replaced by forward ones (`/`).
    /// - If any path is absolute, this function returns an absolute path.
    /// - Any empty segment and trailing path separators, such as in `a/b/` and `a//b` are eliminated.
    pub fn resolve(&self, path2: &str) -> FlexPath {
        FlexPath(flexible::resolve(&self.0, path2, self.1), self.1)
    }

    /// Resolves multiple paths relative to this path. The
    /// behavior is similiar to [`.resolve`]. If the given
    /// set has no items, an empty string is returned.
    pub fn resolve_n<'a, T: IntoIterator<Item = &'a str>>(&self, paths: T) -> FlexPath {
        FlexPath(flexible::resolve(&self.0, &flexible::resolve_n(paths, self.1), self.1), self.1)
    }

    /**
    Finds the relative path from this path to `to_path`.

    # Behavior:

    - If the paths refer to the same path, this function returns
    an empty string.
    - The function ensures that both paths are absolute and resolves
    any `..` and `.` segments inside.
    - If both paths have different prefix, `to_path` is returned.

    # Panics

    Panics if given paths are not absolute.

    # Example

    ```
    use hydroperx_path::FlexPath;
    assert_eq!("", FlexPath::new_common("/a/b").relative("/a/b"));
    assert_eq!("c", FlexPath::new_common("/a/b").relative("/a/b/c"));
    assert_eq!("../../c/d", FlexPath::new_common("/a/b").relative("/c/d"));
    assert_eq!("../c", FlexPath::new_common("/a/b").relative("/a/c"));
    ```
    */
    pub fn relative(&self, to_path: &str) -> String {
        flexible::relative(&self.0, to_path, self.1)
    }

    /// Changes the extension of a path and returns a new string.
    /// This method adds any lacking dot (`.`) prefix automatically to the
    /// `extension` argument.
    ///
    /// This method allows multiple dots per extension. If that is not
    /// desired, use [`.change_last_extension`].
    ///
    /// # Example
    /// 
    /// ```
    /// use hydroperx_path::FlexPath;
    /// assert_eq!("a.y", FlexPath::new_common("a.x").change_extension(".y").to_string());
    /// assert_eq!("a.z", FlexPath::new_common("a.x.y").change_extension(".z").to_string());
    /// assert_eq!("a.z.w", FlexPath::new_common("a.x.y").change_extension(".z.w").to_string());
    /// ```
    ///
    pub fn change_extension(&self, extension: &str) -> FlexPath {
        Self(change_extension(&self.0, extension), self.1)
    }

    /// Changes only the last extension of a path and returns a new string.
    /// This method adds any lacking dot (`.`) prefix automatically to the
    /// `extension` argument.
    ///
    /// # Panics
    ///
    /// Panics if the extension contains more than one dot.
    ///
    pub fn change_last_extension(&self, extension: &str) -> FlexPath {
        Self(change_last_extension(&self.0, extension), self.1)
    }

    /// Checks if a file path has a specific extension.
    /// This method adds any lacking dot (`.`) prefix automatically to the
    /// `extension` argument.
    pub fn has_extension(&self, extension: &str) -> bool {
        has_extension(&self.0, extension)
    }

    /// Checks if a file path has any of multiple specific extensions.
    /// This method adds any lacking dot (`.`) prefix automatically to each
    /// extension argument.
    pub fn has_extensions<'a, T: IntoIterator<Item = &'a str>>(&self, extensions: T) -> bool {
        has_extensions(&self.0, extensions)
    }

    /// Returns the base name of a file path.
    ///
    /// # Example
    /// 
    /// ```
    /// use hydroperx_path::FlexPath;
    /// assert_eq!("qux.html", FlexPath::new_common("foo/qux.html").base_name());
    /// ```
    pub fn base_name(&self) -> String {
        base_name(&self.0)
    }

    /// Returns the base name of a file path, removing any of the specified extensions.
    /// This method adds any lacking dot (`.`) prefix automatically to each
    /// extension argument.
    ///
    /// # Example
    /// 
    /// ```
    /// use hydroperx_path::FlexPath;
    /// assert_eq!("qux", FlexPath::new_common("foo/qux.html").base_name_without_ext([".html"]));
    /// ```
    pub fn base_name_without_ext<'a, T>(&self, extensions: T) -> String
        where T: IntoIterator<Item = &'a str>
    {
        base_name_without_ext(&self.0, extensions)
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from_str(&self.to_string()).unwrap_or(PathBuf::new())
    }
}

impl ToString for FlexPath {
    /// Returns a string representation of the path,
    /// delimiting segments with either a forward slash (`/`) or backward slash (`\`)
    /// depending on the path's `FlexPathVariant`.
    fn to_string(&self) -> String {
        if self.variant() == FlexPathVariant::Windows {
            self.0.replace('/', "\\")
        } else {
            self.0.clone()
        }
    }
}

static STARTS_WITH_PATH_SEPARATOR: Lazy<Regex> = lazy_regex!(r"^[/\\]");

fn change_extension(path: &str, extension: &str) -> String {
    let extension = (if extension.starts_with('.') { "" } else { "." }).to_owned() + extension;
    if regex_find!(r"(\.[^\.]+)+$", path).is_none() {
        return path.to_owned() + &extension;
    }
    regex_replace!(r"(\.[^\.]+)+$", path, |_, _| &extension).into_owned()
}

fn change_last_extension(path: &str, extension: &str) -> String {
    let extension = (if extension.starts_with('.') { "" } else { "." }).to_owned() + extension;
    assert!(
        extension[1..].find('.').is_none(),
        "The argument to hydroperx_path::change_last_extension() must only contain one extension; got {}",
        extension
    );
    if regex_find!(r"(\..+)$", path).is_none() {
        return path.to_owned() + &extension;
    }
    regex_replace!(r"(\..+)$", path, |_, _| &extension).into_owned()
}

/// Adds prefix dot to extension if missing.
fn extension_arg(extension: &str) -> String {
    (if extension.starts_with('.') { "" } else { "." }).to_owned() + extension
}

fn has_extension(path: &str, extension: &str) -> bool {
    let extension = extension.to_lowercase();
    let extension = (if extension.starts_with('.') { "" } else { "." }).to_owned() + &extension;
    path.to_lowercase().ends_with(&extension_arg(&extension))
}

fn has_extensions<'a, T: IntoIterator<Item = &'a str>>(path: &str, extensions: T) -> bool {
    extensions.into_iter().any(|ext| has_extension(path, ext))
}

fn base_name(path: &str) -> String {
    path.split('/').last().map_or("", |s| s).to_owned()
}

fn base_name_without_ext<'a, T>(path: &str, extensions: T) -> String
    where T: IntoIterator<Item = &'a str>
{
    let extensions = extensions.into_iter().map(extension_arg).collect::<Vec<String>>();
    path.split('/').last().map_or("".to_owned(), |base| {
        regex_replace!(r"(\.[^\.]+)+$", base, |_, prev_ext: &str| {
            (if extensions.iter().any(|ext| ext == prev_ext) { "" } else { prev_ext }).to_owned()
        }).into_owned()
    })
}

/// Normalizes a path by resolving relative components and performing some changes.
/// 
/// For Windows, any `\\?\X:`, `X:`, or `\\?\UNC\` prefixes are ensured
/// to be uppercase and UNC host names and rest characters are always returned in lowercase form.
/// 
/// ```ignore
/// assert_eq!(PathBuf::from_str(r"\\?\C:\program files").unwrap(), normalize_path(r"C:/Program Files/"));
/// assert_eq!(PathBuf::from_str(r"\\?\UNC\server\foo").unwrap(), normalize_path(r"\\server\foo\"));
/// assert_eq!(PathBuf::from_str(r"\\?\C:\foo").unwrap(), normalize_path(r"\\?\c:/foo/"));
/// assert_eq!(PathBuf::from_str(r"\\?\UNC\server\foo").unwrap(), normalize_path(r"\\?\unc\server\Foo\"));
/// assert_eq!(PathBuf::from_str(r"\\?\C:").unwrap(), normalize_path(r"\\?\C:\\"));
/// assert_eq!(PathBuf::from_str(r"\\?\C:").unwrap(), normalize_path(r"\\?\C:"));
/// ```
pub fn normalize_path(p: impl AsRef<Path>) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or(PathBuf::from_str("/").unwrap());
    let p = FlexPath::from_n_native([cwd.to_str().unwrap(), &p.as_ref().to_string_lossy().to_owned()]).to_string();
    let p = regex_replace!(r"[^\\/][\\/]+$", &p, |a: &str| {
        a.chars().collect::<Vec<_>>()[0].to_string()
    }).into_owned();

    // If Windows absolute paths use extended-length syntax already,
    // ensure to use uppercase prefixes except for UNC host names.
    if regex_is_match!(r"\\\\\?\\[Uu][Nn][Cc]", &p) {
        return PathBuf::from_str(&(r"\\?\UNC".to_owned() + &p[7..].to_lowercase())).unwrap_or(PathBuf::new());
    }
    if let Some(d) = regex_captures!(r"\\\\\?\\[A-Za-z]\:", &p) {
        return PathBuf::from_str(&(d.to_uppercase() + &p[6..].to_lowercase())).unwrap_or(PathBuf::new());
    }

    // Use extended-length syntax for Windows absolute paths
    if let Some(d) = regex_captures!(r"^[A-Za-z]\:", &p) {
        return PathBuf::from_str(&(r"\\?\".to_owned() + &d.to_uppercase() + &p[2..].to_lowercase())).unwrap_or(PathBuf::new());
    }
    if regex_is_match!(r"^(\\\\([^?]|$))", &p) {
        return PathBuf::from_str(&(r"\\?\UNC".to_owned() + &p[1..].to_lowercase())).unwrap_or(PathBuf::new());
    }

    PathBuf::from_str(&p).unwrap_or(PathBuf::new())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extension_and_base_name() {
        assert!(FlexPath::new_common("a.x").has_extensions([".x", ".y"]));
        assert_eq!("a.y", FlexPath::new_common("a.x").change_extension(".y").to_string());
        assert_eq!("a.0", FlexPath::new_common("a.x.y").change_extension(".0").to_string());
        assert_eq!("a.0.1", FlexPath::new_common("a.x.y").change_extension(".0.1").to_string());

        assert_eq!("qux.html", FlexPath::new_common("foo/qux.html").base_name());
        assert_eq!("qux", FlexPath::new_common("foo/qux.html").base_name_without_ext([".html"]));
    }

    #[test]
    fn resolution() {
        assert_eq!("a", FlexPath::from_n_common(["a/b/.."]).to_string());
        assert_eq!("a", FlexPath::from_n_common(["a", "b", ".."]).to_string());
        assert_eq!("/a/b", FlexPath::new_common("/c").resolve("/a/b").to_string());
        assert_eq!("a", FlexPath::new_common("a/b").resolve("..").to_string());
        assert_eq!("a/b", FlexPath::new_common("a/b/").to_string());
        assert_eq!("a/b", FlexPath::new_common("a//b").to_string());

        let windows = FlexPathVariant::Windows;
        assert_eq!(r"\\Whack\a\Box", FlexPath::from_n(["foo", r"\\Whack////a//Box", "..", "Box"], windows).to_string());
        assert_eq!(r"\\?\X:\", FlexPath::from_n([r"\\?\X:", r".."], windows).to_string());
        assert_eq!(r"\\?\X:\", FlexPath::from_n([r"\\?\X:\", r".."], windows).to_string());
        assert_eq!(r"\\?\UNC\Whack\a\Box", FlexPath::from_n([r"\\?\UNC\Whack\a\Box", r"..", "Box"], windows).to_string());
        assert_eq!(r"C:\a", FlexPath::new("C:/", windows).resolve("a").to_string());
        assert_eq!(r"D:\", FlexPath::new("C:/", windows).resolve("D:/").to_string());
        assert_eq!(r"D:\a", FlexPath::new("D:/a", windows).to_string());
        assert_eq!(r"C:\a\f\b", FlexPath::new("a", windows).resolve("C:/a///f//b").to_string());
    }

    #[test]
    fn relativity() {
        assert_eq!("", FlexPath::new_common("/a/b").relative("/a/b"));
        assert_eq!("c", FlexPath::new_common("/a/b").relative("/a/b/c"));
        assert_eq!("../../c/d", FlexPath::new_common("/a/b/c").relative("/a/c/d"));
        assert_eq!("..", FlexPath::new_common("/a/b/c").relative("/a/b"));
        assert_eq!("../..", FlexPath::new_common("/a/b/c").relative("/a"));
        assert_eq!("..", FlexPath::new_common("/a").relative("/"));
        assert_eq!("a", FlexPath::new_common("/").relative("/a"));
        assert_eq!("", FlexPath::new_common("/").relative("/"));
        assert_eq!("../../c/d", FlexPath::new_common("/a/b").relative("/c/d"));
        assert_eq!("../c", FlexPath::new_common("/a/b").relative("/a/c"));

        let windows = FlexPathVariant::Windows;
        assert_eq!("", FlexPath::new("C:/", windows).relative("C:/"));
        assert_eq!("", FlexPath::new("C:/foo", windows).relative("C:/foo"));
        assert_eq!(r"\\foo", FlexPath::new("C:/", windows).relative(r"\\foo"));
        assert_eq!("../../foo", FlexPath::new(r"\\a/b", windows).relative(r"\\foo"));
        assert_eq!("D:/", FlexPath::new("C:/", windows).relative(r"D:"));
        assert_eq!("../bar", FlexPath::new(r"\\?\C:\foo", windows).relative(r"\\?\C:\bar"));
    }

    #[test]
    fn normalization() {
        assert_eq!(PathBuf::from_str(r"\\?\C:\program files").unwrap(), normalize_path(r"C:/Program Files/"));
        assert_eq!(PathBuf::from_str(r"\\?\UNC\server\foo").unwrap(), normalize_path(r"\\server\foo\"));
        assert_eq!(PathBuf::from_str(r"\\?\C:\foo").unwrap(), normalize_path(r"\\?\c:/foo/"));
        assert_eq!(PathBuf::from_str(r"\\?\UNC\server\foo").unwrap(), normalize_path(r"\\?\unc\server\Foo\"));
    }
}