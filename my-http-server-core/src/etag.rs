//! Entity tags (RFC 9110 §8.8.3) and the `If-None-Match` precondition (RFC 9110 §13.1.2).
//!
//! The hash behind a tag is the caller's business - this module only puts a tag into the shape
//! the `ETag` header wants, and decides whether what a client sent back in `If-None-Match`
//! matches it.

/// The value of an `ETag` header holding `opaque_tag` as a **strong** validator: `"opaque_tag"`.
///
/// A strong ETag promises byte-for-byte identity, so it belongs to the exact bytes it was computed
/// for. Another content-coding of the same content (gzip, deflate, br, ...) is other bytes: it
/// needs an ETag of its own, or a weak one - see [`weak_etag`].
///
/// `opaque_tag` is the bare tag: no quotes and no `W/`. A double quote, a space or a control
/// character in it makes an ETag no client is obliged to understand.
pub fn strong_etag(opaque_tag: &str) -> String {
    let mut result = String::with_capacity(opaque_tag.len() + 2);
    result.push('"');
    result.push_str(opaque_tag);
    result.push('"');
    result
}

/// The value of an `ETag` header holding `opaque_tag` as a **weak** validator: `W/"opaque_tag"`.
///
/// A weak ETag only promises that the content is the same, not the bytes - which is what one
/// tag shared by the compressed variants of a file can promise.
///
/// `opaque_tag` is the bare tag, as for [`strong_etag`].
pub fn weak_etag(opaque_tag: &str) -> String {
    let mut result = String::with_capacity(opaque_tag.len() + 4);
    result.push_str("W/\"");
    result.push_str(opaque_tag);
    result.push('"');
    result
}

/// Whether the `If-None-Match` of a request matches `etag` - that is, whether the client already
/// has the representation, and a `GET` / `HEAD` is answered `304 Not Modified`.
///
/// - `if_none_match` - every `If-None-Match` field line of the request. A header sent as several
///   lines is one comma-separated list (RFC 9110 §5.3), so all of them count. `&str`, `&[u8]` and
///   `&HeaderValue` all fit: `headers.get_all(IF_NONE_MATCH)` can be passed as it is.
/// - `etag` - the `ETag` the response carries: `"x"`, `W/"x"`, or just the opaque tag `x`.
///
/// The field is evaluated as RFC 9110 §13.1.2 says, with the weak comparison of §8.8.3.2:
///
/// - `*` matches whatever the representation is;
/// - otherwise the field is a list of entity tags separated by commas, with optional whitespace
///   around them. A comma inside the quotes is a part of the tag;
/// - `W/` is not looked at, on either side;
/// - a tag sent without quotes is taken as the opaque tag itself. That is how this library wrote
///   ETags earlier - base64 of a SHA-256, padding stripped, no quotes - and the hashes did not
///   change: a client holding one of those still gets its `304` instead of downloading the
///   content again;
/// - a field that is not a valid list matches nothing, whatever else is in it: answering `200` is
///   always safe, answering `304` to a client that does not have the content is not. So does an
///   `etag` that is not a valid entity tag.
pub fn if_none_match_matches(
    if_none_match: impl IntoIterator<Item = impl AsRef<[u8]>>,
    etag: &str,
) -> bool {
    let Some(current) = opaque_tag_of(etag.as_bytes()) else {
        return false;
    };

    let mut matched = false;

    for field_line in if_none_match {
        let valid = for_each_list_element(field_line.as_ref(), |element| match element {
            ListElement::Any => matched = true,
            ListElement::Tag(opaque_tag) => {
                if opaque_tag == current {
                    matched = true;
                }
            }
        });

        if !valid {
            return false;
        }
    }

    matched
}

enum ListElement<'s> {
    /// `*`
    Any,
    /// The opaque tag of an entity tag - without its quotes and without `W/`.
    Tag(&'s [u8]),
}

/// Calls `on_element` for every element of one field line of an `#entity-tag` list. Returns
/// `false` as soon as the line turns out not to be a valid list.
fn for_each_list_element<'s>(line: &'s [u8], mut on_element: impl FnMut(ListElement<'s>)) -> bool {
    let mut pos = 0;

    loop {
        // Whitespace, and the empty elements a list may have: `, ,"a"` is a valid list.
        while pos < line.len() && matches!(line[pos], b' ' | b'\t' | b',') {
            pos += 1;
        }

        if pos == line.len() {
            return true;
        }

        let Some((element, end)) = parse_element(line, pos) else {
            return false;
        };

        on_element(element);

        // An element is followed by optional whitespace and then a comma or the end of the line:
        // `"a" "b"` is not a list.
        pos = skip_ows(line, end);

        if pos < line.len() && line[pos] != b',' {
            return false;
        }
    }
}

/// Parses the element which starts at `start` - neither whitespace nor a comma. Returns it with
/// the position right after it, or `None` if it is not an element of an `If-None-Match` list.
fn parse_element(line: &[u8], start: usize) -> Option<(ListElement<'_>, usize)> {
    let rest = &line[start..];

    // `W/` is a prefix only in front of a quote: a tag written without quotes may start with `W/`
    // itself - base64, which the old ETags are, has both characters.
    let opening_quote = if rest.starts_with(b"W/\"") {
        start + 2
    } else if rest.starts_with(b"\"") {
        start
    } else {
        // No quotes: `*`, or a tag the way this library wrote ETags earlier - base64 of a
        // SHA-256, padding stripped, no quotes.
        let end = rest
            .iter()
            .position(|b| matches!(b, b',' | b' ' | b'\t'))
            .map_or(line.len(), |len| start + len);

        let token = &line[start..end];

        if token == b"*" {
            return Some((ListElement::Any, end));
        }

        if token.iter().all(|b| is_etagc(*b)) {
            return Some((ListElement::Tag(token), end));
        }

        return None;
    };

    let opaque_tag_start = opening_quote + 1;

    for (len, b) in line[opaque_tag_start..].iter().enumerate() {
        if *b == b'"' {
            let closing_quote = opaque_tag_start + len;
            return Some((
                ListElement::Tag(&line[opaque_tag_start..closing_quote]),
                closing_quote + 1,
            ));
        }

        if !is_etagc(*b) {
            return None;
        }
    }

    // No closing quote.
    None
}

/// The opaque tag of the server's own ETag: `"x"`, `W/"x"` or a bare `x`.
fn opaque_tag_of(etag: &[u8]) -> Option<&[u8]> {
    let start = skip_ows(etag, 0);
    let mut end = etag.len();
    while end > start && matches!(etag[end - 1], b' ' | b'\t') {
        end -= 1;
    }

    let etag = &etag[..end];

    if start == etag.len() {
        return None;
    }

    match parse_element(etag, start)? {
        (ListElement::Tag(opaque_tag), end) if end == etag.len() => Some(opaque_tag),
        _ => None,
    }
}

fn skip_ows(line: &[u8], mut pos: usize) -> usize {
    while pos < line.len() && matches!(line[pos], b' ' | b'\t') {
        pos += 1;
    }

    pos
}

/// `etagc = %x21 / %x23-7E / obs-text` - any visible character but the double quote.
fn is_etagc(b: u8) -> bool {
    b == 0x21 || (0x23..=0x7E).contains(&b) || b >= 0x80
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(if_none_match: &str, etag: &str) -> bool {
        if_none_match_matches([if_none_match], etag)
    }

    #[test]
    fn formats_strong_and_weak_etags() {
        assert_eq!(strong_etag("abc"), "\"abc\"");
        assert_eq!(weak_etag("abc"), "W/\"abc\"");
        assert_eq!(strong_etag(""), "\"\"");
    }

    #[test]
    fn exact_match() {
        assert!(matches("\"abc\"", "\"abc\""));
        assert!(!matches("\"abd\"", "\"abc\""));
        assert!(!matches("\"ABC\"", "\"abc\""));
        assert!(!matches("\"abc\"", "\"abcd\""));
    }

    #[test]
    fn weak_prefix_is_ignored_on_either_side() {
        assert!(matches("W/\"abc\"", "\"abc\""));
        assert!(matches("\"abc\"", "W/\"abc\""));
        assert!(matches("W/\"abc\"", "W/\"abc\""));
        assert!(!matches("W/\"abd\"", "W/\"abc\""));
    }

    #[test]
    fn the_etag_may_be_given_as_a_bare_opaque_tag() {
        assert!(matches("\"abc\"", "abc"));
        assert!(matches("W/\"abc\"", "abc"));
        assert!(!matches("\"abd\"", "abc"));
    }

    #[test]
    fn a_list_matches_if_one_of_its_tags_does() {
        assert!(matches("\"x\", \"abc\", \"y\"", "\"abc\""));
        assert!(matches("\"x\",W/\"abc\"", "\"abc\""));
        assert!(!matches("\"x\", \"y\"", "\"abc\""));
    }

    #[test]
    fn star_matches_any_etag() {
        assert!(matches("*", "\"abc\""));
        assert!(matches("*", "W/\"abc\""));
        assert!(matches(" * ", "\"abc\""));
        assert!(matches("\"x\", *", "\"abc\""));
    }

    #[test]
    fn whitespace_around_elements_is_optional() {
        assert!(matches("  \"abc\"  ", "\"abc\""));
        assert!(matches("\"x\"   ,\t\"abc\"", "\"abc\""));
        assert!(matches("\"x\",\"abc\"", "\"abc\""));
        assert!(matches(", ,\"abc\",,", "\"abc\""));
    }

    #[test]
    fn a_comma_inside_quotes_is_a_part_of_the_tag() {
        assert!(matches("\"x\", \"a,b\"", "\"a,b\""));
        assert!(matches("\"a,b\"", "W/\"a,b\""));
        // Split at every comma, this would be `"a`, `b`, `c"` - and the `b` in the middle would
        // pass for a tag written without quotes.
        assert!(!matches("\"a,b,c\"", "\"b\""));
        assert!(!matches("\"a,b\"", "\"a\""));
    }

    #[test]
    fn an_etag_without_quotes_is_taken_as_the_opaque_tag() {
        // The way this library wrote ETags earlier: base64 of a SHA-256, padding stripped, no
        // quotes.
        let legacy = "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg";

        assert!(matches(legacy, &strong_etag(legacy)));
        assert!(matches(legacy, &weak_etag(legacy)));
        assert!(matches(legacy, legacy));
        assert!(matches(&format!("\"x\", {legacy}"), &strong_etag(legacy)));
        assert!(!matches(legacy, "\"n4bQgYhMfWWaL\""));

        // Base64 may start with `W/` - without a quote after it, it is a part of the tag.
        assert!(matches("W/abc", "\"W/abc\""));
        assert!(!matches("W/abc", "\"abc\""));
    }

    #[test]
    fn non_ascii_bytes_are_compared_as_they_are() {
        assert!(matches("\"тег\"", "\"тег\""));
        assert!(!matches("\"тег\"", "\"теж\""));
    }

    #[test]
    fn garbage_matches_nothing() {
        for garbage in [
            "\"abc",
            "abc\"",
            "\"abc\"x",
            "\"abc\" \"abc\"",
            "\"a bc\"",
            "a bc",
            "W/ \"abc\"",
            "w/\"abc\"",
            "\"ab\"c\"",
            "\"abc\u{7}\"",
            "**",
            "\"abc\", \"x",
            "\"abc\", x y",
        ] {
            assert!(!matches(garbage, "\"abc\""), "{garbage}");
        }
    }

    #[test]
    fn an_empty_field_matches_nothing() {
        assert!(!matches("", "\"abc\""));
        assert!(!matches("  ", "\"abc\""));
        assert!(!matches(", ,", "\"abc\""));
        assert!(!if_none_match_matches(Vec::<&str>::new(), "\"abc\""));
    }

    #[test]
    fn every_field_line_counts() {
        assert!(if_none_match_matches(["\"x\"", "\"abc\""], "\"abc\""));
        assert!(if_none_match_matches(["\"x\"", "*"], "\"abc\""));
        assert!(!if_none_match_matches(["\"x\"", "\"y\""], "\"abc\""));
        // A line that is not a valid list spoils the whole field, the matching line included.
        assert!(!if_none_match_matches(["\"abc\"", "\"x"], "\"abc\""));
    }

    #[test]
    fn header_values_and_bytes_are_accepted() {
        let value = hyper::header::HeaderValue::from_static("W/\"abc\"");
        assert!(if_none_match_matches([&value], "\"abc\""));
        assert!(if_none_match_matches([b"\"abc\"".as_slice()], "\"abc\""));
        assert!(if_none_match_matches(Some("\"abc\""), "\"abc\""));
    }

    #[test]
    fn an_invalid_etag_matches_nothing() {
        for etag in ["", " ", "\"abc", "a bc", "\"abc\" x", "*", "W/ \"abc\""] {
            assert!(!matches("*", etag), "{etag}");
            assert!(!matches("\"abc\"", etag), "{etag}");
        }
    }
}
