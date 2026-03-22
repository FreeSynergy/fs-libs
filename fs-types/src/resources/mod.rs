//! Store resource type system — packages that can be published and installed.
//!
//! Every entry in a FreeSynergy store is a **resource**.  This module contains
//! the common envelope ([`ResourceMeta`]), the top-level type discriminant
//! ([`ResourceType`] / [`ValidationStatus`]), and one sub-module per resource
//! category.

pub mod app;
pub mod bot;
pub mod bridge;
pub mod bundle;
pub mod container;
pub mod messenger_adapter;
pub mod meta;
pub mod platform;
pub mod role_api;
pub mod theme;
pub mod validator;
pub mod widget;

pub use app::AppResource;
pub use messenger_adapter::{
    AdapterAuthMethod, ChannelFeature, MessengerAdapterResource, MessengerKind,
};
pub use bot::BotResource;
pub use bridge::BridgeResource;
pub use bundle::BundleResource;
pub use container::ContainerResource;
pub use meta::{Dependency, PackageSource, ResourceMeta, ResourceType, Role, ValidationStatus};
pub use platform::{OsFamily, PlatformFilter, RequiredFeature, platform_filter_from_tags};
pub use theme::{
    AnimationSet, ButtonStyle, ColorScheme, CursorSet, FontSet, IconSet, StyleResource,
    WindowChrome,
};
pub use role_api::{
    Alert, CacheEntry, CacheSet, ChatChannel, ChatMessageSend, DbQueryRequest, DbQueryResult,
    DbSchemaTable, GeoLocation, GitCommit, GitRepo, GitRepoCreate, IamGroup, IamGroupAddMember,
    IamGroupCreate, IamUser, IamUserCreate, IamUserUpdate, LlmCompletionRequest,
    LlmCompletionResponse, LlmModel, MailSend, MapTileRequest, MetricPoint, MetricQuery, Task,
    TaskCreate, TaskUpdate, WikiPage, WikiPageCreate, WikiPageSummary, WikiSearchResult,
};
pub use validator::Validate;
pub use widget::WidgetResource;
