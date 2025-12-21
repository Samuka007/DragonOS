use inner::{UdpInner, UnboundUdp};
use smoltcp;
use system_error::SystemError;

use crate::filesystem::epoll::EPollEventType;
use crate::filesystem::vfs::{fasync::FAsyncItems, vcore::generate_inode_id, InodeId};
use crate::libs::wait_queue::WaitQueue;
use crate::net::socket::common::EPollItems;
use crate::net::socket::{Socket, PMSG};
use crate::process::namespace::net_namespace::NetNamespace;
use crate::process::ProcessManager;
use crate::{libs::rwlock::RwLock, net::socket::endpoint::Endpoint};
use alloc::sync::{Arc, Weak};
use core::sync::atomic::AtomicBool;

use super::InetSocket;

pub mod inner;

type EP = crate::filesystem::epoll::EPollEventType;

// Udp Socket 负责提供状态切换接口、执行状态切换
#[cast_to([sync] Socket)]
#[derive(Debug)]
pub struct UdpSocket {
    inner: RwLock<Option<UdpInner>>,
    nonblock: AtomicBool,
    wait_queue: WaitQueue,
    inode_id: InodeId,
    self_ref: Weak<UdpSocket>,
    netns: Arc<NetNamespace>,
    epoll_items: EPollItems,
    fasync_items: FAsyncItems,
}

impl UdpSocket {
    pub fn new(nonblock: bool) -> Arc<Self> {
        let netns = ProcessManager::current_netns();
        Arc::new_cyclic(|me| Self {
            inner: RwLock::new(Some(UdpInner::Unbound(UnboundUdp::new()))),
            nonblock: AtomicBool::new(nonblock),
            wait_queue: WaitQueue::default(),
            inode_id: generate_inode_id(),
            self_ref: me.clone(),
            netns,
            epoll_items: EPollItems::default(),
            fasync_items: FAsyncItems::default(),
        })
    }

    pub fn is_nonblock(&self) -> bool {
        self.nonblock.load(core::sync::atomic::Ordering::Relaxed)
    }

    pub fn do_bind(&self, local_endpoint: smoltcp::wire::IpEndpoint) -> Result<(), SystemError> {
        let mut inner_guard = self.inner.write();
        let inner = inner_guard.take().ok_or(SystemError::EBADF)?;
        if let UdpInner::Unbound(unbound) = inner {
            match unbound.bind(local_endpoint, self.netns()) {
                Ok(bound) => {
                    bound
                        .inner()
                        .iface()
                        .common()
                        .bind_socket(self.self_ref.upgrade().unwrap());
                    *inner_guard = Some(UdpInner::Bound(bound));
                    return Ok(());
                }
                Err(e) => {
                    *inner_guard = Some(UdpInner::Unbound(UnboundUdp::new()));
                    return Err(e);
                }
            }
        }
        *inner_guard = Some(inner);
        return Err(SystemError::EINVAL);
    }

    pub fn bind_ephemeral(&self, remote: smoltcp::wire::IpAddress) -> Result<(), SystemError> {
        let mut inner_guard = self.inner.write();
        let inner = inner_guard.take().ok_or(SystemError::EBADF)?;
        let bound = match inner {
            UdpInner::Bound(inner) => inner,
            UdpInner::Unbound(unbound) => match unbound.bind_ephemeral(remote, self.netns()) {
                Ok(bound) => bound,
                Err(e) => {
                    inner_guard.replace(UdpInner::Unbound(UnboundUdp::new()));
                    return Err(e);
                }
            },
        };
        inner_guard.replace(UdpInner::Bound(bound));
        return Ok(());
    }

    pub fn is_bound(&self) -> bool {
        let inner = self.inner.read();
        if let Some(UdpInner::Bound(_)) = &*inner {
            return true;
        }
        return false;
    }

    pub fn close(&self) {
        let mut inner = self.inner.write();
        if let Some(UdpInner::Bound(bound)) = &mut *inner {
            bound.close();
            inner.take();
        }
        // unbound socket just drop (only need to free memory)
    }

    pub fn try_recv(
        &self,
        buf: &mut [u8],
    ) -> Result<(usize, smoltcp::wire::IpEndpoint), SystemError> {
        match self.inner.read().as_ref().ok_or(SystemError::EBADF)? {
            UdpInner::Bound(bound) => {
                let ret = bound.try_recv(buf);
                bound.inner().iface().poll();
                ret
            }
            _ => Err(SystemError::EAGAIN_OR_EWOULDBLOCK),
        }
    }

    #[inline]
    pub fn can_recv(&self) -> bool {
        self.check_io_event().contains(EP::EPOLLIN)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn can_send(&self) -> bool {
        self.check_io_event().contains(EP::EPOLLOUT)
    }

    pub fn try_send(
        &self,
        buf: &[u8],
        to: Option<smoltcp::wire::IpEndpoint>,
    ) -> Result<usize, SystemError> {
        let mut inner_guard = self.inner.write();

        // Check if socket is closed
        let inner = inner_guard.as_ref().ok_or(SystemError::EBADF)?;

        // If unbound, bind ephemeral port
        if let UdpInner::Unbound(_) = inner {
            let to_addr = to.ok_or(SystemError::EDESTADDRREQ)?.addr;
            let unbound = match inner_guard.take().unwrap() {
                UdpInner::Unbound(unbound) => unbound,
                _ => unreachable!(),
            };
            match unbound.bind_ephemeral(to_addr, self.netns()) {
                Ok(bound) => {
                    inner_guard.replace(UdpInner::Bound(bound));
                }
                Err(e) => {
                    inner_guard.replace(UdpInner::Unbound(UnboundUdp::new()));
                    return Err(e);
                }
            }
        }

        // Now send the data while holding the write lock
        match inner_guard.as_ref().ok_or(SystemError::EBADF)? {
            UdpInner::Bound(bound) => {
                let ret = bound.try_send(buf, to);
                bound.inner().iface().poll();
                ret
            }
            _ => Err(SystemError::ENOTCONN),
        }
    }

    pub fn netns(&self) -> Arc<NetNamespace> {
        self.netns.clone()
    }
}

impl Socket for UdpSocket {
    fn wait_queue(&self) -> &WaitQueue {
        &self.wait_queue
    }

    fn bind(&self, local_endpoint: Endpoint) -> Result<(), SystemError> {
        if let Endpoint::Ip(local_endpoint) = local_endpoint {
            return self.do_bind(local_endpoint);
        }
        Err(SystemError::EAFNOSUPPORT)
    }

    fn send_buffer_size(&self) -> usize {
        match self.inner.read().as_ref() {
            Some(UdpInner::Bound(bound)) => {
                bound.with_socket(|socket| socket.payload_send_capacity())
            }
            _ => inner::DEFAULT_TX_BUF_SIZE,
        }
    }

    fn recv_buffer_size(&self) -> usize {
        match self.inner.read().as_ref() {
            Some(UdpInner::Bound(bound)) => {
                bound.with_socket(|socket| socket.payload_recv_capacity())
            }
            _ => inner::DEFAULT_RX_BUF_SIZE,
        }
    }

    fn connect(&self, endpoint: Endpoint) -> Result<(), SystemError> {
        if let Endpoint::Ip(remote) = endpoint {
            if !self.is_bound() {
                self.bind_ephemeral(remote.addr)?;
            }
            if let Some(UdpInner::Bound(inner)) = self.inner.read().as_ref() {
                inner.connect(remote);
                return Ok(());
            } else {
                return Err(SystemError::EBADF);
            }
        } else if let Endpoint::Unspecified = endpoint {
            if let Some(UdpInner::Bound(inner)) = self.inner.read().as_ref() {
                inner.disconnect();
                return Ok(());
            } else {
                // Not connected, but not bound either - still valid to disconnect (no-op)
                return Ok(());
            }
        }
        return Err(SystemError::EAFNOSUPPORT);
    }

    fn send(&self, buffer: &[u8], flags: PMSG) -> Result<usize, SystemError> {
        if flags.contains(PMSG::DONTWAIT) {
            log::warn!("Nonblock send is not implemented yet");
        }

        return self.try_send(buffer, None);
    }

    fn send_to(&self, buffer: &[u8], flags: PMSG, address: Endpoint) -> Result<usize, SystemError> {
        if flags.contains(PMSG::DONTWAIT) {
            log::warn!("Nonblock send is not implemented yet");
        }

        if let Endpoint::Ip(remote) = address {
            return self.try_send(buffer, Some(remote));
        }

        return Err(SystemError::EINVAL);
    }

    fn recv(&self, buffer: &mut [u8], flags: PMSG) -> Result<usize, SystemError> {
        return if self.is_nonblock() || flags.contains(PMSG::DONTWAIT) {
            self.try_recv(buffer)
        } else {
            loop {
                match self.try_recv(buffer) {
                    Err(SystemError::EAGAIN_OR_EWOULDBLOCK) => {
                        wq_wait_event_interruptible!(self.wait_queue, self.can_recv(), {})?;
                    }
                    result => break result,
                }
            }
        }
        .map(|(len, _)| len);
    }

    fn recv_from(
        &self,
        buffer: &mut [u8],
        flags: PMSG,
        address: Option<Endpoint>,
    ) -> Result<(usize, Endpoint), SystemError> {
        // could block io
        if let Some(endpoint) = address {
            self.connect(endpoint)?;
        }

        return if self.is_nonblock() || flags.contains(PMSG::DONTWAIT) {
            self.try_recv(buffer)
        } else {
            loop {
                match self.try_recv(buffer) {
                    Err(SystemError::EAGAIN_OR_EWOULDBLOCK) => {
                        wq_wait_event_interruptible!(self.wait_queue, self.can_recv(), {})?;
                        // log::debug!("UdpSocket::recv_from: wake up");
                    }
                    result => break result,
                }
            }
        }
        .map(|(len, remote)| (len, Endpoint::Ip(remote)));
    }

    fn do_close(&self) -> Result<(), SystemError> {
        self.close();
        Ok(())
    }

    fn remote_endpoint(&self) -> Result<Endpoint, SystemError> {
        match self.inner.read().as_ref().ok_or(SystemError::EBADF)? {
            UdpInner::Bound(bound) => Ok(Endpoint::Ip(bound.remote_endpoint()?)),
            // TODO: IPv6 support
            _ => Err(SystemError::ENOTCONN),
        }
    }

    fn local_endpoint(&self) -> Result<Endpoint, SystemError> {
        use smoltcp::wire::{IpAddress::*, IpEndpoint, IpListenEndpoint};
        match self.inner.read().as_ref().ok_or(SystemError::EBADF)? {
            UdpInner::Bound(bound) => {
                let IpListenEndpoint { addr, port } = bound.endpoint();
                // If bound to 0.0.0.0 (unspecified) but connected to a remote peer (e.g. 127.0.0.1),
                // we should return the specific interface address we are using (e.g. 127.0.0.1).
                // TODO: Query routing table to find exact outgoing interface address.
                // For now, if connected to loopback, assume we are on loopback.
                let mut local_addr = addr.unwrap_or(Ipv4([0, 0, 0, 0].into()));
                if local_addr.is_unspecified() {
                    if let Ok(remote) = bound.remote_endpoint() {
                         match remote.addr {
                            Ipv4(addr) if addr.is_loopback() => {
                                local_addr = Ipv4([127, 0, 0, 1].into());
                            }
                            _ => {}
                         }
                    }
                }

                Ok(Endpoint::Ip(IpEndpoint::new(
                    local_addr,
                    port,
                )))
            }
            // TODO: IPv6 support
            _ => Ok(Endpoint::Ip(IpEndpoint::new(Ipv4([0, 0, 0, 0].into()), 0))),
        }
    }

    fn recv_msg(
        &self,
        _msg: &mut crate::net::posix::MsgHdr,
        _flags: PMSG,
    ) -> Result<usize, SystemError> {
        todo!()
    }

    fn send_msg(
        &self,
        _msg: &crate::net::posix::MsgHdr,
        _flags: PMSG,
    ) -> Result<usize, SystemError> {
        todo!()
    }

    fn epoll_items(&self) -> &crate::net::socket::common::EPollItems {
        &self.epoll_items
    }

    fn fasync_items(&self) -> &FAsyncItems {
        &self.fasync_items
    }

    fn option(&self, level: crate::net::socket::PSOL, name: usize, value: &mut [u8]) -> Result<usize, SystemError> {
        use crate::net::posix::{PosixIpSocketOptions, PosixIpv6SocketOptions};
        use crate::net::socket::PSOL;

        match level {
            PSOL::IP => {
                if let Some(name) = PosixIpSocketOptions::from_bits(name as u32) {
                    if name.contains(PosixIpSocketOptions::IP_RECVERR) {
                         // TODO: Actual implementation
                         // For now, return 0 to indicate it's disabled (default)
                         if value.len() < 4 {
                            return Err(SystemError::EINVAL);
                         }
                         value[0..4].copy_from_slice(&0u32.to_ne_bytes());
                         return Ok(4);
                    }
                } else {
                    return Err(SystemError::ENOPROTOOPT);
                }
            }
            PSOL::IPV6 => {
                if let Some(name) = PosixIpv6SocketOptions::from_bits(name as u32) {
                    if name.contains(PosixIpv6SocketOptions::IPV6_RECVERR) {
                         // TODO: Actual implementation
                         // For now, return 0 to indicate it's disabled (default)
                         if value.len() < 4 {
                            return Err(SystemError::EINVAL);
                         }
                         value[0..4].copy_from_slice(&0u32.to_ne_bytes());
                         return Ok(4);
                    }
                } else {
                    return Err(SystemError::ENOPROTOOPT);
                }
            }
            _ => return Err(SystemError::ENOSYS),
        }
        Err(SystemError::ENOSYS)
    }

    fn set_option(&self, level: crate::net::socket::PSOL, name: usize, _value: &[u8]) -> Result<(), SystemError> {
        use crate::net::posix::{PosixIpSocketOptions, PosixIpv6SocketOptions};
        use crate::net::socket::PSOL;
        match level {
            PSOL::IP => {
                if let Some(name) = PosixIpSocketOptions::from_bits(name as u32) {
                    if name.contains(PosixIpSocketOptions::IP_RECVERR) {
                         // TODO: Actual implementation
                         // Accept the setting but warn it's a stub
                         return Ok(());
                    }
                } else {
                    return Err(SystemError::ENOPROTOOPT);
                }
            }
            PSOL::IPV6 => {
                if let Some(name) = PosixIpv6SocketOptions::from_bits(name as u32) {
                    if name.contains(PosixIpv6SocketOptions::IPV6_RECVERR) {
                         // TODO: Actual implementation
                         // Accept the setting but warn it's a stub
                         return Ok(());
                    }
                } else {
                    return Err(SystemError::ENOPROTOOPT);
                }
            }
            _ => return Err(SystemError::ENOSYS),
        }
        Err(SystemError::ENOSYS)
    }
    fn check_io_event(&self) -> EPollEventType {
        let mut event = EPollEventType::empty();
        match self.inner.read().as_ref() {
            Some(UdpInner::Unbound(_)) => {
                event.insert(EP::EPOLLOUT | EP::EPOLLWRNORM | EP::EPOLLWRBAND);
            }
            Some(UdpInner::Bound(bound)) => {
                let (can_recv, can_send) =
                    bound.with_socket(|socket| (socket.can_recv(), socket.can_send()));

                if can_recv {
                    event.insert(EP::EPOLLIN | EP::EPOLLRDNORM);
                }

                if can_send {
                    event.insert(EP::EPOLLOUT | EP::EPOLLWRNORM | EP::EPOLLWRBAND);
                }
            }
            None => {
                event.insert(EP::EPOLLERR | EP::EPOLLHUP);
            }
        }
        return event;
    }

    fn socket_inode_id(&self) -> InodeId {
        self.inode_id
    }
}

impl InetSocket for UdpSocket {
    fn on_iface_events(&self) {
        return;
    }
}

bitflags! {
    pub struct UdpSocketOptions: u32 {
        const ZERO = 0;        /* No UDP options */
        const UDP_CORK = 1;         /* Never send partially complete segments */
        const UDP_ENCAP = 100;      /* Set the socket to accept encapsulated packets */
        const UDP_NO_CHECK6_TX = 101; /* Disable sending checksum for UDP6X */
        const UDP_NO_CHECK6_RX = 102; /* Disable accepting checksum for UDP6 */
        const UDP_SEGMENT = 103;    /* Set GSO segmentation size */
        const UDP_GRO = 104;        /* This socket can receive UDP GRO packets */

        const UDPLITE_SEND_CSCOV = 10; /* sender partial coverage (as sent)      */
        const UDPLITE_RECV_CSCOV = 11; /* receiver partial coverage (threshold ) */
    }
}

bitflags! {
    pub struct UdpEncapTypes: u8 {
        const ZERO = 0;
        const ESPINUDP_NON_IKE = 1;     // draft-ietf-ipsec-nat-t-ike-00/01
        const ESPINUDP = 2;             // draft-ietf-ipsec-udp-encaps-06
        const L2TPINUDP = 3;            // rfc2661
        const GTP0 = 4;                 // GSM TS 09.60
        const GTP1U = 5;                // 3GPP TS 29.060
        const RXRPC = 6;
        const ESPINTCP = 7;             // Yikes, this is really xfrm encap types.
    }
}
