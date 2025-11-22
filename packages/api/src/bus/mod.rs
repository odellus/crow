use std::sync::Arc;
use tokio::sync::broadcast;

pub mod events;

/// Event payload that gets broadcast
#[derive(Clone, Debug, serde::Serialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}

/// Global event bus with broadcast channel
pub struct EventBus {
    /// Broadcast channel for events (1000 event buffer)
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event_type: &str, properties: serde_json::Value) {
        let event = Event {
            event_type: event_type.to_string(),
            properties,
        };
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to events, returns a receiver
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Global singleton
lazy_static::lazy_static! {
    pub static ref BUS: EventBus = EventBus::new();
}

/// Publish helper function
pub fn publish(event_type: &str, properties: serde_json::Value) {
    BUS.publish(event_type, properties);
}

/// Subscribe helper function
pub fn subscribe() -> broadcast::Receiver<Event> {
    BUS.subscribe()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish("test.event", serde_json::json!({"foo": "bar"}));

        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.properties["foo"], "bar");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish("test.event", serde_json::json!({"value": 42}));

        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();

        assert_eq!(event1.event_type, event2.event_type);
        assert_eq!(event1.properties, event2.properties);
    }
}
