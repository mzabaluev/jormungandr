mod context;
mod controller;
mod fragment_node;
mod progress_bar_mode;
pub mod repository;
pub mod settings;

pub use self::{
    context::{Context, ContextChaCha},
    controller::{Controller, ControllerBuilder},
    progress_bar_mode::{parse_progress_bar_mode_from_str, ProgressBarMode},
};
pub use chain_impl_mockchain::{
    block::Block, chaintypes::ConsensusVersion, header::HeaderId, milli::Milli, value::Value,
};
pub use jormungandr_lib::interfaces::{
    ActiveSlotCoefficient, KESUpdateSpeed, NumberOfSlotsPerEpoch, SlotDuration,
};

pub use jormungandr_testing_utils::testing::network_builder::{
    Blockchain, Node, NodeAlias, Seed, SpawnParams, Topology, TopologyBuilder, Wallet, WalletAlias,
    WalletType,
};

error_chain! {
    links {
        Node(crate::node::Error, crate::node::ErrorKind);
        LegacyNode(crate::legacy::Error, crate::legacy::ErrorKind);
    }

    foreign_links {
        Wallet(jormungandr_testing_utils::wallet::WalletError);
        FsFixture(assert_fs::fixture::FixtureError);
        Io(std::io::Error);
        Reqwest(reqwest::Error);
        BlockFormatError(chain_core::mempack::ReadError);
    }

    errors {
        NodeNotFound(node: String) {
            description("Node not found"),
            display("No node with alias {}", node),
        }
        WalletNotFound(wallet: String) {
            description("Wallet was not found"),
            display("Wallet '{}' was not found. Used before or never initialize", wallet)
        }
    }
}

#[macro_export]
macro_rules! prepare_scenario {
    (
        $title:expr,
        $context:expr,
        topology [
            $($topology_tt:tt $(-> $node_link:tt)*),+ $(,)*
        ]
        blockchain {
            consensus = $blockchain_consensus:tt,
            number_of_slots_per_epoch = $slots_per_epoch:tt,
            slot_duration = $slot_duration:tt,
            leaders = [ $($node_leader:tt),* $(,)* ],
            initials = [
                $(account $initial_wallet_name:tt with $initial_wallet_funds:tt $(delegates to $initial_wallet_delegate_to:tt)* ),+ $(,)*
            ] $(,)*
        }
    ) => {{
        let mut builder = $crate::scenario::ControllerBuilder::new($title);
        let mut topology_builder = jormungandr_testing_utils::testing::network_builder::TopologyBuilder::new();
        $(
            #[allow(unused_mut)]
            let mut node = $crate::scenario::Node::new($topology_tt);
            $(
                node.add_trusted_peer($node_link);
            )*
            topology_builder.register_node(node);
        )*
        let topology : jormungandr_testing_utils::testing::network_builder::Topology = topology_builder.build();
        builder.set_topology(topology);

        let mut blockchain = $crate::scenario::Blockchain::new(
            $crate::scenario::ConsensusVersion::$blockchain_consensus,
            $crate::scenario::NumberOfSlotsPerEpoch::new($slots_per_epoch).expect("valid number of slots per epoch"),
            $crate::scenario::SlotDuration::new($slot_duration).expect("valid slot duration in seconds"),
            $crate::scenario::KESUpdateSpeed::new(46800).expect("valid kes update speed in seconds"),
            $crate::scenario::ActiveSlotCoefficient::new($crate::scenario::Milli::from_millis(700)).expect("active slot coefficient in millis"),
        );

        $(
            let node_leader = $node_leader.to_owned();
            blockchain.add_leader(node_leader);
        )*

        $(
            #[allow(unused_mut)]
            let mut wallet = jormungandr_testing_utils::testing::network_builder::WalletTemplate::new_account(
                $initial_wallet_name.to_owned(),
                chain_impl_mockchain::value::Value($initial_wallet_funds).into()
            );

            $(
                assert!(
                    wallet.delegate().is_none(),
                    "we only support delegating once for now, fix delegation for wallet \"{}\"",
                    $initial_wallet_name
                );
                *wallet.delegate_mut() = Some($initial_wallet_delegate_to.to_owned());
            )*


            blockchain.add_wallet(wallet);
        )*
        builder.set_blockchain(blockchain);

        builder.build_settings($context);

        builder
    }};
}
