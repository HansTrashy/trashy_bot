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
    server_configs (id) {
        id -> Int4,
        server_id -> Int4,
    }
}

table! {
    server_settings (id) {
        id -> Int4,
        server_config_id -> Int4,
        key -> Text,
        value -> Text,
    }
}

table! {
    tags (id) {
        id -> Int8,
        fav_id -> Int8,
        label -> Text,
    }
}

joinable!(server_settings -> server_configs (server_config_id));

allow_tables_to_appear_in_same_query!(
    banks,
    favs,
    reaction_roles,
    server_configs,
    server_settings,
    tags,
);
