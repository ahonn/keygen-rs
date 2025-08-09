use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WebhookEvent {
    // Account events
    #[serde(rename = "account.billing.updated")]
    AccountBillingUpdated,
    #[serde(rename = "account.plan.updated")]
    AccountPlanUpdated,
    #[serde(rename = "account.subscription.canceled")]
    AccountSubscriptionCanceled,
    #[serde(rename = "account.subscription.paused")]
    AccountSubscriptionPaused,
    #[serde(rename = "account.subscription.renewed")]
    AccountSubscriptionRenewed,
    #[serde(rename = "account.subscription.resumed")]
    AccountSubscriptionResumed,
    #[serde(rename = "account.updated")]
    AccountUpdated,

    // Artifact events
    #[serde(rename = "artifact.created")]
    ArtifactCreated,
    #[serde(rename = "artifact.deleted")]
    ArtifactDeleted,
    #[serde(rename = "artifact.downloaded")]
    ArtifactDownloaded,
    #[serde(rename = "artifact.updated")]
    ArtifactUpdated,
    #[serde(rename = "artifact.upload.processing")]
    ArtifactUploadProcessing,
    #[serde(rename = "artifact.upload.succeeded")]
    ArtifactUploadSucceeded,
    #[serde(rename = "artifact.upload.failed")]
    ArtifactUploadFailed,
    #[serde(rename = "artifact.uploaded")]
    ArtifactUploaded,

    // Component events
    #[serde(rename = "component.created")]
    ComponentCreated,
    #[serde(rename = "component.deleted")]
    ComponentDeleted,
    #[serde(rename = "component.updated")]
    ComponentUpdated,

    // Entitlement events
    #[serde(rename = "entitlement.created")]
    EntitlementCreated,
    #[serde(rename = "entitlement.deleted")]
    EntitlementDeleted,
    #[serde(rename = "entitlement.updated")]
    EntitlementUpdated,

    // Group events
    #[serde(rename = "group.created")]
    GroupCreated,
    #[serde(rename = "group.deleted")]
    GroupDeleted,
    #[serde(rename = "group.owners.attached")]
    GroupOwnersAttached,
    #[serde(rename = "group.owners.detached")]
    GroupOwnersDetached,
    #[serde(rename = "group.updated")]
    GroupUpdated,

    // License events
    #[serde(rename = "license.check-in-overdue")]
    LicenseCheckInOverdue,
    #[serde(rename = "license.check-in-required-soon")]
    LicenseCheckInRequiredSoon,
    #[serde(rename = "license.checked-in")]
    LicenseCheckedIn,
    #[serde(rename = "license.checked-out")]
    LicenseCheckedOut,
    #[serde(rename = "license.created")]
    LicenseCreated,
    #[serde(rename = "license.deleted")]
    LicenseDeleted,
    #[serde(rename = "license.entitlements.attached")]
    LicenseEntitlementsAttached,
    #[serde(rename = "license.entitlements.detached")]
    LicenseEntitlementsDetached,
    #[serde(rename = "license.expired")]
    LicenseExpired,
    #[serde(rename = "license.expiring-soon")]
    LicenseExpiringSoon,
    #[serde(rename = "license.group.updated")]
    LicenseGroupUpdated,
    #[serde(rename = "license.policy.updated")]
    LicensePolicyUpdated,
    #[serde(rename = "license.reinstated")]
    LicenseReinstated,
    #[serde(rename = "license.renewed")]
    LicenseRenewed,
    #[serde(rename = "license.revoked")]
    LicenseRevoked,
    #[serde(rename = "license.suspended")]
    LicenseSuspended,
    #[serde(rename = "license.updated")]
    LicenseUpdated,
    #[serde(rename = "license.usage.decremented")]
    LicenseUsageDecremented,
    #[serde(rename = "license.usage.incremented")]
    LicenseUsageIncremented,
    #[serde(rename = "license.usage.reset")]
    LicenseUsageReset,
    #[serde(rename = "license.owner.updated")]
    LicenseOwnerUpdated,
    #[serde(rename = "license.users.attached")]
    LicenseUsersAttached,
    #[serde(rename = "license.users.detached")]
    LicenseUsersDetached,
    #[serde(rename = "license.validation.failed")]
    LicenseValidationFailed,
    #[serde(rename = "license.validation.succeeded")]
    LicenseValidationSucceeded,

    // Machine events
    #[serde(rename = "machine.checked-out")]
    MachineCheckedOut,
    #[serde(rename = "machine.created")]
    MachineCreated,
    #[serde(rename = "machine.deleted")]
    MachineDeleted,
    #[serde(rename = "machine.group.updated")]
    MachineGroupUpdated,
    #[serde(rename = "machine.owner.updated")]
    MachineOwnerUpdated,
    #[serde(rename = "machine.heartbeat.dead")]
    MachineHeartbeatDead,
    #[serde(rename = "machine.heartbeat.ping")]
    MachineHeartbeatPing,
    #[serde(rename = "machine.heartbeat.reset")]
    MachineHeartbeatReset,
    #[serde(rename = "machine.heartbeat.resurrected")]
    MachineHeartbeatResurrected,
    #[serde(rename = "machine.updated")]
    MachineUpdated,

    // Package events
    #[serde(rename = "package.created")]
    PackageCreated,
    #[serde(rename = "package.deleted")]
    PackageDeleted,
    #[serde(rename = "package.updated")]
    PackageUpdated,

    // Policy events
    #[serde(rename = "policy.created")]
    PolicyCreated,
    #[serde(rename = "policy.deleted")]
    PolicyDeleted,
    #[serde(rename = "policy.entitlements.attached")]
    PolicyEntitlementsAttached,
    #[serde(rename = "policy.entitlements.detached")]
    PolicyEntitlementsDetached,
    #[serde(rename = "policy.pool.popped")]
    PolicyPoolPopped,
    #[serde(rename = "policy.updated")]
    PolicyUpdated,

    // Product events
    #[serde(rename = "product.created")]
    ProductCreated,
    #[serde(rename = "product.deleted")]
    ProductDeleted,
    #[serde(rename = "product.updated")]
    ProductUpdated,

    // Release events
    #[serde(rename = "release.constraints.attached")]
    ReleaseConstraintsAttached,
    #[serde(rename = "release.constraints.detached")]
    ReleaseConstraintsDetached,
    #[serde(rename = "release.created")]
    ReleaseCreated,
    #[serde(rename = "release.deleted")]
    ReleaseDeleted,
    #[serde(rename = "release.package.updated")]
    ReleasePackageUpdated,
    #[serde(rename = "release.published")]
    ReleasePublished,
    #[serde(rename = "release.updated")]
    ReleaseUpdated,
    #[serde(rename = "release.upgraded")]
    ReleaseUpgraded,
    #[serde(rename = "release.yanked")]
    ReleaseYanked,

    // Second factor events
    #[serde(rename = "second-factor.created")]
    SecondFactorCreated,
    #[serde(rename = "second-factor.deleted")]
    SecondFactorDeleted,
    #[serde(rename = "second-factor.disabled")]
    SecondFactorDisabled,
    #[serde(rename = "second-factor.enabled")]
    SecondFactorEnabled,

    // Token events
    #[serde(rename = "token.generated")]
    TokenGenerated,
    #[serde(rename = "token.regenerated")]
    TokenRegenerated,
    #[serde(rename = "token.revoked")]
    TokenRevoked,

    // User events
    #[serde(rename = "user.banned")]
    UserBanned,
    #[serde(rename = "user.created")]
    UserCreated,
    #[serde(rename = "user.deleted")]
    UserDeleted,
    #[serde(rename = "user.group.updated")]
    UserGroupUpdated,
    #[serde(rename = "user.password-reset")]
    UserPasswordReset,
    #[serde(rename = "user.unbanned")]
    UserUnbanned,
    #[serde(rename = "user.updated")]
    UserUpdated,

    // Wildcard for all events
    #[serde(rename = "*")]
    All,
}
