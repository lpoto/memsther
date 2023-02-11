use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        prelude::{interaction::Interaction, Activity, Reaction, UserId},
    },
    prelude::{Context, EventHandler},
};

use crate::{configuration, datastore::Datastore};
mod application_command;
mod reaction;

pub struct Handler {
    config: configuration::Configuration,
    datastore: Datastore,
}

impl Handler {
    pub async fn new(config: configuration::Configuration) -> Handler {
        let datastore = Datastore::new(&config.postgres);
        datastore.migrate().await;

        Handler {
            config,
            datastore,
        }
    }
}

pub static mut BOT_USER_ID: Option<UserId> = None;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Err(why) = application_command::register(&ctx).await {
            log::error!("Failed to register global commands: {}", why);
        }

        ctx.set_activity(Activity::competing("Rust, I'm in Rust btw.")).await;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        log::info!("Bot ready with username '{}'", ready.user.name);

        unsafe {
            BOT_USER_ID = Some(ready.user.id);
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log::trace!(
            "Received interaction created event: {:?}",
            interaction.id(),
        );
        unsafe {
            if let None = BOT_USER_ID {
                log::trace!("Bot not yet ready, cannot handle event");
                return;
            }
        }
        if let Interaction::ApplicationCommand(command) = interaction {
            let pool = &self.datastore.pool;
            application_command::handle_appliaction_command(
                ctx,
                command,
                pool,
                &self.config,
            )
            .await;
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        log::trace!("Received reaction add event");

        unsafe {
            if let None = BOT_USER_ID {
                log::trace!("Bot not yet ready, cannot handle event");
                return;
            }
        }

        let pool = &self.datastore.pool;
        reaction::handle_reaction_add(ctx, reaction, pool).await;
    }

    // Handle the reaction removed event. This is where we handle the
    // reactions removed from the meme message, and the logic behind
    // decreasing the user's score.
    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        log::trace!("Received reaction remove event");

        unsafe {
            if let None = BOT_USER_ID {
                log::trace!("Bot not yet ready, cannot handle event");
                return;
            }
        }

        let pool = &self.datastore.pool;
        reaction::handle_reaction_remove(ctx, reaction, pool).await;
    }
}
