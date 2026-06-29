use std::net::{TcpListener, TcpStream, Ipv4Addr, SocketAddrV4};
// Убрали UdpSocket

#[allow(dead_code)]
pub struct NxTcpListener(TcpListener);
#[allow(dead_code)]
pub struct NxTcpStream(TcpStream);

#[no_mangle]
pub extern "C" fn nx_net_tcp_listen(port: u16) -> *mut NxTcpListener {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    match TcpListener::bind(addr) {
        Ok(l) => Box::into_raw(Box::new(NxTcpListener(l))),
        Err(_) => std::ptr::null_mut(),
    }
}