use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use iwdrs::modes::Mode;
use tokio::sync::mpsc;

use crate::{app::AppResult, notification::Notification};

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Notification(Notification),
    Reset(Mode),
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
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = _sender.closed() => {
                    break;
                  }
                  _ = tick_delay => {
                    _sender.send(Event::Tick).unwrap();
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      CrosstermEvent::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          _sender.send(Event::Key(key)).unwrap();
                        }
                      },
                      CrosstermEvent::Resize(x, y) => {
                        _sender.send(Event::Resize(x, y)).unwrap();
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

    pub async fn next(&mut self) -> AppResult<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(Box::new(std::io::Error::other("This is an IO error")))
    }
}
