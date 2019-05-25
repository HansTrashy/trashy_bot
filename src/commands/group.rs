use serenity::framework::standard::{macros::{command, group, help, check}};

use super::{about::*, roll::*, choose::*, xkcd::*, quote::*, fav::*};

group!({
    name: "general",
    options: {},
    commands: [about, roll, choose, xkcd, quote],
});

group!({
    name: "greenbook",
    options: {
        prefix: "fav",
        description: "Saving your favourite messages.",
        default_command: fav,
    },
    commands: [],
});

group!({
    name: "rules",
    options: {},
    commands: [],
});

group!({
    name: "reaction_roles",
    options: {},
    commands: [],
});

