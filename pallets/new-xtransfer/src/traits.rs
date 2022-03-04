use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::dispatch::DispatchResult;
use sp_std::vec;
use xcm::latest::{MultiAsset, MultiLocation};

#[derive(Clone, Decode, Encode, Eq, PartialEq, Ord, PartialOrd, Debug, TypeInfo)]
pub enum XAsset<AssetId, ClassId, InstanceId> {
    Native,
    Fungible(AssetId),
    NonFungible(ClassId, InstanceId),
}

impl<AssetId: From<u32>, ClassId: From<u32>, InstanceId: From<u32>> Into<MultiLocation> for XAsset<AssetId, ClassId, InstanceId> {
    fn into(x: Self) -> MultiLocation {
        match self {
            Native => {
                MultiLocation::new(0, Here),
            },
            Fungible(id) => {
                // id => asset_reg.location
            },
            NonFungible(class_id, instance_id) => {
                // id => unique_reg.location
            },
        }
    }
}

pub trait XTransact {
    fn transfer_fungible(
        sender: MultiLocation,
        asset: MultiAsset,
        dest: MultiLocation,
    ) -> DispatchResult;

    fn transfer_nonfungible(
        sender: MultiLocation,
        asset: MultiAsset,
        dest: MultiLocation,
    ) -> DispatchResult;

    fn transfer_generic(
        sender: MultiLocation,
        data: Vec<u8>,
        dest: MultiLocation,
    ) -> DispatchResult;
}

impl XTransact for () {
    fn transfer_fungible(
        sender: MultiLocation,
        asset: MultiAsset,
        dest: MultiLocation,
    ) -> DispatchResult {
        Ok(())
    }

    fn transfer_nonfungible(
        sender: MultiLocation,
        asset: MultiAsset,
        dest: MultiLocation,
    ) -> DispatchResult {
        Ok(())
    }

    fn transfer_generic(
        sender: MultiLocation,
        data: Vec<u8>,
        dest: MultiLocation,
    ) -> DispatchResult {
        Ok(())
    }
}