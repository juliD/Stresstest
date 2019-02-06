extern crate actor_model;

use actor_model::address::*;

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
    ConnectToMaster,
    StartListenForTcp(Address<Message>),
    IncomingTcpMessage(String),
    SendTcpMessage(u32, Box<Message>),
    // InputActor
    StartWatchInput(Address<Message>),
}