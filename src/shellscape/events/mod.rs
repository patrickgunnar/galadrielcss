use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info, warn};

/// Enum representing the types of terminal events for Shellscape.
#[allow(dead_code)]
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
    // The task handler that manages the event loop.
    handler: tokio::task::JoinHandle<()>,
    // A sender used to transmit events to other components.
    pub sender: UnboundedSender<ShellscapeTerminalEvents>,
}

impl ShellscapeEvents {
    /// Creates a new `ShellscapeEvents` instance with the specified tick rate and sender.
    /// This function spawns a new asynchronous task to handle terminal events.
    ///
    /// # Parameters
    /// - `tick_rate`: The rate at which the "Tick" event is generated, in milliseconds.
    /// - `sender`: A sender to communicate terminal events.
    ///
    /// # Returns
    /// A `ShellscapeEvents` struct containing the handler and sender.
    pub fn new(tick_rate: u64, sender: UnboundedSender<ShellscapeTerminalEvents>) -> Self {
        // Convert the tick rate to a Tokio Duration
        let tick_rate = tokio::time::Duration::from_millis(tick_rate);
        // Clone the sender to use it in the spawned task
        let _sender = sender.clone();

        // Spawn a new asynchronous task that listens for events
        let handler = tokio::spawn(async move {
            // Create an event stream reader from crossterm for capturing terminal events
            let mut reader = crossterm::event::EventStream::new();
            // Create a Tokio interval for the tick event based on the specified tick rate
            let mut tick = tokio::time::interval(tick_rate);

            info!(
                "Shellscape terminal events handler started with tick rate of {} ms",
                tick_rate.as_millis()
            );

            loop {
                // Await the next tick or event
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    // Handle channel closure: If the sender is closed, exit the loop.
                    _ = _sender.closed() => {
                        warn!("Shellscape terminal events handler detected that the sender channel is closed. Exiting loop.");

                        break;
                    }
                    // On tick event: Send the "Tick" event to the sender.
                    _ = tick_delay => {
                        if let Err(err) = _sender.send(ShellscapeTerminalEvents::Tick) {
                            error!("Failed to send Tick event to Shellscape terminal event receiver: {}", err);
                        }
                    }
                    // On crossterm event: Handle key, mouse, or resize events.
                    Some(Ok(event)) = crossterm_event => {
                        match event {
                            // Handle key press events, only processing 'Press' events.
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    if let Err(err) = _sender.send(ShellscapeTerminalEvents::Key(key)) {
                                        error!("Failed to send Key event {:?} to Shellscape terminal event receiver: {}", key, err);
                                    } else {
                                        info!("Key event {:?} sent to Shellscape terminal event receiver", key);
                                    }
                                }
                            }
                            // Handle mouse events, sending them through the sender.
                            CrosstermEvent::Mouse(mouse) => {
                                if let Err(err) = _sender.send(ShellscapeTerminalEvents::Mouse(mouse)) {
                                    error!("Failed to send Mouse event {:?} to Shellscape terminal event receiver: {}", mouse, err);
                                }
                                /*else {
                                    info!("Mouse event {:?} sent to Shellscape terminal event receiver", mouse);
                                }*/
                            }
                            // Handle terminal resize events, sending them with width and height.
                            /*CrosstermEvent::Resize(x, y) => {
                                if let Err(err) = _sender.send(ShellscapeTerminalEvents::Resize(x, y)) {
                                    error!("Failed to send Resize event (width: {}, height: {}) to Shellscape terminal event receiver: {}", x, y, err);
                                } else {
                                    info!("Resize event (width: {}, height: {}) sent to Shellscape terminal event receiver", x, y);
                                }
                            }*/
                            _ => {
                                warn!("Unhandled event type received from crossterm");
                            }
                        }
                    }
                }
            }
        });

        // Return a ShellscapeEvents instance containing the handler and sender
        Self { handler, sender }
    }
}

#[cfg(test)]
mod tests {
    use super::{ShellscapeEvents, ShellscapeTerminalEvents};
    use std::{
        io::ErrorKind,
        pin::Pin,
        task::{Context, Poll},
    };

    use crossterm::{
        self,
        event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers},
    };
    use futures::{Stream, StreamExt};
    use tokio::sync::mpsc;

    struct MockEventStream {
        events: Vec<CrosstermEvent>,
    }

    impl Stream for MockEventStream {
        type Item = Result<CrosstermEvent, ErrorKind>;

        fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if let Some(event) = self.events.pop() {
                Poll::Ready(Some(Ok(event)))
            } else {
                Poll::Pending
            }
        }
    }

    #[tokio::test]
    async fn test_shellscape_events_tick_event() {
        let (sender, mut receiver) = mpsc::unbounded_channel();

        let tick_rate = 100;
        let _shellscape_events = ShellscapeEvents::new(tick_rate, sender.clone());

        if let Some(event) = receiver.recv().await {
            assert_eq!(event, ShellscapeTerminalEvents::Tick);
        } else {
            panic!("Expected tick event, but none received");
        }
    }

    #[tokio::test]
    async fn test_shellscape_events_key_event() {
        let (sender, mut receiver) = mpsc::unbounded_channel();

        let mut mock_event_stream = MockEventStream {
            events: vec![CrosstermEvent::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            ))],
        };

        let handler = tokio::spawn(async move {
            if let Some(event) = mock_event_stream.next().await {
                if let CrosstermEvent::Key(key) = event.unwrap() {
                    sender.send(ShellscapeTerminalEvents::Key(key)).unwrap();
                }
            }
        });

        if let Some(ShellscapeTerminalEvents::Key(key)) = receiver.recv().await {
            assert_eq!(key.code, KeyCode::Enter);
        } else {
            panic!("Expected key event, but none received");
        }

        handler.await.unwrap();
    }

    #[tokio::test]
    async fn test_shellscape_events_resize_event() {
        let (sender, mut receiver) = mpsc::unbounded_channel();

        let mut mock_event_stream = MockEventStream {
            events: vec![CrosstermEvent::Resize(80, 24)],
        };

        let handler = tokio::spawn(async move {
            if let Some(event) = mock_event_stream.next().await {
                if let CrosstermEvent::Resize(x, y) = event.unwrap() {
                    sender.send(ShellscapeTerminalEvents::Resize(x, y)).unwrap();
                }
            }
        });

        if let Some(ShellscapeTerminalEvents::Resize(width, height)) = receiver.recv().await {
            assert_eq!((width, height), (80, 24));
        } else {
            panic!("Expected resize event, but none received");
        }

        handler.await.unwrap();
    }
}
