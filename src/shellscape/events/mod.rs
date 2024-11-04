use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info, warn};

#[derive(Clone, PartialEq, Debug)]
pub enum ShellscapeTerminalEvents {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ShellscapeEvents {
    handler: tokio::task::JoinHandle<()>,
    pub sender: UnboundedSender<ShellscapeTerminalEvents>,
}

impl ShellscapeEvents {
    pub fn new(tick_rate: u64, sender: UnboundedSender<ShellscapeTerminalEvents>) -> Self {
        let tick_rate = tokio::time::Duration::from_millis(tick_rate);
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);

            info!(
                "Shellscape terminal events handler started with tick rate of {} ms",
                tick_rate.as_millis()
            );

            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = _sender.closed() => {
                        warn!("Shellscape terminal events handler detected that the sender channel is closed. Exiting loop.");

                        break;
                    }
                    _ = tick_delay => {
                        if let Err(err) = _sender.send(ShellscapeTerminalEvents::Tick) {
                            error!("Failed to send Tick event to Shellscape terminal event receiver: {}", err);
                        } else {
                            info!("Tick event sent to Shellscape terminal event receiver");
                        }
                    }
                    Some(Ok(event)) = crossterm_event => {
                        match event {
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    if let Err(err) = _sender.send(ShellscapeTerminalEvents::Key(key)) {
                                        error!("Failed to send Key event {:?} to Shellscape terminal event receiver: {}", key, err);
                                    } else {
                                        info!("Key event {:?} sent to Shellscape terminal event receiver", key);
                                    }
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                if let Err(err) = _sender.send(ShellscapeTerminalEvents::Mouse(mouse)) {
                                    error!("Failed to send Mouse event {:?} to Shellscape terminal event receiver: {}", mouse, err);
                                } else {
                                    info!("Mouse event {:?} sent to Shellscape terminal event receiver", mouse);
                                }
                            }
                            CrosstermEvent::Resize(x, y) => {
                                if let Err(err) = _sender.send(ShellscapeTerminalEvents::Resize(x, y)) {
                                    error!("Failed to send Resize event (width: {}, height: {}) to Shellscape terminal event receiver: {}", x, y, err);
                                } else {
                                    info!("Resize event (width: {}, height: {}) sent to Shellscape terminal event receiver", x, y);
                                }
                            }
                            _ => {
                                warn!("Unhandled event type received from crossterm");
                            }
                        }
                    }
                }
            }
        });

        Self { handler, sender }
    }
}
