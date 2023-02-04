use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        prelude::{interaction::Interaction, Activity, Reaction},
    },
    prelude::{Context, EventHandler},
};

use crate::datastore::Datastore;
mod application_command;
mod reaction;

pub struct Handler {
    datastore: Datastore,
}

impl Handler {
    pub fn new(datastore: Datastore) -> Handler {
        Handler {
            datastore,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Err(why) = application_command::register(&ctx).await {
            log::error!("Failed to register global commands: {}", why);
        }

        ctx.set_activity(Activity::competing("Rust, I'm in Rust btw.")).await;

        log::info!("Bot ready with username '{}'", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log::trace!(
            "Received interaction created event: {:?}",
            interaction.id(),
        );
        if let Interaction::ApplicationCommand(command) = interaction {
            let pool = &self.datastore.pool;
            application_command::handle_appliaction_command(ctx, command, pool)
                .await;
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        log::trace!("Received reaction added event");

        let pool = &self.datastore.pool;
        reaction::handle_reaction_add(ctx, reaction, pool).await;
    }

    // Handle the reaction removed event. This is where we handle the
    // reactions removed from the meme message, and the logic behind
    // decreasing the user's score.
    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        log::trace!("Received reaction removed event");

        let pool = &self.datastore.pool;
        reaction::handle_reaction_remove(ctx, reaction, pool).await;
    }
}
