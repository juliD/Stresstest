extern crate actor_model;

use crate::application::tcp_connection::TcpConnection;
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
    ConnectToMaster(Address<Message>),
    StartListenForTcp(Address<Message>),
    IncomingTcpMessage(String),
    IncomingTcpConnection(TcpConnection),
    SendTcpMessage(Box<Message>),
    // InputActor
    StartWatchInput(Address<Message>),
}