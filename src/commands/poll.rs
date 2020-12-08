use serenity::collector::ReactionAction;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    futures::stream::StreamExt,
    model::channel::Message,
    model::user::User,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[command]
#[description = "Create an adhoc poll"]
#[usage = "*\"question\"* *\"answer_1\"* *\"answer_2\"*"]
#[example = "\"Do you freeze bread?\" \"Yes\" \"No\""]
#[only_in("guilds")]
async fn poll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let question = args.single::<String>()?;
    let answers = args
        .iter()
        .filter_map(|a| a.map(Some).unwrap_or(None))
        .collect::<Vec<String>>();

    if answers.len() > 9 {
        return Err("Only up to 9 answers are allowed".into());
    }

    //TODO: react with all possible answer reactions
    let question_msg = msg
        .channel_id
        .say(&ctx, ask_question(&msg.author, &question, &answers).await)
        .await?;

    let _ = msg.delete(ctx).await;

    let collector = question_msg
        .await_reactions(&ctx)
        .timeout(Duration::from_secs(60))
        .await;

    let reactions: Vec<Arc<ReactionAction>> = collector.collect().await;

    let num_reactions = reactions
        .iter()
        .filter_map(reaction_to_num)
        .filter(|&num| num <= answers.len())
        .map(|num| num - 1) // reduce num to prevent index out of bounds
        .fold(HashMap::new(), |mut acc, x| {
            let entry = acc.entry(x).or_insert(0_usize);
            *entry += 1;

            acc
        });

    let poll_results = num_reactions
        .into_iter()
        .map(|(key, val)| (answers[key].clone(), val))
        .collect::<HashMap<_, _>>();

    msg.channel_id
        .say(&ctx, render_poll_results(&question, &poll_results))
        .await?;

    Ok(())
}

async fn ask_question(user: &User, question: &str, answers: &[String]) -> String {
    let mut rendered = format!("{} asks: {}\nPossible answers:\n", user.name, question);
    for (i, a) in answers.iter().enumerate() {
        rendered.push_str(&format!("{} | {}\n", i + 1, a));
    }
    rendered
}

fn reaction_to_num(reaction: &Arc<ReactionAction>) -> Option<usize> {
    let reaction = reaction.as_inner_ref();
    match &reaction.emoji {
        ReactionType::Unicode(s) => match s.as_ref() {
            "1\u{fe0f}\u{20e3}" => Some(1),
            "2\u{fe0f}\u{20e3}" => Some(2),
            "3\u{fe0f}\u{20e3}" => Some(3),
            "4\u{fe0f}\u{20e3}" => Some(4),
            "5\u{fe0f}\u{20e3}" => Some(5),
            "6\u{fe0f}\u{20e3}" => Some(6),
            "7\u{fe0f}\u{20e3}" => Some(7),
            "8\u{fe0f}\u{20e3}" => Some(8),
            "9\u{fe0f}\u{20e3}" => Some(9),
            _ => None,
        },
        _ => None,
    }
}

fn render_poll_results(question: &str, poll_results: &HashMap<String, usize>) -> String {
    let mut rendered = String::new();
    rendered.push_str(&format!("On the question of: {}\n", question));
    rendered.push_str("people answered: \n");
    for (answer, votes) in poll_results {
        rendered.push_str(&format!("{} with {} vote(s)!\n", answer, votes));
    }
    if let Some(winner) = poll_results.iter().max_by_key(|&(_, v)| v) {
        rendered.push_str(&format!(
            "**{}** won with **{}** vote(s)!",
            winner.0, winner.1
        ));
    }

    rendered
}
