use tokio::sync::watch::Sender;
use tokio::time::{sleep, Duration};
use crate::config::CONFIG;
use tracing::error;

pub async fn cpu_usage(tx: Sender<f32>){
    let mut sys = sysinfo::System::new();
    sys.refresh_cpu_usage();
    sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
    loop {
        sys.refresh_cpu_usage();
        let cpu = sys.global_cpu_usage();
        match tx.send(cpu) {
            Ok(()) => {},
            Err(e) => error!(error = ?e)
        }
        sleep(Duration::from_secs(CONFIG.system.cpu_usage_update_interval)).await;
    }

}