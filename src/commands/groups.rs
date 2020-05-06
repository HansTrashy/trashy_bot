pub mod general {
    use crate::commands::{
        about::*, choose::*, emoji::*, poll::*, quote::*, remindme::*, roll::*, selfmute::*,
        spongebob::*, userinfo::*, xkcd::*,
    };
    use serenity::framework::standard::macros::group;

    #[group]
    #[commands(
        about, roll, choose, xkcd, quote, userinfo, remindme, spongebob, selfmute, katzer, poll
    )]
    pub struct General;
}

pub mod lastfm {
    use crate::commands::lastfm::*;
    use serenity::framework::standard::macros::group;

    #[group]
    #[prefix("lastfm")]
    #[commands(register, now, recent, artists, albums, tracks)]
    pub struct Lastfm;
}

pub mod misc {
    use crate::commands::{copypasta::*, emoji::*, optout::*, shiny::*};
    use serenity::framework::standard::macros::group;

    #[group]
    #[prefix("misc")]
    #[commands(shiny, list, setshiny, removeshiny, goldt, emoji, optout)]
    pub struct Misc;
}

pub mod config {
    use crate::commands::config::*;
    use serenity::framework::standard::macros::group;

    #[group]
    #[prefix("cfg")]
    #[commands(show_config, set_modlog, set_muterole, set_userlog)]
    pub struct Config;
}

pub mod moderation {
    use crate::commands::moderation::*;
    use serenity::framework::standard::macros::group;
    #[group]
    #[prefix("mod")]
    #[commands(mute, unmute, kick, ban)]
    pub struct Moderation;
}

pub mod account {
    use crate::commands::account::{general::*, slot::*};
    use serenity::framework::standard::macros::group;

    #[group]
    #[prefix("acc")]
    #[commands(create, payday, leaderboard, transfer, slot)]
    pub struct Account;
}

pub mod greenbook {
    use crate::commands::fav::*;
    use serenity::framework::standard::macros::group;

    #[group]
    #[prefix("fav")]
    #[default_command(post)]
    #[commands(post, untagged, add, tags)]
    pub struct Greenbook;
}

// pub mod rules {
//     use crate::commands::rules::*;
//     use serenity::framework::standard::macros::group;

//     #[group]
//     #[prefix("rules")]
//     #[commands(de, en, setde, addde, seten, adden, post)]
//     pub struct Rules;
// }

pub mod roles {
    use crate::commands::reaction_roles::*;
    use serenity::framework::standard::macros::group;

    #[group]
    #[prefix("roles")]
    #[commands(list, create, remove, postgroups)]
    pub struct Roles;
}
