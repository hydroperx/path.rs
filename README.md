# fairyvoid::path

Work with textual file paths, including relativity and resolution. Features:

- _Variant_: `FlexPath` methods consider absolute paths according to the path's `FlexPathVariant`. Two variants are supported: `Common` and `Windows`. The native variant can be deduced directly through `_native` suffixed methods.
- _Normalization_: normalize paths with the `normalize_path()` function, whether they exist or not (does not solve for symlinks and real letter case).

Requirements:

- The Rust standard library (`std`).

# Example

```rust
use fairyvoid_path::FlexPath;

assert_eq!("a", FlexPath::new_common("a/b").resolve("..").to_string());
assert_eq!("a", FlexPath::new_common("a/b/..").to_string());
assert_eq!("a/b/c/d/e", FlexPath::from_n_common(["a/b", "c/d", "e/f", ".."]).to_string());
assert_eq!("../../c/d", FlexPath::new_common("/a/b").relative("/c/d"))
```