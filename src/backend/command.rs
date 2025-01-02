use std::cmp::Ordering;

#[derive(Eq, PartialEq, Debug)]
pub enum BackendCommand {
    Shutdown,
    IncreaseCounter,
    DecreaseCounter,
    GetCounterValue,
}

#[derive(Eq, PartialEq, Debug)]
pub enum CommandPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Eq, PartialEq, Debug)]
pub struct PrioritizedCommand {
    priority: CommandPriority,
    cmd: BackendCommand,
}

impl PrioritizedCommand {
    pub fn command(&self) -> &BackendCommand {
        &self.cmd
    }
    pub fn priority(&self) -> &CommandPriority {
        &self.priority
    }
}

impl BackendCommand {
    pub fn low(self) -> PrioritizedCommand {
        PrioritizedCommand {
            priority: CommandPriority::Low,
            cmd: self,
        }
    }
    pub fn medium(self) -> PrioritizedCommand {
        PrioritizedCommand {
            priority: CommandPriority::Medium,
            cmd: self,
        }
    }
    pub fn high(self) -> PrioritizedCommand {
        PrioritizedCommand {
            priority: CommandPriority::High,
            cmd: self,
        }
    }
    pub fn critical(self) -> PrioritizedCommand {
        PrioritizedCommand {
            priority: CommandPriority::Critical,
            cmd: self,
        }
    }
}

impl PartialOrd for PrioritizedCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for PrioritizedCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        use CommandPriority as C;
        use Ordering as O;
        match (&self.priority, &other.priority) {
            (C::Low, C::Low) => O::Equal,
            (C::Low, C::Medium) => O::Less,
            (C::Low, C::High) => O::Less,
            (C::Low, C::Critical) => O::Less,
            (C::Medium, C::Low) => O::Greater,
            (C::Medium, C::Medium) => O::Equal,
            (C::Medium, C::High) => O::Less,
            (C::Medium, C::Critical) => O::Less,
            (C::High, C::Low) => O::Greater,
            (C::High, C::Medium) => O::Greater,
            (C::High, C::High) => O::Equal,
            (C::High, C::Critical) => O::Less,
            (C::Critical, C::Low) => O::Greater,
            (C::Critical, C::Medium) => O::Greater,
            (C::Critical, C::High) => O::Greater,
            (C::Critical, C::Critical) => O::Equal,
        }
    }
}
