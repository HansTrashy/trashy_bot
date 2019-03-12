use crate::schema::banks;
use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable, Debug)]
pub struct Bank {
    pub id: i64,
    pub user_id: i64,
    pub amount: i64,
    pub last_payday: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "banks"]
pub struct NewBank {
    user_id: i64,
    amount: i64,
    last_payday: NaiveDateTime,
}

pub fn create_bank(
    conn: &PgConnection,
    user_id: i64,
    amount: i64,
    last_payday: NaiveDateTime,
) -> Bank {
    use crate::schema::banks;

    let new_bank = NewBank {
        user_id,
        amount,
        last_payday,
    };

    diesel::insert_into(banks::table)
        .values(&new_bank)
        .get_result(conn)
        .expect("Error saving bank")
}