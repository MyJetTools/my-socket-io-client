use std::sync::Arc;

pub trait SocketIoSubscribeEventModel {
    const NAME_SPACE: &'static str;
    const EVENT_NAME: &'static str;
    fn deserialize(payload: &str) -> Self;
}

pub trait SocketIoSubscribeOutModel {
    fn serialize(&self) -> String;
}

impl SocketIoSubscribeOutModel for () {
    fn serialize(&self) -> String {
        String::new()
    }
}

#[async_trait::async_trait]
pub trait SocketIoEventSubscriberCallback<
    TInModel: SocketIoSubscribeEventModel + Send + Sync + 'static,
    TOutModel: SocketIoSubscribeOutModel + Send + Sync + 'static,
>
{
    async fn on_event(&self, event_payload: TInModel) -> TOutModel;
}

#[async_trait::async_trait]
pub trait SocketIoEventSubscriberNotification {
    async fn on_event(&self, event_payload: &str) -> String;
}

pub struct SocketIoEventSubscriber<
    TModel: SocketIoSubscribeEventModel + Send + Sync + 'static,
    TOutModel: SocketIoSubscribeOutModel + Send + Sync + 'static,
> {
    pub callbacks:
        Arc<dyn SocketIoEventSubscriberCallback<TModel, TOutModel> + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl<
        TModel: SocketIoSubscribeEventModel + Send + Sync + 'static,
        TOutModel: SocketIoSubscribeOutModel + Send + Sync + 'static,
    > SocketIoEventSubscriberNotification for SocketIoEventSubscriber<TModel, TOutModel>
{
    async fn on_event(&self, event_payload: &str) -> String {
        let event_model: TModel = TModel::deserialize(event_payload);
        let response = self.callbacks.on_event(event_model).await;

        response.serialize()
    }
}
