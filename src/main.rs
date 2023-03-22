// Will create an exporter with a single metric that will randomize the value
// of the metric everytime the exporter is called.

use anyhow::Context;
use env_logger::{
    Builder,
    Env,
};
use log::{info, error};
use prometheus_exporter::prometheus::register_counter_vec;
use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    
    #[arg(short, long)]
    url: String,

    /// Name of the metric to export
    #[arg(short, long)]
    output: Vec<String>,
}

fn fetch_metric(iotawatt: &str, metric: &str) -> anyhow::Result<f64> {
    let resp = reqwest::blocking::get(format!("{}/query?select=[{}.wh]&begin=d&end=s&group=d", iotawatt, metric))?
        .json::<Vec<Vec<f64>>>()
        .context("http request failed")?;

    let elem = resp.first().context("invalid json response")?;
    let value = elem.first().context("invalid json response")?;
    Ok(*value)
}

fn main() {
    // Setup logger with default level info so we can see the messages from
    // prometheus_exporter.
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Parse address used to bind exporter to.
    let addr_raw = "0.0.0.0:9185";
    let addr: SocketAddr = addr_raw.parse().expect("can not parse listen addr");

    // Start exporter
    let exporter = prometheus_exporter::start(addr).expect("can not start exporter");

    // Create metric
    let output_watthours_total = register_counter_vec!(
        "iotawatt_output_watthours_total", "total watt-hours for the given output on the iotawatt",
        &["output"]
    ).expect("can not create counter iotawatt_output_watthours_total");

    loop {
        // Will block until a new request comes in.
        let _guard = exporter.wait_request();
        
        for output in &cli.output {
            info!("updating metric {}", output);
            let data = match fetch_metric(&cli.url, &output).context("failed to fetch metrics") {
                Ok(data) => data,
                Err(e) => {
                    error!("{}", e);
                    let _ = output_watthours_total.remove_label_values(&[&output]);
                    continue
                }
            };

            let metric = output_watthours_total.with_label_values(&[&output]);
            info!("metric data: {}", data);
            metric.reset();
            metric.inc_by(data);
        }
    }
}
