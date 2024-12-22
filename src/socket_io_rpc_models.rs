pub trait SocketIoRpcInModel {
    const NAME_SPACE: &'static str;
    const EVENT_NAME: &'static str;
    fn serialize(&self) -> String;
}

pub trait SocketIoRpcOutModel {
    fn deserialize(payload: &str) -> Self;
}
