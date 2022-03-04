pub use self::pallet::*;

mod trait;
pub use trait::{XAsset, XTransact};

#[allow(unused_variables)]
#[frame_support::pallet]
pub mod pallet {
    use crate::trait
	use crate::pallet_assets_wrapper;
	use crate::xcm_helper::{ConcrateAsset, NativeAssetChecker};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, StorageVersion},
		transactional,
		weights::Weight,
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_runtime::{traits::AccountIdConversion, DispatchError};
	use sp_std::{convert::TryInto, prelude::*, vec};

	/// The logging target.
	const LOG_TARGET: &str = "runtime::xtransfer";
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		/// Required origin for executing XCM messages, including the teleport functionality. If successful,
		/// then it resolves to `MultiLocation` which exists as an interior location within this chain's XCM
		/// context.
		type ExecuteXcmOrigin: EnsureOrigin<Self::Origin, Success = MultiLocation>;

        type ChainBridgeTransactor: XTransact;
        type CelerBridgeTransactor: XTransact;
        type XcmTransactor: XTransact;
	}


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Assets being sent to other chains
		Transfered {
			asset: MultiAsset,
			origin: MultiLocation,
			dest: MultiLocation,
		},
		/// Assets being deposited, including being deposited to reserve account of solo chains.
		Deposited {
			what: MultiAsset,
			who: MultiLocation,
		},
		/// Assets being withdrawn
		Withdrawn {
			what: MultiAsset,
			who: MultiLocation,
		},
        /// Assets being forwarded
        Forwarded {
            what: MultiAsset,
            who: MultiLocation,
        }
	}

	#[pallet::error]
	pub enum Error<T> {
		UnknownError,
		CannotReanchor,
		UnweighableMessage,
		FeePaymentEmpty,
		ExecutionFailed,
		UnknownTransfer,
		AssetNotFound,
		LocationInvertFailed,
		IllegalDestination,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: Into<[u8; 32]> + From<[u8; 32]>,
		BalanceOf<T>: Into<u128>,
	{
		#[pallet::weight(195_000_000 + Pallet::<T>::estimate_transfer_weight())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			asset: XAsset,
			dest: MultiLocation,
			amount: BalanceOf<T>,
			dest_weight: Weight,
		) -> DispatchResult {
            let origin = T::ExecuteXcmOrigin::ensure_origin(origin)?;
            // Convert asset to MultiAsset
            let asset = match asset {
                NonFungible(_) => {
                    MultiAsset  {
                        fn: ,
                        id: ,
                    }
                },
                _ => {
                    MultiAsset {
                        fun: Fungible(amount),
                        id: asset.into(),
                    }
                },
            };
            match (dest.parents, &dest.interior) {
                (
                    0,
                    Junctions::X3(GeneralKey(cb_key), GeneralIndex(dest_id), GeneralKey(recipient)),
                ) => {  // Transfer to solo chain through ChainBridge
                    if (cb_key == "cb") {
                        T::ChainBridgeTransactor::transfer_fungible(origin, asset, dest);
                    } else if (cb_key == "ce") {    // Transfer to solo chain through CelerBridge
                        T::CelerBridgeTransactor::transfer_fungible(origin, asset, dest);
                    } else {
                        // Error
                    }
                },
                (
                    1,
                    AccountId32 {

                    }
                ) | (1, Junctions::X2(Parachain(paraid), AccountId32 {})) => {
                    T::XcmTransactor::transfer_fungible(origin, asset, dest);
                },
                _ => {
                    // Error
                }
		}
	}
}
