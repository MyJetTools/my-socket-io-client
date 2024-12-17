#[async_trait::async_trait]
pub trait SocketIoNamespaceCallbacks {
    fn get_namespace(&self) -> &'static str;
    async fn on_connect(&self);
    async fn on_payload(&self, payload: String);
}
