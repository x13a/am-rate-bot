use crate::collector::Rate;
use crate::generator::generate_table;
use crate::sources::{Currency, RateType, Source};
use crate::DUNNO;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use teloxide::adaptors::DefaultParseMode;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands, utils::html};
use tokio::sync::Mutex;

type Bot = DefaultParseMode<teloxide::Bot>;

#[derive(Debug)]
pub struct Storage {
    pub data: Mutex<Data>,
}

#[derive(Debug)]
pub struct Data {
    pub map: HashMap<Source, Vec<Rate>>,
    pub updated_at: SystemTime,
}

impl Storage {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            data: Mutex::new(Data {
                map: HashMap::new(),
                updated_at: SystemTime::now(),
            }),
        })
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "AMD/USD")]
    Usd,
    #[command(description = "AMD/EUR")]
    Eur,
    #[command(description = "RUB/AMD")]
    Rub,
    #[command(description = "RUB/USD")]
    RubUsd,
    #[command(description = "RUB/EUR")]
    RubEur,
    #[command(description = "USD/EUR")]
    UsdEur,
    #[command(description = "<FROM> <TO>", parse_with = "split")]
    FromTo { from: String, to: String },
    #[command(description = "<FROM> <TO> inverted", parse_with = "split")]
    FromToInv { from: String, to: String },
    #[command(description = "AMD/USD cash")]
    UsdCash,
    #[command(description = "AMD/EUR cash")]
    EurCash,
    #[command(description = "RUB/AMD cash")]
    RubCash,
    #[command(description = "RUB/USD cash")]
    RubUsdCash,
    #[command(description = "RUB/EUR cash")]
    RubEurCash,
    #[command(description = "USD/EUR cash")]
    UsdEurCash,
    #[command(description = "<FROM> <TO> cash", parse_with = "split")]
    FromToCash { from: String, to: String },
    #[command(description = "<FROM> <TO> cash inverted", parse_with = "split")]
    FromToCashInv { from: String, to: String },
    #[command(description = "help")]
    Help,
    #[command(description = "welcome")]
    Start,
}

pub async fn run(db: Arc<Storage>) {
    let bot = teloxide::Bot::from_env().parse_mode(ParseMode::Html);
    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("panic");
    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<Command>()
            .endpoint(command),
    );
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db])
        .default_handler(|_| async {})
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: Arc<Storage>,
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
            bot.send_message(msg.chat.id, "Meow!").await?;
        }
        Command::Usd | Command::UsdCash => {
            exchange_repl(
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
            exchange_repl(
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
            exchange_repl(
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
            exchange_repl(
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
            exchange_repl(
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
            exchange_repl(
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
            exchange_repl(
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
    }
    Ok(())
}

async fn exchange_repl(
    mut from: Currency,
    mut to: Currency,
    rate_type: RateType,
    inv: i32,
    bot: Bot,
    msg: Message,
    db: Arc<Storage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if to == from {
        bot.send_message(msg.chat.id, DUNNO).await?;
        return Ok(());
    }
    let mut rates = HashMap::new();
    {
        let data = db.data.lock().await;
        rates.clone_from(&data.map);
    }
    for idx in 0..2 {
        let s = generate_table(from.clone(), to.clone(), &rates, rate_type, idx % 2 == inv);
        bot.send_message(msg.chat.id, html::code_block(&s)).await?;
        std::mem::swap(&mut from, &mut to);
    }
    Ok(())
}
