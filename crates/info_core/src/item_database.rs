use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemDatabase {
    items: Vec<WarframeItem>,
    market_items: Vec<MarketItem>,
}

impl ItemDatabase {
    pub fn new(items: Vec<WarframeItem>) -> Self {
        Self {
            items,
            market_items: Vec::new(),
        }
    }

    pub fn with_market_items(items: Vec<WarframeItem>, market_items: Vec<MarketItem>) -> Self {
        Self {
            items,
            market_items,
        }
    }

    pub fn items(&self) -> &[WarframeItem] {
        &self.items
    }

    pub fn market_items(&self) -> &[MarketItem] {
        &self.market_items
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
                let distance =
                    levenshtein_distance(&searchable_item_name(&item.drop_name), &needle);
                let current_threshold = threshold.unwrap_or(item.drop_name.len() / 3);

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

    pub fn find_market_item(&self, name: &str) -> Option<&MarketItem> {
        self.market_items.iter().find(|item| {
            item.name == name || item.localized_names.values().any(|value| value == name)
        })
    }

    pub fn find_market_item_fuzzy(
        &self,
        text: &str,
        threshold: Option<usize>,
    ) -> Option<&MarketItem> {
        let needle = searchable_item_name(text);

        if needle.is_empty() {
            return None;
        }

        self.market_items
            .iter()
            .filter_map(|item| {
                let distance = market_item_search_names(item)
                    .into_iter()
                    .map(|name| levenshtein_distance(&searchable_item_name(name), &needle))
                    .min()?;
                let current_threshold = threshold.unwrap_or(item.name.len() / 3);

                (distance <= current_threshold).then_some((item, distance))
            })
            .min_by_key(|(_, distance)| *distance)
            .map(|(item, _)| item)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WarframeItem {
    pub name: String,
    pub drop_name: String,
    pub market_slug: Option<String>,
    pub platinum: f32,
    pub ducats: Option<u32>,
    pub volume: u32,
    pub vaulted: bool,
}

impl WarframeItem {
    pub fn platinum_rounded(&self) -> u32 {
        self.platinum.round().max(0.0) as u32
    }

    pub fn summary(&self) -> String {
        let mut details = Vec::new();

        if self.platinum_rounded() > 0 {
            details.push(format!("{}p", self.platinum_rounded()));
        }

        if let Some(ducats) = self.ducats {
            details.push(format!("{ducats} ducats"));
        }

        if self.volume > 0 {
            details.push(format!("{} sold", self.volume));
        }

        if self.vaulted {
            details.push("vaulted".to_owned());
        }

        if details.is_empty() {
            self.drop_name.clone()
        } else {
            format!("{} ({})", self.drop_name, details.join(", "))
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MarketItem {
    pub slug: String,
    pub name: String,
    pub localized_names: HashMap<String, String>,
}

fn market_item_search_names(item: &MarketItem) -> Vec<&str> {
    let mut names = Vec::with_capacity(item.localized_names.len() + 1);
    names.push(item.name.as_str());
    names.extend(item.localized_names.values().map(String::as_str));
    names
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
    use std::collections::HashMap;

    use super::{levenshtein_distance, ItemDatabase, MarketItem, WarframeItem};

    #[test]
    fn database_fuzzy_matches_ocr_text_without_matching_sets_by_default() {
        let database = test_database();

        let item = database
            .find_item("ash prime systerns blueprint", None)
            .expect("fuzzy match");

        assert_eq!(item.drop_name, "Ash Prime Systems Blueprint");
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
        assert_eq!(items[0].drop_name, "Ash Prime Systems Blueprint");
        assert_eq!(
            items[0].summary(),
            "Ash Prime Systems Blueprint (22p, 65 ducats, 7 sold, vaulted)"
        );
    }

    #[test]
    fn database_fuzzy_matches_market_item_names_and_translations() {
        let database = test_database();

        let market_item = database
            .find_market_item_fuzzy("Plan de Systemes d Ash Prime", None)
            .expect("localized fuzzy match");

        assert_eq!(market_item.slug, "ash_prime_systems_blueprint");
    }

    #[test]
    fn database_matches_market_item_names_and_translations_exactly() {
        let database = test_database();

        let market_item = database
            .find_market_item("Plan de Systemes d'Ash Prime")
            .expect("localized market item");

        assert_eq!(market_item.slug, "ash_prime_systems_blueprint");
    }

    #[test]
    fn summary_omits_missing_or_zero_details() {
        let item = WarframeItem {
            name: "Arcane Concentration".to_owned(),
            drop_name: "Arcane Concentration".to_owned(),
            market_slug: Some("arcane_concentration".to_owned()),
            platinum: 0.0,
            ducats: None,
            volume: 0,
            vaulted: false,
        };

        assert_eq!(item.summary(), "Arcane Concentration");
    }

    #[test]
    fn levenshtein_distance_handles_insertions_deletions_and_substitutions() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    fn test_database() -> ItemDatabase {
        ItemDatabase::with_market_items(
            vec![
                WarframeItem {
                    name: "Ash Prime Systems".to_owned(),
                    drop_name: "Ash Prime Systems Blueprint".to_owned(),
                    market_slug: Some("ash_prime_systems_blueprint".to_owned()),
                    platinum: 22.0,
                    ducats: Some(65),
                    volume: 7,
                    vaulted: true,
                },
                WarframeItem {
                    name: "Ash Prime Set".to_owned(),
                    drop_name: "Ash Prime Set".to_owned(),
                    market_slug: Some("ash_prime_set".to_owned()),
                    platinum: 80.0,
                    ducats: None,
                    volume: 15,
                    vaulted: true,
                },
            ],
            vec![MarketItem {
                slug: "ash_prime_systems_blueprint".to_owned(),
                name: "Ash Prime Systems Blueprint".to_owned(),
                localized_names: HashMap::from([
                    ("en".to_owned(), "Ash Prime Systems Blueprint".to_owned()),
                    ("fr".to_owned(), "Plan de Systemes d'Ash Prime".to_owned()),
                ]),
            }],
        )
    }
}
