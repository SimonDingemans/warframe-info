use crate::pipeline::TextNormalizer;

#[derive(Debug, Clone, Copy, Default)]
pub struct WarframeTextNormalizer;

impl TextNormalizer for WarframeTextNormalizer {
    fn normalize(&self, text: &str) -> Option<String> {
        let mut cleaned = text.trim().to_string();

        if let Some(prime_start) = cleaned.find("Prime") {
            if cleaned[..prime_start]
                .chars()
                .all(|c| c.is_ascii_lowercase())
            {
                cleaned.replace_range(..prime_start, "");
            }
        }

        if cleaned.is_empty() {
            return None;
        }

        if cleaned
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_lowercase())
        {
            let mut chars = cleaned.chars();
            cleaned = match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            };
        }

        let mut spaced_text = String::new();
        let mut chars = cleaned.chars().peekable();

        while let Some(c) = chars.next() {
            spaced_text.push(c);

            if let Some(&next_c) = chars.peek() {
                if c.is_ascii_lowercase() && next_c.is_ascii_uppercase() {
                    spaced_text.push(' ');
                }
            }
        }

        Some(spaced_text)
    }
}
