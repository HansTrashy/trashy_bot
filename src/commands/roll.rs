use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::{
    branch::alt, combinator::opt, error::ErrorKind, multi::separated_list1, sequence::tuple,
    IResult,
};
use rand::prelude::*;
use twilight_model::application::callback::{CallbackData, InteractionResponse};
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::ApplicationCommand;

use crate::error::TrashyCommandError;
use crate::TrashyContext;

pub async fn roll(
    cmd: Box<ApplicationCommand>,
    ctx: &TrashyContext,
) -> Result<(), TrashyCommandError> {
    let die_str = match &cmd.data.options.get(0) {
        Some(CommandDataOption::String { value, .. }) => value,
        _ => {
            tracing::error!("wrong or no command option dataype received!");
            return Ok(());
        }
    };

    match parse_multiple_dice_str(die_str) {
        Ok((_, dice)) => {
            let mut total: isize = 0;
            let mut papertrail: Vec<String> = Vec::new();
            {
                let rng = &mut ctx.rng.lock().await;
                for die in dice {
                    let mut rolls = Vec::new();
                    for _ in 0..die.number {
                        rolls.push(rng.gen_range(1..=(die.sides as isize)));
                    }

                    papertrail.extend(
                        rolls
                            .iter()
                            .map(ToString::to_string)
                            .chain(vec![die.flat.to_string()].into_iter())
                            .collect::<Vec<_>>(),
                    );
                    total += rolls.iter().sum::<isize>() + die.flat;
                }
            }

            let interaction_resp = InteractionResponse::ChannelMessageWithSource(CallbackData {
                allowed_mentions: None,
                components: None,
                content: Some(format!("Your Roll ({}): {}", papertrail.join("+"), total)),
                embeds: Vec::new(),
                flags: None,
                tts: None,
            });

            let resp = ctx
                .http
                .interaction_callback(cmd.id, &cmd.token, &interaction_resp)
                .exec()
                .await;
            tracing::debug!(?resp);
        }
        Err(e) => {
            tracing::error!(?e, "Failed parsing input");
            let interaction_resp = InteractionResponse::ChannelMessageWithSource(CallbackData {
                allowed_mentions: None,
                components: None,
                content: Some("the given die string is invalid".to_string()),
                embeds: Vec::new(),
                flags: None,
                tts: None,
            });

            let resp = ctx
                .http
                .interaction_callback(cmd.id, &cmd.token, &interaction_resp)
                .exec()
                .await;
            tracing::debug!(?resp);
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

fn parse_flat_part(input: &str) -> IResult<&str, Option<(&str, &str)>> {
    opt(tuple((alt((tag("+"), tag("-"))), digit1)))(input)
}

fn parse_dice_str(input: &str) -> IResult<&str, Die> {
    let (input, number_digits) = digit1(input)?;
    let (input, _) = tag("d")(input)?;
    let (input, side_digits) = digit1(input)?;
    let (input, flat_part) = parse_flat_part(input)?;

    let number = number_digits
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?;

    let sides = side_digits
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?;

    let flat = match flat_part {
        Some((flat_sign, flat_digits)) => match flat_sign {
            "+" => flat_digits
                .parse::<isize>()
                .map_err(|_| nom::Err::Error(nom::error::Error::new(input, ErrorKind::Char)))?,
            "-" => -flat_digits
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

fn parse_multiple_dice_str(input: &str) -> IResult<&str, Vec<Die>> {
    separated_list1(tag(" "), parse_dice_str)(input)
}

#[cfg(test)]
mod tests {
    use crate::commands::roll::{parse_dice_str, parse_multiple_dice_str};

    #[test]
    fn test_nom_parser() {
        let die_str = "1d6+2";

        let (_, die) = parse_dice_str(die_str).unwrap();

        println!("{:?}", die);
    }

    #[test]
    fn test_nom_parser_multi() {
        let die_str = "1d6+2 2d20-3";

        let (_, die) = parse_multiple_dice_str(die_str).unwrap();

        println!("{:?}", die);
    }
}
