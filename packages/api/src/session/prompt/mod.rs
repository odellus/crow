//! Session prompt reminders and injections

/// Plan mode read-only reminder (matches OpenCode exactly)
pub const PROMPT_PLAN: &str = include_str!("plan.txt");

/// Insert agent-specific reminders into user messages
/// This matches OpenCode's insertReminders() function
pub fn insert_reminders(agent_name: &str) -> Option<String> {
    match agent_name {
        "plan" => Some(PROMPT_PLAN.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_reminder() {
        let reminder = insert_reminders("plan");
        assert!(reminder.is_some());
        assert!(reminder.unwrap().contains("READ-ONLY"));
    }

    #[test]
    fn test_no_reminder_for_build() {
        let reminder = insert_reminders("build");
        assert!(reminder.is_none());
    }
}
