use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use rtrb::Producer;

pub const DEFAULT_STREAMING_PORT: u16 = 48100;

/// UDP Audio Receiver for Phone -> Laptop real-time streaming
pub struct PhoneAudioReceiver {
    port: u16,
    running: Arc<AtomicBool>,
}

impl PhoneAudioReceiver {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Spawns a background thread listening for UDP audio packets from the Android companion app
    pub fn start(&mut self, mut ring_producer: Producer<f32>) -> Result<(), String> {
        let addr = format!("0.0.0.0:{}", self.port);
        let socket = UdpSocket::bind(&addr).map_err(|e| format!("Failed to bind UDP socket {}: {}", addr, e))?;
        socket.set_read_timeout(Some(Duration::from_millis(100))).map_err(|e| e.to_string())?;

        self.running.store(true, Ordering::Release);
        let is_running = self.running.clone();

        println!("[+] Phone Audio Streaming Receiver listening on UDP 0.0.0.0:{}", self.port);

        thread::Builder::new()
            .name("mx5ic-phone-receiver".into())
            .spawn(move || {
                let mut buf = [0u8; 2048]; // Max UDP packet size

                while is_running.load(Ordering::Acquire) {
                    match socket.recv_from(&mut buf) {
                        Ok((bytes_read, _peer)) => {
                            if bytes_read >= 4 && bytes_read % 4 == 0 {
                                // Raw 32-bit float PCM samples (Little-Endian)
                                let num_samples = bytes_read / 4;
                                for i in 0..num_samples {
                                    let sample_bytes = [
                                        buf[i * 4 + 0],
                                        buf[i * 4 + 1],
                                        buf[i * 4 + 2],
                                        buf[i * 4 + 3],
                                    ];
                                    let sample = f32::from_le_bytes(sample_bytes);
                                    let _ = ring_producer.push(sample);
                                }
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                            // Timeout, loop and check running flag
                        }
                        Err(e) => {
                            eprintln!("[!] UDP socket error: {}", e);
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
                println!("[*] Phone Audio Receiver stopped.");
            })
            .map_err(|e| format!("Failed to spawn receiver thread: {}", e))?;

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }
}
