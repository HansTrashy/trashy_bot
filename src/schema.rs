table! {
    banks (id) {
        id -> Int8,
        user_id -> Int8,
        user_name -> Text,
        amount -> Int8,
        last_payday -> Timestamp,
    }
}

table! {
    favs (id) {
        id -> Int8,
        server_id -> Int8,
        channel_id -> Int8,
        msg_id -> Int8,
        user_id -> Int8,
        author_id -> Int8,
    }
}

table! {
    reaction_roles (id) {
        id -> Int8,
        server_id -> Int8,
        role_id -> Int8,
        role_name -> Text,
        role_group -> Text,
        emoji -> Text,
    }
}

table! {
    tags (id) {
        id -> Int8,
        fav_id -> Int8,
        label -> Text,
    }
}

table! {
    twitch_configs (id) {
        id -> Int8,
        guild_id -> Int8,
        channel_ids -> Array<Int8>,
        delete_offline -> Bool,
        allow_everyone -> Bool,
    }
}

table! {
    twitch_streams (id) {
        id -> Int8,
        twitch_user_id -> Text,
        profile_image_url -> Text,
    }
}

table! {
    twitch_subs (id) {
        id -> Int8,
        twitch_stream_id -> Int8,
        channel_id -> Int8,
        user_id -> Int8,
        message_id -> Nullable<Int8>,
    }
}

joinable!(twitch_subs -> twitch_streams (twitch_stream_id));

allow_tables_to_appear_in_same_query!(
    banks,
    favs,
    reaction_roles,
    tags,
    twitch_configs,
    twitch_streams,
    twitch_subs,
);
