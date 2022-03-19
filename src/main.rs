// use std::net::UdpSocket;
use rosc::{self, encoder, OscPacket, OscMessage, OscType};
use ascii::{AsciiChar};
use std::{time::Duration, net::UdpSocket};
//use tokio::{time, task};
//use tokio::net::UdpSocket;

use eframe::{epi, egui};

struct Egui7DisplayApp {
    text: String,
    socket: UdpSocket,
}

impl epi::App for Egui7DisplayApp {
    fn name(&self) -> &str {
        "7Display"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.spacing_mut().item_spacing = Vec2::new(1f32, 200f32);
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                let last_text = self.text.clone();
                let last_text_len = last_text.len();
                let response = ui.add(egui::TextEdit::singleline(&mut self.text));
                let text_len = self.text.len();
                if response.changed() {
                    if text_len == 0 {
                        self.send_char(AsciiChar::LineFeed.as_char());
                    } else if text_len > last_text_len {
                        let last_char = self.text.chars().last().unwrap();
                        self.send_char(last_char);
                    } else if text_len < last_text_len {
                        self.send_char(AsciiChar::BackSpace.as_char());
                    }
                }
                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    // …
                }
            });
            
        });
    }
}

impl Egui7DisplayApp {
    fn send_char(&self, c: char) {
        self.socket.send(&encoder::encode(&OscPacket::Message(OscMessage {
            addr: "/avatar/parameters/7Display".to_string(),
            args: vec![
                OscType::Int(c as i32)
            ],
        })).unwrap()).unwrap();
    }
    fn sync_display(&self) {

    }
    fn init() -> Egui7DisplayApp {
        let app = Egui7DisplayApp {
            text: "".to_string(),
            socket: UdpSocket::bind("127.0.0.1:9002").unwrap(),
        };
    
        app.socket.connect("127.0.0.1:9000").unwrap();
        
        //let mut interval = time::interval(Duration::from_millis(10));

        // let forever_app = &app;
        // let forever = task::spawn(async {
            
        //     let mut interval = time::interval(Duration::from_millis(10));
    
        //     loop {
        //         interval.tick();
        //         forever_app.sync_display();
        //     }
        // });
                
        app
    }
}

fn main() {
    let app = Egui7DisplayApp::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}