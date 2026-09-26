use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use sokol_biophoton::{BioMetabolism, BiophotonEmitter, BiophotonPulse, BiophotonReceptor};

fn main() {
    println!("=== Advanced Sokol-Biophoton Physics & Immune Test ===");

    thread::spawn(|| {
        let socket = UdpSocket::bind("127.0.0.1:5052").unwrap();
        socket.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
        let mut receptor = BiophotonReceptor::new(0.5, 0.08);
        let mut metabolism = BioMetabolism::default();
        let mut buf = [0u8; 64];

        loop {
            if let Ok((sz, _)) = socket.recv_from(&mut buf) {
                let absorbed = receptor.absorb(&buf[..sz], 1.0);
                if !absorbed {
                    println!("  [Immune System] Packet dropped or node neutralized!");
                }
            }

            let prev = metabolism.stress_level;
            receptor.tick(&mut metabolism);

            if (metabolism.stress_level - prev).abs() > 0.005 {
                println!(
                    "  [Receptor] Energy: {:.3} | Stress: {:.2} | Neutralized nodes: {}",
                    receptor.current_energy(),
                    metabolism.stress_level,
                    receptor.blacklist_count
                );
            }
        }
    });

    thread::sleep(Duration::from_millis(100));


    let socket = UdpSocket::bind("127.0.0.1:5051").unwrap();
    let mut emitter_valid = BiophotonEmitter::new(101, 0.3); // node_id: u16 = 101
    let mut meta_valid = BioMetabolism { atp: 1.0, stress_level: 0.8, atp_recovery_rate: 0.02 };
    let mut buf = [0u8; BiophotonPulse::SIZE];

    println!("\n--- 1. Valid biophoton transmission (Node 101) ---");
    
    let content_hash = 0x8F3A;
    if let Some(sz) = emitter_valid.emit_if_stressed(&mut meta_valid, 15, content_hash, &mut buf) {
        socket.send_to(&buf[..sz], "127.0.0.1:5052").unwrap();
    }
    thread::sleep(Duration::from_millis(200));

    println!("\n--- 2. Anomaly attack attempt (Node 666) ---");
    let rogue_pulse = BiophotonPulse {
        magic: BiophotonPulse::MAGIC,
        content_hash: 0xDEAD, 
        threat_type: 2,       
        stress_intensity: 255,
        phase: 128,
        emitter_id: 666,      
    };
    socket.send_to(rogue_pulse.as_bytes(), "127.0.0.1:5052").unwrap();
    thread::sleep(Duration::from_millis(200));

    println!("\n--- 3. Repeated attack attempt from Node 666 (Immunity check) ---");
    socket.send_to(rogue_pulse.as_bytes(), "127.0.0.1:5052").unwrap();
    thread::sleep(Duration::from_millis(300));
}