table! {
    banks (id) {
        id -> Int8,
        user_id -> Int8,
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
    tags (id) {
        id -> Int8,
        fav_id -> Int8,
        label -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    banks,
    favs,
    tags,
);
