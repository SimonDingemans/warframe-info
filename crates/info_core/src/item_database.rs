use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemDatabase {
    items: Vec<WarframeItem>,
}

impl ItemDatabase {
    pub fn new(items: Vec<WarframeItem>) -> Self {
        Self { items }
    }

    pub fn find_item(&self, text: &str, threshold: Option<usize>) -> Option<&WarframeItem> {
        let needle = searchable_item_name(text);
        let needle_is_set = needle.ends_with("set");

        if needle.is_empty() {
            return None;
        }

        self.items
            .iter()
            .filter(|item| needle_is_set || !item.name.ends_with(" Set"))
            .filter_map(|item| {
                let distance = levenshtein_distance(&searchable_item_name(&item.name), &needle);
                let current_threshold = threshold.unwrap_or(item.name.len() / 3);

                (distance <= current_threshold).then_some((item, distance))
            })
            .min_by_key(|(_, distance)| *distance)
            .map(|(item, _)| item)
    }

    pub fn find_items<'a>(&self, texts: impl IntoIterator<Item = &'a str>) -> Vec<WarframeItem> {
        texts
            .into_iter()
            .filter_map(|text| self.find_item(text, None).cloned())
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WarframeItem {
    pub name: String,
    pub market_slug: Option<String>,
    pub platinum: f32,
    pub ducats: Option<u32>,
    pub vaulted: bool,
}

impl WarframeItem {
    pub fn platinum_rounded(&self) -> u32 {
        self.platinum.round().max(0.0) as u32
    }
}

fn searchable_item_name(name: &str) -> String {
    name.chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let mut previous = (0..=right.chars().count()).collect::<Vec<_>>();
    let mut current = vec![0; previous.len()];

    for (left_index, left_character) in left.chars().enumerate() {
        current[0] = left_index + 1;

        for (right_index, right_character) in right.chars().enumerate() {
            let insertion = current[right_index] + 1;
            let deletion = previous[right_index + 1] + 1;
            let substitution =
                previous[right_index] + usize::from(left_character != right_character);
            current[right_index + 1] = insertion.min(deletion).min(substitution);
        }

        std::mem::swap(&mut previous, &mut current);
    }

    previous[right.chars().count()]
}

#[cfg(test)]
mod tests {
    use super::{levenshtein_distance, ItemDatabase, WarframeItem};

    #[test]
    fn database_fuzzy_matches_ocr_text_without_matching_sets_by_default() {
        let database = test_database();

        let item = database
            .find_item("ash prime systerns blueprint", None)
            .expect("fuzzy match");

        assert_eq!(item.name, "Ash Prime Systems Blueprint");
        assert_eq!(database.find_item("Ash Prime", None), None);

        let set = database
            .find_item("Ash Prime Set", None)
            .expect("set should match when set was requested");
        assert_eq!(set.name, "Ash Prime Set");
    }

    #[test]
    fn database_returns_warframe_items_for_ocr_text() {
        let database = test_database();
        let items = database.find_items(["Ash Prime Systerns Blueprint", "unknown"]);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Ash Prime Systems Blueprint");
    }

    #[test]
    fn levenshtein_distance_handles_insertions_deletions_and_substitutions() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    fn test_database() -> ItemDatabase {
        ItemDatabase::new(vec![
            WarframeItem {
                name: "Ash Prime Systems Blueprint".to_owned(),
                market_slug: Some("ash_prime_systems_blueprint".to_owned()),
                platinum: 22.0,
                ducats: Some(65),
                vaulted: true,
            },
            WarframeItem {
                name: "Ash Prime Set".to_owned(),
                market_slug: Some("ash_prime_set".to_owned()),
                platinum: 80.0,
                ducats: None,
                vaulted: true,
            },
        ])
    }
}
