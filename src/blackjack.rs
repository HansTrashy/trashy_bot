use diesel::pg::PgConnection;
use log::*;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use serenity::model::id::ChannelId;
use serenity::prelude::Mutex;
use std::fmt;
use std::sync::Arc;

pub struct State(Vec<Game>);

impl State {
    fn save(&self) {
        std::fs::write(
            "blackjack_state.storage",
            serde_json::to_string(&self.0).expect("Could not serialize blackjack state"),
        )
        .expect("coult not write blackjack state to file");
    }

    pub fn add_game(
        &mut self,
        conn: Arc<Mutex<PgConnection>>,
        user: u64,
        bet: i64,
        channel_id: u64,
        message_id: u64,
    ) {
        self.0.retain(|g| g.player != user);
        self.0
            .push(Game::new(conn, user, bet, channel_id, message_id));
        self.save();
        self.new_game(user, message_id);
    }

    pub fn hit(&mut self, user: u64, message_id: u64) {
        for g in self.0.iter_mut() {
            if g.player == user && g.message_id == message_id {
                g.update(PlayerAction::Hit);
                break;
            }
        }
    }

    pub fn stay(&mut self, user: u64, message_id: u64) {
        for g in self.0.iter_mut() {
            if g.player == user && g.message_id == message_id {
                g.update(PlayerAction::Stay);
                break;
            }
        }
    }

    pub fn new_game(&mut self, user: u64, message_id: u64) {
        for g in self.0.iter_mut() {
            if g.player == user && g.message_id == message_id {
                g.update(PlayerAction::Init);
                break;
            }
        }
    }

    pub fn load(conn: Arc<Mutex<PgConnection>>) -> Self {
        match std::fs::read_to_string("blackjack_state.storage") {
            Ok(data) => {
                let mut games: Vec<Game> =
                    serde_json::from_str(&data).expect("could not deserialize blackjack state");
                for g in games.iter_mut() {
                    g.conn = Some(conn.clone());
                }
                Self(games)
            }
            Err(_e) => Self(Vec::default()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy, Serialize, Deserialize)]
enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::Spades => "♠",
                Suit::Hearts => "♥",
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Copy, Serialize, Deserialize)]
enum Color {
    Red,
    Black,
}

#[derive(Clone, Debug, PartialEq, Copy, Serialize, Deserialize)]
enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::Two => "2",
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "10",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    fn color(&self) -> Color {
        match self.suit {
            Suit::Hearts | Suit::Diamonds => Color::Red,
            Suit::Clubs | Suit::Spades => Color::Black,
        }
    }

    fn value(&self) -> usize {
        match self.rank {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 10,
            Rank::Queen => 10,
            Rank::King => 10,
            Rank::Ace => 11,
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|{}{}|", self.suit, self.rank,)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Deck(Vec<Card>);

impl Deck {
    fn new() -> Self {
        let mut cards: Vec<Card> = Vec::with_capacity(52);
        for s in &[Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            for r in &[
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ] {
                cards.push(Card::new(*s, *r));
            }
        }
        Self(cards)
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.0.shuffle(&mut rng);
    }

    fn draw(&mut self) -> Option<Card> {
        self.0.pop()
    }

    fn draw_multiple(&mut self, n: usize) -> Vec<Card> {
        let mut stack = Vec::new();
        for i in 0..n {
            if let Some(c) = self.draw() {
                stack.push(c);
            } else {
                break;
            }
        }
        stack
    }

    fn add_cards(&mut self, cards: &mut Vec<Card>) {
        self.0.append(cards);
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn size(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Stack(Vec<Card>);

impl Stack {
    fn new() -> Self {
        Self(Vec::default())
    }

    fn add(&mut self, card: Card) {
        self.0.push(card);
    }

    fn clear_stack(&mut self) -> Vec<Card> {
        self.0.drain(..).collect()
    }

    fn value(&self) -> usize {
        if self.0.iter().all(|c| c.rank != Rank::Ace) {
            self.0.iter().fold(0, |acc, c| acc + c.value())
        } else {
            let ace_count = self.0.iter().filter(|c| c.rank == Rank::Ace).count();
            let mut possible_values: Vec<usize> = (0..=ace_count)
                .map(|n| self.0.iter().fold(0, |acc, c| acc + c.value()) - n * 10)
                .collect();
            possible_values.sort();
            possible_values.reverse();
            for p in &possible_values {
                if *p <= 21 {
                    return *p;
                }
            }
            *possible_values.last().unwrap()
        }
    }

    fn is_blackjack(&self) -> bool {
        self.value() == 21 && self.0.len() == 2
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut stack = String::new();
        for c in &self.0 {
            stack.push_str(&c.to_string());
            stack.push_str(" ");
        }
        write!(f, "{}", stack)
    }
}

#[derive(Debug, Clone, Copy)]
enum PlayerAction {
    Init,
    Hit,
    Stay,
}

#[derive(Serialize, Deserialize)]
struct Game {
    deck: Deck,
    removed_cards: Vec<Card>,
    player: u64,
    bet: i64,
    bank_cards: Stack,
    player_cards: Stack,
    channel_id: u64,
    message_id: u64,
    state: GameState,
    #[serde(skip)]
    conn: Option<Arc<Mutex<PgConnection>>>,
}

impl Game {
    fn new(
        conn: Arc<Mutex<PgConnection>>,
        player: u64,
        bet: i64,
        channel_id: u64,
        message_id: u64,
    ) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();
        Self {
            deck,
            removed_cards: Vec::with_capacity(52),
            player,
            bet,
            bank_cards: Stack::new(),
            player_cards: Stack::new(),
            channel_id,
            message_id,
            state: GameState::Draw,
            conn: Some(conn),
        }
    }

    fn update(&mut self, action: PlayerAction) {
        // Have at least 15 cards remaining, or add removed_cards and shuffle
        if !self.deck.size() >= 15 {
            self.deck.add_cards(&mut self.removed_cards);
            self.deck.shuffle();
        }

        let player_account = PlayerAccount(self.conn.clone().unwrap(), self.player);
        let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();

        match action {
            PlayerAction::Init
                if self.state == GameState::BankWins
                    || self.state == GameState::Draw
                    || self.state == GameState::PlayerWins
                    || self.state == GameState::PlayerWinsThreeToTwo =>
            {
                if player_account.can_pay(self.bet) {
                    player_account.change_amount(-self.bet);

                    // move cards from hand to removed_cards
                    self.removed_cards
                        .append(&mut self.player_cards.clear_stack());
                    self.removed_cards
                        .append(&mut self.bank_cards.clear_stack());

                    // start with a fresh draw
                    self.bank_cards.add(self.deck.draw().unwrap());
                    self.player_cards.add(self.deck.draw().unwrap());

                    // visualize state
                    msg.edit(|m| m.content(&self.visualize(GameState::Playing)))
                        .unwrap();

                    self.state = GameState::Playing;
                } else {
                    msg.edit(|m| m.content(&self.visualize(GameState::NotEnoughMoney)))
                        .unwrap();
                    self.state = GameState::NotEnoughMoney;
                }
            }
            PlayerAction::Hit if self.state == GameState::Playing => {
                self.player_cards.add(self.deck.draw().unwrap());

                // make card checks
                if self.player_cards.value() > 21 {
                    // bust
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::BankWins)))
                        .unwrap();

                    player_account.change_amount(-self.bet);
                    self.state = GameState::BankWins;
                } else if self.player_cards.value() == 21 {
                    // blackjack or triple
                    if self.player_cards.0.iter().all(|c| c.rank == Rank::Seven) {
                        let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                        msg.edit(|m| m.content(&self.visualize(GameState::PlayerWinsThreeToTwo)))
                            .unwrap();

                        player_account.change_amount(3 * self.bet);
                        self.state = GameState::PlayerWinsThreeToTwo;
                    } else {
                        self.update(PlayerAction::Stay);
                    }
                } else {
                    // allow more draws
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::Playing)))
                        .unwrap();
                }
            }
            PlayerAction::Stay if self.state == GameState::Playing => {
                // make remaining draws for the bank
                let mut bank_value = self.bank_cards.value();
                while bank_value <= 17 {
                    self.bank_cards.add(self.deck.draw().unwrap());
                    bank_value = self.bank_cards.value();
                }

                // check winner
                // Both have Blackjack
                if self.bank_cards.is_blackjack() && self.player_cards.is_blackjack() {
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::Draw)))
                        .unwrap();

                    player_account.change_amount(self.bet);
                    self.state = GameState::Draw;
                // Bank has Blackjack, player doesnt
                } else if self.bank_cards.is_blackjack() && !self.player_cards.is_blackjack() {
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::BankWins)))
                        .unwrap();
                    self.state = GameState::BankWins;
                // Bank has no blackjack, player does
                } else if !self.bank_cards.is_blackjack() && self.player_cards.is_blackjack() {
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::PlayerWinsThreeToTwo)))
                        .unwrap();

                    player_account.change_amount(3 * self.bet);
                    self.state = GameState::PlayerWinsThreeToTwo;
                // Bank loses because of bust
                } else if self.bank_cards.value() > 21 {
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::PlayerWins)))
                        .unwrap();

                    player_account.change_amount(2 * self.bet);
                    self.state = GameState::PlayerWins;
                // Both have some value
                } else if self.bank_cards.value() > self.player_cards.value() {
                    // bank wins
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::BankWins)))
                        .unwrap();
                    self.state = GameState::BankWins;
                } else if self.bank_cards.value() < self.player_cards.value() {
                    // player wins
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::PlayerWins)))
                        .unwrap();

                    player_account.change_amount(2 * self.bet);
                    self.state = GameState::PlayerWins;
                } else {
                    // draw
                    let mut msg = ChannelId(self.channel_id).message(self.message_id).unwrap();
                    msg.edit(|m| m.content(&self.visualize(GameState::Draw)))
                        .unwrap();

                    player_account.change_amount(self.bet);
                    self.state = GameState::Draw;
                }
            }
            // ignore invalid player updates
            _ => (),
        }
    }

    fn visualize(&self, state: GameState) -> String {
        match state {
            GameState::Playing => {
                format!("Bank: {}\nYou: {}\n", self.bank_cards, self.player_cards)
            }
            GameState::BankWins => format!(
                "Bank: {}\nYou: {}\nYou lost! Play again?",
                self.bank_cards, self.player_cards
            ),
            GameState::PlayerWins => format!(
                "Bank: {}\nYou: {}\nYou won {}! Play again?",
                self.bank_cards,
                self.player_cards,
                self.bet * 2
            ),
            GameState::PlayerWinsThreeToTwo => format!(
                "Bank: {}\nYou: {}\nYou won {}! Play again?",
                self.bank_cards,
                self.player_cards,
                self.bet * 3
            ),
            GameState::Draw => format!(
                "Bank: {}\nYou: {}\nYou Tied! Play again?",
                self.bank_cards, self.player_cards
            ),
            GameState::NotEnoughMoney => "You have not enough money left to play!".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum GameState {
    Playing,
    BankWins,
    PlayerWins,
    PlayerWinsThreeToTwo,
    Draw,
    NotEnoughMoney,
}

struct PlayerAccount(Arc<Mutex<PgConnection>>, u64);

impl PlayerAccount {
    fn can_pay(&self, amount: i64) -> bool {
        use crate::models::bank::Bank;
        use crate::schema::banks::dsl;
        use diesel::prelude::*;

        debug!("Check if user {} can pay: {}", &self.1, &amount);

        let results = dsl::banks
            .filter(dsl::user_id.eq(self.1 as i64))
            .load::<Bank>(&*self.0.lock())
            .expect("could not retrieve banks");

        !results.is_empty() && results[0].amount >= amount
    }

    fn change_amount(&self, amount: i64) {
        use crate::models::bank::Bank;
        use crate::schema::banks::dsl;
        use diesel::prelude::*;

        let results = dsl::banks
            .filter(dsl::user_id.eq(self.1 as i64))
            .load::<Bank>(&*self.0.lock())
            .expect("could not retrieve banks");

        let mut new_amount = results[0].amount;
        new_amount += amount;

        debug!(
            "Change {} amount by: {}, new amount: {}",
            &self.1, &amount, &new_amount
        );

        diesel::update(dsl::banks.filter(dsl::user_id.eq(self.1 as i64)))
            .set(dsl::amount.eq(new_amount))
            .execute(&*self.0.lock())
            .expect("failed update bank");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_creation() {
        let deck = Deck::new();

        dbg!(&deck);

        assert_eq!(deck.0.len(), 52);
    }

    #[test]
    fn test_deck_draw() {
        let mut deck = Deck::new();

        let card = deck.draw();

        dbg!(&card);

        assert!(card.is_some());
    }

    #[test]
    fn test_deck_draw_multiple() {
        let mut deck = Deck::new();

        let stack = deck.draw_multiple(52);

        assert_eq!(stack.len(), 52);
        assert!(deck.is_empty());
    }

    #[test]
    fn test_deck_overdraw_multiple() {
        let mut deck = Deck::new();

        let stack = deck.draw_multiple(55);

        assert_eq!(stack.len(), 52);
        assert!(deck.is_empty());
    }
}
