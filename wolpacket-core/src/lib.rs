//! Construct and send [Wake-on-LAN](https://en.wikipedia.org/wiki/Wake-on-LAN) magic packets.
//!
//! # Example
//!
//! ```no_run
//! use wolpacket_core::wake;
//!
//! let mac = "74:CA:60:27:82:0A".parse()?;
//! wake(mac, "192.168.103.255:9")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use macaddr::MacAddr6;
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};
use thiserror::Error;
use tracing::debug;

/// Errors that can occur sending a magic packet.
#[derive(Error, Debug)]
pub enum Error {
    /// The broadcast address could not be resolved.
    #[error("invalid broadcast address: {0}")]
    InvalidBroadcastAddress(String),

    /// An I/O error occurred opening or writing to the socket.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// A constructed Wake-on-LAN magic packet, ready to be sent.
///
/// The magic packet consists of 6 bytes of `0xFF` followed by the target
/// MAC address repeated 16 times (102 bytes total).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MagicPacket {
    payload: [u8; 102],
}

impl MagicPacket {
    /// Build a magic packet for the given MAC address.
    pub fn new(mac: MacAddr6) -> Self {
        let mut payload = [0xFFu8; 102];
        for i in 0..16 {
            let offset = 6 + i * 6;
            payload[offset..offset + 6].copy_from_slice(mac.as_bytes());
        }

        debug!(%mac, "built magic packet");
        Self { payload }
    }

    /// Send the magic packet to `broadcast_addr` via UDP.
    ///
    /// The socket will be created with `SO_BROADCAST` enabled.
    /// Uses port 9 by default (the standard WOL port).
    pub fn send_to(&self, broadcast_addr: impl ToSocketAddrs) -> Result<(), Error> {
        let addr = broadcast_addr
            .to_socket_addrs()
            .map_err(|e| Error::InvalidBroadcastAddress(e.to_string()))?
            .next()
            .ok_or_else(|| Error::InvalidBroadcastAddress("no addresses resolved".into()))?;

        debug!(%addr, "sending magic packet");

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.send_to(&self.payload, addr)?;

        debug!("magic packet sent");
        Ok(())
    }

    /// The raw 102-byte payload.
    pub fn as_bytes(&self) -> &[u8; 102] {
        &self.payload
    }
}

/// Convenience: construct and send a magic packet in one call.
///
/// ```no_run
/// use wolpacket_core::wake;
///
/// let mac = "74:CA:60:27:82:0A".parse()?;
/// wake(mac, "192.168.103.255:9")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn wake(mac: MacAddr6, broadcast_addr: impl ToSocketAddrs) -> Result<(), Error> {
    MagicPacket::new(mac).send_to(broadcast_addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_mac() -> MacAddr6 {
        "74:CA:60:27:82:0A".parse().unwrap()
    }

    #[test]
    fn magic_packet_starts_with_six_ff() {
        let packet = MagicPacket::new(test_mac());
        assert_eq!(packet.as_bytes()[..6], [0xFF; 6]);
    }

    #[test]
    fn magic_packet_length() {
        let packet = MagicPacket::new(test_mac());
        assert_eq!(packet.as_bytes().len(), 102);
    }

    #[test]
    fn magic_packet_contains_mac_16_times() {
        let mac = test_mac();
        let packet = MagicPacket::new(mac);
        for i in 0..16 {
            let offset = 6 + i * 6;
            assert_eq!(&packet.as_bytes()[offset..offset + 6], mac.as_bytes());
        }
    }
}
