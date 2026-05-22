#[derive(Debug)]
pub enum ScanError {
    NotFound,
    Multiple,
}

pub fn find_unique(haystack: &[u8], needle: &[u8]) -> Result<usize, ScanError> {
    if needle.is_empty() {
        return Err(ScanError::NotFound);
    }
    let mut found: Option<usize> = None;
    for window_start in 0..=haystack.len().saturating_sub(needle.len()) {
        if &haystack[window_start..window_start + needle.len()] == needle {
            if found.is_some() {
                return Err(ScanError::Multiple);
            }
            found = Some(window_start);
        }
    }
    found.ok_or(ScanError::NotFound)
}

#[must_use]
pub fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    if needle.is_empty() {
        return out;
    }
    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        if &haystack[start..start + needle.len()] == needle {
            out.push(start);
            start += needle.len();
        } else {
            start += 1;
        }
    }
    out
}
