//! Gerenciador de rede P2P via iroh.
//!
//! Roda em uma task tokio separada para nunca bloquear a UI.
//! A UI se comunica com este manager via canais:
//!   - `NetworkCommand`: UI → Manager (hospedar, conectar, enviar jogada)
//!   - `NetworkEvent`: Manager → UI (ticket pronto, peer conectou, recebeu jogada)

use std::str::FromStr;

use anyhow::{Context, Result};
use iroh::{
    endpoint::{Connection, RecvStream, SendStream},
    ticket::NodeTicket,
    Endpoint,
};
use tokio::sync::mpsc;

use crate::network::protocol::GameMessage;

/// ALPN do protocolo Velha 2.0 — identifica nossa aplicação no handshake iroh.
const ALPN_VELHA: &[u8] = b"velha2/0.1";

// ──────────────────────────────────────────────────────────
// Tipos de canal UI ↔ Rede
// ──────────────────────────────────────────────────────────

/// Comandos enviados da UI para o NetworkManager.
#[derive(Debug)]
pub enum NetworkCommand {
    /// Hospedar uma nova sessão P2P.
    Hospedar { nosso_nome: String },
    /// Conectar a uma sessão existente usando o ticket do host.
    Conectar {
        ticket_str: String,
        nosso_nome: String,
    },
    /// Enviar uma jogada para o peer.
    EnviarJogada { quad: usize, cell: usize },
    /// Encerrar a conexão.
    Desconectar,
}

/// Eventos enviados do NetworkManager para a UI.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Host pronto — compartilhe este ticket com o amigo.
    HostPronto { ticket: String },
    /// Peer conectou — partida pode começar.
    PeerConectado { nome_peer: String },
    /// Recebemos uma jogada do peer.
    JogadaRecebida { quad: usize, cell: usize },
    /// Peer desconectou ou a conexão caiu.
    PeerDesconectado,
    /// Erro de rede — mensagem para exibir na UI.
    Erro { mensagem: String },
}

/// Handle retornado pela inicialização do NetworkManager.
/// A UI mantém este handle para enviar comandos e receber eventos.
pub struct NetworkHandle {
    pub tx_cmd: mpsc::Sender<NetworkCommand>,
    pub rx_evt: mpsc::Receiver<NetworkEvent>,
}

/// Inicializa o NetworkManager em uma task tokio separada.
///
/// Retorna um `NetworkHandle` para comunicação com a UI.
pub fn iniciar_network_manager() -> NetworkHandle {
    let (tx_cmd, rx_cmd) = mpsc::channel::<NetworkCommand>(32);
    let (tx_evt, rx_evt) = mpsc::channel::<NetworkEvent>(32);

    tokio::spawn(async move {
        if let Err(e) = run_manager(rx_cmd, tx_evt.clone()).await {
            let _ = tx_evt
                .send(NetworkEvent::Erro {
                    mensagem: format!("Erro interno de rede: {e}"),
                })
                .await;
        }
    });

    NetworkHandle { tx_cmd, rx_evt }
}

// ──────────────────────────────────────────────────────────
// Loop principal do manager
// ──────────────────────────────────────────────────────────

async fn run_manager(
    mut rx_cmd: mpsc::Receiver<NetworkCommand>,
    tx_evt: mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    // Aguarda o primeiro comando para saber o modo (hospedar ou conectar)
    let primeiro_cmd = match rx_cmd.recv().await {
        Some(cmd) => cmd,
        None => return Ok(()), // Canal fechado antes de receber qualquer comando
    };

    match primeiro_cmd {
        NetworkCommand::Hospedar { nosso_nome } => {
            run_host(nosso_nome, rx_cmd, tx_evt).await?;
        }
        NetworkCommand::Conectar { ticket_str, nosso_nome } => {
            run_guest(ticket_str, nosso_nome, rx_cmd, tx_evt).await?;
        }
        _ => {} // Comandos prematuros são ignorados
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────
// Modo HOST
// ──────────────────────────────────────────────────────────

async fn run_host(
    nosso_nome: String,
    mut rx_cmd: mpsc::Receiver<NetworkCommand>,
    tx_evt: mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    // Cria o endpoint com ALPN configurado
    let endpoint = Endpoint::builder()
        .alpns(vec![ALPN_VELHA.to_vec()])
        .bind()
        .await
        .context("Falha ao criar endpoint iroh")?;

    // Gera o NodeTicket para compartilhamento
    // Aguarda termos pelo menos um endereço relay disponível
    let addr = endpoint.node_addr().await.context("Falha ao obter NodeAddr")?;
    let ticket = NodeTicket::new(addr);
    let ticket_str = ticket.to_string();

    // Notifica a UI que o ticket está pronto
    tx_evt
        .send(NetworkEvent::HostPronto {
            ticket: ticket_str,
        })
        .await
        .ok();

    // Aguarda o guest conectar
    let connecting = endpoint
        .accept()
        .await
        .context("Falha ao aguardar conexão")?;

    let conn = connecting
        .await
        .context("Falha ao aceitar conexão")?;

    // Handshake: troca de nomes
    let nome_peer = executar_handshake_host(&conn, &nosso_nome).await?;

    tx_evt
        .send(NetworkEvent::PeerConectado { nome_peer })
        .await
        .ok();

    // Loop de jogo
    run_game_loop(conn, rx_cmd, tx_evt).await
}

// ──────────────────────────────────────────────────────────
// Modo GUEST
// ──────────────────────────────────────────────────────────

async fn run_guest(
    ticket_str: String,
    nosso_nome: String,
    rx_cmd: mpsc::Receiver<NetworkCommand>,
    tx_evt: mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    // Parse do ticket
    let ticket = NodeTicket::from_str(&ticket_str)
        .context("Ticket inválido — verifique se copiou corretamente")?;

    // Cria endpoint do guest
    let endpoint = Endpoint::builder()
        .bind()
        .await
        .context("Falha ao criar endpoint iroh")?;

    // Conecta ao host usando o NodeAddr do ticket
    let conn = endpoint
        .connect(ticket.node_addr().clone(), ALPN_VELHA)
        .await
        .context("Falha ao conectar ao host — verifique o ticket e tente novamente")?;

    // Handshake
    let nome_host = executar_handshake_guest(&conn, &nosso_nome).await?;

    tx_evt
        .send(NetworkEvent::PeerConectado { nome_peer: nome_host })
        .await
        .ok();

    run_game_loop(conn, rx_cmd, tx_evt).await
}

// ──────────────────────────────────────────────────────────
// Handshake: troca de nomes após conexão
// ──────────────────────────────────────────────────────────

async fn executar_handshake_host(conn: &Connection, nosso_nome: &str) -> Result<String> {
    // Abre stream bidirecional
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .context("Falha ao abrir stream bidirecional")?;

    // Envia nosso nome
    escrever_mensagem(&mut send, &GameMessage::Handshake { nome: nosso_nome.to_owned() }).await?;

    // Lê nome do guest
    match ler_mensagem(&mut recv).await? {
        GameMessage::Handshake { nome } => Ok(nome),
        _ => anyhow::bail!("Handshake inesperado do peer"),
    }
}

async fn executar_handshake_guest(conn: &Connection, nosso_nome: &str) -> Result<String> {
    // Aceita o stream aberto pelo host
    let (mut send, mut recv) = conn
        .accept_bi()
        .await
        .context("Falha ao aceitar stream bidirecional")?;

    // Lê nome do host
    let nome_host = match ler_mensagem(&mut recv).await? {
        GameMessage::Handshake { nome } => nome,
        _ => anyhow::bail!("Handshake inesperado do host"),
    };

    // Envia nosso nome
    escrever_mensagem(&mut send, &GameMessage::Handshake { nome: nosso_nome.to_owned() }).await?;

    Ok(nome_host)
}

// ──────────────────────────────────────────────────────────
// Loop de jogo: recebe jogadas do peer e envia as nossas
// ──────────────────────────────────────────────────────────

async fn run_game_loop(
    conn: Connection,
    mut rx_cmd: mpsc::Receiver<NetworkCommand>,
    tx_evt: mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    // Abre stream unidirecional para enviar jogadas
    let (mut send_jog, mut recv_jog) = conn
        .open_bi()
        .await
        .context("Falha ao abrir stream de jogo")?;

    loop {
        tokio::select! {
            // Comando da UI (enviar jogada ou desconectar)
            cmd = rx_cmd.recv() => {
                match cmd {
                    Some(NetworkCommand::EnviarJogada { quad, cell }) => {
                        let msg = GameMessage::Jogada { quad, cell };
                        if let Err(e) = escrever_mensagem(&mut send_jog, &msg).await {
                            tx_evt.send(NetworkEvent::Erro {
                                mensagem: format!("Erro ao enviar jogada: {e}"),
                            }).await.ok();
                            break;
                        }
                    }
                    Some(NetworkCommand::Desconectar) | None => {
                        conn.close(0u32.into(), b"bye");
                        break;
                    }
                    _ => {}
                }
            }

            // Mensagem chegando do peer
            resultado = ler_mensagem(&mut recv_jog) => {
                match resultado {
                    Ok(GameMessage::Jogada { quad, cell }) => {
                        tx_evt.send(NetworkEvent::JogadaRecebida { quad, cell }).await.ok();
                    }
                    Ok(GameMessage::Desistir) => {
                        tx_evt.send(NetworkEvent::PeerDesconectado).await.ok();
                        break;
                    }
                    Ok(_) => {} // Outras mensagens ignoradas nesta fase
                    Err(_) => {
                        // Conexão caiu
                        tx_evt.send(NetworkEvent::PeerDesconectado).await.ok();
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────
// Serialização de mensagens sobre streams QUIC
// ──────────────────────────────────────────────────────────

/// Serializa e escreve uma mensagem no stream com um prefixo de tamanho (4 bytes, big-endian).
async fn escrever_mensagem(send: &mut SendStream, msg: &GameMessage) -> Result<()> {
    let dados = serde_json::to_vec(msg).context("Falha ao serializar mensagem")?;
    let tamanho = (dados.len() as u32).to_be_bytes();

    send.write_all(&tamanho).await.context("Falha ao escrever tamanho")?;
    send.write_all(&dados).await.context("Falha ao escrever dados")?;

    Ok(())
}

/// Lê uma mensagem de um stream com prefixo de tamanho.
async fn ler_mensagem(recv: &mut RecvStream) -> Result<GameMessage> {
    // Lê os 4 bytes de tamanho
    let mut buf_tamanho = [0u8; 4];
    recv.read_exact(&mut buf_tamanho)
        .await
        .context("Conexão encerrada (tamanho)")?;

    let tamanho = u32::from_be_bytes(buf_tamanho) as usize;

    // Sanidade: mensagens maiores que 64KB são suspeitas
    if tamanho > 65_536 {
        anyhow::bail!("Mensagem muito grande: {} bytes", tamanho);
    }

    // Lê os dados
    let mut dados = vec![0u8; tamanho];
    recv.read_exact(&mut dados)
        .await
        .context("Conexão encerrada (dados)")?;

    serde_json::from_slice(&dados).context("Falha ao desserializar mensagem")
}
