use crate::collector::Rate;
use crate::generator::generate_table;
use crate::sources::{Currency, Source};
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
    #[command(description = "USD/AMD")]
    USD,
    #[command(description = "EUR/AMD")]
    EUR,
    #[command(description = "RUB/AMD")]
    RUB,
    #[command(description = "USD/RUB")]
    UsdRub,
    #[command(description = "EUR/RUB")]
    EurRub,
    #[command(description = "EUR/USD")]
    EurUsd,
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
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Start => {
            bot.send_message(msg.chat.id, "You are welcome!").await?;
        }
        Command::USD => exchange_repl(Currency::usd(), Currency::base(), bot, msg, db).await?,
        Command::EUR => exchange_repl(Currency::eur(), Currency::base(), bot, msg, db).await?,
        Command::RUB => exchange_repl(Currency::rub(), Currency::base(), bot, msg, db).await?,
        Command::UsdRub => exchange_repl(Currency::usd(), Currency::rub(), bot, msg, db).await?,
        Command::EurRub => exchange_repl(Currency::eur(), Currency::rub(), bot, msg, db).await?,
        Command::EurUsd => exchange_repl(Currency::eur(), Currency::usd(), bot, msg, db).await?,
    }
    Ok(())
}

async fn exchange_repl(
    mut from: Currency,
    mut to: Currency,
    bot: Bot,
    msg: Message,
    db: Arc<Storage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut rates = HashMap::new();
    {
        let data = db.data.lock().await;
        rates.clone_from(&data.map);
    }
    for idx in 0..2 {
        let s = generate_table(to.clone(), from.clone(), &rates, idx % 2 == 0);
        bot.send_message(msg.chat.id, html::code_block(&s)).await?;
        std::mem::swap(&mut from, &mut to);
    }
    Ok(())
}
