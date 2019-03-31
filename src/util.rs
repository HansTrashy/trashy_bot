static OTHER_MOD_CMD: [char; 3] = ['%', '=', '$'];

pub fn sanitize_for_other_bot_commands(output: &str) -> String {
    output
        .chars()
        .filter(|&c| !OTHER_MOD_CMD.contains(&c))
        .collect::<String>()
}
