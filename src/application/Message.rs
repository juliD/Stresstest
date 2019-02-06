#[derive(Clone)]
pub enum Message {
    // MasterActor
    Log,
    ReportRequests(u64),
    Help,
    IncomingTcpMessage(String),
    // MasterActor + WorkerActor
    Start,
    Stop,
    SetTarget(String),
    // TcpActor
    StartListenTcp,
    // InputActor
    StartWatchInput,
}