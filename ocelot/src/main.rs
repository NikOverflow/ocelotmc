use std::{
    fmt::{Display, Formatter},
    io::{self, Error},
    sync::Arc,
};

use num_bigint::BigInt;
use ocelot_data::registry::SYNCED_REGISTRIES;
use ocelot_protocol::{
    buffer::PacketBuffer,
    codec::{BoundedPrefixedArray, MinecraftCodec, PrefixedArray},
    packet::{
        MinecraftPacket,
        configuration::{
            clientbound as configuration_clientbound, serverbound as configuration_serverbound,
        },
        handshaking::serverbound as handshaking_serverbound,
        login::{clientbound as login_clientbound, serverbound as login_serverbound},
        play::{clientbound as play_clientbound, serverbound as play_serverbound},
        types::{GameEvent, GameMode, Intent, KnownPack, RegistryEntry, TeleportFlags},
    },
};
use ocelot_types::{BoundedString, ResourceLocation, VarInt};
use openssl::{
    pkey::Private,
    rsa::{Padding, Rsa},
};
use rand::{RngCore, SeedableRng};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::{
    io::AsyncRead,
    net::{TcpListener, TcpStream},
};
use uuid::Uuid;

// The written code here is only a proof of concept and for testing purposes.

pub fn get_server_hash(server_id: &str, shared_secret: &[u8], public_key_der: &[u8]) -> String {
    let mut hasher = openssl::sha::Sha1::new();
    hasher.update(server_id.as_bytes());
    hasher.update(shared_secret);
    hasher.update(public_key_der);
    let hash_result = hasher.finish();
    let big_int = BigInt::from_signed_bytes_be(&hash_result);
    format!("{:x}", big_int)
}

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

async fn read_varint<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<i32> {
    const SEGMENT_BITS: u32 = 0x7F;
    const CONTINUE_BITS: u32 = 0x80;
    let mut value = 0;
    let mut position = 0;
    let mut byte = [0u8; 1];
    loop {
        reader.read_exact(&mut byte).await?;
        let current_byte = byte[0];
        value |= ((current_byte & SEGMENT_BITS as u8) as i32) << position;
        if (current_byte & CONTINUE_BITS as u8) == 0 {
            break;
        }
        position += 7;
        if position >= 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt is too big!",
            ));
        }
    }
    Ok(value)
}

pub struct Connection {
    state: ConnectionState,
}
impl Connection {
    async fn send_packet<P: MinecraftPacket>(&self, packet: &P, stream: &mut TcpStream) {
        let mut buffer = Vec::new();
        let mut packet_data = packet.serialize().unwrap();
        VarInt(packet_data.len() as i32)
            .encode(&mut buffer)
            .unwrap();
        buffer.append(&mut packet_data);
        stream.write_all(&buffer).await.unwrap();
        stream.flush().await.unwrap();
        println!(
            "[Server -> Client] {} (State: {}, ID: {})",
            format_packet_name(std::any::type_name::<P>()),
            self.state,
            packet.get_id()
        );
    }
    fn read_packet<P: MinecraftPacket>(&self, packet_buffer: &mut PacketBuffer) -> P {
        let packet = P::deserialize(packet_buffer).unwrap();
        println!(
            "[Client -> Server] {} (State: {}, ID: {})",
            format_packet_name(std::any::type_name::<P>()),
            self.state,
            packet.get_id()
        );
        packet
    }
    async fn handle_connection(&mut self, mut stream: TcpStream, rsa_key_pair: Arc<Rsa<Private>>) {
        let mut player: Player = Player {
            username: None,
            uuid: None,
        };
        let mut rng = rand::rngs::StdRng::from_os_rng();
        let mut sent_verify_token = [0; 4];
        rng.fill_bytes(&mut sent_verify_token);
        loop {
            let size = match read_varint(&mut stream).await {
                Ok(value) => value as usize,
                Err(_) => break,
            };
            let mut buffer = vec![0u8; size];
            if let Err(_) = stream.read_exact(&mut buffer).await {
                break;
            }
            let mut packet_buffer = PacketBuffer::new(&buffer);
            let packet_id = VarInt::decode(&mut packet_buffer).unwrap().0;
            match self.state {
                ConnectionState::HANDSHAKING => match packet_id {
                    handshaking_serverbound::HandshakePacket::ID => {
                        let packet = self.read_packet::<handshaking_serverbound::HandshakePacket>(
                            &mut packet_buffer,
                        );
                        println!("Packet Data:");
                        println!("Protocol Version: {}", packet.get_protocol_version().0);
                        println!("Server Address: {}", packet.get_server_address().0);
                        println!("Server Port: {}", packet.get_server_port());
                        println!("Intent: {}", packet.get_intent());

                        match packet.get_intent() {
                            Intent::Status => self.state = ConnectionState::STATUS,
                            Intent::Login => self.state = ConnectionState::LOGIN,
                            Intent::Transfer => self.state = ConnectionState::LOGIN,
                        }
                    }
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        self.state, packet_id
                    ),
                },
                ConnectionState::STATUS => match packet_id {
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        self.state, packet_id
                    ),
                },
                ConnectionState::LOGIN => match packet_id {
                    login_serverbound::LoginStartPacket::ID => {
                        let packet = self
                            .read_packet::<login_serverbound::LoginStartPacket>(&mut packet_buffer);
                        println!("Packet Data:");
                        println!("Name: {}", packet.get_name().0);
                        println!("Player UUID: {}", packet.get_player_uuid());

                        player.username = Some(packet.get_name().0.clone());
                        player.uuid = Some(*packet.get_player_uuid());

                        let login_success = login_clientbound::LoginSuccessPacket::new(
                            player.uuid.unwrap(),
                            BoundedString::new(player.username.as_ref().unwrap()).unwrap(),
                            BoundedPrefixedArray::new(Vec::new()),
                        );
                        self.send_packet(&login_success, &mut stream).await;

                        /*let encryption_request_packet = ClientboundEncryptionRequestPacket::new(
                            BoundedString::new("").unwrap(),
                            PrefixedArray(rsa_key_pair.public_key_to_der().unwrap()),
                            PrefixedArray(sent_verify_token.to_vec()),
                            true,
                        );
                        self.send_packet(&encryption_request_packet, &mut stream, &player)
                            .await;*/
                    }
                    login_serverbound::EncryptionResponsePacket::ID => {
                        let packet = self
                            .read_packet::<login_serverbound::EncryptionResponsePacket>(
                                &mut packet_buffer,
                            );
                        let shared_secret = &packet.get_shared_secret().0;
                        let verify_token = &packet.get_verify_token().0;
                        println!("Packet Data:");
                        println!("Shared Secret: {:?}", packet.get_shared_secret().0);
                        println!("Verify Token: {:?}", packet.get_verify_token().0);
                        let mut decrypted_shared_secret = [0; 128];
                        rsa_key_pair
                            .private_decrypt(
                                &shared_secret,
                                &mut decrypted_shared_secret,
                                Padding::PKCS1,
                            )
                            .unwrap();
                        let decrypted_shared_secret = &decrypted_shared_secret[..16];
                        println!("Decrypted Shared Secret: {:?}", decrypted_shared_secret);
                        let mut decrypted_verify_token = [0; 128];
                        rsa_key_pair
                            .private_decrypt(
                                &verify_token,
                                &mut decrypted_verify_token,
                                Padding::PKCS1,
                            )
                            .unwrap();
                        println!("Decrypted Verify Token: {:?}", decrypted_verify_token);
                        if sent_verify_token != &decrypted_verify_token[..4] {
                            println!("Token invalid!");
                            break;
                        }

                        // TODO: encrypt and decrypt for online mode
                        let server_hash = get_server_hash(
                            "",
                            &decrypted_shared_secret[..16],
                            &rsa_key_pair.public_key_to_der().unwrap(),
                        );

                        let client = reqwest::Client::new();

                        let username = player.username.as_ref().unwrap();

                        let response = client
                            .get("https://sessionserver.mojang.com/session/minecraft/hasJoined")
                            .query(&[("username", username), ("serverId", &server_hash)])
                            .send()
                            .await
                            .unwrap();
                        if response.status() == 200 {
                            let body = response.text().await.unwrap_or_default();
                            println!("{}", body);
                            let login_success = login_clientbound::LoginSuccessPacket::new(
                                player.uuid.unwrap(),
                                BoundedString::new(username).unwrap(),
                                BoundedPrefixedArray::new(Vec::new()),
                            );
                            self.send_packet(&login_success, &mut stream).await;
                        } else {
                            println!("{}", response.status());
                        }
                    }
                    login_serverbound::LoginAcknowledgedPacket::ID => {
                        let _ = self.read_packet::<login_serverbound::LoginAcknowledgedPacket>(
                            &mut packet_buffer,
                        );
                        self.state = ConnectionState::CONFIGURATION;
                    }
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        self.state, packet_id
                    ),
                },
                ConnectionState::CONFIGURATION => match packet_id {
                    configuration_serverbound::ClientInformationPacket::ID => {
                        let packet = self
                            .read_packet::<configuration_serverbound::ClientInformationPacket>(
                                &mut packet_buffer,
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
                            configuration_clientbound::KnownPacksPacket::new(PrefixedArray(vec![
                                KnownPack {
                                    namespace: BoundedString::<_>::new("minecraft").unwrap(),
                                    id: BoundedString::<_>::new("core").unwrap(),
                                    version: BoundedString::<_>::new("1.21.11").unwrap(),
                                },
                            ]));
                        self.send_packet(&known_packs_packet, &mut stream).await;
                    }
                    configuration_serverbound::PluginMessagePacket::ID => {
                        let packet = self
                            .read_packet::<configuration_serverbound::PluginMessagePacket>(
                                &mut packet_buffer,
                            );
                        println!("Packet Data:");
                        println!("Channel: {}", packet.get_channel().to_string());
                        println!("Data: {:?}", packet.get_data());
                    }
                    configuration_serverbound::KnownPacksPacket::ID => {
                        let packet = self
                            .read_packet::<configuration_serverbound::KnownPacksPacket>(
                                &mut packet_buffer,
                            );
                        println!("Packet Data:");
                        println!("Known Packs:");
                        for known_pack in &packet.get_known_packs().0 {
                            println!("Namespace: {}", known_pack.namespace.0);
                            println!("ID: {}", known_pack.id.0);
                            println!("Version: {}", known_pack.version.0);
                        }

                        for registry in SYNCED_REGISTRIES {
                            let mut entries = Vec::new();
                            for entry in registry.entries {
                                entries.push(RegistryEntry {
                                    id: BoundedString::<32767>::new(entry.name)
                                        .unwrap()
                                        .0
                                        .try_into()
                                        .unwrap(),
                                    data: Some(entry.nbt_bytes.to_vec()),
                                });
                            }
                            let registry_data_packet =
                                configuration_clientbound::RegistryDataPacket::new(
                                    BoundedString::<32767>::new(registry.registry_id)
                                        .unwrap()
                                        .0
                                        .try_into()
                                        .unwrap(),
                                    PrefixedArray(entries),
                                );
                            self.send_packet(&registry_data_packet, &mut stream).await;
                        }

                        let finish_configuration_packet =
                            configuration_clientbound::FinishConfigurationPacket::new();
                        self.send_packet(&finish_configuration_packet, &mut stream)
                            .await;
                    }
                    configuration_serverbound::AcknowledgeFinishConfigurationPacket::ID => {
                        let _ = self
                            .read_packet::<configuration_serverbound::AcknowledgeFinishConfigurationPacket>(
                                &mut packet_buffer,
                            );
                        self.state = ConnectionState::PLAY;

                        let login_packet = play_clientbound::LoginPacket::new(
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
                            ResourceLocation::from_vanilla("overworld").unwrap(),
                            0,
                            GameMode::Survival,
                            GameMode::Undefined,
                            false,
                            false,
                            None,
                            VarInt(0),
                            VarInt(60),
                            false,
                        );
                        self.send_packet(&login_packet, &mut stream).await;
                        let game_event_packet = play_clientbound::GameEventPacket::new(
                            GameEvent::StartWaitingForLevelChunks,
                            0.0,
                        );
                        self.send_packet(&game_event_packet, &mut stream).await;
                        let synchronize_player_position_packet =
                            play_clientbound::SynchronizePlayerPositionPacket::new(
                                VarInt(1),
                                0.0,
                                -128.0,
                                0.0,
                                0.0,
                                -128.0,
                                0.0,
                                0.0,
                                0.0,
                                TeleportFlags::empty(),
                            );
                        self.send_packet(&synchronize_player_position_packet, &mut stream)
                            .await;
                    }
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        self.state, packet_id
                    ),
                },
                ConnectionState::PLAY => match packet_id {
                    play_serverbound::ClientTickEndPacket::ID => {
                        let _ = self.read_packet::<play_serverbound::ClientTickEndPacket>(
                            &mut packet_buffer,
                        );
                    }
                    _ => eprintln!(
                        "[Client -> Server] ??? (State: {}, ID: {})",
                        self.state, packet_id
                    ),
                },
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let rsa_key_pair: Arc<Rsa<Private>> = Arc::new(Rsa::generate(1024).unwrap());
    println!("Hello, world!");

    let listener = TcpListener::bind("0.0.0.0:25565").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        let copy_key_pair = Arc::clone(&rsa_key_pair);
        tokio::spawn(async move {
            let mut connection = Connection {
                state: ConnectionState::HANDSHAKING,
            };
            connection.handle_connection(socket, copy_key_pair).await;
        });
    }
}
