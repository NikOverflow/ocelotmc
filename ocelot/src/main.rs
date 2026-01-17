use std::{
    fmt::{Display, Formatter},
    io::{Error, Read, Write},
};

use ocelot_protocol::{
    buffer::PacketBuffer,
    codec::{BoundedPrefixedArray, BoundedString, MinecraftCodec, PrefixedArray, VarInt},
    packet::{
        MinecraftPacket,
        configuration::{
            ClientboundFinishConfigurationPacket, ClientboundKnownPacksPacket, KnownPacks,
            ServerboundAcknowledgeFinishConfigurationPacket, ServerboundClientInformationPacket,
            ServerboundKnownPacksPacket,
        },
        handshaking::{Intent, ServerboundHandshakePacket},
        login::{
            ClientboundLoginSuccessPacket, Properties, ServerboundLoginAcknowledgedPacket,
            ServerboundLoginStartPacket,
        }, play,
    },
    types::Identifier,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::io::SyncIoBridge;
use uuid::Uuid;

// The written code here is only a proof of concept and for testing purposes.

enum ConnectionState {
    HANDSHAKING,
    STATUS,
    LOGIN,
    CONFIGURATION,
    PLAY,
}
impl Display for ConnectionState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let name = match self {
            Self::HANDSHAKING => "Handshaking",
            Self::STATUS => "Status",
            Self::LOGIN => "Login",
            Self::CONFIGURATION => "Configuration",
            Self::PLAY => "Play",
        };
        write!(f, "{}", name)
    }
}

struct Player {
    username: Option<String>,
    uuid: Option<Uuid>,
    state: ConnectionState,
}

fn format_packet_name(full_packet_name: &str) -> String {
    let mut packet_name = full_packet_name
        .split("::")
        .last()
        .unwrap_or(full_packet_name);
    packet_name = packet_name
        .strip_prefix("Clientbound")
        .unwrap_or(packet_name);
    packet_name = packet_name
        .strip_prefix("Serverbound")
        .unwrap_or(packet_name);
    let mut final_packet_name = String::new();
    for (i, c) in packet_name
        .strip_suffix("Packet")
        .unwrap()
        .chars()
        .enumerate()
    {
        if i > 0 && c.is_uppercase() {
            final_packet_name.push(' ');
        }
        final_packet_name.push(c);
    }
    final_packet_name
}

fn send_packet<P: MinecraftPacket>(
    packet: &P,
    bridge: &mut SyncIoBridge<&mut TcpStream>,
    connection_state: &ConnectionState,
) {
    let mut buffer = Vec::new();
    let mut packet_data = packet.serialize().unwrap();
    VarInt(packet_data.len() as i32)
        .encode(&mut buffer)
        .unwrap();
    buffer.append(&mut packet_data);
    bridge.write_all(&buffer).unwrap();
    bridge.flush().unwrap();
    println!(
        "[Server -> Client] {} (State: {}, ID: {})",
        format_packet_name(std::any::type_name::<P>()),
        connection_state,
        packet.get_id()
    );
}
fn read_packet<P: MinecraftPacket>(
    packet_buffer: &mut PacketBuffer,
    connection_state: &ConnectionState,
) -> P {
    let packet = P::deserialize(packet_buffer).unwrap();
    println!(
        "[Client -> Server] {} (State: {}, ID: {})",
        format_packet_name(std::any::type_name::<P>()),
        connection_state,
        packet.get_id()
    );
    packet
}

async fn handle_connection(mut stream: TcpStream) {
    tokio::task::spawn_blocking(move || {
        let mut player: Player = Player {
            username: None,
            uuid: None,
            state: ConnectionState::HANDSHAKING,
        };
        let mut bridge = SyncIoBridge::new(&mut stream);
        loop {
            let size = match VarInt::decode(&mut bridge) {
                Ok(value) => value.0 as usize,
                Err(_) => {
                    eprintln!("An error occured!");
                    break;
                }
            };
            let mut buffer = vec![0u8; size];
            if let Err(_) = bridge.read_exact(&mut buffer) {
                eprintln!("An error occured!");
                break;
            }
            let mut packet_buffer = PacketBuffer::new(&buffer);
            let packet_id = VarInt::decode(&mut packet_buffer).unwrap().0;
            match player.state {
                ConnectionState::HANDSHAKING => match packet_id {
                    0x00 => {
                        let packet = read_packet::<ServerboundHandshakePacket>(
                            &mut packet_buffer,
                            &player.state,
                        );
                        println!("Packet Data:");
                        println!("Protocol Version: {}", packet.get_protocol_version().0);
                        println!("Server Address: {}", packet.get_server_address().0);
                        println!("Server Port: {}", packet.get_server_port());
                        println!("Intent: {}", packet.get_intent());
                        println!("");

                        match packet.get_intent() {
                            Intent::STATUS => player.state = ConnectionState::STATUS,
                            Intent::LOGIN => player.state = ConnectionState::LOGIN,
                            Intent::TRANSFER => player.state = ConnectionState::LOGIN,
                        }
                    }
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        player.state, packet_id
                    ),
                },
                ConnectionState::STATUS => match packet_id {
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        player.state, packet_id
                    ),
                },
                ConnectionState::LOGIN => match packet_id {
                    0x00 => {
                        let packet = read_packet::<ServerboundLoginStartPacket>(
                            &mut packet_buffer,
                            &player.state,
                        );
                        println!("Packet Data:");
                        println!("Name: {}", packet.get_name().0);
                        println!("Player UUID: {}", packet.get_player_uuid());
                        println!("");

                        player.username = Some(packet.get_name().0.clone());
                        player.uuid = Some(*packet.get_player_uuid());

                        let login_success_packet = ClientboundLoginSuccessPacket::new(
                            *packet.get_player_uuid(),
                            packet.get_name().clone(),
                            BoundedPrefixedArray::<Properties, 16>::new(Vec::new()),
                        );
                        send_packet(&login_success_packet, &mut bridge, &player.state);
                    }
                    0x03 => {
                        let packet = read_packet::<ServerboundLoginAcknowledgedPacket>(
                            &mut packet_buffer,
                            &player.state,
                        );
                        player.state = ConnectionState::CONFIGURATION;
                    }
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        player.state, packet_id
                    ),
                },
                ConnectionState::CONFIGURATION => match packet_id {
                    0x00 => {
                        let packet = read_packet::<ServerboundClientInformationPacket>(
                            &mut packet_buffer,
                            &player.state,
                        );
                        println!("Packet Data:");
                        println!("Locale: {}", packet.get_locale().0);
                        println!("View Distance: {}", packet.get_view_distance());
                        println!("Chat Mode: {}", packet.get_chat_mode());
                        println!("Chat Colors: {}", packet.get_chat_colors());
                        println!(
                            "Displayed Skin Parts: {}",
                            packet.get_displayed_skin_parts()
                        );
                        println!("Main Hand: {}", packet.get_main_hand());
                        println!(
                            "Enable text filtering: {}",
                            packet.get_enable_text_filtering()
                        );
                        println!(
                            "Allow server listings: {}",
                            packet.get_allow_server_listings()
                        );
                        println!("Particle Status: {}", packet.get_particle_status());

                        let known_packs_packet =
                            ClientboundKnownPacksPacket::new(PrefixedArray(vec![KnownPacks {
                                namespace: BoundedString::<_>::new("minecraft").unwrap(),
                                id: BoundedString::<_>::new("core").unwrap(),
                                version: BoundedString::<_>::new("1.21.11").unwrap(),
                            }]));
                        send_packet(&known_packs_packet, &mut bridge, &player.state);
                    }
                    0x07 => {
                        let packet = read_packet::<ServerboundKnownPacksPacket>(
                            &mut packet_buffer,
                            &player.state,
                        );
                        println!("Packet Data:");
                        println!("Known Packs:");
                        for known_pack in &packet.get_known_packs().0 {
                            println!("Namespace: {}", known_pack.namespace.0);
                            println!("ID: {}", known_pack.id.0);
                            println!("Version: {}", known_pack.version.0);
                        }

                        let finish_configuration_packet =
                            ClientboundFinishConfigurationPacket::new();
                        send_packet(&finish_configuration_packet, &mut bridge, &player.state);
                    }
                    0x03 => {
                        let _packet = read_packet::<ServerboundAcknowledgeFinishConfigurationPacket>(
                            &mut packet_buffer,
                            &player.state,
                        );
                        player.state = ConnectionState::PLAY;
                            
                        let login_packet = play::ClientboundLoginPacket::new(
                            0,
                            false,
                            PrefixedArray(Vec::new()),
                            VarInt(1),
                            VarInt(8),
                            VarInt(8),
                            false,
                            false,
                            false,
                            VarInt(0),
                            Identifier::from_string(
                                BoundedString::new("minecraft:overworld").unwrap(),
                            ),
                            0,
                            0,
                            -1,
                            false,
                            false,
                            None,
                            VarInt(0),
                            VarInt(60),
                            false,
                        );
                        send_packet(&login_packet, &mut bridge, &player.state);
                    }
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        player.state, packet_id
                    ),
                },
                ConnectionState::PLAY => match packet_id {
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        player.state, packet_id
                    ),
                },
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
