use am_rate_bot::bot;
use am_rate_bot::collector;
use argh::FromArgs;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, FromArgs)]
/// options:
struct Opts {
    /// reqwest timeout
    #[argh(option, default = "10")]
    timeout: u64,
    /// rates collect interval
    #[argh(option, default = "3 * 60")]
    collect_interval: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let opts: Opts = argh::from_env();
    let db = bot::Storage::new();
    let task1 = async {
        let db = db.clone();
        collect(db, opts).await;
    };
    let task2 = async {
        let db = db.clone();
        bot::run(db).await;
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
        {
            let mut map = db.map.lock().await;
            map.clear();
            map.clone_from(&rates);
        }
    };
    loop {
        get_rates().await;
        let sleep = tokio::time::sleep(Duration::from_secs(opts.collect_interval));
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
