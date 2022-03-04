use cumulus_primitives_core::ParaId;
use frame_support::pallet_prelude::*;
use sp_core::U256;
use sp_runtime::traits::CheckedConversion;
use sp_std::{
    convert::{Into, TryFrom, TryInto},
    marker::PhantomData,
    result,
};
use xcm::latest::{
    prelude::*, AssetId::Concrete, Error as XcmError, Fungibility::Fungible, MultiAsset,
    MultiLocation, Result as XcmResult,
};
use xcm_builder::TakeRevenue;
use xcm_executor::{
    traits::{
        Error as MatchError, FilterAssetLocation, MatchesFungible, MatchesFungibles,
        TransactAsset,
    },
    Assets,
};

pub struct XcmAdapter<
    NativeAdapter,
    FungibleAdapter,
    DepositHandler,
    WithdrawHandler,
    NativeChecker,
    BridgeTransactor,
    BridgeFeeInfo,
    AccountId,
    Treasury,
>(
PhantomData<(
    NativeAdapter,
    FungibleAdapter,
    DepositHandler,
    WithdrawHandler,
    NativeChecker,
    BridgeTransactor,
    BridgeFeeInfo,
    AccountId,
    Treasury,
)>,
);

impl<
    NativeAdapter: TransactAsset,
    FungibleAdapter: TransactAsset,
    DepositHandler: XcmOnDeposited,
    WithdrawHandler: XcmOnWithdrawn,
    NativeChecker: NativeAssetChecker,
    BridgeTransactor: BridgeTransact,
    BridgeFeeInfo: GetBridgeFee,
    AccountId: From<[u8; 32]> + Into<[u8; 32]> + Clone,
    Treasury: Get<AccountId>,
> TransactAsset
for XcmAdapter<
    NativeAdapter,
    FungibleAdapter,
    DepositHandler,
    WithdrawHandler,
    NativeChecker,
    BridgeTransactor,
    BridgeFeeInfo,
    AccountId,
    Treasury,
>
{
fn can_check_in(_origin: &MultiLocation, what: &MultiAsset) -> XcmResult {
    if NativeChecker::is_native_asset(what) {
        NativeAdapter::can_check_in(_origin, what)
    } else {
        FungibleAdapter::can_check_in(_origin, what)
    }
}

fn check_in(_origin: &MultiLocation, what: &MultiAsset) {
    if NativeChecker::is_native_asset(what) {
        NativeAdapter::check_in(_origin, what)
    } else {
        FungibleAdapter::check_in(_origin, what)
    }
}

fn check_out(_dest: &MultiLocation, what: &MultiAsset) {
    if NativeChecker::is_native_asset(what) {
        NativeAdapter::check_out(_dest, what)
    } else {
        FungibleAdapter::check_out(_dest, what)
    }
}

fn deposit_asset(what: &MultiAsset, who: &MultiLocation) -> XcmResult {
    log::trace!(
        target: LOG_TARGET,
        "XcmAdapter deposit_asset, what: {:?}, who: {:?}.",
        &what,
        &who,
    );

    match (who.parents, &who.interior) {
        // Destnation is a foreign chain. Forward it through the bridge
        (
            0,
            Junctions::X3(GeneralKey(cb_key), GeneralIndex(dest_id), GeneralKey(recipient)),
        ) => {
            ensure!(
                &cb_key == &CB_ASSET_KEY,
                XcmError::FailedToTransactAsset("DismatchPath")
            );
            let (&amount, location) = match (&what.fun, &what.id) {
                (Fungible(amount), Concrete(id)) => (amount, id),
                _ => return Err(XcmError::Unimplemented),
            };
            let dest_id: BridgeChainId = dest_id
                .clone()
                .try_into()
                .map_err(|_| XcmError::FailedToTransactAsset("ChainIdConversionFailed"))?;
            log::trace!(
                target: LOG_TARGET,
                "Forward transaction to chain: {:?}, with asset: {:?}",
                dest_id,
                &what,
            );
            // Deduct some fees if assets would be forwarded to solo chains.
            let fee = BridgeFeeInfo::get_fee(dest_id, what)
                .ok_or(XcmError::FailedToTransactAsset("FailedGetFee"))?;
            log::trace!(
                target: LOG_TARGET,
                "Deduct some fees before transfer to solochain, fee: {:?}, amount: {:?}",
                fee,
                amount
            );
            ensure!(
                amount > fee,
                XcmError::FailedToTransactAsset("Insufficient")
            );
            let fee_asset: MultiAsset = (location.clone(), fee).into();
            // Transfer fee to treasury
            let treasury = MultiLocation::new(
                0,
                X1(AccountId32 {
                    network: NetworkId::Any,
                    id: Treasury::get().into(),
                }),
            );
            log::trace!(
                target: LOG_TARGET,
                "Send fee to treasury, fee: {:?}, treasury: {:?}",
                &fee_asset,
                &treasury
            );
            if NativeChecker::is_native_asset(&fee_asset) {
                NativeAdapter::deposit_asset(&fee_asset, &treasury)
                    .map_err(|_| XcmError::FailedToTransactAsset("FeeTransferFailed"))?;
            } else {
                FungibleAdapter::deposit_asset(&fee_asset, &treasury)
                    .map_err(|_| XcmError::FailedToTransactAsset("FeeTransferFailed"))?;
            }

            // transfering_amount > 0
            let transfering_amount = amount - fee;
            let transfering_asset: MultiAsset =
                (location.clone(), transfering_amount).into();
            let dest_reserve: MultiLocation = (
                0,
                X2(
                    GeneralKey(CB_ASSET_KEY.to_vec()),
                    GeneralIndex(dest_id as u128),
                ),
            )
                .into();
            let asset_reserve_location = location
                .reserve_location()
                .ok_or(XcmError::FailedToTransactAsset("FailedGetreserve"))?;
            // If we are forwarding asset to its non-reserve destination, deposit assets
            // to reserve account first
            if asset_reserve_location != dest_reserve {
                log::trace!(
                    target: LOG_TARGET,
                    "XcmAdapter, reserve of asset and dest dismatch, deposit asset to reserve account.",
                );
                let reserve_account: MultiLocation = (
                    0,
                    X1(AccountId32 {
                        network: NetworkId::Any,
                        id: dest_reserve.into_account().into(),
                    }),
                )
                    .into();

                if NativeChecker::is_native_asset(&transfering_asset) {
                    NativeAdapter::deposit_asset(&transfering_asset, &reserve_account)
                        .map_err(|_| {
                            XcmError::FailedToTransactAsset("ReserveTransferFailed")
                        })?;
                } else {
                    FungibleAdapter::deposit_asset(&transfering_asset, &reserve_account)
                        .map_err(|_| {
                            XcmError::FailedToTransactAsset("ReserveTransferFailed")
                        })?;
                }
            }

            // Currently, we use chainbridge forwards transfer to solo chain, which use resource id
            // to indicate an asset.
            //
            // When asset is native token, e.g. PHA, we need transfer the location to MultiLocation::here()
            // from (1, X1(Parachain(id))) to match resource id registered in solo chains.
            let rid = if NativeChecker::is_native_asset(&transfering_asset) {
                XTransferAsset(MultiLocation::here()).into_rid(dest_id)
            } else {
                let xtransfer_asset: XTransferAsset = location.clone().into();
                xtransfer_asset.into_rid(dest_id)
            };

            // This operation will not do real transfer, it just emits FungibleTransfer event
            // to notify relayers submit proposal to our bridge contract that deployed on EVM chains.
            BridgeTransactor::transfer_fungible(
                dest_id,
                rid,
                recipient.to_vec(),
                U256::from(transfering_amount),
            )
            .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
        }
        // Try handle it with transfer adapter
        _ => {
            if NativeChecker::is_native_asset(what) {
                NativeAdapter::deposit_asset(what, who)?;
            } else {
                FungibleAdapter::deposit_asset(what, who)?;
            }
        }
    }

    DepositHandler::on_deposited(what.clone(), who.clone())
        .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
    Ok(())
}

fn withdraw_asset(
    what: &MultiAsset,
    who: &MultiLocation,
) -> result::Result<Assets, XcmError> {
    log::trace!(
        target: LOG_TARGET,
        "XcmAdapter withdraw_asset, what: {:?}, who: {:?}.",
        &what,
        &who,
    );
    let assets = if NativeChecker::is_native_asset(what) {
        NativeAdapter::withdraw_asset(what, who)?
    } else {
        FungibleAdapter::withdraw_asset(what, who)?
    };
    WithdrawHandler::on_withdrawn(what.clone(), who.clone())
        .map_err(|e| return XcmError::FailedToTransactAsset(e.into()))?;
    Ok(assets)
}
}