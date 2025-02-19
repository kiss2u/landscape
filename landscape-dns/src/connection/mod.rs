use std::net::SocketAddr;
use std::os::fd::RawFd;
use std::os::unix::io::AsRawFd;
use std::{future::Future, io, pin::Pin};

use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_client::proto::TokioTime;
use hickory_resolver::{
    name_server::{GenericConnector, RuntimeProvider},
    TokioHandle,
};

use libc::{setsockopt, SOL_SOCKET, SO_MARK};
use tokio::net::TcpStream as TokioTcpStream;
use tokio::net::UdpSocket as TokioUdpSocket;

pub type MarkConnectionProvider = GenericConnector<MarkRuntimeProvider>;

/// The Tokio Runtime for async execution
#[derive(Clone)]
pub struct MarkRuntimeProvider {
    handler: TokioHandle,
    mark_value: u32,
}

impl MarkRuntimeProvider {
    /// Create a Tokio runtime with a specific mark value
    pub fn new(mark_value: u32) -> Self {
        MarkRuntimeProvider { handler: TokioHandle::default(), mark_value }
    }
}

impl RuntimeProvider for MarkRuntimeProvider {
    type Handle = TokioHandle;
    type Timer = TokioTime;
    type Udp = TokioUdpSocket;
    type Tcp = AsyncIoTokioAsStd<TokioTcpStream>;

    fn create_handle(&self) -> Self::Handle {
        self.handler.clone()
    }

    fn connect_tcp(
        &self,
        server_addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Tcp>>>> {
        let mark_value = self.mark_value;
        Box::pin(async move {
            let socket = TokioTcpStream::connect(server_addr).await?;
            let fd = socket.as_raw_fd();
            set_socket_mark(fd, mark_value)?;
            Ok(AsyncIoTokioAsStd(socket))
        })
    }

    fn bind_udp(
        &self,
        local_addr: SocketAddr,
        _server_addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Udp>>>> {
        let mark_value = self.mark_value;
        Box::pin(async move {
            let socket = TokioUdpSocket::bind(local_addr).await?;
            let fd = socket.as_raw_fd();
            set_socket_mark(fd, mark_value)?;
            Ok(socket)
        })
    }
}

fn set_socket_mark(fd: RawFd, mark_value: u32) -> io::Result<()> {
    // 设置 SO_MARK 选项
    let result = unsafe {
        setsockopt(
            fd,
            SOL_SOCKET,
            SO_MARK,
            &mark_value as *const u32 as *const libc::c_void,
            std::mem::size_of::<u32>() as libc::socklen_t,
        )
    };

    if result == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
