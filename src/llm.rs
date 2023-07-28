pub fn get_prompt_from_bytes(prompt_bytes: &[u8]) -> Result<String, anyhow::Error> {
    let prompt = std::str::from_utf8(prompt_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to convert byte array to string: {}", e))?
        .trim()
        .to_string();
    Ok(prompt)
}

fn form_prompt(action_text: &str, command_content: &str) -> Result<String, anyhow::Error> {
    let ACTION_PROMPT_BYTES: &[u8] = include_bytes!("../prompts/action_text.md");
    let action_prompt = get_prompt_from_bytes(ACTION_PROMPT_BYTES)?;
    Ok(action_prompt.replace("{action_text}", action_text).replace("{user_input}", command_content))
}

pub fn form_prompt_add_idea(command_content: &str) -> Result<String, anyhow::Error> {
    let add_action = "add the idea provided by user";
    Ok(form_prompt(add_action, command_content)?)
}

pub fn form_prompt_update_idea(command_content: &str) -> Result<String, anyhow::Error> {
    let update_action = "update the corresponding idea intended by user";
    Ok(form_prompt(update_action, command_content)?)
}