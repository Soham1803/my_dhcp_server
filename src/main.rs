use std::{collections::HashMap, net::{Ipv4Addr, UdpSocket}, time::{Duration, SystemTime}};

#[derive(Debug)]
struct LeaseEntry {
    mac_address : [u8; 6],
    ip_address : Ipv4Addr,
    lease_expiry : SystemTime,
}

struct DHCPServer {
    socket : UdpSocket,
    available_pool : Vec<Ipv4Addr>,
    leases : HashMap<[u8; 6], LeaseEntry>,
    subnet_mask : Ipv4Addr,
    gateway : Ipv4Addr,
    dns_servers : Vec<Ipv4Addr>,
    lease_duration : Duration,
}

impl DHCPServer {
    fn new() -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:67")?;
        socket.set_broadcast(true)?;

        let mut available_pool = Vec::new();
        for i in 100..200 {
            available_pool.push(Ipv4Addr::new(192, 168, 1, i));
        }

        Ok(DHCPServer {
            socket,
            available_pool,
            leases : HashMap::new(),
            subnet_mask : Ipv4Addr::new(255, 255, 255, 0),
            gateway : Ipv4Addr::new(192, 168, 1, 1),
            dns_servers : vec![Ipv4Addr::new(8, 8, 8, 8)],
            lease_duration : Duration::from_secs(86400),
        })
    }

    fn process_discover(&mut self, mac_address : [u8; 6]) -> Option<Ipv4Addr>{
        // Clean expired leases
        self.clean_expired_leases();

        // Check if client already has a lease
        if let Some(lease) = self.leases.get(&mac_address) {
            return Some(lease.ip_address);
        }

        // Find available IP
        if let Some(ip) = self.available_pool.pop() {
            let lease = LeaseEntry {
                mac_address,
                ip_address : ip,
                lease_expiry : SystemTime::now() + self.lease_duration,
            };

            self.leases.insert(mac_address, lease);
            Some(ip)
        } else {
            None
        }
    }

    fn clean_expired_leases (&mut self) {
        let now = SystemTime::now();
        let expired : Vec<_> = self.leases
            .iter()
            .filter(|(_, lease)| lease.lease_expiry <= now)
            .map(|(mac, lease)| (*mac, lease.ip_address))
            .collect();

        for (mac, ip) in expired {
            self.leases.remove(&mac);
            self.available_pool.push(ip);
        }
    }

    fn run(&mut self) -> std::io::Result<()> {
        let mut buffer = [0u8; 1024];

        loop {
            let (size, src) = self.socket.recv_from(&mut buffer)?;
            if size < 241 {
                continue;
            }

            let message_type = match self.get_dhcp_message_type(&buffer[..size]) {
                Some(t) => t,
                None => continue,
            };

            let mac_address = self.get_mac_address(&buffer);

            match message_type {
                1 => {
                    if let Some(offer_ip) = self.process_discover(mac_address) {
                        self.send_offer(src, mac_address, offer_ip)?;
                    }
                }
                3 => {
                    self.send_ack(src, mac_address)?;
                }
                _ => continue,
            }
        }
    }

    fn get_dhcp_message_type(&self, packet: &[u8]) -> Option<u8> {
        Some(packet[0])
    }

    fn get_mac_address(&self, packet: &[u8]) -> [u8; 6] {
        let mut mac = [0u8; 6];
        mac.copy_from_slice(&packet[28..34]);
        mac
    }

    fn send_offer(&self, dest : std::net::SocketAddr, mac : [u8; 6], ip : Ipv4Addr) -> std::io::Result<()> {
        // Simplified DHCP offer packet construction
        let mut response = vec![0u8; 300];

        //...Fill in DHCP offer packet fields...
        self.socket.send_to(&response, dest)?;
        Ok(())
    }

    fn send_ack(&self, dest : std::net::SocketAddr, mac : [u8; 6]) -> std::io::Result<()>{
        // Simplified DHCP ACK packet construction
        let mut response = vec![0u8; 300];
        // ...Fill in DHCP ACK packet fields...
        self.socket.send_to(&response, dest)?;
        Ok(())
    }
}



fn main() -> std::io::Result<()> {
    let mut server = DHCPServer::new()?;
    server.run()
}
