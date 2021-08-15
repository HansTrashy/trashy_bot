use std::str::FromStr;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1},
    combinator::map_res,
    multi::many1,
    multi::separated_list1,
    sequence::tuple,
    IResult,
};
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::error;

#[command]
#[description = "Show a combo moveset with easy to understand symbols"]
#[usage = "*move instructions*"]
#[example = "632146P"]
pub async fn combo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let combo_str = args.rest().trim();

    match parse_combo_str(combo_str) {
        Ok((_, moves)) => {
            msg.reply(ctx, format!("{:?}", moves)).await?;
        }
        Err(e) => {
            error!(?e, "Failed parsing input");
            msg.reply(ctx, "Sorry that is not a valid combo sequence!")
                .await?;
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Combo {
    moves: Vec<Move>,
}

#[derive(Debug)]
struct Move {
    directionals: Vec<Directional>,
    buttons: Vec<Button>,
}

#[derive(Debug)]
enum Directional {
    Neutral,
    Left,
    Right,
    Up,
    Down,
    DownLeft,
    DownRight,
    UpLeft,
    UpRight,
    QuarterCircleRight,
    QuarterCircleLeft,
    DragonPunchRight,
    DragonPunchLeft,
    HalfCircleRight,
    HalfCircleLeft,
    FullCircle,
}

impl FromStr for Directional {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "1" => Self::DownLeft,
            "2" => Self::Down,
            "3" => Self::DownRight,
            "4" => Self::Left,
            "5" => Self::Neutral,
            "6" => Self::Right,
            "7" => Self::UpLeft,
            "8" => Self::Up,
            "9" => Self::UpRight,
            "214" => Self::QuarterCircleLeft,
            "236" => Self::QuarterCircleRight,
            "623" => Self::DragonPunchRight,
            "421" => Self::DragonPunchLeft,
            "412364" => Self::HalfCircleLeft,
            "632146" => Self::HalfCircleRight,
            "632147896" => Self::FullCircle,
            _ => return Err(String::from("failed to parse directional").into()),
        })
    }
}

#[derive(Debug)]
enum Button {
    Punch,
    Kick,
    Slash,
    HeavySlash,
    Dust,
}

impl FromStr for Button {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "P" => Self::Punch,
            "K" => Self::Kick,
            "S" => Self::Slash,
            "H" => Self::HeavySlash,
            "D" => Self::Dust,
            _ => return Err(String::from("failed to parse button").into()),
        })
    }
}

fn parse_combo_str(combo_str: &str) -> IResult<&str, Combo> {
    let directional = separated_list1(
        tag(","),
        map_res(digit1, |s: &str| s.parse::<Directional>()),
    );

    let directionals = many1(directional);
    let button = map_res(alpha1, |s: &str| s.parse::<Button>());
    let buttons = many1(button);

    let (input, moves) = separated_list1(tag(" "), tuple((directionals, buttons)))(combo_str)?;

    Ok((
        input,
        Combo {
            moves: moves
                .into_iter()
                .map(|(directionals, buttons)| Move {
                    directionals: directionals.into_iter().flatten().collect(),
                    buttons,
                })
                .collect(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::commands::fighting::parse_combo_str;

    #[test]
    fn test_nom_parser() {
        let combo_str = "623,623P 5P 5P";

        let (_, combo) = parse_combo_str(combo_str).unwrap();

        println!("{:?}", combo);
    }
}
