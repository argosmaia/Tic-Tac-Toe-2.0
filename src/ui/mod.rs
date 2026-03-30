//! Camada de apresentação egui do Velha 2.0.
//!
//! Depende de todas as camadas abaixo (game/, ai/, storage/, network/) via abstrações.
//! Nenhuma lógica de negócio vive aqui — apenas apresentação e roteamento de eventos.

pub mod components;
pub mod screens;
pub mod theme;
