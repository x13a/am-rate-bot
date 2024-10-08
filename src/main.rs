use am_rate_bot::{bot, collector, Opts};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts: Opts = argh::from_env();
    let db = bot::Storage::new();
    let task1 = async {
        let db = db.clone();
        collect(db, opts).await;
    };
    let task2 = async {
        let db = db.clone();
        bot::run(db, opts).await;
    };
    tokio::join!(task1, task2);
    Ok(())
}

async fn collect(db: Arc<bot::Storage>, opts: Opts) {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(opts.timeout))
        .build()
        .expect("panic");

    let get_rates = || async {
        log::debug!("get rates");
        let results = collector::collect_all(&client).await;
        let rates = collector::filter_collection(results);
        db.clear_cache().await;
        db.set_rates(&rates).await;
    };
    loop {
        get_rates().await;
        let sleep = tokio::time::sleep(Duration::from_secs(opts.update_interval));
        tokio::pin!(sleep);
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::debug!("ctrl+c");
                break;
            }
            _ = &mut sleep => {}
        }
    }
}
