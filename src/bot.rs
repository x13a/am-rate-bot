use crate::{
    config::Config,
    database::Database,
    generate,
    source::{Currency, RateType, Source},
    DUNNO,
};
use chrono::{DateTime, Utc};
use std::{env, str::FromStr, sync::Arc};
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

type Bot = DefaultParseMode<Throttle<teloxide::Bot>>;
const ENV_BOT_TOKEN: &str = "TELOXIDE_TOKEN";

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
    #[command(description = "<FROM> <TO>?", parse_with = parse_conv)]
    Conv { from: Currency, to: Currency },
    #[command(description = "<FROM> <TO>? cash", parse_with = parse_conv)]
    ConvCash { from: Currency, to: Currency },
    #[command(description = "<SOURCE>")]
    Get { src: Source },
    #[command(description = "<SOURCE> cash")]
    GetCash { src: Source },
    #[command(description = "<SOURCE> card")]
    GetCard { src: Source },
    #[command(description = "list sources", aliases = ["ls"])]
    List,
    #[command(description = "bot info")]
    Info,
    #[command(description = "help", aliases = ["h", "?"], hide)]
    Help,
    #[command(description = "welcome", hide)]
    Start(String),
}

pub async fn run(db: Arc<Database>, cfg: Arc<Config>) -> anyhow::Result<()> {
    let bot = teloxide::Bot::from_env()
        .throttle(Limits::default())
        .parse_mode(ParseMode::Html);
    unsafe {
        env::remove_var(ENV_BOT_TOKEN);
    }
    bot.set_my_commands(Command::bot_commands()).await?;
    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<Command>()
            .endpoint(command),
    );
    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![db, cfg.clone()])
        .enable_ctrlc_handler()
        .default_handler(|_| async move {})
        .build();
    if cfg.bot.polling {
        dispatcher.dispatch().await;
    } else {
        let url = cfg.bot.webhook.url.parse()?;
        let listener = webhooks::axum(
            bot.clone(),
            webhooks::Options {
                address: ([0, 0, 0, 0], cfg.bot.webhook.port).into(),
                url,
                path: "/".into(),
                certificate: Some(InputFile::file(&cfg.bot.webhook.cert)),
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
    db: Arc<Database>,
    cfg: Arc<Config>,
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
            start_repl(s, bot, msg, db, cfg).await?;
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
        Command::Get { src } | Command::GetCash { src } | Command::GetCard { src } => {
            src_repl(
                src,
                match cmd {
                    Command::Get { .. } => RateType::NoCash,
                    Command::GetCash { .. } => RateType::Cash,
                    _ => RateType::Card,
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
            info_repl(bot, msg, db, cfg).await?;
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
    db: Arc<Database>,
    cfg: Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if value.is_empty() {
        bot.send_message(msg.chat.id, html::escape(&cfg.bot.welcome_msg))
            .await?;
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
    db: Arc<Database>,
    cfg: Arc<Config>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let updated_at = db.get_updated_at().await;
    let update_interval = cfg.bot.update_interval;
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
    db: Arc<Database>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cached = db.get_cache_src(src, rate_type).await;
    let s = match cached {
        Some(s) => s,
        None => {
            log::debug!("empty cache src");
            let rates = db.get_rates().await;
            let mut s = generate::src_table(src, &rates, rate_type);
            if !s.is_empty() {
                s = html::code_inline(&s);
                db.set_cache_src(src, rate_type, s.clone()).await;
            } else {
                s = DUNNO.into()
            }
            s
        }
    };
    bot.send_message(msg.chat.id, s).await?;
    Ok(())
}

async fn conv_repl(
    mut from: Currency,
    mut to: Currency,
    rate_type: RateType,
    inv: bool,
    bot: Bot,
    msg: Message,
    db: Arc<Database>,
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
                log::debug!("empty cache conv");
                let mut s =
                    generate::conv_table(from.clone(), to.clone(), &rates, rate_type, is_inv);
                if !s.is_empty() {
                    s = html::code_block(&s);
                    db.set_cache_conv(from.clone(), to.clone(), rate_type, is_inv, s.clone())
                        .await;
                } else {
                    s = DUNNO.into()
                }
                s
            }
        };
        bot.send_message(msg.chat.id, s).await?;
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
