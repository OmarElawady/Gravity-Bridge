package keeper

import (
	"fmt"
	"log"

	"github.com/Gravity-Bridge/Gravity-Bridge/module/x/gravity/types"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (k Keeper) GetEthOriginatedDenom(ctx sdk.Context, tokenContract types.EthAddress) (string, bool) {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get([]byte(types.GetEthERC20ToDenomKey(tokenContract)))
	log.Printf("found token corresponding to contract %s: %s", tokenContract, string(bz))
	if bz != nil {
		return string(bz), true
	}
	return "", false
}

func (k Keeper) Log(ctx sdk.Context, tokenContract types.EthAddress) (string, bool) {
	store := ctx.KVStore(k.storeKey)
	it := store.Iterator([]byte{0}, []byte{255, 255, 255})
	for it.Valid() {
		log.Printf("in eth originated denoms %s: %s", it.Key(), it.Value())
		it.Next()
	}
	return "", false
}

func (k Keeper) GetEthOriginatedERC20(ctx sdk.Context, denom string) (*types.EthAddress, bool) {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get([]byte(types.GetEthDenomToERC20Key(denom)))
	if bz != nil {
		ethAddr, err := types.NewEthAddress(string(bz))
		if err != nil {
			panic(fmt.Errorf("discovered invalid ERC20 address under key %v", string(bz)))
		}

		return ethAddr, true
	}
	return nil, false
}

func (k Keeper) setEthOriginatedDenomToERC20(ctx sdk.Context, denom string, tokenContract types.EthAddress) {
	store := ctx.KVStore(k.storeKey)
	store.Set([]byte(types.GetEthDenomToERC20Key(denom)), []byte(tokenContract.GetAddress()))
	store.Set([]byte(types.GetEthERC20ToDenomKey(tokenContract)), []byte(denom))
}

// IterateERC20ToDenom iterates over erc20 to denom relations
func (k Keeper) IterateEthERC20ToDenom(ctx sdk.Context, cb func([]byte, *types.ERC20ToDenom) bool) {
	prefixStore := prefix.NewStore(ctx.KVStore(k.storeKey), []byte(types.EthERC20ToDenomKey))
	iter := prefixStore.Iterator(nil, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		erc20ToDenom := types.ERC20ToDenom{
			Erc20: string(iter.Key()),
			Denom: string(iter.Value()),
		}
		// cb returns true to stop early
		if cb(iter.Key(), &erc20ToDenom) {
			break
		}
	}
}
