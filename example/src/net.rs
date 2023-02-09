
use core::ptr::NonNull;

use alloc::{vec, string::String};
use lose_net_stack::{LoseStack, IPv4, MacAddress, results::Packet};
use opensbi_rt::{print, println};

use crate::e1000_impls::Kernfn;


pub fn init() {

    e1000_driver::pci::pci_init();

    let mut net = e1000_driver::e1000::E1000Device::<Kernfn>::new(
        e1000_driver::pci::E1000_REGS as usize
    ).unwrap();

    let lose_stack = LoseStack::new(IPv4::new(10, 0, 2, 15), MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]));

    loop {
        let datas = net.e1000_recv();
        
        if datas.is_none() {
            continue;
        }

        let datas = datas.unwrap();

        for data in datas {
            info!("receive {} bytes from net", data.len());
            hexdump(&data);
    
            let packet = lose_stack.analysis(&data);
            info!("packet: {:?}", packet);
    
            match packet {
                Packet::ARP(arp_packet) => {
                    let reply_packet = arp_packet.reply_packet(lose_stack.ip, lose_stack.mac).expect("can't build reply");
                    info!("reply_packet: {:?}", reply_packet);
                    let reply_data = reply_packet.build_data();
                    hexdump(&reply_data);
                    net.e1000_transmit(&reply_data);
                },
                Packet::UDP(udp_packet) => {
                    info!("{}:{}(MAC:{}) -> {}:{}(MAC:{})  len:{}", udp_packet.source_ip, udp_packet.source_port, udp_packet.source_mac, 
                        udp_packet.dest_ip, udp_packet.dest_port, udp_packet.dest_mac, udp_packet.data_len);
                    info!("data: {}", String::from_utf8_lossy(udp_packet.data.as_ref()));
                    hexdump(udp_packet.data.as_ref());
    
                    if String::from_utf8_lossy(udp_packet.data.as_ref()) == "this is a ping!" {
                        let data = r"reply".as_bytes();
                        let udp_reply_packet = udp_packet.reply(data);
                        net.e1000_transmit(&udp_reply_packet.build_data());
                        info!("net stack example test successed!");
                        return;
                    }
    
                    // let response_udp = 
                }
                _ => {}
            }
        }
    }
}

pub fn hexdump(data: &[u8]) {
    const PRELAND_WIDTH: usize = 70;
    println!("{:-^1$}", " hexdump ", PRELAND_WIDTH);
    for offset in (0..data.len()).step_by(16) {
        for i in 0..16 {
            if offset + i < data.len() {
                print!("{:02x} ", data[offset + i]);
            } else {
                print!("{:02} ", "");
            }
        }

        print!("{:>6}", ' ');

        for i in 0..16 {
            if offset + i < data.len() {
                let c = data[offset + i];
                if c >= 0x20 && c <= 0x7e {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            } else {
                print!("{:02} ", "");
            }
        }
        
        println!("");
    }
    println!("{:-^1$}", " hexdump end ", PRELAND_WIDTH);
}

/* UDP SEND HEXDUMP
------------------------------ hexdump -------------------------------
ff ff ff ff ff ff 52 54 00 12 34 56 08 00 45 00       ......RT..4V..E.
28 00 00 00 00 00 20 11 61 ff 0a 00 02 0f 0a 00       (..... .a.......
02 02 38 18 39 18 14 00 5e ff 48 65 6c 6c 6f 20       ..8.9...^.Hello 
57 6f 72 6c 64 21                                     World!                              
---------------------------- hexdump end -----------------------------
*/