use am_rate_bot::{bot, collector, config::Config, database::Database, source::Source};
use std::{sync::Arc, time::Duration};
use strum::EnumCount;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cfg = Config::load()?;
    let db = Database::new();
    let task1 = async {
        let db = db.clone();
        let cfg = cfg.clone();
        collect_loop(db, cfg).await.expect("panic");
    };
    let task2 = async {
        let db = db.clone();
        let cfg = cfg.clone();
        bot::run(db, cfg).await.expect("panic");
    };
    tokio::join!(task1, task2);
    Ok(())
}

async fn collect_loop(db: Arc<Database>, cfg: Arc<Config>) -> anyhow::Result<()> {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(cfg.bot.reqwest_timeout))
        .build()?;
    let get_rates = || async {
        log::debug!("get rates");
        let (tx, mut rx) = mpsc::channel(Source::COUNT);
        let client = client.clone();
        let cfg = cfg.clone();
        {
            let tx = tx.clone();
            tokio::spawn(async move {
                collector::collect(&client, cfg, tx).await;
            });
        }
        drop(tx);
        while let Some((src, rates)) = rx.recv().await {
            db.set_rates(src, rates).await;
        }
        db.clear_cache().await;
    };
    loop {
        get_rates().await;
        let sleep = tokio::time::sleep(Duration::from_secs(cfg.bot.update_interval));
        tokio::pin!(sleep);
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::debug!("ctrl+c");
                break;
            }
            _ = &mut sleep => {}
        }
    }
    Ok(())
}
