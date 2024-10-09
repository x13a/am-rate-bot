use crate::generator::{generate_from_to_table, generate_src_table};
use crate::sources::{Currency, Rate, RateType, Source};
use crate::{Opts, DUNNO};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::SystemTime;
use strum::IntoEnumIterator;
use teloxide::adaptors::{
    throttle::{Limits, Throttle},
    DefaultParseMode,
};
use teloxide::types::{InputFile, ParseMode};
use teloxide::update_listeners::webhooks;
use teloxide::{prelude::*, requests::RequesterExt, utils::command::BotCommands, utils::html};
use tokio::sync::Mutex;

type Bot = DefaultParseMode<Throttle<teloxide::Bot>>;
const ENV_POLLING: &str = "POLLING";
const ENV_HOST: &str = "HOST";
const ENV_PORT: &str = "PORT";
const ENV_CERT: &str = "CERT";
const WELCOME_MSG: &str = "Meow!";

#[derive(Debug)]
pub struct Storage {
    data: Mutex<Data>,
    cache: Mutex<CacheData>,
}

#[derive(Debug)]
pub struct Data {
    rates: HashMap<Source, Vec<Rate>>,
    updated_at: SystemTime,
}

impl Data {
    fn get_rates(&self) -> HashMap<Source, Vec<Rate>> {
        self.rates.clone()
    }

    fn set_rates(&mut self, value: &HashMap<Source, Vec<Rate>>) {
        self.rates.clone_from(value);
        self.updated_at = SystemTime::now();
    }

    fn get_updated_at(&self) -> SystemTime {
        self.updated_at
    }
}

#[derive(Debug)]
pub struct CacheData {
    from_to: HashMap<String, String>,
    src: HashMap<String, String>,
}

impl CacheData {
    const KEY_SEP: &'static str = "_";

    fn clear(&mut self) {
        self.from_to.clear();
        self.src.clear();
    }

    fn add_from_to(
        &mut self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
        value: String,
    ) {
        self.from_to
            .insert(self.format_from_to_key(from, to, rate_type, is_inv), value);
    }

    fn add_src(&mut self, src: Source, rate_type: RateType, value: String) {
        self.src.insert(self.format_src_key(src, rate_type), value);
    }

    fn format_src_key(&self, src: Source, rate_type: RateType) -> String {
        [
            src.to_string().to_lowercase(),
            (rate_type as u8).to_string(),
        ]
        .join(Self::KEY_SEP)
    }

    fn get_from_to(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
    ) -> Option<String> {
        self.from_to
            .get(&self.format_from_to_key(from, to, rate_type, is_inv))
            .cloned()
    }

    fn get_src(&self, src: Source, rate_type: RateType) -> Option<String> {
        self.src.get(&self.format_src_key(src, rate_type)).cloned()
    }

    fn format_from_to_key(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
    ) -> String {
        [
            from.to_string().to_lowercase(),
            to.to_string().to_uppercase(),
            (rate_type as u8).to_string(),
            (is_inv as i32).to_string(),
        ]
        .join(Self::KEY_SEP)
    }
}

impl Storage {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            data: Mutex::new(Data {
                rates: HashMap::new(),
                updated_at: SystemTime::now(),
            }),
            cache: Mutex::new(CacheData {
                from_to: HashMap::new(),
                src: HashMap::new(),
            }),
        })
    }

    pub async fn get_rates(&self) -> HashMap<Source, Vec<Rate>> {
        let data = self.data.lock().await;
        data.get_rates()
    }

    pub async fn set_rates(&self, value: &HashMap<Source, Vec<Rate>>) {
        let mut data = self.data.lock().await;
        data.set_rates(value);
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    pub async fn get_cache_src(&self, src: Source, rate_type: RateType) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get_src(src, rate_type)
    }

    pub async fn get_cache_from_to(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
    ) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get_from_to(from, to, rate_type, is_inv)
    }

    pub async fn set_cache_src(&self, src: Source, rate_type: RateType, value: String) {
        let mut cache = self.cache.lock().await;
        cache.add_src(src, rate_type, value);
    }

    pub async fn set_cache_from_to(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
        value: String,
    ) {
        let mut cache = self.cache.lock().await;
        cache.add_from_to(from, to, rate_type, is_inv, value);
    }

    pub async fn get_updated_at(&self) -> SystemTime {
        let data = self.data.lock().await;
        data.get_updated_at()
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "AMD/USD (֏ - $)")]
    Usd,
    #[command(description = "AMD/EUR (֏ - €)")]
    Eur,
    #[command(description = "RUB/AMD (₽ - ֏)")]
    Rub,
    #[command(description = "RUB/USD (₽ - $)")]
    RubUsd,
    #[command(description = "RUB/EUR (₽ - €)")]
    RubEur,
    #[command(description = "USD/EUR ($ - €)")]
    UsdEur,
    #[command(description = "<FROM> <TO>", parse_with = "split")]
    FromTo { from: String, to: String },
    #[command(description = "<FROM> <TO> inverted", parse_with = "split")]
    FromToInv { from: String, to: String },
    #[command(description = "<SOURCE>")]
    Get { src: Source },
    #[command(description = "AMD/USD cash (֏ - $)")]
    UsdCash,
    #[command(description = "AMD/EUR cash (֏ - €)")]
    EurCash,
    #[command(description = "RUB/AMD cash (₽ - ֏)")]
    RubCash,
    #[command(description = "RUB/USD cash (₽ - $)")]
    RubUsdCash,
    #[command(description = "RUB/EUR cash (₽ - €)")]
    RubEurCash,
    #[command(description = "USD/EUR cash ($ - €)")]
    UsdEurCash,
    #[command(description = "<FROM> <TO> cash", parse_with = "split")]
    FromToCash { from: String, to: String },
    #[command(description = "<FROM> <TO> cash inverted", parse_with = "split")]
    FromToCashInv { from: String, to: String },
    #[command(description = "<SOURCE> cash")]
    GetCash { src: Source },
    #[command(description = "list sources")]
    List,
    #[command(description = "bot status")]
    Status,
    #[command(description = "help")]
    Help,
    #[command(description = "welcome")]
    Start,
}

pub async fn run(db: Arc<Storage>, opts: Opts) {
    let bot = teloxide::Bot::from_env()
        .throttle(Limits::default())
        .parse_mode(ParseMode::Html);
    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("panic");
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(command),
    );
    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![db, opts])
        .enable_ctrlc_handler()
        .default_handler(|_| async move {})
        .build();
    let is_polling = env::var(ENV_POLLING)
        .expect("panic")
        .parse()
        .expect("panic");
    if is_polling {
        dispatcher.dispatch().await;
    } else {
        let host = env::var(ENV_HOST).expect("panic");
        let port = env::var(ENV_PORT).expect("panic").parse().expect("panic");
        let cert = env::var(ENV_CERT).expect("panic");
        let url = format!("https://{host}/am-rate-bot/webhook/")
            .parse()
            .expect("panic");
        let listener = webhooks::axum(
            bot.clone(),
            webhooks::Options {
                address: ([0, 0, 0, 0], port).into(),
                url,
                path: "/".into(),
                certificate: Some(InputFile::file(cert)),
                max_connections: None,
                drop_pending_updates: false,
                secret_token: None,
            },
        )
        .await
        .expect("panic");
        let error_handler =
            LoggingErrorHandler::with_custom_text("An error from the update listener");
        dispatcher
            .dispatch_with_listener(listener, error_handler)
            .await;
    }
}

async fn command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: Arc<Storage>,
    opts: Opts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        Command::Help => {
            bot.send_message(
                msg.chat.id,
                html::escape(&Command::descriptions().to_string()),
            )
            .await?;
        }
        Command::Start => {
            bot.send_message(msg.chat.id, WELCOME_MSG).await?;
        }
        Command::Usd | Command::UsdCash => {
            from_to_repl(
                Currency::default(),
                Currency::usd(),
                match cmd {
                    Command::UsdCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                0,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::Eur | Command::EurCash => {
            from_to_repl(
                Currency::default(),
                Currency::eur(),
                match cmd {
                    Command::EurCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                0,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::Rub | Command::RubCash => {
            from_to_repl(
                Currency::rub(),
                Currency::default(),
                match cmd {
                    Command::RubCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                1,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::RubUsd | Command::RubUsdCash => {
            from_to_repl(
                Currency::rub(),
                Currency::usd(),
                match cmd {
                    Command::RubUsdCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                0,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::RubEur | Command::RubEurCash => {
            from_to_repl(
                Currency::rub(),
                Currency::eur(),
                match cmd {
                    Command::RubEurCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                0,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::UsdEur | Command::UsdEurCash => {
            from_to_repl(
                Currency::usd(),
                Currency::eur(),
                match cmd {
                    Command::UsdEurCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                0,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::FromTo { ref from, ref to }
        | Command::FromToInv { ref from, ref to }
        | Command::FromToCash { ref from, ref to }
        | Command::FromToCashInv { ref from, ref to } => {
            from_to_repl(
                Currency::new(from),
                Currency::new(to),
                match cmd {
                    Command::FromToCash { .. } | Command::FromToCashInv { .. } => RateType::Cash,
                    _ => RateType::NoCash,
                },
                match cmd {
                    Command::FromToInv { .. } | Command::FromToCashInv { .. } => 1,
                    _ => 0,
                },
                bot,
                msg,
                db,
            )
            .await?;
        }
        Command::Get { src } | Command::GetCash { src } => {
            src_repl(
                src,
                match cmd {
                    Command::GetCash { .. } => RateType::Cash,
                    _ => RateType::NoCash,
                },
                bot,
                msg,
                db,
            )
            .await?;
        }
        Command::List => {
            let mut srcs = Source::iter()
                .map(|v| {
                    let mut s = v.to_string().to_lowercase();
                    for c in ["'", " "] {
                        s = s.replace(c, "");
                    }
                    s
                })
                .collect::<Vec<_>>();
            srcs.sort();
            bot.send_message(msg.chat.id, srcs.join(", ")).await?;
        }
        Command::Status => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            let updated_at = db.get_updated_at().await;
            let update_interval = opts.update_interval;
            let lines = [
                format!("version: {VERSION}"),
                format!("update_interval: {update_interval}"),
                format!(
                    "updated_at: {}",
                    DateTime::<Utc>::from(updated_at).format("%F %T %Z"),
                ),
            ];
            bot.send_message(msg.chat.id, lines.join("\n")).await?;
        }
    }
    Ok(())
}

async fn src_repl(
    src: Source,
    rate_type: RateType,
    bot: Bot,
    msg: Message,
    db: Arc<Storage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cached = db.get_cache_src(src, rate_type).await;
    let s = match cached {
        Some(s) => s,
        None => {
            log::debug!("empty cache src");
            let rates = db.get_rates().await;
            let s = generate_src_table(src, &rates, rate_type);
            db.set_cache_src(src, rate_type, s.clone()).await;
            s
        }
    };
    bot.send_message(msg.chat.id, html::code_inline(&s)).await?;
    Ok(())
}

async fn from_to_repl(
    mut from: Currency,
    mut to: Currency,
    rate_type: RateType,
    inv: i32,
    bot: Bot,
    msg: Message,
    db: Arc<Storage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if from == to || from.is_empty() || to.is_empty() {
        bot.send_message(msg.chat.id, DUNNO).await?;
        return Ok(());
    }
    let rates = db.get_rates().await;
    for idx in 0..2 {
        let is_inv = idx % 2 == inv;
        let cached = db
            .get_cache_from_to(from.clone(), to.clone(), rate_type, is_inv)
            .await;
        let s = match cached {
            Some(s) => s,
            None => {
                log::debug!("empty cache from to");
                let s = generate_from_to_table(from.clone(), to.clone(), &rates, rate_type, is_inv);
                db.set_cache_from_to(from.clone(), to.clone(), rate_type, is_inv, s.clone())
                    .await;
                s
            }
        };
        bot.send_message(msg.chat.id, html::code_block(&s)).await?;
        std::mem::swap(&mut from, &mut to);
    }
    Ok(())
}
