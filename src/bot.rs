use crate::{
    generator::{generate_conv_table, generate_src_table},
    sources::{Currency, Rate, RateType, Source},
    Opts, DUNNO,
};
use chrono::{DateTime, Utc};
use std::{collections::HashMap, env, str::FromStr, sync::Arc, time::SystemTime};
use strum::IntoEnumIterator;
use teloxide::{
    adaptors::{
        throttle::{Limits, Throttle},
        DefaultParseMode,
    },
    prelude::*,
    requests::RequesterExt,
    types::{InputFile, ParseMode},
    update_listeners::webhooks,
    utils::{
        command::{BotCommands, ParseError},
        html,
    },
};
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
    cache: Mutex<Cache>,
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
pub struct Cache {
    conv: HashMap<String, String>,
    src: HashMap<String, String>,
}

impl Cache {
    const KEY_SEP: &'static str = "_";

    fn clear(&mut self) {
        self.conv.clear();
        self.src.clear();
    }

    fn add_conv(
        &mut self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
        value: String,
    ) {
        self.conv
            .insert(self.format_conv_key(from, to, rate_type, is_inv), value);
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

    fn get_conv(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
    ) -> Option<String> {
        self.conv
            .get(&self.format_conv_key(from, to, rate_type, is_inv))
            .cloned()
    }

    fn get_src(&self, src: Source, rate_type: RateType) -> Option<String> {
        self.src.get(&self.format_src_key(src, rate_type)).cloned()
    }

    fn format_conv_key(
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
            cache: Mutex::new(Cache {
                conv: HashMap::new(),
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

    pub async fn get_cache_conv(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
    ) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get_conv(from, to, rate_type, is_inv)
    }

    pub async fn set_cache_src(&self, src: Source, rate_type: RateType, value: String) {
        let mut cache = self.cache.lock().await;
        cache.add_src(src, rate_type, value);
    }

    pub async fn set_cache_conv(
        &self,
        from: Currency,
        to: Currency,
        rate_type: RateType,
        is_inv: bool,
        value: String,
    ) {
        let mut cache = self.cache.lock().await;
        cache.add_conv(from, to, rate_type, is_inv, value);
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
    #[command(description = "USD ($)")]
    Usd,
    #[command(description = "EUR (€)")]
    Eur,
    #[command(description = "RUB (₽)")]
    Rub,
    #[command(description = "GEL (₾)")]
    Gel,
    #[command(description = "RUB/USD (₽ - $)")]
    RubUsd,
    #[command(description = "RUB/EUR (₽ - €)")]
    RubEur,
    #[command(description = "USD/EUR ($ - €)")]
    UsdEur,
    #[command(description = "<FROM> <TO>?", parse_with = parse_conv)]
    Conv { from: Currency, to: Currency },
    #[command(description = "<SOURCE>")]
    Get { src: Source },
    #[command(description = "USD cash ($)")]
    UsdCash,
    #[command(description = "EUR cash (€)")]
    EurCash,
    #[command(description = "RUB cash (₽)")]
    RubCash,
    #[command(description = "GEL cash (₾)")]
    GelCash,
    #[command(description = "RUB/USD cash (₽ - $)")]
    RubUsdCash,
    #[command(description = "RUB/EUR cash (₽ - €)")]
    RubEurCash,
    #[command(description = "USD/EUR cash ($ - €)")]
    UsdEurCash,
    #[command(description = "<FROM> <TO>? cash", parse_with = parse_conv)]
    ConvCash { from: Currency, to: Currency },
    #[command(description = "<SOURCE> cash")]
    GetCash { src: Source },
    #[command(description = "list sources", aliases = ["ls"])]
    List,
    #[command(description = "bot info")]
    Info,
    #[command(description = "help", aliases = ["h", "?"])]
    Help,
    #[command(description = "welcome", hide)]
    Start(String),
}

pub async fn run(db: Arc<Storage>, opts: Opts) -> anyhow::Result<()> {
    let bot = teloxide::Bot::from_env()
        .throttle(Limits::default())
        .parse_mode(ParseMode::Html);
    bot.set_my_commands(Command::bot_commands()).await?;
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
    let is_polling = env::var(ENV_POLLING)?.parse()?;
    if is_polling {
        dispatcher.dispatch().await;
    } else {
        let host = env::var(ENV_HOST)?;
        let port = env::var(ENV_PORT)?.parse()?;
        let cert = env::var(ENV_CERT)?;
        let url = format!("https://{host}/am-rate-bot/webhook/").parse()?;
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
        .await?;
        let error_handler =
            LoggingErrorHandler::with_custom_text("An error from the update listener");
        dispatcher
            .dispatch_with_listener(listener, error_handler)
            .await;
    }
    Ok(())
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
        Command::Start(s) => {
            start_repl(s, bot, msg, db).await?;
        }
        Command::Usd | Command::UsdCash => {
            conv_repl(
                Currency::default(),
                Currency::usd(),
                match cmd {
                    Command::UsdCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                false,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::Eur | Command::EurCash => {
            conv_repl(
                Currency::default(),
                Currency::eur(),
                match cmd {
                    Command::EurCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                false,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::Rub | Command::RubCash => {
            conv_repl(
                Currency::rub(),
                Currency::default(),
                match cmd {
                    Command::RubCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                true,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::Gel | Command::GelCash => {
            conv_repl(
                Currency::default(),
                Currency::new("GEL"),
                match cmd {
                    Command::GelCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                false,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::RubUsd | Command::RubUsdCash => {
            conv_repl(
                Currency::rub(),
                Currency::usd(),
                match cmd {
                    Command::RubUsdCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                false,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::RubEur | Command::RubEurCash => {
            conv_repl(
                Currency::rub(),
                Currency::eur(),
                match cmd {
                    Command::RubEurCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                false,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::UsdEur | Command::UsdEurCash => {
            conv_repl(
                Currency::usd(),
                Currency::eur(),
                match cmd {
                    Command::UsdEurCash => RateType::Cash,
                    _ => RateType::NoCash,
                },
                false,
                bot,
                msg,
                db,
            )
            .await?
        }
        Command::Conv { ref from, ref to } | Command::ConvCash { ref from, ref to } => {
            conv_repl(
                from.clone(),
                to.clone(),
                match cmd {
                    Command::ConvCash { .. } => RateType::Cash,
                    _ => RateType::NoCash,
                },
                *to == Currency::default(),
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
            ls_repl(bot, msg).await?;
        }
        Command::Info => {
            info_repl(bot, msg, db, opts).await?;
        }
    }
    Ok(())
}

fn parse_conv(s: String) -> Result<(Currency, Currency), ParseError> {
    if let Some((from, to)) = s.split_once('/') {
        return Ok((Currency::new(from), Currency::new(to)));
    }
    let mut ws = s.split_whitespace();
    if let (Some(from), Some(to)) = (ws.next(), ws.next()) {
        return Ok((Currency::new(from), Currency::new(to)));
    }
    Ok((Currency::default(), Currency::new(s)))
}

async fn start_repl(
    value: String,
    bot: Bot,
    msg: Message,
    db: Arc<Storage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if value.is_empty() {
        bot.send_message(msg.chat.id, WELCOME_MSG).await?;
        return Ok(());
    }
    let mut value = value.clone();
    let mut rate_type = RateType::NoCash;
    if let Some((main, param)) = value.split_once(':') {
        if let Ok(v) = RateType::from_str(param.trim()) {
            rate_type = v;
        }
        value = main.into();
    }
    if let Ok(src) = Source::from_str(value.trim()) {
        src_repl(src, rate_type, bot, msg, db).await?;
        return Ok(());
    }
    let (from, to) = parse_conv(value).unwrap();
    conv_repl(
        from.clone(),
        to.clone(),
        rate_type,
        to == Currency::default(),
        bot,
        msg,
        db,
    )
    .await?;
    Ok(())
}

async fn ls_repl(bot: Bot, msg: Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut srcs = Source::iter()
        .map(|v| v.to_string().to_lowercase())
        .collect::<Vec<_>>();
    srcs.sort();
    bot.send_message(msg.chat.id, srcs.join(", ")).await?;
    Ok(())
}

async fn info_repl(
    bot: Bot,
    msg: Message,
    db: Arc<Storage>,
    opts: Opts,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

async fn conv_repl(
    mut from: Currency,
    mut to: Currency,
    rate_type: RateType,
    inv: bool,
    bot: Bot,
    msg: Message,
    db: Arc<Storage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if from.is_empty() || to.is_empty() {
        return dunno_repl(bot, msg).await;
    }
    let rates = db.get_rates().await;
    for idx in 0..2 {
        let is_inv = idx % 2 == inv as usize;
        let cached = db
            .get_cache_conv(from.clone(), to.clone(), rate_type, is_inv)
            .await;
        let s = match cached {
            Some(s) => s,
            None => {
                log::debug!("empty cache from to");
                let s = generate_conv_table(from.clone(), to.clone(), &rates, rate_type, is_inv);
                db.set_cache_conv(from.clone(), to.clone(), rate_type, is_inv, s.clone())
                    .await;
                s
            }
        };
        bot.send_message(msg.chat.id, html::code_block(&s)).await?;
        std::mem::swap(&mut from, &mut to);
    }
    Ok(())
}

async fn dunno_repl(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.send_message(msg.chat.id, DUNNO).await?;
    Ok(())
}
