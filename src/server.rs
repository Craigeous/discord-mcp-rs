use std::sync::Arc;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use twilight_http::Client as DiscordClient;

use crate::tools;

#[derive(Clone)]
pub struct DiscordMcpServer {
    discord: Arc<DiscordClient>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DiscordMcpServer {
    pub fn from_env() -> anyhow::Result<Self> {
        let token = std::env::var("DISCORD_TOKEN")
            .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN environment variable not set"))?;
        let discord = Arc::new(DiscordClient::new(token));
        Ok(Self {
            discord,
            tool_router: Self::tool_router(),
        })
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
