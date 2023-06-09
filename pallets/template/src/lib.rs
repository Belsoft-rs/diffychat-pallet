#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{DispatchResult, OptionQuery, StorageMap, *},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::{OriginFor, *};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct ContactByAccountId {
		// encoded name
		pub name: EncodedContactName,
	}

	impl Default for ContactByAccountId {
		fn default() -> Self {
			ContactByAccountId { name: [0_u8; 1000] }
		}
	}

	pub type EncodedContactName = [u8; 1000];
	pub type EncodedContactAddr = [u8; 1000];

	#[pallet::storage]
	#[pallet::getter(fn get_contact_by_account_id)]
	pub type ContactByAccountIdStore<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		EncodedContactAddr,
		ContactByAccountId,
		ValueQuery,
	>;

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, Default, TypeInfo,
	)]
	pub struct ItemByAccountId {
		pub address: [u8; 32],
		pub nickname: [u8; 21],
	}

	#[pallet::storage]
	#[pallet::getter(fn get_address_by_nickname)]
	pub type ItemByNicknameStore<T: Config> =
		StorageMap<_, Blake2_128Concat, [u8; 21], T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_address_by_account_id)]
	pub type ItemByAccountIdStore<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, ItemByAccountId, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Offer {
			offer: [u8; 2048],
			offered_by: T::AccountId,
			offered_to: T::AccountId,
			welcome_msg: [u8; 300],
		},
		Answer {
			answer: [u8; 2048],
			answer_from: T::AccountId,
			answer_to: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// AlreadyRegistered - nickname <-> address is already registered
		AccountIdAlreadyRegistered,
		NicknameAlreadyRegistered,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// open chat request
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn offer_chat(
			origin: OriginFor<T>,
			welcome_msg: [u8; 300],
			offer: [u8; 2048],
			to: T::AccountId,
		) -> DispatchResult {
			// who wanna open discuss
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::Offer {
				offer,
				offered_by: who,
				offered_to: to,
				welcome_msg,
			});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1).ref_time())]
		pub fn register(
			origin: OriginFor<T>,
			nickname: [u8; 21],
			address: [u8; 32],
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			if <ItemByAccountIdStore<T>>::contains_key(owner.clone()) {
				return Err(Error::<T>::AccountIdAlreadyRegistered.into())
			}

			if <ItemByNicknameStore<T>>::contains_key(nickname.clone()) {
				return Err(Error::<T>::NicknameAlreadyRegistered.into())
			}

			<ItemByNicknameStore<T>>::insert(nickname, owner.clone());
			<ItemByAccountIdStore<T>>::insert(owner, ItemByAccountId { address, nickname });

			Ok(())
		}
		// answering on open chat request
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn answer_chat(
			origin: OriginFor<T>,
			answer: [u8; 2048],
			to: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::Answer { answer, answer_from: who, answer_to: to });
			Ok(())
		}
		// updating or inserting contact to sender contact list
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn upsert_contact(
			origin: OriginFor<T>,
			contact_name: EncodedContactName,
			contact_addr: EncodedContactAddr,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<ContactByAccountIdStore<T>>::set(
				who,
				contact_addr,
				ContactByAccountId { name: contact_name },
			);
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_contact(
			origin: OriginFor<T>,
			contact_addr: EncodedContactAddr,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<ContactByAccountIdStore<T>>::remove(who, contact_addr);
			Ok(())
		}
	}
}
