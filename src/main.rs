use am_rate_bot::{bot, collector, sources::Config, Opts};
use std::{env, fs, sync::Arc, time::Duration};

const ENV_SRC_CONFIG: &str = "SRC_CONFIG";
const ENV_REQWEST_TIMEOUT: &str = "REQWEST_TIMEOUT";
const ENV_UPDATE_INTERVAL: &str = "UPDATE_INTERVAL";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = get_opts()?;
    let db = bot::Storage::new();
    let task1 = async {
        let db = db.clone();
        collect(db, opts).await.expect("panic");
    };
    let task2 = async {
        let db = db.clone();
        bot::run(db, opts).await.expect("panic");
    };
    tokio::join!(task1, task2);
    Ok(())
}

fn get_opts() -> anyhow::Result<Opts> {
    let opts = Opts {
        reqwest_timeout: env::var(ENV_REQWEST_TIMEOUT)?.parse()?,
        update_interval: env::var(ENV_UPDATE_INTERVAL)?.parse()?,
    };
    Ok(opts)
}

fn load_src_config() -> anyhow::Result<Config> {
    let cfg = toml::from_str(fs::read_to_string(env::var(ENV_SRC_CONFIG)?)?.as_str())?;
    Ok(cfg)
}

async fn collect(db: Arc<bot::Storage>, opts: Opts) -> anyhow::Result<()> {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(opts.reqwest_timeout))
        .build()?;
    let cfg = load_src_config()?;
    let get_rates = || async {
        log::debug!("get rates");
        let results = collector::collect_all(&client, &cfg).await;
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
    Ok(())
}
