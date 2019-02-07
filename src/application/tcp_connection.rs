use std::net::TcpStream;

pub struct TcpConnection(pub TcpStream);
impl Clone for TcpConnection {
    fn clone(&self) -> Self {
        TcpConnection(self.0.try_clone().expect(""))
    }
}