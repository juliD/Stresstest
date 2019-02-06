#[derive(Clone)]
pub enum Message {
    // MasterActor
    Log,
    ReportRequests(u64),
    Help,
    // MasterActor + WorkerActor
    Start,
    Stop,
    SetTarget(String),
    // TcpActor
    IncomingTcpMessage(String),
    SendTcpMessage(u32, Box<Message>),
    // InputActor
    StartWatchInput,
}