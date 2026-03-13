use std::sync::Arc;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use twilight_http::Client as DiscordClient;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::tools;

#[derive(Clone)]
pub struct DiscordMcpServer {
    discord: Arc<DiscordClient>,
    http: Arc<reqwest::Client>,
    token: String,
    application_id: Id<ApplicationMarker>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DiscordMcpServer {
    pub async fn from_env() -> anyhow::Result<Self> {
        let token = std::env::var("DISCORD_TOKEN")
            .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN environment variable not set"))?;
        let discord = Arc::new(DiscordClient::new(token.clone()));
        let http = Arc::new(reqwest::Client::new());

        // Resolve application ID: env var or auto-detect from current user
        let application_id = if let Ok(app_id_str) = std::env::var("DISCORD_APPLICATION_ID") {
            let raw: u64 = app_id_str.parse().map_err(|_| {
                anyhow::anyhow!("DISCORD_APPLICATION_ID must be a numeric snowflake ID")
            })?;
            Id::new(raw)
        } else {
            tracing::info!("DISCORD_APPLICATION_ID not set, auto-detecting from current user...");
            let user = discord
                .current_user()
                .await?
                .model()
                .await?;
            let id = user.id.get();
            tracing::info!("Auto-detected application ID: {id}");
            Id::new(id)
        };

        Ok(Self {
            discord,
            http,
            token,
            application_id,
            tool_router: Self::tool_router(),
        })
    }

    /// Build a raw Discord API request with bot authorization.
    pub fn raw_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("https://discord.com/api/v10{path}");
        self.http
            .request(method, &url)
            .header("Authorization", format!("Bot {}", self.token))
    }

    /// Get the application ID.
    #[allow(dead_code)]
    pub fn application_id(&self) -> Id<ApplicationMarker> {
        self.application_id
    }

    // ========================
    // DISCOVERY
    // ========================

    #[tool(description = "Get information about the bot's current Discord user account")]
    async fn get_current_user(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::discovery::get_current_user(&self.discord).await
    }

    #[tool(description = "List all guilds (servers) the bot is a member of")]
    async fn list_guilds(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::discovery::list_guilds(&self.discord).await
    }

    #[tool(description = "Get detailed information about a specific guild including roles, features, and settings")]
    async fn get_guild(
        &self,
        Parameters(params): Parameters<tools::discovery::GetGuildParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::discovery::get_guild(&self.discord, params).await
    }

    // ========================
    // CHANNELS
    // ========================

    #[tool(description = "List all channels in a guild (server)")]
    async fn list_guild_channels(
        &self,
        Parameters(params): Parameters<tools::channels::ListGuildChannelsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::list_guild_channels(&self.discord, params).await
    }

    #[tool(description = "Get detailed information about a specific channel")]
    async fn get_channel(
        &self,
        Parameters(params): Parameters<tools::channels::GetChannelParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::get_channel(&self.discord, params).await
    }

    #[tool(description = "Create a new channel in a guild. Types: 0=text, 2=voice, 4=category, 5=announcement, 13=stage, 15=forum")]
    async fn create_channel(
        &self,
        Parameters(params): Parameters<tools::channels::CreateChannelParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::create_channel(&self.discord, params).await
    }

    #[tool(description = "Update a channel's settings (name, topic, nsfw, position, parent category)")]
    async fn update_channel(
        &self,
        Parameters(params): Parameters<tools::channels::UpdateChannelParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::update_channel(&self.discord, params).await
    }

    #[tool(description = "Delete a channel permanently")]
    async fn delete_channel(
        &self,
        Parameters(params): Parameters<tools::channels::DeleteChannelParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::delete_channel(&self.discord, params).await
    }

    #[tool(description = "Reorder channels in a guild by specifying new positions")]
    async fn update_channel_positions(
        &self,
        Parameters(params): Parameters<tools::channels::UpdateChannelPositionsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::update_channel_positions(&self.discord, params).await
    }

    // ========================
    // MESSAGES
    // ========================

    #[tool(description = "List messages in a channel. Supports pagination with before/after/around parameters.")]
    async fn list_messages(
        &self,
        Parameters(params): Parameters<tools::messages::ListMessagesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::list_messages(&self.discord, params).await
    }

    #[tool(description = "Get a specific message by ID")]
    async fn get_message(
        &self,
        Parameters(params): Parameters<tools::messages::GetMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::get_message(&self.discord, params).await
    }

    #[tool(description = "Send a message to a channel. Can include text content and reply to another message.")]
    async fn send_message(
        &self,
        Parameters(params): Parameters<tools::messages::SendMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::send_message(&self.discord, params).await
    }

    #[tool(description = "Edit an existing message's content")]
    async fn edit_message(
        &self,
        Parameters(params): Parameters<tools::messages::EditMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::edit_message(&self.discord, params).await
    }

    #[tool(description = "Delete a specific message")]
    async fn delete_message(
        &self,
        Parameters(params): Parameters<tools::messages::DeleteMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::delete_message(&self.discord, params).await
    }

    #[tool(description = "Bulk delete messages (2-100 messages, must be less than 14 days old)")]
    async fn bulk_delete_messages(
        &self,
        Parameters(params): Parameters<tools::messages::BulkDeleteMessagesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::bulk_delete_messages(&self.discord, params).await
    }

    #[tool(description = "Pin a message in a channel")]
    async fn pin_message(
        &self,
        Parameters(params): Parameters<tools::messages::PinMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::pin_message(&self.discord, params).await
    }

    #[tool(description = "Unpin a message from a channel")]
    async fn unpin_message(
        &self,
        Parameters(params): Parameters<tools::messages::UnpinMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::unpin_message(&self.discord, params).await
    }

    #[tool(description = "Get all pinned messages in a channel")]
    async fn get_pinned_messages(
        &self,
        Parameters(params): Parameters<tools::messages::GetPinnedMessagesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::get_pinned_messages(&self.discord, params).await
    }

    #[tool(description = "Crosspost (publish) a message from an announcement channel to following channels")]
    async fn crosspost_message(
        &self,
        Parameters(params): Parameters<tools::messages::CrosspostMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::messages::crosspost_message(&self.discord, params).await
    }

    // ========================
    // MEMBERS
    // ========================

    #[tool(description = "List members of a guild. Supports pagination with limit and after parameters.")]
    async fn list_guild_members(
        &self,
        Parameters(params): Parameters<tools::members::ListGuildMembersParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::members::list_guild_members(&self.discord, params).await
    }

    #[tool(description = "Get a specific guild member by user ID")]
    async fn get_guild_member(
        &self,
        Parameters(params): Parameters<tools::members::GetGuildMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::members::get_guild_member(&self.discord, params).await
    }

    #[tool(description = "Search guild members by username or nickname")]
    async fn search_guild_members(
        &self,
        Parameters(params): Parameters<tools::members::SearchGuildMembersParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::members::search_guild_members(&self.discord, params).await
    }

    #[tool(description = "Update a guild member's nickname, roles, mute/deaf status, or voice channel")]
    async fn update_guild_member(
        &self,
        Parameters(params): Parameters<tools::members::UpdateGuildMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::members::update_guild_member(&self.discord, params).await
    }

    #[tool(description = "Kick (remove) a member from a guild")]
    async fn kick_member(
        &self,
        Parameters(params): Parameters<tools::members::KickMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::members::kick_member(&self.discord, params).await
    }

    // ========================
    // REACTIONS
    // ========================

    #[tool(description = "Add a reaction to a message. Emoji can be unicode (e.g. 👍) or custom (name:id)")]
    async fn add_reaction(
        &self,
        Parameters(params): Parameters<tools::reactions::AddReactionParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::reactions::add_reaction(&self.discord, params).await
    }

    #[tool(description = "Remove a reaction from a message. Omit user_id to remove the bot's own reaction.")]
    async fn remove_reaction(
        &self,
        Parameters(params): Parameters<tools::reactions::RemoveReactionParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::reactions::remove_reaction(&self.discord, params).await
    }

    #[tool(description = "Get users who reacted with a specific emoji on a message")]
    async fn get_reactions(
        &self,
        Parameters(params): Parameters<tools::reactions::GetReactionsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::reactions::get_reactions(&self.discord, params).await
    }

    #[tool(description = "Remove all reactions from a message")]
    async fn clear_all_reactions(
        &self,
        Parameters(params): Parameters<tools::reactions::ClearAllReactionsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::reactions::clear_all_reactions(&self.discord, params).await
    }

    #[tool(description = "Remove all reactions of a specific emoji from a message")]
    async fn clear_emoji_reactions(
        &self,
        Parameters(params): Parameters<tools::reactions::ClearEmojiReactionsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::reactions::clear_emoji_reactions(&self.discord, params).await
    }

    // ========================
    // BANS
    // ========================

    #[tool(description = "List all bans in a guild")]
    async fn list_bans(
        &self,
        Parameters(params): Parameters<tools::bans::ListBansParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::bans::list_bans(&self.discord, params).await
    }

    #[tool(description = "Get ban information for a specific user")]
    async fn get_ban(
        &self,
        Parameters(params): Parameters<tools::bans::GetBanParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::bans::get_ban(&self.discord, params).await
    }

    #[tool(description = "Ban a member from a guild. Optionally delete their recent messages.")]
    async fn ban_member(
        &self,
        Parameters(params): Parameters<tools::bans::BanMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::bans::ban_member(&self.discord, params).await
    }

    #[tool(description = "Unban a user from a guild")]
    async fn unban_member(
        &self,
        Parameters(params): Parameters<tools::bans::UnbanMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::bans::unban_member(&self.discord, params).await
    }

    // ========================
    // ROLES
    // ========================

    #[tool(description = "List all roles in a guild")]
    async fn list_roles(
        &self,
        Parameters(params): Parameters<tools::roles::ListRolesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::roles::list_roles(&self.discord, params).await
    }

    #[tool(description = "Create a new role in a guild")]
    async fn create_role(
        &self,
        Parameters(params): Parameters<tools::roles::CreateRoleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::roles::create_role(&self.discord, params).await
    }

    #[tool(description = "Update a role's name, color, permissions, hoist, or mentionable status")]
    async fn update_role(
        &self,
        Parameters(params): Parameters<tools::roles::UpdateRoleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::roles::update_role(&self.discord, params).await
    }

    #[tool(description = "Delete a role from a guild")]
    async fn delete_role(
        &self,
        Parameters(params): Parameters<tools::roles::DeleteRoleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::roles::delete_role(&self.discord, params).await
    }

    #[tool(description = "Add a role to a guild member")]
    async fn add_role_to_member(
        &self,
        Parameters(params): Parameters<tools::roles::AddRoleToMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::roles::add_role_to_member(&self.discord, params).await
    }

    #[tool(description = "Remove a role from a guild member")]
    async fn remove_role_from_member(
        &self,
        Parameters(params): Parameters<tools::roles::RemoveRoleFromMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::roles::remove_role_from_member(&self.discord, params).await
    }

    // ========================
    // THREADS
    // ========================

    #[tool(description = "Create a new thread in a channel. Types: 11=public, 12=private")]
    async fn create_thread(
        &self,
        Parameters(params): Parameters<tools::threads::CreateThreadParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::create_thread(&self.discord, params).await
    }

    #[tool(description = "Create a thread from an existing message")]
    async fn create_thread_from_message(
        &self,
        Parameters(params): Parameters<tools::threads::CreateThreadFromMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::create_thread_from_message(&self.discord, params).await
    }

    #[tool(description = "Join a thread as the bot")]
    async fn join_thread(
        &self,
        Parameters(params): Parameters<tools::threads::JoinThreadParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::join_thread(&self.discord, params).await
    }

    #[tool(description = "Leave a thread as the bot")]
    async fn leave_thread(
        &self,
        Parameters(params): Parameters<tools::threads::LeaveThreadParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::leave_thread(&self.discord, params).await
    }

    #[tool(description = "Add a user to a thread")]
    async fn add_thread_member(
        &self,
        Parameters(params): Parameters<tools::threads::AddThreadMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::add_thread_member(&self.discord, params).await
    }

    #[tool(description = "Remove a user from a thread")]
    async fn remove_thread_member(
        &self,
        Parameters(params): Parameters<tools::threads::RemoveThreadMemberParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::remove_thread_member(&self.discord, params).await
    }

    #[tool(description = "List members of a thread")]
    async fn list_thread_members(
        &self,
        Parameters(params): Parameters<tools::threads::ListThreadMembersParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::list_thread_members(&self.discord, params).await
    }

    #[tool(description = "List all active threads in a guild")]
    async fn list_active_threads(
        &self,
        Parameters(params): Parameters<tools::threads::ListActiveThreadsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::list_active_threads(&self.discord, params).await
    }

    #[tool(description = "List public archived threads in a channel")]
    async fn list_public_archived_threads(
        &self,
        Parameters(params): Parameters<tools::threads::ListPublicArchivedThreadsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::list_public_archived_threads(&self.discord, params).await
    }

    // ========================
    // WEBHOOKS
    // ========================

    #[tool(description = "List all webhooks in a channel")]
    async fn list_channel_webhooks(
        &self,
        Parameters(params): Parameters<tools::webhooks::ListChannelWebhooksParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::webhooks::list_channel_webhooks(&self.discord, params).await
    }

    #[tool(description = "List all webhooks in a guild")]
    async fn list_guild_webhooks(
        &self,
        Parameters(params): Parameters<tools::webhooks::ListGuildWebhooksParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::webhooks::list_guild_webhooks(&self.discord, params).await
    }

    #[tool(description = "Create a new webhook in a channel")]
    async fn create_webhook(
        &self,
        Parameters(params): Parameters<tools::webhooks::CreateWebhookParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::webhooks::create_webhook(&self.discord, params).await
    }

    #[tool(description = "Get a webhook by ID")]
    async fn get_webhook(
        &self,
        Parameters(params): Parameters<tools::webhooks::GetWebhookParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::webhooks::get_webhook(&self.discord, params).await
    }

    #[tool(description = "Update a webhook's name or channel")]
    async fn update_webhook(
        &self,
        Parameters(params): Parameters<tools::webhooks::UpdateWebhookParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::webhooks::update_webhook(&self.discord, params).await
    }

    #[tool(description = "Delete a webhook")]
    async fn delete_webhook(
        &self,
        Parameters(params): Parameters<tools::webhooks::DeleteWebhookParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::webhooks::delete_webhook(&self.discord, params).await
    }

    #[tool(description = "Execute a webhook to send a message")]
    async fn execute_webhook(
        &self,
        Parameters(params): Parameters<tools::webhooks::ExecuteWebhookParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::webhooks::execute_webhook(&self.discord, params).await
    }

    // ========================
    // INVITES
    // ========================

    #[tool(description = "List all invites for a channel")]
    async fn list_channel_invites(
        &self,
        Parameters(params): Parameters<tools::invites::ListChannelInvitesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::invites::list_channel_invites(&self.discord, params).await
    }

    #[tool(description = "List all invites for a guild")]
    async fn list_guild_invites(
        &self,
        Parameters(params): Parameters<tools::invites::ListGuildInvitesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::invites::list_guild_invites(&self.discord, params).await
    }

    #[tool(description = "Get information about an invite by its code")]
    async fn get_invite(
        &self,
        Parameters(params): Parameters<tools::invites::GetInviteParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::invites::get_invite(&self.discord, params).await
    }

    #[tool(description = "Create an invite for a channel")]
    async fn create_invite(
        &self,
        Parameters(params): Parameters<tools::invites::CreateInviteParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::invites::create_invite(&self.discord, params).await
    }

    #[tool(description = "Delete an invite by its code")]
    async fn delete_invite(
        &self,
        Parameters(params): Parameters<tools::invites::DeleteInviteParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::invites::delete_invite(&self.discord, params).await
    }

    // ========================
    // EMOJIS
    // ========================

    #[tool(description = "List all custom emojis in a guild")]
    async fn list_emojis(
        &self,
        Parameters(params): Parameters<tools::emojis::ListEmojisParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::list_emojis(&self.discord, params).await
    }

    #[tool(description = "Get a specific custom emoji by ID")]
    async fn get_emoji(
        &self,
        Parameters(params): Parameters<tools::emojis::GetEmojiParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::get_emoji(&self.discord, params).await
    }

    #[tool(description = "Create a custom emoji in a guild (requires base64 image data)")]
    async fn create_emoji(
        &self,
        Parameters(params): Parameters<tools::emojis::CreateEmojiParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::create_emoji(&self.discord, params).await
    }

    #[tool(description = "Update a custom emoji's name")]
    async fn update_emoji(
        &self,
        Parameters(params): Parameters<tools::emojis::UpdateEmojiParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::update_emoji(&self.discord, params).await
    }

    #[tool(description = "Delete a custom emoji from a guild")]
    async fn delete_emoji(
        &self,
        Parameters(params): Parameters<tools::emojis::DeleteEmojiParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::delete_emoji(&self.discord, params).await
    }

    // ========================
    // STICKERS
    // ========================

    #[tool(description = "List all custom stickers in a guild")]
    async fn list_guild_stickers(
        &self,
        Parameters(params): Parameters<tools::stickers::ListGuildStickersParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stickers::list_guild_stickers(&self.discord, params).await
    }

    #[tool(description = "Get a specific guild sticker by ID")]
    async fn get_guild_sticker(
        &self,
        Parameters(params): Parameters<tools::stickers::GetGuildStickerParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stickers::get_guild_sticker(&self.discord, params).await
    }

    #[tool(description = "Delete a guild sticker")]
    async fn delete_guild_sticker(
        &self,
        Parameters(params): Parameters<tools::stickers::DeleteGuildStickerParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stickers::delete_guild_sticker(&self.discord, params).await
    }

    #[tool(description = "Create a guild sticker from an image file (png, apng, gif, or Lottie json)")]
    async fn create_guild_sticker(
        &self,
        Parameters(params): Parameters<tools::stickers::CreateGuildStickerParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stickers::create_guild_sticker(&self.discord, params).await
    }

    // ========================
    // AUTO MODERATION
    // ========================

    #[tool(description = "List all auto-moderation rules in a guild")]
    async fn list_automod_rules(
        &self,
        Parameters(params): Parameters<tools::automod::ListAutomodRulesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::automod::list_automod_rules(&self.discord, params).await
    }

    #[tool(description = "Get a specific auto-moderation rule")]
    async fn get_automod_rule(
        &self,
        Parameters(params): Parameters<tools::automod::GetAutomodRuleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::automod::get_automod_rule(&self.discord, params).await
    }

    #[tool(description = "Delete an auto-moderation rule")]
    async fn delete_automod_rule(
        &self,
        Parameters(params): Parameters<tools::automod::DeleteAutomodRuleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::automod::delete_automod_rule(&self.discord, params).await
    }

    // ========================
    // SCHEDULED EVENTS
    // ========================

    #[tool(description = "List all scheduled events in a guild")]
    async fn list_scheduled_events(
        &self,
        Parameters(params): Parameters<tools::scheduled_events::ListScheduledEventsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::scheduled_events::list_scheduled_events(&self.discord, params).await
    }

    #[tool(description = "Get a specific scheduled event")]
    async fn get_scheduled_event(
        &self,
        Parameters(params): Parameters<tools::scheduled_events::GetScheduledEventParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::scheduled_events::get_scheduled_event(&self.discord, params).await
    }

    #[tool(description = "Delete a scheduled event")]
    async fn delete_scheduled_event(
        &self,
        Parameters(params): Parameters<tools::scheduled_events::DeleteScheduledEventParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::scheduled_events::delete_scheduled_event(&self.discord, params).await
    }

    #[tool(description = "List users interested in a scheduled event")]
    async fn list_scheduled_event_users(
        &self,
        Parameters(params): Parameters<tools::scheduled_events::ListScheduledEventUsersParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::scheduled_events::list_scheduled_event_users(&self.discord, params).await
    }

    // ========================
    // AUDIT LOG
    // ========================

    #[tool(description = "Get the audit log for a guild. Filter by user, action type, or get entries before a specific ID.")]
    async fn get_audit_log(
        &self,
        Parameters(params): Parameters<tools::audit_log::GetAuditLogParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::audit_log::get_audit_log(&self.discord, params).await
    }

    // ========================
    // GUILD SETTINGS
    // ========================

    #[tool(description = "Update a guild's settings (name, description)")]
    async fn update_guild(
        &self,
        Parameters(params): Parameters<tools::guild_settings::UpdateGuildParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::update_guild(&self.discord, params).await
    }

    #[tool(description = "Get the number of members that would be pruned for inactivity")]
    async fn get_guild_prune_count(
        &self,
        Parameters(params): Parameters<tools::guild_settings::GetGuildPruneCountParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::get_guild_prune_count(&self.discord, params).await
    }

    #[tool(description = "Begin pruning inactive guild members")]
    async fn begin_guild_prune(
        &self,
        Parameters(params): Parameters<tools::guild_settings::BeginGuildPruneParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::begin_guild_prune(&self.discord, params).await
    }

    #[tool(description = "Get a guild's vanity URL invite code")]
    async fn get_guild_vanity_url(
        &self,
        Parameters(params): Parameters<tools::guild_settings::GetGuildVanityUrlParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::get_guild_vanity_url(&self.discord, params).await
    }

    #[tool(description = "Get a guild's welcome screen configuration")]
    async fn get_guild_welcome_screen(
        &self,
        Parameters(params): Parameters<tools::guild_settings::GetGuildWelcomeScreenParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::get_guild_welcome_screen(&self.discord, params).await
    }

    #[tool(description = "Get a guild's widget data")]
    async fn get_guild_widget(
        &self,
        Parameters(params): Parameters<tools::guild_settings::GetGuildWidgetParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::get_guild_widget(&self.discord, params).await
    }

    #[tool(description = "Get available voice regions for a guild")]
    async fn get_guild_voice_regions(
        &self,
        Parameters(params): Parameters<tools::guild_settings::GetGuildVoiceRegionsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::get_guild_voice_regions(&self.discord, params).await
    }

    #[tool(description = "Get a guild's preview (available for discoverable guilds)")]
    async fn get_guild_preview(
        &self,
        Parameters(params): Parameters<tools::guild_settings::GetGuildPreviewParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::get_guild_preview(&self.discord, params).await
    }

    // ========================
    // PERMISSIONS
    // ========================

    #[tool(description = "Set a permission overwrite for a role or member on a channel. Provide allow/deny as decimal permission bitfield strings.")]
    async fn update_channel_permission(
        &self,
        Parameters(params): Parameters<tools::permissions::UpdateChannelPermissionParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::permissions::update_channel_permission(&self.discord, params).await
    }

    #[tool(description = "Delete a permission overwrite for a role or member on a channel")]
    async fn delete_channel_permission(
        &self,
        Parameters(params): Parameters<tools::permissions::DeleteChannelPermissionParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::permissions::delete_channel_permission(&self.discord, params).await
    }

    // ========================
    // STAGE INSTANCES
    // ========================

    #[tool(description = "Create a stage instance for a stage channel")]
    async fn create_stage_instance(
        &self,
        Parameters(params): Parameters<tools::stage_instances::CreateStageInstanceParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stage_instances::create_stage_instance(&self.discord, params).await
    }

    #[tool(description = "Get the stage instance for a stage channel")]
    async fn get_stage_instance(
        &self,
        Parameters(params): Parameters<tools::stage_instances::GetStageInstanceParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stage_instances::get_stage_instance(&self.discord, params).await
    }

    #[tool(description = "Update a stage instance's topic or privacy level")]
    async fn update_stage_instance(
        &self,
        Parameters(params): Parameters<tools::stage_instances::UpdateStageInstanceParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stage_instances::update_stage_instance(&self.discord, params).await
    }

    #[tool(description = "Delete a stage instance")]
    async fn delete_stage_instance(
        &self,
        Parameters(params): Parameters<tools::stage_instances::DeleteStageInstanceParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stage_instances::delete_stage_instance(&self.discord, params).await
    }

    // ========================
    // VOICE
    // ========================

    #[tool(description = "List available voice regions")]
    async fn list_voice_regions(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::voice::list_voice_regions(&self.discord).await
    }

    #[tool(description = "Update the current user's voice state in a guild (e.g., suppress/request to speak in stage)")]
    async fn update_current_user_voice_state(
        &self,
        Parameters(params): Parameters<tools::voice::UpdateCurrentUserVoiceStateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::voice::update_current_user_voice_state(&self.discord, params).await
    }

    #[tool(description = "Update another user's voice state in a guild (e.g., suppress in stage channel)")]
    async fn update_user_voice_state(
        &self,
        Parameters(params): Parameters<tools::voice::UpdateUserVoiceStateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::voice::update_user_voice_state(&self.discord, params).await
    }

    // ========================
    // USERS
    // ========================

    #[tool(description = "Get a user by ID")]
    async fn get_user(
        &self,
        Parameters(params): Parameters<tools::users::GetUserParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::users::get_user(&self.discord, params).await
    }

    #[tool(description = "Create a DM channel with a user")]
    async fn create_dm(
        &self,
        Parameters(params): Parameters<tools::users::CreateDmParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::users::create_dm(&self.discord, params).await
    }

    #[tool(description = "Leave a guild (server)")]
    async fn leave_guild(
        &self,
        Parameters(params): Parameters<tools::users::LeaveGuildParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::users::leave_guild(&self.discord, params).await
    }

    #[tool(description = "Get the current user's connected accounts (Twitch, YouTube, etc.)")]
    async fn get_current_user_connections(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::users::get_current_user_connections(&self.discord).await
    }

    #[tool(description = "Update the current bot user's username or avatar image")]
    async fn update_current_user(
        &self,
        Parameters(params): Parameters<tools::users::UpdateCurrentUserParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::users::update_current_user(&self.discord, params).await
    }

    // ========================
    // APPLICATION COMMANDS
    // ========================

    #[tool(description = "List all global application commands")]
    async fn list_global_commands(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::list_global_commands(&self.discord, self.application_id).await
    }

    #[tool(description = "Create a global slash command")]
    async fn create_global_command(
        &self,
        Parameters(params): Parameters<tools::application_commands::CreateGlobalCommandParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::create_global_command(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Update a global application command")]
    async fn update_global_command(
        &self,
        Parameters(params): Parameters<tools::application_commands::UpdateGlobalCommandParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::update_global_command(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Delete a global application command")]
    async fn delete_global_command(
        &self,
        Parameters(params): Parameters<tools::application_commands::DeleteGlobalCommandParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::delete_global_command(&self.discord, self.application_id, params).await
    }

    #[tool(description = "List all application commands for a guild")]
    async fn list_guild_commands(
        &self,
        Parameters(params): Parameters<tools::application_commands::ListGuildCommandsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::list_guild_commands(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Create a guild-specific slash command")]
    async fn create_guild_command(
        &self,
        Parameters(params): Parameters<tools::application_commands::CreateGuildCommandParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::create_guild_command(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Update a guild-specific application command")]
    async fn update_guild_command(
        &self,
        Parameters(params): Parameters<tools::application_commands::UpdateGuildCommandParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::update_guild_command(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Delete a guild-specific application command")]
    async fn delete_guild_command(
        &self,
        Parameters(params): Parameters<tools::application_commands::DeleteGuildCommandParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::application_commands::delete_guild_command(&self.discord, self.application_id, params).await
    }

    // ========================
    // INTERACTIONS
    // ========================

    #[tool(description = "Respond to an interaction (e.g., slash command). Types: 1=pong, 4=channel_message, 5=deferred_channel_message, 6=deferred_update, 7=update_message")]
    async fn create_interaction_response(
        &self,
        Parameters(params): Parameters<tools::interactions::CreateInteractionResponseParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::create_interaction_response(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Get the original interaction response message")]
    async fn get_original_response(
        &self,
        Parameters(params): Parameters<tools::interactions::GetOriginalResponseParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::get_original_response(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Edit the original interaction response")]
    async fn edit_original_response(
        &self,
        Parameters(params): Parameters<tools::interactions::EditOriginalResponseParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::edit_original_response(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Delete the original interaction response")]
    async fn delete_original_response(
        &self,
        Parameters(params): Parameters<tools::interactions::DeleteOriginalResponseParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::delete_original_response(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Create a followup message for an interaction")]
    async fn create_followup_message(
        &self,
        Parameters(params): Parameters<tools::interactions::CreateFollowupMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::create_followup_message(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Get a followup message for an interaction")]
    async fn get_followup_message(
        &self,
        Parameters(params): Parameters<tools::interactions::GetFollowupMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::get_followup_message(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Edit a followup message for an interaction")]
    async fn edit_followup_message(
        &self,
        Parameters(params): Parameters<tools::interactions::EditFollowupMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::edit_followup_message(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Delete a followup message for an interaction")]
    async fn delete_followup_message(
        &self,
        Parameters(params): Parameters<tools::interactions::DeleteFollowupMessageParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::interactions::delete_followup_message(&self.discord, self.application_id, params).await
    }

    // ========================
    // GUILD TEMPLATES
    // ========================

    #[tool(description = "Get a guild template by its code")]
    async fn get_template(
        &self,
        Parameters(params): Parameters<tools::guild_templates::GetTemplateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_templates::get_template(&self.discord, params).await
    }

    #[tool(description = "List all templates for a guild")]
    async fn list_guild_templates(
        &self,
        Parameters(params): Parameters<tools::guild_templates::ListGuildTemplatesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_templates::list_guild_templates(&self.discord, params).await
    }

    #[tool(description = "Create a template from a guild")]
    async fn create_guild_template(
        &self,
        Parameters(params): Parameters<tools::guild_templates::CreateGuildTemplateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_templates::create_guild_template(&self.discord, params).await
    }

    #[tool(description = "Sync a guild template to the guild's current state")]
    async fn sync_guild_template(
        &self,
        Parameters(params): Parameters<tools::guild_templates::SyncGuildTemplateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_templates::sync_guild_template(&self.discord, params).await
    }

    #[tool(description = "Update a guild template's name or description")]
    async fn update_guild_template(
        &self,
        Parameters(params): Parameters<tools::guild_templates::UpdateGuildTemplateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_templates::update_guild_template(&self.discord, params).await
    }

    #[tool(description = "Delete a guild template")]
    async fn delete_guild_template(
        &self,
        Parameters(params): Parameters<tools::guild_templates::DeleteGuildTemplateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_templates::delete_guild_template(&self.discord, params).await
    }

    // ========================
    // POLLS
    // ========================

    #[tool(description = "Get voters for a specific poll answer")]
    async fn get_poll_answer_voters(
        &self,
        Parameters(params): Parameters<tools::polls::GetPollAnswerVotersParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::polls::get_poll_answer_voters(&self.discord, params).await
    }

    #[tool(description = "Immediately end a poll")]
    async fn end_poll(
        &self,
        Parameters(params): Parameters<tools::polls::EndPollParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::polls::end_poll(&self.discord, params).await
    }

    // ========================
    // APPLICATION EMOJIS
    // ========================

    #[tool(description = "List all emojis for the application")]
    async fn list_application_emojis(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::list_application_emojis(&self.discord, self.application_id).await
    }

    #[tool(description = "Create a custom emoji for the application (requires base64 image data)")]
    async fn create_application_emoji(
        &self,
        Parameters(params): Parameters<tools::emojis::CreateApplicationEmojiParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::create_application_emoji(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Update an application emoji's name")]
    async fn update_application_emoji(
        &self,
        Parameters(params): Parameters<tools::emojis::UpdateApplicationEmojiParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::update_application_emoji(&self.discord, self.application_id, params).await
    }

    #[tool(description = "Delete an application emoji")]
    async fn delete_application_emoji(
        &self,
        Parameters(params): Parameters<tools::emojis::DeleteApplicationEmojiParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::emojis::delete_application_emoji(&self.discord, self.application_id, params).await
    }

    // ========================
    // AUTOMOD (new tools)
    // ========================

    #[tool(description = "Create an auto-moderation rule in a guild. Trigger types: 1=keyword, 3=spam, 4=keyword_preset, 5=mention_spam")]
    async fn create_automod_rule(
        &self,
        Parameters(params): Parameters<tools::automod::CreateAutomodRuleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::automod::create_automod_rule(&self.discord, params).await
    }

    #[tool(description = "Update an auto-moderation rule")]
    async fn update_automod_rule(
        &self,
        Parameters(params): Parameters<tools::automod::UpdateAutomodRuleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::automod::update_automod_rule(&self.discord, params).await
    }

    // ========================
    // SCHEDULED EVENTS (new tools)
    // ========================

    #[tool(description = "Create a scheduled event in a guild. Entity types: 1=stage_instance, 2=voice, 3=external")]
    async fn create_scheduled_event(
        &self,
        Parameters(params): Parameters<tools::scheduled_events::CreateScheduledEventParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::scheduled_events::create_scheduled_event(&self.discord, params).await
    }

    #[tool(description = "Update a scheduled event")]
    async fn update_scheduled_event(
        &self,
        Parameters(params): Parameters<tools::scheduled_events::UpdateScheduledEventParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::scheduled_events::update_scheduled_event(&self.discord, params).await
    }

    // ========================
    // STICKERS (new tools)
    // ========================

    #[tool(description = "Get a sticker by ID (any sticker, not guild-specific)")]
    async fn get_sticker(
        &self,
        Parameters(params): Parameters<tools::stickers::GetStickerParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stickers::get_sticker(&self.discord, params).await
    }

    #[tool(description = "List all available Nitro sticker packs")]
    async fn list_sticker_packs(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::stickers::list_sticker_packs(&self.discord).await
    }

    // ========================
    // GUILD SETTINGS (new tools)
    // ========================

    #[tool(description = "Delete a guild (server). The bot must be the owner.")]
    async fn delete_guild(
        &self,
        Parameters(params): Parameters<tools::guild_settings::DeleteGuildParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::delete_guild(&self.discord, params).await
    }

    #[tool(description = "Get a guild's integrations (bots, apps, etc.)")]
    async fn get_guild_integrations(
        &self,
        Parameters(params): Parameters<tools::guild_settings::GetGuildIntegrationsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::get_guild_integrations(&self.discord, params).await
    }

    #[tool(description = "Create a new guild (server). Only available for bots in fewer than 10 guilds.")]
    async fn create_guild(
        &self,
        Parameters(params): Parameters<tools::guild_settings::CreateGuildParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::guild_settings::create_guild(self, params).await
    }

    // ========================
    // CHANNELS (new tools)
    // ========================

    #[tool(description = "Follow an announcement channel so messages are crossposted to a target channel")]
    async fn follow_announcement_channel(
        &self,
        Parameters(params): Parameters<tools::channels::FollowAnnouncementChannelParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::follow_announcement_channel(&self.discord, params).await
    }

    #[tool(description = "Trigger a typing indicator in a channel (lasts ~10 seconds)")]
    async fn trigger_typing_indicator(
        &self,
        Parameters(params): Parameters<tools::channels::TriggerTypingIndicatorParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::channels::trigger_typing_indicator(&self.discord, params).await
    }

    // ========================
    // THREADS (new tools)
    // ========================

    #[tool(description = "List private archived threads in a channel")]
    async fn list_private_archived_threads(
        &self,
        Parameters(params): Parameters<tools::threads::ListPrivateArchivedThreadsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::threads::list_private_archived_threads(&self.discord, params).await
    }

    // ========================
    // BANS (new tools)
    // ========================

    #[tool(description = "Bulk ban up to 200 users from a guild")]
    async fn bulk_guild_ban(
        &self,
        Parameters(params): Parameters<tools::bans::BulkGuildBanParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::bans::bulk_guild_ban(self, params).await
    }

    // ========================
    // SOUNDBOARD
    // ========================

    #[tool(description = "Send a soundboard sound to a voice channel")]
    async fn send_soundboard_sound(
        &self,
        Parameters(params): Parameters<tools::soundboard::SendSoundboardSoundParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::soundboard::send_soundboard_sound(self, params).await
    }

    #[tool(description = "List default soundboard sounds available to all guilds")]
    async fn list_default_soundboard_sounds(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::soundboard::list_default_soundboard_sounds(self).await
    }

    #[tool(description = "List all soundboard sounds in a guild")]
    async fn list_guild_soundboard_sounds(
        &self,
        Parameters(params): Parameters<tools::soundboard::ListGuildSoundboardSoundsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::soundboard::list_guild_soundboard_sounds(self, params).await
    }

    #[tool(description = "Get a specific soundboard sound in a guild")]
    async fn get_guild_soundboard_sound(
        &self,
        Parameters(params): Parameters<tools::soundboard::GetGuildSoundboardSoundParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::soundboard::get_guild_soundboard_sound(self, params).await
    }

    #[tool(description = "Create a soundboard sound in a guild")]
    async fn create_guild_soundboard_sound(
        &self,
        Parameters(params): Parameters<tools::soundboard::CreateGuildSoundboardSoundParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::soundboard::create_guild_soundboard_sound(self, params).await
    }

    #[tool(description = "Update a soundboard sound in a guild")]
    async fn update_guild_soundboard_sound(
        &self,
        Parameters(params): Parameters<tools::soundboard::UpdateGuildSoundboardSoundParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::soundboard::update_guild_soundboard_sound(self, params).await
    }

    #[tool(description = "Delete a soundboard sound from a guild")]
    async fn delete_guild_soundboard_sound(
        &self,
        Parameters(params): Parameters<tools::soundboard::DeleteGuildSoundboardSoundParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::soundboard::delete_guild_soundboard_sound(self, params).await
    }

    // ========================
    // MONETIZATION
    // ========================

    #[tool(description = "List all SKUs for the application")]
    async fn list_skus(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::monetization::list_skus(self, self.application_id).await
    }

    #[tool(description = "List entitlements for the application, with optional filters")]
    async fn list_entitlements(
        &self,
        Parameters(params): Parameters<tools::monetization::ListEntitlementsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::monetization::list_entitlements(self, self.application_id, params).await
    }

    #[tool(description = "Get a specific entitlement by ID")]
    async fn get_entitlement(
        &self,
        Parameters(params): Parameters<tools::monetization::GetEntitlementParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::monetization::get_entitlement(self, self.application_id, params).await
    }

    #[tool(description = "Create a test entitlement for testing monetization")]
    async fn create_test_entitlement(
        &self,
        Parameters(params): Parameters<tools::monetization::CreateTestEntitlementParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::monetization::create_test_entitlement(self, self.application_id, params).await
    }

    #[tool(description = "Delete a test entitlement")]
    async fn delete_test_entitlement(
        &self,
        Parameters(params): Parameters<tools::monetization::DeleteTestEntitlementParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::monetization::delete_test_entitlement(self, self.application_id, params).await
    }

    #[tool(description = "Mark a one-time purchase entitlement as consumed")]
    async fn consume_entitlement(
        &self,
        Parameters(params): Parameters<tools::monetization::ConsumeEntitlementParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::monetization::consume_entitlement(self, self.application_id, params).await
    }

    #[tool(description = "List subscriptions for a SKU")]
    async fn list_sku_subscriptions(
        &self,
        Parameters(params): Parameters<tools::monetization::ListSkuSubscriptionsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tools::monetization::list_sku_subscriptions(self, params).await
    }
}

#[tool_handler]
impl ServerHandler for DiscordMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Discord MCP Server: Interact with the Discord API. \
                 Start with get_current_user and list_guilds to discover available servers, \
                 then use specific tools for channels, messages, members, roles, etc. \
                 All ID parameters accept string snowflake IDs."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: rmcp::model::Implementation::from_build_env(),
            ..Default::default()
        }
    }
}
