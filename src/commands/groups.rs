pub mod general {
    use crate::commands::{
        about::*, choose::*, quote::*, remindme::*, roll::*, selfmute::*, spongebob::*,
        userinfo::*, xkcd::*,
    };
    use serenity::framework::standard::macros::{check, command, group, help};

    group!({
        name: "general",
        options: {},
        commands: [about, roll, choose, xkcd, quote, userinfo, remindme, spongebob, selfmute,],
    });
}

pub mod lastfm {
    use crate::commands::lastfm::*;
    use serenity::framework::standard::macros::{check, command, group, help};

    group!({
        name: "lastfm",
        options: {
            prefix: "lastfm",
            description: "Lastfm commands",
        },
        commands: [register, now]
    });
}

pub mod misc {
    use crate::commands::{copypasta::*, shiny::*};
    use serenity::framework::standard::macros::{check, command, group, help};

    group!({
        name: "misc",
        options: {
            prefix: "misc",
            description: "Miscellaneous commands",
        },
        commands: [shiny, list, setshiny, removeshiny, goldt]
    });
}

pub mod config {
    use crate::commands::config::*;
    use serenity::framework::standard::macros::{check, command, group, help};

    group!({
        name: "config",
        options: {
            prefix: "cfg",
            description: "Config commands",
            default_command: status,
        },
        commands: [status, show_config, set_modlog, set_muterole, set_userlog]
    });
}

pub mod moderation {
    use crate::commands::moderation::*;
    use serenity::framework::standard::macros::{check, command, group, help};

    group!({
        name: "moderation",
        options: {
            prefix: "mod",
            description: "Moderation commands",
            allowed_roles: [
                "Mods",
            ]
        },
        commands: [mute, unmute, kick, ban]
    });
}

pub mod account {
    use crate::commands::account::{blackjack::*, general::*, slot::*};
    use serenity::framework::standard::macros::{check, command, group, help};
    group!({
        name: "account",
        options: {
            prefix: "acc",
            description: "Having fun with some games",
            default_command: payday,
        },
        commands: [createaccount, payday, leaderboard, transfer, slot, blackjack]
    });
}

pub mod greenbook {
    use crate::commands::fav::*;
    use serenity::framework::standard::macros::{check, command, group, help};
    group!({
        name: "greenbook",
        options: {
            prefix: "fav",
            description: "Saving your favourite messages.",
            default_command: post,
        },
        commands: [post, untagged, add, tags],
    });
}

pub mod rules {
    use crate::commands::rules::*;
    use serenity::framework::standard::macros::{check, command, group, help};
    group!({
        name: "rules",
        options: {
            prefix: "rules",
            description: "Rules to be sent by the bot",
            default_command: de,
        },
        commands: [de, en, setde, addde, seten, adden, post],
    });
}

pub mod reaction_roles {
    use crate::commands::reaction_roles::*;
    use serenity::framework::standard::macros::{check, command, group, help};
    group!({
        name: "reaction_roles",
        options: {
            prefix: "rr",
            description: "Let users easily add roles to themselves with reactions",
            default_command: list,
        },
        commands: [list, create, remove, postgroups],
    });
}

pub mod voice {
    use crate::commands::voice::*;
    use serenity::framework::standard::macros::{check, command, group, help};
    group!({
        name: "voice",
        options: {
            prefix: "v",
            description: "Let the bot sing for you",
        },
        commands: [deafen, join, leave, mute, play, undeafen, unmute]
    });
}
