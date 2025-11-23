/// Event type constants matching OpenCode

// Session events
pub const SESSION_CREATED: &str = "session.created";
pub const SESSION_UPDATED: &str = "session.updated";
pub const SESSION_DELETED: &str = "session.deleted";
pub const SESSION_ERROR: &str = "session.error";

// Session status
pub const SESSION_STATUS: &str = "session.status";
pub const SESSION_IDLE: &str = "session.idle";

// Message events
pub const MESSAGE_UPDATED: &str = "message.updated";
pub const MESSAGE_REMOVED: &str = "message.removed";
pub const MESSAGE_PART_UPDATED: &str = "message.part.updated";
pub const MESSAGE_PART_REMOVED: &str = "message.part.removed";

// File events
pub const FILE_EDITED: &str = "file.edited";

// Server events
pub const SERVER_CONNECTED: &str = "server.connected";
