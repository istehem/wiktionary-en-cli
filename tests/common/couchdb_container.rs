use anyhow::Result;
use rustainers::images::GenericImage;
use rustainers::runner::{RunOption, Runner};
use rustainers::Container;
use rustainers::{ImageName, WaitStrategy};
use tokio::time::{sleep, Duration};

pub const COUCH_DB_PORT: u16 = 5984;

pub type CouchDBContainer = Container<GenericImage>;

pub async fn start_couchdb() -> Result<CouchDBContainer> {
    let name = ImageName::new_with_tag("docker.io/couchdb", "3.5.2");
    let mut image = GenericImage::new(name);
    image.add_env_var("COUCHDB_PASSWORD", env!("COUCH_DB_PASSWORD"));
    image.add_env_var("COUCHDB_USER", env!("COUCH_DB_USER"));
    image.add_port_mapping(COUCH_DB_PORT);
    image.set_wait_strategy(WaitStrategy::HttpSuccess {
        path: "/_up".to_string(),
        container_port: COUCH_DB_PORT.into(),
        https: false,
        require_valid_certs: false,
    });

    let run_option = RunOption::builder().with_remove(true).build();
    let runner = Runner::podman().unwrap();
    let container = runner.start_with_options(image, run_option).await?;

    // couchdb /up endpoint returns ok before users are initialized; this may cause 401.
    sleep(Duration::from_millis(2000)).await;
    Ok(container)
}
