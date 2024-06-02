/// Formats every T as `...`
pub fn mask_fmt<T>(_: &T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    f.write_str("...")
}
