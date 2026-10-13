use std::borrow::Cow;

/// `true`: the path has a `..` segment - written as it is or percent-encoded.
///
/// The middleware glues the request path to its folder as it was received, and the file system
/// follows a `..`: `/../../etc/hosts` glued to `./wwwroot` is a file of the host, not of the
/// folder. A path like that names no file of a folder served here, so it is refused before any
/// file is looked at.
///
/// The path is decoded for the check only, and once. The middleware looks a file up by the name
/// as it was received, so an encoded `..` leaves no folder today - it is refused with the plain
/// one, to keep it that way whatever decodes a path later. A backslash is taken for a separator
/// as well: on Windows it is one.
pub fn has_parent_segment(path: &str) -> bool {
    percent_decode(path.as_bytes())
        .split(|b| *b == b'/' || *b == b'\\')
        .any(|segment| segment == b"..")
}

/// One pass: `%XX` with two hex digits becomes the byte, anything else stays as it is - no `+`
/// for a space, no second pass. A `%` which starts no escape does not hide the escape next to
/// it.
fn percent_decode(src: &[u8]) -> Cow<'_, [u8]> {
    if !src.contains(&b'%') {
        return Cow::Borrowed(src);
    }

    let mut result = Vec::with_capacity(src.len());
    let mut i = 0;

    while i < src.len() {
        if src[i] == b'%' && i + 2 < src.len() {
            if let (Some(high), Some(low)) = (hex_digit(src[i + 1]), hex_digit(src[i + 2])) {
                result.push(high << 4 | low);
                i += 3;
                continue;
            }
        }

        result.push(src[i]);
        i += 1;
    }

    Cow::Owned(result)
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::has_parent_segment;

    #[test]
    fn a_parent_segment_is_found_however_it_is_written() {
        for path in [
            "/..",
            "/../",
            "/../secret.txt",
            "/a/../b",
            "/a/..",
            "/a/b/../../../secret.txt",
            "/%2e%2e/secret.txt",
            "/%2E%2E/secret.txt",
            "/%2e%2E/secret.txt",
            "/%2e./secret.txt",
            "/.%2e/secret.txt",
            "/.%2E/secret.txt",
            "/a/%2e%2e",
            "/..%2fsecret.txt",
            "/..%2Fsecret.txt",
            "/a%2f..%2fb",
            "/a%2F%2e%2e%2Fb",
            "/%2f..",
            "/..\\secret.txt",
            "/a\\..\\b",
            "/..%5csecret.txt",
            "/..%5Csecret.txt",
            "/%2e%2e%5csecret.txt",
            // A `%` which starts no escape does not hide the escape next to it.
            "/%zz/%2e%2e/secret.txt",
            "/%/../secret.txt",
        ] {
            assert!(has_parent_segment(path), "{path}");
        }
    }

    #[test]
    fn dots_in_a_name_are_not_a_segment() {
        for path in [
            "/",
            "/index.html",
            "/a/b/c.css",
            "/a/",
            "/.a",
            "/a.",
            "/.well-known/acme-challenge/x",
            "/.env",
            "/a..b",
            "/..a",
            "/a..",
            "/a../b",
            "/...",
            "/.../x",
            "/.",
            "/./a",
            "/%2e",
            "/%2e/a",
            "/a%2eb",
            "/file%20name.txt",
            "/100%.txt",
            "/a%2",
            "/a%",
            "/%zz",
            "/..;/x",
        ] {
            assert!(!has_parent_segment(path), "{path}");
        }
    }

    /// Nothing behind the check decodes a path, so what is encoded twice stays a name.
    #[test]
    fn a_path_is_decoded_once() {
        assert!(!has_parent_segment("/%252e%252e/secret.txt"));
        assert!(!has_parent_segment("/%252e%252e%252fsecret.txt"));
    }
}
