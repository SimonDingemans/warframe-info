use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use info_core::{ItemDatabase, MarketItem, ScanOutput, WarframeItem};
use serde::{Deserialize, Serialize};
use wf_market::{ApiCache, Client, Item, SerializableCache, Unauthenticated};

const CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const ITEM_CACHE_FILE: &str = "wf_market_cache.json";
const PRICE_CACHE_FILE: &str = "wf_market_price_cache.json";

pub(crate) struct MarketData {
    pub(crate) database: ItemDatabase,
    client: Option<Client<Unauthenticated>>,
    price_cache: PriceCache,
    price_cache_path: PathBuf,
}

impl MarketData {
    pub(crate) async fn load() -> Result<Self, String> {
        load_market_index().await
    }

    pub(crate) async fn enrich_scan_output(&mut self, mut output: ScanOutput) -> ScanOutput {
        let Some(client) = &self.client else {
            return output;
        };

        for item in &mut output.items {
            let Some(slug) = item.market_slug.as_deref() else {
                continue;
            };

            if let Some(price) = self.price_cache.get_fresh(slug) {
                if let Some(price) = price {
                    item.platinum = price as f32;
                }

                continue;
            }

            if let Ok(top_orders) = client.get_top_orders(slug, None).await {
                let price = top_orders.best_sell_price();
                self.price_cache.insert(slug, price);

                if let Some(price) = price {
                    item.platinum = price as f32;
                }
            }
        }

        let _ = save_price_cache(&self.price_cache_path, &self.price_cache);
        output
    }
}

pub(crate) fn run_cache_command_from_args(
    mut args: impl Iterator<Item = OsString>,
) -> Option<Result<(), String>> {
    let command = args.next()?;

    if command != "cache" {
        return None;
    }

    match args.next().as_deref().and_then(|arg| arg.to_str()) {
        Some("clear") | Some("invalidate") => Some(invalidate_caches()),
        Some(command) => Some(Err(format!(
            "unknown cache command {command:?}; expected `cache clear`"
        ))),
        None => Some(Err(
            "missing cache command; expected `cache clear`".to_owned()
        )),
    }
}

pub(crate) fn invalidate_caches() -> Result<(), String> {
    let cache_dir = default_cache_dir();
    let paths = [
        cache_dir.join(ITEM_CACHE_FILE),
        cache_dir.join(PRICE_CACHE_FILE),
    ];
    let mut errors = Vec::new();

    for path in paths {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => errors.push(format!("could not remove {}: {error}", path.display())),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

async fn load_market_index() -> Result<MarketData, String> {
    let cache_dir = default_cache_dir();
    let cache_path = cache_dir.join(ITEM_CACHE_FILE);
    let price_cache_path = cache_dir.join(PRICE_CACHE_FILE);
    let (serializable_cache, cache_load_error) = match load_serializable_cache(&cache_path) {
        Ok(cache) => (cache, None),
        Err(error) => (SerializableCache::default(), Some(error)),
    };
    let cached_items = cached_items(&serializable_cache);
    let mut cache = serializable_cache.into_api_cache();
    cache.invalidate_items_if_older_than(CACHE_TTL);

    match Client::builder().build_with_cache(&mut cache).await {
        Ok(client) => {
            let _ = save_cache(&cache_path, &cache);

            let database = item_database_from_market_items(client.items().as_slice());
            let price_cache = load_price_cache(&price_cache_path).unwrap_or_default();

            Ok(MarketData {
                database,
                client: Some(client),
                price_cache,
                price_cache_path,
            })
        }
        Err(error) => {
            let Some(items) = cached_items else {
                let cache_context = cache_load_error
                    .map(|error| format!("; cached item index was unavailable: {error}"))
                    .unwrap_or_default();

                return Err(format!(
                    "could not load Warframe Market item index: {error}{cache_context}"
                ));
            };

            Ok(MarketData {
                database: item_database_from_market_items(&items),
                client: None,
                price_cache: PriceCache::default(),
                price_cache_path,
            })
        }
    }
}

fn load_serializable_cache(path: &Path) -> Result<SerializableCache, String> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str::<SerializableCache>(&json)
            .map_err(|error| format!("could not parse {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(SerializableCache::default())
        }
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

fn cached_items(cache: &SerializableCache) -> Option<Vec<Item>> {
    cache
        .items
        .as_ref()
        .map(|items| items.data.clone())
        .filter(|items| !items.is_empty())
}

fn save_cache(path: &Path, cache: &ApiCache) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }

    let cache = SerializableCache::from(cache);
    let json = serde_json::to_string(&cache)
        .map_err(|error| format!("could not serialize WFM cache: {error}"))?;

    fs::write(path, json).map_err(|error| format!("could not write {}: {error}", path.display()))
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct PriceCache {
    #[serde(default)]
    prices: HashMap<String, CachedPrice>,
}

impl PriceCache {
    fn get_fresh(&self, slug: &str) -> Option<Option<u32>> {
        let cached = self.prices.get(slug)?;
        let age = current_unix_time().saturating_sub(cached.fetched_at_unix);

        (age < CACHE_TTL.as_secs()).then_some(cached.platinum)
    }

    fn insert(&mut self, slug: &str, platinum: Option<u32>) {
        self.prices.insert(
            slug.to_owned(),
            CachedPrice {
                platinum,
                fetched_at_unix: current_unix_time(),
            },
        );
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CachedPrice {
    platinum: Option<u32>,
    fetched_at_unix: u64,
}

fn load_price_cache(path: &Path) -> Result<PriceCache, String> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str::<PriceCache>(&json)
            .map_err(|error| format!("could not parse {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(PriceCache::default()),
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

fn save_price_cache(path: &Path, cache: &PriceCache) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }

    let json = serde_json::to_string(cache)
        .map_err(|error| format!("could not serialize WFM price cache: {error}"))?;

    fs::write(path, json).map_err(|error| format!("could not write {}: {error}", path.display()))
}

fn current_unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn item_database_from_market_items(items: &[Item]) -> ItemDatabase {
    let mut market_items = Vec::with_capacity(items.len());
    let mut warframe_items = Vec::with_capacity(items.len());

    for item in items {
        if item.tradable == Some(false) {
            continue;
        }

        let localized_names = localized_names(item);
        let name = item_name(item, &localized_names);

        if name.is_empty() {
            continue;
        }

        market_items.push(MarketItem {
            slug: item.slug.clone(),
            name: name.clone(),
            localized_names,
        });

        warframe_items.push(WarframeItem {
            name: name.clone(),
            drop_name: name,
            market_slug: Some(item.slug.clone()),
            platinum: 0.0,
            ducats: item.ducats,
            volume: 0,
            vaulted: item.is_vaulted(),
        });
    }

    ItemDatabase::with_market_items(warframe_items, market_items)
}

fn localized_names(item: &Item) -> HashMap<String, String> {
    item.i18n
        .iter()
        .map(|(language, translation)| (language.clone(), translation.name.clone()))
        .collect()
}

fn item_name(item: &Item, localized_names: &HashMap<String, String>) -> String {
    localized_names
        .get("en")
        .or_else(|| localized_names.values().next())
        .cloned()
        .unwrap_or_else(|| item.name().to_owned())
}

fn default_cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wf-info")
}
