use testcontainers::{clients::Cli, Container, RunnableImage};

use super::micro::Micro;

pub async fn micro_endpoint(micro_url: &str, page: &str) -> serde_json::Value {
    let resp = reqwest::get(micro_url.to_string() + "/micro/" + page)
        .await
        .unwrap();
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap()
}

pub fn setup(docker: &Cli) -> (Container<Micro>, String) {
    let micro_image = Micro::default();
    // We cannot call `$(pwd)` as usual in a path for a docker volume, so we need to get the current working directory
    let pwd = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        + "/tests/common/micro_config";

    let micro = RunnableImage::from(micro_image).with_volume((pwd, "/config"));
    let container = docker.run(micro);
    let host_port = container.get_host_port_ipv4(9090);
    let micro_url = format!("http://0.0.0.0:{host_port}");

    (container, micro_url)
}

pub async fn wait_for_events(micro_url: &str, page: &str, number: usize) {
    loop {
        let response = micro_endpoint(micro_url, page).await;
        match response.as_array() {
            Some(events) => {
                if events.len() >= number {
                    break;
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            _ => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
        }
    }
}
