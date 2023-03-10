use deadpool_postgres::Pool;
use serenity::{
    model::{
        application::command::Command,
        prelude::interaction::application_command::ApplicationCommandInteraction,
    },
    prelude::Context,
};

pub mod gif;
pub mod leaderboard;
pub mod link;
pub mod meme;
pub mod score;

/// Fetch all global commands. Delete those that are no longer required,
/// and register those that are not yet registered.
pub async fn register(ctx: &Context) -> Result<(), String> {
    let commands = Command::get_global_application_commands(&ctx.http)
        .await
        .map_err(|err| format!("Failed to fetch global commands: {:?}", err))?;

    let to_register = vec![
        meme::name(),
        score::name(),
        link::name(),
        gif::name(),
        leaderboard::name(),
    ];

    log::debug!("Registering slash commands ...");
    for command in commands.iter() {
        if to_register.iter().find(|&name| command.name == *name).is_none() {
            log::debug!("Deleting '{}' app. command", command.name,);
            Command::delete_global_application_command(&ctx.http, command.id)
                .await
                .map_err(|err| {
                    format!(
                        "Failed to delete '{}' app. command: {:?}",
                        command.name, err
                    )
                })?;
        }
    }
    // NOTE: register only slash commands that are not yet present in
    // the already registered commands.
    // This is a safety to avoid being blocked from discord for registering
    // commands too often.
    if commands.iter().find(|command| command.name == meme::name()).is_none() {
        meme::register(ctx).await;
    };
    if commands.iter().find(|command| command.name == score::name()).is_none() {
        score::register(ctx).await;
    };
    if commands.iter().find(|command| command.name == link::name()).is_none() {
        link::register(ctx).await;
    };
    if commands.iter().find(|command| command.name == gif::name()).is_none() {
        gif::register(ctx).await;
    };
    if commands
        .iter()
        .find(|command| command.name == leaderboard::name())
        .is_none()
    {
        leaderboard::register(ctx).await;
    };

    log::info!("Slash commands registered");
    return Ok(());
}

pub async fn handle_appliaction_command(
    ctx: Context,
    command: ApplicationCommandInteraction,
    pool: &Pool,
    giphy_key: &String,
) {
    log::trace!("Handling command interaction: {:?}", command.data.name,);
    let name = command.data.name.to_string();
    if name == meme::name() {
        meme::handle_command(ctx, command).await
    } else if name == score::name() {
        score::handle_command(ctx, command, pool).await
    } else if name == link::name() {
        link::handle_command(ctx, command).await
    } else if name == gif::name() {
        gif::handle_command(ctx, command, giphy_key.as_str()).await
    } else if name == leaderboard::name() {
        leaderboard::handle_command(ctx, command, pool).await
    };
}
