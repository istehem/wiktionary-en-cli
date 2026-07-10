#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::rstest;
    use rustainers::images::GenericImage;
    use rustainers::runner::Runner;
    use rustainers::{HealthCheck, ImageName, WaitStrategy};

    async fn start_couchdb() -> Result<()> {
        let name = ImageName::new_with_tag("docker.io/couchdb", "3.5.2");
        let container_port = 6984;

        let mut image = GenericImage::new(name);
        image.add_env_var("COUCHDB_PASSWORD", env!("COUCH_DB_PASSWORD"));
        image.add_env_var("COUCHDB_USER", env!("COUCH_DB_USER"));
        image.add_port_mapping(container_port);
        let health_check = HealthCheck::builder()
            //.with_command(format!("curl --fail http://localhost:{}/", container_port))
            .with_command(format!("bash -c 'echo > /dev/tcp/127.0.0.1/{}'", 5984))
            .build();
        image.set_wait_strategy(WaitStrategy::custom_health_check(health_check));

        let runner = Runner::auto()?;
        let container = runner.start(image).await?;
        let _port = container.host_port(container_port).await?;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn start_containers() -> Result<()> {
        start_couchdb().await?;
        Ok(())
    }
}
