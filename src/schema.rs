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
    mutes (id) {
        id -> Int8,
        server_id -> Int8,
        user_id -> Int8,
        end_time -> Timestamptz,
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
        id -> Int8,
        server_id -> Int8,
        config -> Jsonb,
    }
}

table! {
    tags (id) {
        id -> Int8,
        fav_id -> Int8,
        label -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    banks,
    favs,
    mutes,
    reaction_roles,
    server_configs,
    tags,
);
