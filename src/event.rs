use anyhow::Result;
use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use iwdrs::modes::Mode;
use tokio::sync::mpsc;

use crate::notification::Notification;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Notification(Notification),
    Reset(Mode),
    Auth(String),
    EapNeworkConfigured(String),
    ConfigureNewEapNetwork(String),
    AuthRequestPassword((String, Option<String>)),
    AuthReqKeyPassphrase(String),
    AuthReqUsernameAndPassword(String),
    UsernameAndPasswordSubmit,
    ConnectToHiddenNetwork(String),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    pub sender: mpsc::UnboundedSender<Event>,
    pub receiver: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_cloned = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  () = sender_cloned.closed() => {
                    break;
                  }
                  _ = tick_delay => {
                    sender_cloned.send(Event::Tick).unwrap();
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      CrosstermEvent::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          sender_cloned.send(Event::Key(key)).unwrap();
                        }
                      },
                      CrosstermEvent::Resize(x, y) => {
                        sender_cloned.send(Event::Resize(x, y)).unwrap();
                      },
                      _ => {}
                    }
                  }
                };
            }
        });
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(std::io::Error::other("This is an IO error").into())
    }
}
