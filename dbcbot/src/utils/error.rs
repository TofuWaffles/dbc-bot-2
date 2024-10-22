#[derive(Debug)]
pub enum CommonError {
    NoSelection,
    NotInAGuild,
    RoleNotExists(String),
    ChannelNotExists(String),
    UserNotExists(String),
    GuildNotExists(String),
    RoundNotExists(String),
    MatchNotExists(String),
    TournamentNotExists(String),
    UnableToSendMessage,
    UnableToEditMessage,
    UnableToAssignRole,
}

impl std::fmt::Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CommonError::*;
        match self {
            NoSelection => write!(f, "No selection was made."),
            NotInAGuild => write!(f, "You are not in a guild."),
            RoleNotExists(id) => write!(f, "Role {} does not exist.", id),
            RoundNotExists(id) => write!(f, "Round {} does not exist.", id),
            MatchNotExists(id) => write!(f, "Match {} does not exist.", id),
            ChannelNotExists(id) => write!(f, "Channel {} does not exist.", id),
            UserNotExists(id) => write!(f, "User {} does not exist.", id),
            GuildNotExists(id) => write!(f, "Guild {} does not exist.", id),
            TournamentNotExists(id) => write!(f, "Tournament {} not exist.", id),
            UnableToSendMessage => write!(f, "Unable to send message."),
            UnableToEditMessage => write!(f, "Unable to edit message."),
            UnableToAssignRole => write!(f, "Unable to assign role."),
        }
    }
}

impl std::error::Error for CommonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use CommonError::*;
        match self {
            NoSelection => Some(self),
            NotInAGuild => Some(self),
            RoleNotExists(_) => Some(self),
            ChannelNotExists(_) => Some(self),
            UserNotExists(_) => Some(self),
            GuildNotExists(_) => Some(self),
            RoundNotExists(_) => Some(self),
            MatchNotExists(_) => Some(self),
            TournamentNotExists(_) => Some(self),
            UnableToSendMessage => Some(self),
            UnableToEditMessage => Some(self),
            UnableToAssignRole => Some(self),
        }
    }
}
