use std::{
    fs,
    path::{Path, PathBuf},
};

use info_core::WarframeItem;
use serde::{Deserialize, Serialize};
use wf_market::{
    Authenticated, Client, CreateOrder, Credentials, Item, OrderType, OwnedOrder, OwnedOrderId,
    TopOrderFilters, UpdateOrder,
};

use super::MarketData;

const SESSION_FILE: &str = "wf_market_session.json";

#[derive(Clone)]
pub(crate) struct OrderSession {
    pub(crate) email: String,
    pub(crate) client: Client<Authenticated>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub(crate) const ALL: [Self; 2] = [Self::Sell, Self::Buy];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Buy => "Buy",
            Self::Sell => "Sell",
        }
    }
}

impl From<OrderType> for OrderSide {
    fn from(value: OrderType) -> Self {
        match value {
            OrderType::Buy => Self::Buy,
            OrderType::Sell => Self::Sell,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrderItemOption {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) max_rank: Option<u32>,
    pub(crate) max_charges: Option<u32>,
    pub(crate) max_amber_stars: Option<u32>,
    pub(crate) max_cyan_stars: Option<u32>,
    pub(crate) subtypes: Vec<String>,
}

impl OrderItemOption {
    pub(crate) fn from_scan_item(item: &WarframeItem) -> Option<Self> {
        Some(Self {
            name: item.name.clone(),
            slug: item.market_slug.clone()?,
            max_rank: None,
            max_charges: None,
            max_amber_stars: None,
            max_cyan_stars: None,
            subtypes: Vec::new(),
        })
    }

    fn from_market_item(item: &Item) -> Option<Self> {
        if item.tradable == Some(false) {
            return None;
        }

        let name = super::item_name(item);
        if name.is_empty() {
            return None;
        }

        Some(Self {
            name,
            slug: item.slug.clone(),
            max_rank: item.max_rank,
            max_charges: item.max_charges,
            max_amber_stars: item.max_amber_stars,
            max_cyan_stars: item.max_cyan_stars,
            subtypes: item.subtypes.clone().unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarketOrder {
    pub(crate) id: String,
    pub(crate) side: OrderSide,
    pub(crate) item_name: String,
    pub(crate) item_slug: String,
    pub(crate) platinum: u32,
    pub(crate) quantity: u32,
    pub(crate) visible: bool,
    pub(crate) rank: Option<u8>,
    pub(crate) charges: Option<u8>,
    pub(crate) amber_stars: Option<u8>,
    pub(crate) cyan_stars: Option<u8>,
    pub(crate) subtype: Option<String>,
}

impl MarketOrder {
    pub(crate) fn from_owned(order: OwnedOrder) -> Self {
        let item = order.get_item();
        let item_name = item
            .map(super::item_name)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| order.item_id().to_owned());
        let item_slug = item
            .map(|item| item.slug.clone())
            .unwrap_or_else(|| order.item_id().to_owned());

        Self {
            id: order.id().as_str().to_owned(),
            side: order.order_type().into(),
            item_name,
            item_slug,
            platinum: order.platinum(),
            quantity: order.quantity(),
            visible: order.is_visible(),
            rank: order.order.rank,
            charges: order.order.charges,
            amber_stars: order.order.amber_stars,
            cyan_stars: order.order.cyan_stars,
            subtype: order.order.subtype.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DraftMode {
    Create,
    Edit(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrderDraft {
    pub(crate) mode: DraftMode,
    pub(crate) side: OrderSide,
    pub(crate) item_name: String,
    pub(crate) item_slug: String,
    pub(crate) platinum: String,
    pub(crate) quantity: String,
    pub(crate) visible: bool,
    pub(crate) rank: String,
    pub(crate) charges: String,
    pub(crate) amber_stars: String,
    pub(crate) cyan_stars: String,
    pub(crate) subtype: String,
    pub(crate) capabilities: OrderItemOption,
}

impl OrderDraft {
    pub(crate) fn empty() -> Self {
        let capabilities = OrderItemOption {
            name: String::new(),
            slug: String::new(),
            max_rank: None,
            max_charges: None,
            max_amber_stars: None,
            max_cyan_stars: None,
            subtypes: Vec::new(),
        };

        Self {
            mode: DraftMode::Create,
            side: OrderSide::Sell,
            item_name: String::new(),
            item_slug: String::new(),
            platinum: String::new(),
            quantity: "1".to_owned(),
            visible: true,
            rank: String::new(),
            charges: String::new(),
            amber_stars: String::new(),
            cyan_stars: String::new(),
            subtype: String::new(),
            capabilities,
        }
    }

    pub(crate) fn create(item: OrderItemOption, side: OrderSide, price: Option<u32>) -> Self {
        let rank = item
            .max_rank
            .filter(|rank| *rank <= u8::MAX as u32)
            .map(|rank| rank.to_string())
            .unwrap_or_default();

        Self {
            mode: DraftMode::Create,
            side,
            item_name: item.name.clone(),
            item_slug: item.slug.clone(),
            platinum: price.map(|price| price.to_string()).unwrap_or_default(),
            quantity: "1".to_owned(),
            visible: true,
            rank,
            charges: String::new(),
            amber_stars: String::new(),
            cyan_stars: String::new(),
            subtype: String::new(),
            capabilities: item,
        }
    }

    pub(crate) fn edit(order: &MarketOrder) -> Self {
        let capabilities = OrderItemOption {
            name: order.item_name.clone(),
            slug: order.item_slug.clone(),
            max_rank: order.rank.map(u32::from),
            max_charges: order.charges.map(u32::from),
            max_amber_stars: order.amber_stars.map(u32::from),
            max_cyan_stars: order.cyan_stars.map(u32::from),
            subtypes: order.subtype.clone().into_iter().collect(),
        };

        Self {
            mode: DraftMode::Edit(order.id.clone()),
            side: order.side,
            item_name: order.item_name.clone(),
            item_slug: order.item_slug.clone(),
            platinum: order.platinum.to_string(),
            quantity: order.quantity.to_string(),
            visible: order.visible,
            rank: order
                .rank
                .map(|value| value.to_string())
                .unwrap_or_default(),
            charges: order
                .charges
                .map(|value| value.to_string())
                .unwrap_or_default(),
            amber_stars: order
                .amber_stars
                .map(|value| value.to_string())
                .unwrap_or_default(),
            cyan_stars: order
                .cyan_stars
                .map(|value| value.to_string())
                .unwrap_or_default(),
            subtype: order.subtype.clone().unwrap_or_default(),
            capabilities,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PendingOrderAction {
    Create(OrderDraft),
    Update { order_id: String, draft: OrderDraft },
    Delete { order_id: String },
    Close { order_id: String, quantity: u32 },
}

impl PendingOrderAction {
    pub(crate) fn description(&self) -> String {
        match self {
            Self::Create(draft) => format!(
                "Create {} order for {} at {}p x{}?",
                draft.side.label(),
                draft.item_name,
                draft.platinum,
                draft.quantity
            ),
            Self::Update { draft, .. } => {
                format!(
                    "Update {} to {}p x{}?",
                    draft.item_name, draft.platinum, draft.quantity
                )
            }
            Self::Delete { .. } => "Delete this order permanently?".to_owned(),
            Self::Close { quantity, .. } => {
                format!(
                    "Close {quantity} unit{} from this order?",
                    plural_suffix(*quantity)
                )
            }
        }
    }
}

pub(crate) fn session_path_for_settings(settings_path: &Path) -> PathBuf {
    settings_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(SESSION_FILE)
}

pub(crate) async fn restore_session(session_path: PathBuf) -> Result<Option<OrderSession>, String> {
    let Some(credentials) = read_session_credentials(&session_path)? else {
        return Ok(None);
    };

    match Client::validate_credentials(&credentials).await {
        Ok(true) => {}
        Ok(false) => {
            let _ = fs::remove_file(&session_path);
            return Ok(None);
        }
        Err(error) => return Err(format!("could not validate saved WFM session: {error}")),
    }

    let client = Client::from_credentials(credentials)
        .await
        .map_err(|error| format!("could not restore WFM session: {error}"))?;
    save_session(&session_path, &client.export_session())?;

    Ok(Some(OrderSession {
        email: client.credentials().email.clone(),
        client,
    }))
}

pub(crate) async fn login(
    session_path: PathBuf,
    email: String,
    password: String,
) -> Result<OrderSession, String> {
    let email = email.trim().to_owned();
    let password = password.trim().to_owned();

    if email.is_empty() {
        return Err("email is required".to_owned());
    }

    if password.is_empty() {
        return Err("password is required".to_owned());
    }

    let device_id = read_session_credentials(&session_path)
        .ok()
        .flatten()
        .filter(|credentials| credentials.email == email)
        .map(|credentials| credentials.device_id)
        .unwrap_or_else(Credentials::generate_device_id);

    let credentials = Credentials::new(email, password, device_id);
    let client = Client::from_credentials(credentials)
        .await
        .map_err(|error| format!("WFM login failed: {error}"))?;

    save_session(&session_path, &client.export_session())?;

    Ok(OrderSession {
        email: client.credentials().email.clone(),
        client,
    })
}

pub(crate) fn logout(session_path: &Path) -> Result<(), String> {
    match fs::remove_file(session_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "could not remove WFM session {}: {error}",
            session_path.display()
        )),
    }
}

pub(crate) async fn load_item_options() -> Result<Vec<OrderItemOption>, String> {
    let market = MarketData::load().await?;
    let mut items = if let Some(client) = &market.client {
        client
            .items()
            .iter()
            .filter_map(OrderItemOption::from_market_item)
            .collect::<Vec<_>>()
    } else {
        market
            .database
            .items()
            .iter()
            .filter_map(OrderItemOption::from_scan_item)
            .collect::<Vec<_>>()
    };

    items.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(items)
}

pub(crate) fn search_item_options(
    items: &[OrderItemOption],
    query: &str,
    limit: usize,
) -> Vec<OrderItemOption> {
    let needle = searchable(query);
    if needle.is_empty() || limit == 0 {
        return Vec::new();
    }

    let mut matches = items
        .iter()
        .filter_map(|item| {
            let haystack = searchable(&item.name);
            haystack
                .find(&needle)
                .map(|position| (item, position, item.name.len()))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.0.name.cmp(&right.0.name))
    });

    matches
        .into_iter()
        .take(limit)
        .map(|(item, _, _)| item.clone())
        .collect()
}

pub(crate) async fn create_draft_with_price(
    item: OrderItemOption,
    side: OrderSide,
    fallback_price: Option<u32>,
) -> OrderDraft {
    let item = match enrich_item_option(item.clone()).await {
        Some(enriched) => enriched,
        None => item,
    };
    let draft = OrderDraft::create(item, side, None);

    refresh_draft_price(draft, fallback_price).await
}

pub(crate) async fn refresh_draft_price(
    mut draft: OrderDraft,
    fallback_price: Option<u32>,
) -> OrderDraft {
    let existing_price = parse_positive_u32(&draft.platinum, "price").ok();
    let price = default_price_for_draft(&draft)
        .await
        .ok()
        .flatten()
        .or(fallback_price)
        .or(existing_price)
        .filter(|price| *price > 0);

    draft.platinum = price.map(|price| price.to_string()).unwrap_or_default();
    draft
}

pub(crate) async fn load_orders(client: Client<Authenticated>) -> Result<Vec<MarketOrder>, String> {
    let orders = client
        .my_orders()
        .await
        .map_err(|error| format!("could not load WFM orders: {error}"))?;

    Ok(orders.into_iter().map(MarketOrder::from_owned).collect())
}

pub(crate) async fn commit_action(
    client: Client<Authenticated>,
    action: PendingOrderAction,
) -> Result<Vec<MarketOrder>, String> {
    match action {
        PendingOrderAction::Create(draft) => {
            let request = create_request(&draft)?;
            client
                .create_order(request)
                .await
                .map_err(|error| format!("could not create WFM order: {error}"))?;
        }
        PendingOrderAction::Update { order_id, draft } => {
            let update = update_request(&draft)?;
            if update.is_empty() {
                return Err("no order changes to send".to_owned());
            }

            client
                .update_order(&OwnedOrderId::from_raw(order_id), update)
                .await
                .map_err(|error| format!("could not update WFM order: {error}"))?;
        }
        PendingOrderAction::Delete { order_id } => {
            client
                .delete_order(&OwnedOrderId::from_raw(order_id))
                .await
                .map_err(|error| format!("could not delete WFM order: {error}"))?;
        }
        PendingOrderAction::Close { order_id, quantity } => {
            client
                .close_order(&OwnedOrderId::from_raw(order_id), quantity)
                .await
                .map_err(|error| format!("could not close WFM order: {error}"))?;
        }
    }

    load_orders(client).await
}

pub(crate) fn pending_action_from_draft(draft: &OrderDraft) -> Result<PendingOrderAction, String> {
    validate_draft(draft)?;

    match &draft.mode {
        DraftMode::Create => Ok(PendingOrderAction::Create(draft.clone())),
        DraftMode::Edit(order_id) => Ok(PendingOrderAction::Update {
            order_id: order_id.clone(),
            draft: draft.clone(),
        }),
    }
}

fn create_request(draft: &OrderDraft) -> Result<CreateOrder, String> {
    let platinum = parse_positive_u32(&draft.platinum, "price")?;
    let quantity = parse_positive_u32(&draft.quantity, "quantity")?;

    let mut request = match draft.side {
        OrderSide::Buy => CreateOrder::buy(&draft.item_slug, platinum, quantity),
        OrderSide::Sell => CreateOrder::sell(&draft.item_slug, platinum, quantity),
    };

    if !draft.visible {
        request = request.hidden();
    }

    if let Some(value) = parse_optional_u8(&draft.rank, "rank")? {
        request = request.with_mod_rank(value);
    }

    if let Some(value) = parse_optional_u8(&draft.charges, "charges")? {
        request = request.with_charges(value);
    }

    if let Some(value) = parse_optional_u8(&draft.amber_stars, "amber stars")? {
        let cyan = parse_optional_u8(&draft.cyan_stars, "cyan stars")?.unwrap_or(0);
        request = request.with_sculpture_stars(value, cyan);
    } else if let Some(cyan) = parse_optional_u8(&draft.cyan_stars, "cyan stars")? {
        request = request.with_sculpture_stars(0, cyan);
    }

    if !draft.subtype.trim().is_empty() {
        request = request.with_subtype(draft.subtype.trim());
    }

    Ok(request)
}

fn update_request(draft: &OrderDraft) -> Result<UpdateOrder, String> {
    let mut update = UpdateOrder::new()
        .platinum(parse_positive_u32(&draft.platinum, "price")?)
        .quantity(parse_positive_u32(&draft.quantity, "quantity")?)
        .visible(draft.visible);

    if let Some(value) = parse_optional_u8(&draft.rank, "rank")? {
        update = update.rank(value);
    }

    if let Some(value) = parse_optional_u8(&draft.charges, "charges")? {
        update = update.charges(value);
    }

    if let Some(value) = parse_optional_u8(&draft.amber_stars, "amber stars")? {
        update = update.amber_stars(value);
    }

    if let Some(value) = parse_optional_u8(&draft.cyan_stars, "cyan stars")? {
        update = update.cyan_stars(value);
    }

    if !draft.subtype.trim().is_empty() {
        update = update.subtype(draft.subtype.trim());
    }

    Ok(update)
}

fn validate_draft(draft: &OrderDraft) -> Result<(), String> {
    if draft.item_slug.trim().is_empty() {
        return Err("choose an item before creating an order".to_owned());
    }

    parse_positive_u32(&draft.platinum, "price")?;
    parse_positive_u32(&draft.quantity, "quantity")?;
    parse_optional_u8(&draft.rank, "rank")?;
    parse_optional_u8(&draft.charges, "charges")?;
    parse_optional_u8(&draft.amber_stars, "amber stars")?;
    parse_optional_u8(&draft.cyan_stars, "cyan stars")?;

    Ok(())
}

async fn default_price_for_draft(draft: &OrderDraft) -> Result<Option<u32>, String> {
    let filters = top_order_filters_from_draft(draft)?;
    default_price(&draft.item_slug, draft.side, filters.as_ref()).await
}

async fn enrich_item_option(item: OrderItemOption) -> Option<OrderItemOption> {
    let market = MarketData::load().await.ok()?;
    let client = market.client.as_ref()?;
    client
        .get_item_by_slug(&item.slug)
        .and_then(OrderItemOption::from_market_item)
}

async fn default_price(
    slug: &str,
    side: OrderSide,
    filters: Option<&TopOrderFilters>,
) -> Result<Option<u32>, String> {
    let market = MarketData::load().await?;
    let Some(client) = market.client else {
        return Ok(None);
    };

    let top_orders = client
        .get_top_orders(slug, filters)
        .await
        .map_err(|error| format!("could not fetch WFM prices for {slug}: {error}"))?;

    Ok(match side {
        OrderSide::Buy => top_orders.best_buy_price(),
        OrderSide::Sell => top_orders.best_sell_price(),
    })
}

fn top_order_filters_from_draft(draft: &OrderDraft) -> Result<Option<TopOrderFilters>, String> {
    let mut filters = TopOrderFilters::new();

    if let Some(value) = parse_optional_u8(&draft.rank, "rank")? {
        filters = filters.rank(value);
    }

    if let Some(value) = parse_optional_u8(&draft.charges, "charges")? {
        filters = filters.charges(value);
    }

    if let Some(value) = parse_optional_u8(&draft.amber_stars, "amber stars")? {
        filters = filters.amber_stars(value);
    }

    if let Some(value) = parse_optional_u8(&draft.cyan_stars, "cyan stars")? {
        filters = filters.cyan_stars(value);
    }

    if !draft.subtype.trim().is_empty() {
        filters = filters.subtype(draft.subtype.trim());
    }

    Ok((!filters.is_empty()).then_some(filters))
}

fn read_session_credentials(path: &Path) -> Result<Option<Credentials>, String> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str::<Credentials>(&json)
            .map(Some)
            .map_err(|error| format!("could not parse WFM session {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "could not read WFM session {}: {error}",
            path.display()
        )),
    }
}

fn save_session(path: &Path, credentials: &Credentials) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }

    let json = serde_json::to_string(credentials)
        .map_err(|error| format!("could not serialize WFM session: {error}"))?;

    write_session_file(path, json.as_bytes())
}

#[cfg(unix)]
fn write_session_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    use std::{
        fs::{OpenOptions, Permissions},
        io::Write,
        os::unix::fs::{OpenOptionsExt, PermissionsExt},
    };

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("could not write WFM session {}: {error}", path.display()))?;
    file.write_all(contents)
        .map_err(|error| format!("could not write WFM session {}: {error}", path.display()))?;
    file.set_permissions(Permissions::from_mode(0o600))
        .map_err(|error| {
            format!(
                "could not secure WFM session permissions {}: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn write_session_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    fs::write(path, contents)
        .map_err(|error| format!("could not write WFM session {}: {error}", path.display()))
}

fn parse_positive_u32(value: &str, label: &str) -> Result<u32, String> {
    let value = value.trim();
    let parsed = value
        .parse::<u32>()
        .map_err(|_| format!("{label} must be a whole number greater than 0"))?;

    if parsed == 0 {
        Err(format!("{label} must be greater than 0"))
    } else {
        Ok(parsed)
    }
}

fn parse_optional_u8(value: &str, label: &str) -> Result<Option<u8>, String> {
    if value.trim().is_empty() {
        return Ok(None);
    }

    value
        .trim()
        .parse::<u8>()
        .map(Some)
        .map_err(|_| format!("{label} must be a whole number from 0 to 255"))
}

fn searchable(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

fn plural_suffix(count: u32) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_path_lives_next_to_settings_file() {
        let path = session_path_for_settings(Path::new("/tmp/warframe-info/settings.toml"));

        assert_eq!(
            path,
            PathBuf::from("/tmp/warframe-info/wf_market_session.json")
        );
    }

    #[test]
    fn missing_session_file_reads_as_no_session() {
        let path = std::env::temp_dir().join(format!(
            "warframe-info-missing-session-{}",
            std::process::id()
        ));

        assert!(read_session_credentials(&path).unwrap().is_none());
    }

    #[test]
    fn serialized_login_credentials_do_not_include_password() {
        let credentials = Credentials::new("user@example.com", "secret", "device-id");
        let json = serde_json::to_string(&credentials).unwrap();

        assert!(!json.contains("secret"));
        assert!(!json.contains("password"));
    }

    #[test]
    fn search_item_options_prefers_prefix_and_shorter_names() {
        let items = vec![
            test_item("Ash Prime Systems Blueprint", "ash_prime_systems_blueprint"),
            test_item("Paris Prime String", "paris_prime_string"),
            test_item("Ash Prime Set", "ash_prime_set"),
        ];

        let matches = search_item_options(&items, "ash prime", 10);

        assert_eq!(matches[0].name, "Ash Prime Set");
        assert_eq!(matches[1].name, "Ash Prime Systems Blueprint");
    }

    #[test]
    fn draft_defaults_to_visible_quantity_one_and_side() {
        let item = test_item("Forma Blueprint", "forma_blueprint");

        let draft = OrderDraft::create(item, OrderSide::Buy, Some(10));

        assert!(draft.visible);
        assert_eq!(draft.quantity, "1");
        assert_eq!(draft.side, OrderSide::Buy);
        assert_eq!(draft.platinum, "10");
    }

    #[test]
    fn rankable_draft_defaults_to_max_rank() {
        let mut item = test_item("Serration", "serration");
        item.max_rank = Some(10);

        let draft = OrderDraft::create(item, OrderSide::Sell, Some(12));

        assert_eq!(draft.rank, "10");
    }

    #[test]
    fn top_order_filters_include_rank_from_draft() {
        let mut item = test_item("Serration", "serration");
        item.max_rank = Some(10);
        let draft = OrderDraft::create(item, OrderSide::Sell, Some(12));

        let filters = top_order_filters_from_draft(&draft).unwrap().unwrap();

        assert_eq!(filters.rank, Some(10));
    }

    #[test]
    fn pending_action_requires_item_and_positive_numbers() {
        let mut draft = OrderDraft::empty();
        assert!(pending_action_from_draft(&draft).is_err());

        draft.item_name = "Forma Blueprint".to_owned();
        draft.item_slug = "forma_blueprint".to_owned();
        draft.platinum = "0".to_owned();
        assert!(pending_action_from_draft(&draft).is_err());

        draft.platinum = "8".to_owned();
        assert!(pending_action_from_draft(&draft).is_ok());
    }

    fn test_item(name: &str, slug: &str) -> OrderItemOption {
        OrderItemOption {
            name: name.to_owned(),
            slug: slug.to_owned(),
            max_rank: None,
            max_charges: None,
            max_amber_stars: None,
            max_cyan_stars: None,
            subtypes: Vec::new(),
        }
    }
}
