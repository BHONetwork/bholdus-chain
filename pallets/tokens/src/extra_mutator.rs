//! Datatype for easy mutation of the extra "sidecar" data.

use super::*;

/// A mutator type allowing inspection and possible modification of the extra "sidecare" data.
///
/// This may be used as a `Deref` for the pallet's extra data.

pub struct ExtraMutator<T: Config<I>, I: 'static = ()> {
	id: T::AssetId,
	who: T::AccountId,
	original: T::Extra,
	pending: Option<T::Extra>,
}

impl<T: Config<I>, I: 'static> Drop for ExtraMutator<T, I> {
	fn drop(&mut self) {
		debug_assert!(self.commit().is_ok(), "attempt to write to non-existent asset account");
	}
}

impl<T: Config<I>, I: 'static> sp_std::ops::Deref for ExtraMutator<T, I> {
	type Target = T::Extra;
	fn deref(&self) -> &T::Extra {
		match self.pending {
			Some(ref value) => value,
			None => &self.original,
		}
	}
}

impl<T: Config<I>, I: 'static> ExtraMutator<T, I> {
	pub(super) fn maybe_new(
		id: T::AssetId,
		who: impl sp_std::borrow::Borrow<T::AccountId>,
	) -> Option<ExtraMutator<T, I>> {
		if Account::<T, I>::contains_key(id, who.borrow()) {
			Some(ExtraMutator::<T, I> {
				id,
				who: who.borrow().clone(),
				original: Account::<T, I>::get(id, who.borrow()).extra,
				pending: None,
			})
		} else {
			None
		}
	}
	/// Commit any changes to storage.
	pub fn commit(&mut self) -> Result<(), ()> {
		if let Some(extra) = self.pending.take() {
			Account::<T, I>::try_mutate_exists(self.id, self.who.borrow(), |maybe_account| {
				if let Some(ref mut account) = maybe_account {
					account.extra = extra;
					Ok(())
				} else {
					Err(())
				}
			})
		} else {
			Ok(())
		}
	}
	/// Revert any changes, even those already committed by `Self` and drop self.
	pub fn revert(mut self) -> Result<(), ()> {
		self.pending = None;
		Account::<T, I>::try_mutate_exists(self.id, self.who.borrow(), |maybe_account| {
			if let Some(ref mut account) = maybe_account {
				account.extra = self.original.clone();
				Ok(())
			} else {
				Err(())
			}
		})
	}
}
