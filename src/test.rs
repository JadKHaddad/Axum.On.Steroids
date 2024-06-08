use crate::server::ServerConfig;

#[tokio::test]
async fn example_config_is_valid() {
    ServerConfig::from_config_file("config.example.yaml")
        .await
        .expect("Example config is not parsable");
}
