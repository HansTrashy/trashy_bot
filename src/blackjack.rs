use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

pub struct State(Vec<Game>);

impl State {
    fn save(&self) {
        // also save those settings
        let data = serde_json::to_string(&self.0).expect("Could not serialize blackjack state");
        std::fs::write("blackjack_state.storage", data)
            .expect("coult not write blackjack state to file");
    }

    pub fn new() -> Self {
        Self(Vec::default())
    }

    pub fn add_game(&mut self, user: u64, bet: u64) {
        self.0.push(Game::new(user, bet));
    }

    pub fn load() -> Self {
        match std::fs::read_to_string("blackjack_state.storage") {
            Ok(data) => {
                Self(serde_json::from_str(&data).expect("could not deserialize blackjack state"))
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
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.suit, self.rank,)
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
}

#[derive(Debug, Clone, Copy)]
enum PlayerAction {
    Init,
    Hit,
    Stay,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    deck: Deck,
    removed_cards: Vec<Card>,
    player: u64,
    bet: u64,
    bank_cards: Stack,
    player_cards: Stack,
}

impl Game {
    fn new(player: u64, bet: u64) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();
        Self {
            deck,
            removed_cards: Vec::with_capacity(52),
            player,
            bet,
            bank_cards: Stack::new(),
            player_cards: Stack::new(),
        }
    }

    fn update(&mut self, action: PlayerAction) {
        // Have at least 15 cards remaining, or add removed_cards and shuffle
        if !self.deck.size() >= 15 {
            self.deck.add_cards(&mut self.removed_cards);
            self.deck.shuffle();
        }

        match action {
            PlayerAction::Init => {
                self.bank_cards.add(self.deck.draw().unwrap());
                self.player_cards.add(self.deck.draw().unwrap());

                // visualize state
            }
            PlayerAction::Hit => {
                self.player_cards.add(self.deck.draw().unwrap());

                // make card checks

                // visualize state
            }
            PlayerAction::Stay => {
                // make remaining draws for the bank

                // check winner

                // visualize result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game() {
        let mut game = Game::new(0, 1000);
        game.update(PlayerAction::Init);

        dbg!(&game);
    }

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
