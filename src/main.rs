#![windows_subsystem = "windows"]
use rosc::{self, encoder, OscPacket, OscMessage, OscType};
use ascii::{AsciiChar};

use eframe::{epi, egui, epaint::Vec2};

struct Egui7DisplayApp {
    text: String,
    tx: tokio::sync::watch::Sender<String>,
    font_id: egui::FontId,
}

impl epi::App for Egui7DisplayApp {
    fn name(&self) -> &str {
        "7Display"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                let response = ui.add(egui::TextEdit::singleline(&mut self.text).font(self.font_id.clone()));
                
                if response.changed() {
                    self.text = self.text.chars().take(50).collect();
                    self.tx.send(self.text.clone()).expect("Couldn't send string to other thread");
                }

                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    self.text = "".to_string();
                    self.tx.send(self.text.clone()).expect("Couldn't send string to other thread");
                    response.request_focus();
                }
            });
            
        });
    }
}

impl Egui7DisplayApp {
    fn init(tx: tokio::sync::watch::Sender<String>) -> Egui7DisplayApp {
        let app = Egui7DisplayApp {
            text: "".to_string(),
            font_id: egui::FontId::monospace(60.0),
            tx
        };
                
        app
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::watch::channel("".to_string());

    tokio::spawn(async move {
        keystroke_processor(rx).await;
    });

    let app = Egui7DisplayApp::init(tx);

    let native_options = eframe::NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: true,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_pos: None,
        initial_window_size: Some(Vec2 { x: 1600.0, y: 80.0 }),
        min_window_size: None,
        max_window_size: None,
        resizable: true,
        transparent: false,
    };
    eframe::run_native(Box::new(app), native_options);
}

/**
 * controls the synchronization of the vrchat 7display and the rust 7display
 */
async fn keystroke_processor(mut rx: tokio::sync::watch::Receiver<String>) {
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:9003").await.expect("Couldn't bind a udp socket");
    socket.connect("127.0.0.1:9000").await.expect("Failed to connect");
    let mut current_display_value = "".to_string();
    let mut last_sent_char: char = AsciiChar::LineFeed.as_char();

    while rx.changed().await.is_ok() {
        //ignore non-ascii chars
        //TODO: this line is repeated below, figure out how to deduplicate it
        let mut next_display_value = (*rx.borrow()).replace(|c: char| !c.is_ascii(), "");
        while next_display_value != current_display_value {
            // println!("{:?} -> {:?}", current_display_value, next_display_value);
            let char_to_send = if next_display_value.len() == 0 {
                //empty string, clear it
                current_display_value.clear();
                AsciiChar::LineFeed.as_char()
            } else if next_display_value.len() * 2 + 1 < current_display_value.len() {
                current_display_value.clear();
                //would need to backspace a lot, just clear it
                AsciiChar::LineFeed.as_char()
            } else if next_display_value.starts_with(&current_display_value) {
                //characters typed, send 1 char
                let cur_len = current_display_value.chars().count();
                let next_char = next_display_value.chars().skip(cur_len).next().expect("Couldn't find another char somehow");
                current_display_value.push(next_char);
                next_char
            } else if current_display_value.starts_with(&next_display_value) {
                //characters deleted, delete 1 char
                current_display_value.pop();
                AsciiChar::BackSpace.as_char()
            } else {
                //totally different string, just clear it
                current_display_value.clear();
                AsciiChar::LineFeed.as_char()
            };
            send_char(&socket, last_sent_char, char_to_send).await.unwrap();
            last_sent_char = char_to_send;
            
            //TODO: this line is repeated above, figure out how to deduplicate it
            next_display_value = (*rx.borrow()).replace(|c: char| !c.is_ascii(), "");
        }

    }
}

async fn send_char(socket: &tokio::net::UdpSocket, last_sent_char: char, c: char) -> Result<(), std::io::Error> {
    if last_sent_char == c {
        send_char_no_throttle(socket, AsciiChar::Null.as_char()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    send_char_no_throttle(socket, c).await?;
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    Ok(())
}

async fn send_char_no_throttle(socket: &tokio::net::UdpSocket, c: char) -> Result<usize, std::io::Error> {
    socket.send(
        &encoder::encode(
            &OscPacket::Message(OscMessage {
                addr: "/avatar/parameters/7Display".to_string(),
                args: vec![
                    OscType::Int(c as i32)
                ],
            })
        ).expect("Couldn't encode an OscMessage")
    ).await
}