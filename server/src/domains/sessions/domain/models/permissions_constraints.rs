//! Test on the system of permissions

use super::{Permission, PermissionDiscriminants, UserRole};
use std::iter::once;
use strum::IntoEnumIterator;

fn all_permissions() -> impl Iterator<Item = Permission> {
    use Permission::*;
    PermissionDiscriminants::iter().flat_map(|d| {
        let iter: Box<dyn Iterator<Item = Permission>> = match d {
            PermissionDiscriminants::GetData => Box::new(once(GetData)),
            PermissionDiscriminants::SetData => Box::new(once(SetData)),
            PermissionDiscriminants::GetHistory => Box::new(once(GetHistory)),
            PermissionDiscriminants::Delete => Box::new(once(Delete)),
            PermissionDiscriminants::GetUsers => Box::new(once(GetUsers)),
            PermissionDiscriminants::AddUser => {
                Box::new(UserRole::iter().map(|role| AddUser { role }))
            }
            PermissionDiscriminants::RemoveUser => {
                Box::new(UserRole::iter().map(|role| RemoveUser { role }))
            }
            PermissionDiscriminants::RemoveSelf => Box::new(once(RemoveSelf)),
            PermissionDiscriminants::SetRole => Box::new(
                UserRole::iter()
                    .flat_map(|from| UserRole::iter().map(move |to| SetRole { from, to })),
            ),
            PermissionDiscriminants::SetSelfRole => {
                Box::new(UserRole::iter().map(|to| SetSelfRole { to }))
            }
            PermissionDiscriminants::SendCommand => Box::new(once(SendCommand)),
        };
        iter.inspect(move |p| assert_eq!(PermissionDiscriminants::from(p), d))
    })
}

#[test]
/// Ensure that impossible role edit cannot be obtained with removing and adding.
fn impossible_role_edits_cannot_be_obtained_with_removing_and_adding() {
    for role_self in UserRole::iter() {
        for role_other in UserRole::iter() {
            for new_role_other in UserRole::iter() {
                if !role_self.can(Permission::SetRole {
                    from: role_other,
                    to: role_other,
                }) {
                    assert!(
                        !(role_self.can(Permission::RemoveUser { role: role_other })
                            && role_self.can(Permission::AddUser {
                            role: new_role_other
                        })),
                        "Users with role {role_self:?} cannot change users with role {role_other:?} to role {new_role_other:?}, but can remove then and add them with the new role",
                    )
                }
            }
        }
    }
}

#[test]
/// Ensure that admins are valid players
fn admins_are_valid_players() {
    for permission in all_permissions() {
        if UserRole::Player.can(permission) {
            assert!(
                UserRole::Admin.can(permission),
                "Admins cannot {permission:?}, but players can"
            );
        }
    }
}
#[test]
/// Ensure that admins are valid players
fn players_are_valid_observers() {
    for permission in all_permissions() {
        if UserRole::Observer.can(permission) {
            assert!(
                UserRole::Player.can(permission),
                "Players cannot {permission:?}, but observers can"
            );
        }
    }
}
