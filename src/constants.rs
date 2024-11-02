
pub(crate) const HELP_CMD: &str = "!help";

pub(crate) const ANY_GAMERS_CMD: &str = "!any-gamers";
pub(crate) const REGISTER_CMD: &str = "!register";
pub(crate) const GAME_NOTIFICATION_ON_CMD: &str = "!game-notification-on";
pub(crate) const GAME_NOTIFICATION_OFF_CMD: &str = "!game-notification-off";
pub(crate) const ADD_ADMINS_CMD: &str = "!admin";

pub(crate) fn help_text() -> String {
    format!("Here are my commands:
{HELP_CMD}: show this message
{REGISTER_CMD}: add yourself to the list of users I interact with
{GAME_NOTIFICATION_ON_CMD}: enable notifications in the current channel when another registered user invokes the {ADD_ADMINS_CMD} command
{GAME_NOTIFICATION_OFF_CMD}: disable game search notifications in the current channel
{ANY_GAMERS_CMD}: send a dm to all registered users who have enabled game notifications in the current channel
-----------ADMIN ONLY------------
{ADD_ADMINS_CMD}: adds all mentioned users as admins. For example, '{ADD_ADMINS_CMD} @<some guy> would add <some guy> as an admin")
}
