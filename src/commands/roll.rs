use nom::{
    branch::alt, bytes::complete::take_while, character::complete::char, character::is_digit,
    combinator::opt, error::ErrorKind, multi::separated_list1, sequence::tuple, IResult,
};
use rand::prelude::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Roll some dice"]
#[num_args(2)]
#[usage = "*dice_str* *dice_str*"]
#[example = "1d6"]
#[example = "2d20-3"]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let dice_str = args.rest().trim();

    match parse_multiple_dice_str(dice_str.as_bytes()) {
        Ok((_, dice)) => {
            let mut total: isize = 0;
            {
                // dont hold rng over await points
                let mut rng = rand::thread_rng();
                for die in dice {
                    let mut rolls = Vec::new();
                    for _ in 0..die.number {
                        rolls.push(rng.gen_range(1..=(die.sides as isize)));
                    }
                    total += rolls.iter().sum::<isize>() + die.flat;
                }
            }

            msg.reply(ctx, &format!("Your Roll: {}", total)).await?;
        }
        Err(_) => {
            let _ = msg.reply(ctx, "Sorry that is not a valid die roll!").await;
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
struct Die {
    number: usize,
    sides: usize,
    flat: isize,
}

fn parse_flat_part(input: &[u8]) -> IResult<&[u8], Option<(char, &[u8])>> {
    opt(tuple((alt((char('+'), char('-'))), take_while(is_digit))))(input)
}

fn parse_dice_str(input: &[u8]) -> IResult<&[u8], Die> {
    let (input, number_digits) = take_while(is_digit)(input)?;
    let (input, _) = char('d')(input)?;
    let (input, side_digits) = take_while(is_digit)(input)?;
    let (input, flat_part) = parse_flat_part(input)?;

    let number = std::str::from_utf8(number_digits)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?;

    let sides = std::str::from_utf8(side_digits)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?;

    let flat = match flat_part {
        Some((flat_sign, flat_digits)) => match flat_sign {
            '+' => std::str::from_utf8(flat_digits)
                .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?
                .parse::<isize>()
                .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?,
            '-' => -std::str::from_utf8(flat_digits)
                .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?
                .parse::<isize>()
                .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?,
            _ => unreachable!("Reaching this means the nom parser failed"),
        },
        None => 0,
    };

    let die = Die {
        number,
        sides,
        flat,
    };

    Ok((input, die))
}

fn parse_multiple_dice_str(input: &[u8]) -> IResult<&[u8], Vec<Die>> {
    separated_list1(char(' '), parse_dice_str)(input)
}

#[test]
fn test_nom_parser() {
    let die_str = "1d6+2";

    let (_, die) = parse_dice_str(die_str.as_bytes()).unwrap();

    println!("{:?}", die);
}

#[test]
fn test_nom_parser_multi() {
    let die_str = "1d6+2 2d20-3";

    let (_, die) = parse_multiple_dice_str(die_str.as_bytes()).unwrap();

    println!("{:?}", die);
}
