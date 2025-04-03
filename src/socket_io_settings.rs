#[async_trait::async_trait]
pub trait SocketIoClientSettings {
    async fn get_server_url(&self, client_name: &str) -> String;
    async fn get_handshake_path(&self, client_name: &str) -> String;
    async fn get_headers(&self, client_name: &str) -> Vec<(String, String)>;
    async fn get_query_params(&self, client_name: &str) -> Vec<(String, String)>;
}
