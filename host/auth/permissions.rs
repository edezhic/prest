use std::collections::HashSet;

use axum_login::AuthzBackend;

use crate::*;

pub type Permission = String;

#[async_trait]
impl AuthzBackend for Prest {
    type Permission = Permission;

    async fn get_user_permissions(
        &self,
        user: &Self::User,
    ) -> std::result::Result<HashSet<Self::Permission>, Self::Error> {
        Ok(user.permissions.iter().map(|s| s.to_owned()).collect())
    }

    async fn get_group_permissions(
        &self,
        user: &Self::User,
    ) -> std::result::Result<HashSet<Self::Permission>, Self::Error> {
        Ok(user.permissions.iter().map(|s| s.to_owned()).collect())
    }

    async fn get_all_permissions(
        &self,
        user: &Self::User,
    ) -> std::result::Result<HashSet<Self::Permission>, Self::Error> {
        Ok(user.permissions.iter().map(|s| s.to_owned()).collect())
    }

    async fn has_perm(
        &self,
        user: &Self::User,
        perm: Self::Permission,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(user.permissions.iter().find(|p| **p == perm).is_some())
    }
}
