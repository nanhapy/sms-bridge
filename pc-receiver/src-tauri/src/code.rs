use regex::Regex;
use std::sync::OnceLock;

static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

pub fn valid_explicit_code(value: &str) -> Option<String> {
    Regex::new(r"^[0-9]{4,8}$")
        .expect("valid explicit-code pattern")
        .is_match(value)
        .then(|| value.to_string())
}

pub fn extract_code(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    patterns().iter().find_map(|pattern| {
        pattern
            .captures(text)
            .and_then(|captures| captures.get(1))
            .map(|capture| capture.as_str().to_string())
    })
}

fn patterns() -> &'static Vec<Regex> {
    PATTERNS.get_or_init(|| {
        [
            r"(?:验证码|校验码|动态验证码|动态码|安全码|确认码|短信码)\s*(?:是|为|:|：|，|,)?\s*[^0-9]{0,10}?([0-9]{4,8})",
            r"(?:验证码|校验码|动态验证码|动态码|安全码|确认码|短信码)[^0-9]{0,20}?([0-9]{4,8})",
            r"(?i)(?:code|verification\s*code|otp)[^0-9]{0,20}?([0-9]{4,8})",
            r"([0-9]{4,8})[^0-9]{0,10}?(?:验证码|校验码|动态验证码|动态码|安全码|短信码)",
            r"(?:【|\[)([0-9]{4,8})(?:】|\])",
            r"(?-u:\b([0-9]{6})\b)",
            r"(?-u:\b([0-9]{4})\b)",
        ]
        .into_iter()
        .map(|pattern| Regex::new(pattern).expect("valid verification-code pattern"))
        .collect()
    })
}
