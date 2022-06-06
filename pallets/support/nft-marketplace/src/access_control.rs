use crate::*;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Permission {
	BanUser,
	UnbanUser,
	Ban,
	Unban,
	ApproveListing,
	RejectListing,
	None,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum RoleType {
	Manager(ManagerRole),
	Member(MemberRole),
}

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum ManagerRole {
	Admin,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MemberRole {
	Mod,
	Copywriter,
}

impl<T: Config> Pallet<T> {
	pub fn grant_role(
		origin: OriginFor<T>,
		role: &RoleType,
		account: &T::AccountId,
	) -> DispatchResult {
		if Self::has_role(&role, account) {
			return Err(Error::<T>::RoleRedundant.into());
		}
		match role {
			RoleType::Manager(ManagerRole::Admin) => {
				ensure_root(origin)?;
				let access_item = vec![
					Permission::BanUser,
					Permission::UnbanUser,
					Permission::Ban,
					Permission::Unban,
					Permission::ApproveListing,
					Permission::RejectListing,
				];
				AccessControl::<T>::insert(role, account, access_item);
			},
			RoleType::Member(_) => {
				let who = ensure_signed(origin)?;
				Self::check_admin_role(&who)?;
				AccessControl::<T>::insert(role, account, vec![Permission::None]);
			},
		}
		Ok(())
	}

	pub fn revoke_role(
		origin: OriginFor<T>,
		role: &RoleType,
		account: &T::AccountId,
	) -> DispatchResult {
		Self::check_role(role, account)?;
		match role {
			RoleType::Manager(ManagerRole::Admin) => ensure_root(origin)?,
			RoleType::Member(_) => {
				let who = ensure_signed(origin)?;
				Self::check_admin_role(&who)?
			},
		};
		Self::do_revoke_role(&role, account);
		Ok(())
	}

	pub fn do_revoke_role(role: &RoleType, account: &T::AccountId) {
		AccessControl::<T>::remove(role, account)
	}
}

impl<T: Config> Pallet<T> {
	pub fn check_role(role: &RoleType, account: &T::AccountId) -> DispatchResult {
		if !Self::has_role(role, account) {
			return Err(Error::<T>::MissingRole.into());
		}
		Ok(())
	}

	pub fn check_permission(
		permission: &Permission,
		role: &RoleType,
		account: &T::AccountId,
	) -> DispatchResult {
		if !Self::has_permission(permission, role, account) {
			return Err(Error::<T>::MissingPermission.into());
		}
		Ok(())
	}

	pub fn check_admin_permission(
		permission: &Permission,
		account: &T::AccountId,
	) -> DispatchResult {
		Self::check_permission(permission, &RoleType::Manager(ManagerRole::Admin), account)
	}

	pub fn check_admin_role(account: &T::AccountId) -> DispatchResult {
		Self::check_role(&RoleType::Manager(ManagerRole::Admin), account)
	}

	pub fn has_admin_role(account: &T::AccountId) -> bool {
		Self::has_role(&RoleType::Manager(ManagerRole::Admin), account)
	}

	pub fn has_role(role: &RoleType, account: &T::AccountId) -> bool {
		AccessControl::<T>::contains_key(role, account)
	}

	// Permission Management
	pub fn has_permission(
		permission: &Permission,
		role: &RoleType,
		account: &T::AccountId,
	) -> bool {
		if let Some(permissions) = AccessControl::<T>::get(role, account) {
			permissions.contains(permission)
		} else {
			false
		}
	}

	pub fn has_admin_permission(permission: &Permission, account: &T::AccountId) -> bool {
		Self::has_permission(permission, &RoleType::Manager(ManagerRole::Admin), account)
	}
}
