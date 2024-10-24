use am_rate_bot::{
    bot,
    source::{collect_all, filter_collection},
    Config,
};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cfg = Config::load()?;
    let db = bot::Storage::new();
    let task1 = async {
        let db = db.clone();
        let cfg = cfg.clone();
        collect(db, cfg).await.expect("panic");
    };
    let task2 = async {
        let db = db.clone();
        let cfg = cfg.clone();
        bot::run(db, cfg).await.expect("panic");
    };
    tokio::join!(task1, task2);
    Ok(())
}

async fn collect(db: Arc<bot::Storage>, cfg: Arc<Config>) -> anyhow::Result<()> {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(cfg.bot.reqwest_timeout))
        .build()?;
    let get_rates = || async {
        log::debug!("get rates");
        let results = collect_all(&client, &cfg.src).await;
        let rates = filter_collection(results);
        db.clear_cache().await;
        db.set_rates(&rates).await;
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
