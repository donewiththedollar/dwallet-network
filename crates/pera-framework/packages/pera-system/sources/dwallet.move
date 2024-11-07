// Copyright (c) dWallet Labs, Ltd.
// SPDX-License-Identifier: BSD-3-Clause-Clear

module pera_system::dwallet {

    #[allow(unused_field)]
    /// `DWallet` represents a wallet that is created after the DKG process.
    public struct DWallet<phantom T> has key, store {
        id: UID,
        session_id: ID,
        dwallet_cap_id: ID,
        // The output of the second DKG round.
        output: vector<u8>,
    }

    /// `DWalletCap` holder controls a corresponding `Dwallet`.
    public struct DWalletCap has key, store {
        id: UID,
    }

    /// A generic function to create a new [`DWallet`] object of type T.
    public(package) fun create_dwallet<T: drop>(
        session_id: ID,
        dwallet_cap_id: ID,
        output: vector<u8>,
        ctx: &mut TxContext
    ): DWallet<T> {
        DWallet<T> {
            id: object::new(ctx),
            session_id,
            dwallet_cap_id,
            output,
        }
    }

    /// Create a new [`DWalletCap`] object.
    /// The holder of this capability owns the `DWallet`.
    public(package) fun create_dwallet_cap(ctx: &mut TxContext): ID {
        let cap = DWalletCap {
            id: object::new(ctx),
        };
    let id = object::id(&cap);
        transfer::transfer(cap, ctx.sender());
        id
    }

    /// `MessageApproval` represents a message that was approved.
    /// Bound to a `DWalletCap`.
    public struct MessageApproval has store, drop {
        dwallet_cap_id: ID,
        message: vector<u8>,
    }

    /// Create a set of message approvals.
    /// The messages must be approved in the same order as they were created.
    /// The messages must be approved by the same `dwallet_cap_id`.
    public fun approve_messages(
        dwallet_cap: &DWalletCap,
        mut messages: vector<vector<u8>>
    ): vector<MessageApproval> {
        let dwallet_cap_id = object::id(dwallet_cap);
        let mut message_approvals = vector::empty<MessageApproval>();
        while (vector::length(&messages) > 0) {
            let message = vector::pop_back(&mut messages);
            vector::push_back(&mut message_approvals, MessageApproval {
                dwallet_cap_id,
                message,
            });
        };
        message_approvals
    }

    public(package) fun get_dwallet_cap_id<T: drop>(dwallet: &DWallet<T>): ID {
        dwallet.dwallet_cap_id
    }

    public(package) fun get_dwallet_output<T: drop>(dwallet: &DWallet<T>): vector<u8> {
        dwallet.output
    }


}
