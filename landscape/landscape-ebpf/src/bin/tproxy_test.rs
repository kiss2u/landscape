use std::net::TcpListener;
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::os::unix::io::AsRawFd as _;
use std::str::FromStr;
use std::thread;

use nix::sys::socket::bind;
use nix::sys::socket::getsockopt;
use nix::sys::socket::listen;
use nix::sys::socket::setsockopt;
use nix::sys::socket::socket;
use nix::sys::socket::sockopt::BindToDevice;
use nix::sys::socket::sockopt::IpTransparent;
use nix::sys::socket::sockopt::ReuseAddr;
use nix::sys::socket::sockopt::ReusePort;
use nix::sys::socket::AddressFamily;
use nix::sys::socket::Backlog;
use nix::sys::socket::SockFlag;
use nix::sys::socket::SockType;
use nix::sys::socket::SockaddrIn;
use std::ffi::OsStr;

fn run() {
    // Create listener socket
    let fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();

    println!("fd is : {:?}", fd);
    // Set some sockopts
    setsockopt(&fd, ReuseAddr, &true).unwrap();
    // setsockopt(&fd, ReusePort, &true).unwrap();
    setsockopt(&fd, IpTransparent, &true).unwrap();

    let transparent_enabled: bool = getsockopt(&fd, IpTransparent).unwrap();
    println!("IP_TRANSPARENT enabled: {}", transparent_enabled);

    // let if_name = "br0".to_string();
    // setsockopt(&fd, BindToDevice, &OsStr::new("br0").to_os_string()).unwrap();

    // Bind to addr
    let addr = format!("{}:{}", "0.0.0.0", 12345);
    let addr = SockaddrIn::from_str(&addr).unwrap();
    bind(fd.as_raw_fd(), &addr).unwrap();

    // Start listening
    listen(&fd, Backlog::new(128).unwrap()).unwrap();
    let listener = TcpListener::from(fd);

    println!("start");
    // while let Ok((client, _addr)) = listener.accept() {
    //     // let client = client.unwrap();
    //     handle_client(client);
    // }
    for client in listener.incoming() {
        let client = client.unwrap();
        handle_client(client);
    }
}

fn handle_client(client: TcpStream) {
    let local_addr = client.local_addr().unwrap();
    let peer_addr = client.peer_addr().unwrap();

    println!("New connection:");
    println!("\tlocal: {local_addr}");
    println!("\tpeer: {peer_addr}");
    println!();
}
pub fn main() {
    // ip netns exec tproxy curl -vvv http://223.6.6.6:2234
    // echo asdf | nc -s 127.0.0.5 127.0.0.1 2234
    // ip netns exec tproxy sh -c 'echo asdf | nc -s 10.1.2.2 -p 123 223.6.6.6 2234'

    // ip rule add fwmark 0x1/0x1 lookup 100
    // ip route add local default dev lo table 100

    // thread::spawn(|| run());
    landscape_ebpf::tproxy::run()
}
