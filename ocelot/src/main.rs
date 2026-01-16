use std::io::{Error, Read, Write};

use ocelot_protocol::{
    buffer::PacketBuffer,
    codec::{MinecraftCodec, VarInt},
    packet::{
        MinecraftPacket,
        configuration::ClientboundFinishConfigurationPacket,
        handshaking::{Intent, ServerboundHandshakePacket},
        login::{ClientboundLoginSuccessPacket, ServerboundLoginStartPacket},
    },
};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::io::SyncIoBridge;

// The written code here is only proof of concept and testing.

enum ConnectionState {
    STATUS,
    LOGIN,
    CONFIGURATION,
    PLAY,
}
struct Player {
    username: Option<String>,
    current_state: ConnectionState,
}
fn send_packet<P: MinecraftPacket>(packet: &P, bridge: &mut SyncIoBridge<&mut TcpStream>) {
    let mut buffer = Vec::new();
    let mut packet_data = packet.serialize().unwrap();
    VarInt(packet_data.len() as i32)
        .encode(&mut buffer)
        .unwrap();
    buffer.append(&mut packet_data);
    bridge.write_all(&buffer).unwrap();
    bridge.flush().unwrap();
}
async fn handle_connection(mut stream: TcpStream) {
    tokio::task::spawn_blocking(move || {
        let mut player_info: Option<Player> = None;
        let mut bridge = SyncIoBridge::new(&mut stream);
        loop {
            let size = VarInt::decode(&mut bridge).unwrap().0 as usize;
            let mut buffer = vec![0u8; size];
            bridge.read_exact(&mut buffer).unwrap();
            let mut packet_buffer = PacketBuffer::new(&buffer);
            let packet_id = VarInt::decode(&mut packet_buffer).unwrap().0;
            if let Some(ref mut player) = player_info {
                match player.current_state {
                    ConnectionState::STATUS => match packet_id {
                        _ => {
                            eprintln!(
                                "I can't handle packet id: {} in the Status state at the moment :(",
                                packet_id
                            );
                        }
                    },
                    ConnectionState::LOGIN => match packet_id {
                        0x00 => {
                            println!(
                                "[Client -> Server] Login Start (State: Login, ID: {}):",
                                packet_id
                            );
                            let packet =
                                ServerboundLoginStartPacket::deserialize(&mut packet_buffer)
                                    .unwrap();
                            println!("Name: {}", packet.get_name().0);
                            player.username = Some(packet.get_name().0.clone());
                            println!("Player UUID: {}", packet.get_player_uuid());

                            let (uuid, name) = (packet.get_player_uuid(), packet.get_name());
                            let login_success_packet =
                                ClientboundLoginSuccessPacket::new(*uuid, name.clone(), VarInt(0));
                            send_packet(&login_success_packet, &mut bridge);
                            println!(
                                "[Server -> Client] Login Success (State: Login, ID: {})",
                                login_success_packet.get_id()
                            );
                        }
                        0x03 => {
                            println!(
                                "[Client -> Server] Login Acknowledged (State: Login, ID: {}):",
                                packet_id
                            );
                            player.current_state = ConnectionState::CONFIGURATION;
                        }
                        _ => {
                            eprintln!(
                                "I can't handle packet id: {} in the Login state at the moment :(",
                                packet_id
                            );
                        }
                    },
                    ConnectionState::CONFIGURATION => match packet_id {
                        0x00 => {
                            println!(
                                "[Client -> Server] Client Information (State: Configuration, ID: {}):",
                                packet_id
                            );
                            let finish_configuration_packet =
                                ClientboundFinishConfigurationPacket::new();
                            send_packet(&finish_configuration_packet, &mut bridge);
                            println!(
                                "[Server -> Client] Finish Configuration (State: Configuration, ID: {})",
                                finish_configuration_packet.get_id()
                            );
                        }
                        0x03 => {
                            println!(
                                "[Client -> Server] Acknowledge Finish Configuration (State: Configuration, ID: {}):",
                                packet_id
                            );
                            player.current_state = ConnectionState::PLAY;
                        }
                        _ => {
                            eprintln!(
                                "I can't handle packet id: {} in the Configuration state at the moment :(",
                                packet_id
                            );
                        }
                    },
                    ConnectionState::PLAY => match packet_id {
                        _ => {
                            eprintln!(
                                "I can't handle packet id: {} in the Play state at the moment :(",
                                packet_id
                            );
                        }
                    },
                }
            } else {
                match packet_id {
                    0x00 => {
                        println!(
                            "[Client -> Server] Handshake Packet (State: Handshaking, ID: {}):",
                            packet_id
                        );
                        let packet =
                            ServerboundHandshakePacket::deserialize(&mut packet_buffer).unwrap();
                        println!("Protocol Version: {}", packet.get_protocol_version().0);
                        println!("Server Address: {}", packet.get_server_address().0);
                        println!("Server Port: {}", packet.get_server_port());
                        println!("Intent: {}", packet.get_intent());
                        if matches!(packet.get_intent(), Intent::LOGIN) {
                            player_info = Some(Player {
                                username: None,
                                current_state: ConnectionState::LOGIN,
                            });
                        } else if matches!(packet.get_intent(), Intent::STATUS) {
                            player_info = Some(Player {
                                username: None,
                                current_state: ConnectionState::STATUS,
                            });
                        }
                    }
                    _ => {
                        eprintln!(
                            "I can't handle packet id: {} in the Handshake state at the moment :(",
                            packet_id
                        );
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Hello, world!");

    let listener = TcpListener::bind("0.0.0.0:25565").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}
