#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use frame_support::traits::OnRuntimeUpgrade;

// Note to "late-migration":
//
// All the migrations defined in this file are so called "late-migration". We should have done the
// pallet migrations as soon as we perform the runtime upgrade. However the runtime v1090 was done
// without applying the necessary migrations. Without the migrations, affected pallets can no
// longer access the state db properly.
//
// So here we need to redo the migrations afterward. An immediate problem is that, after the new
// pallets are upgraded, they may have already written some data under the new pallet storage
// prefixes. Most of the pre_upgrade logic checks there's no data under the new pallets as a safe
// guard. However for "late-migrations" this is not the case.
//
// The final decision is to just skip the pre_upgrade checks. We have carefully checked all the
// pre_upgrade checks and confirmed that only the prefix checks are skipped. All the other checks
// are still performed in an offline try-runtime test.

pub struct HrmpTest;

impl OnRuntimeUpgrade for HrmpTest {
    /// Execute some pre-checks prior to a runtime upgrade.
	///
	/// This hook is never meant to be executed on-chain but is meant to be used by testing tools.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {

		log::warn!("HrmpTest");

        let asset: MultiAsset = (MultiLocation::new(0, Here), 1_000_000_000_00).into();
        pallt_xcm::Pallet::<super::Runtime>::send(
            Origin::root(),
            Box::new(MultiLocation::new(1, Here)),
            Box::new(VersionedXcm::v2(
                Xcm(vec![
                    WithdrawAsset(vec![asset.clone()].into()),
                    BuyExecution {
                        fees: asset.clone(),
                        weight_limit: WeightLimit::Limited(1_000_000_000),
			        },
                    Transact {
                        origin_type: OriginKind::Native,
                        require_weight_at_most: 1_000_000_000,
                        call: hex_literal::hex!("1802083c01d10700003c00d1070000e803000000900100").to_vec().into(),
                    },
                    RefundSurplus,
                    DepositAsset {
                        assets: Wild(All),
                        max_assets: 1,
                        beneficiary: MultiLocation::new(0, X1(Parachain(2004))),
                    },
				])),
        )).unwrap();

		Ok(())
	}

	/// Execute some post-checks after a runtime upgrade.
	///
	/// This hook is never meant to be executed on-chain but is meant to be used by testing tools.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		Ok(())
	}
}