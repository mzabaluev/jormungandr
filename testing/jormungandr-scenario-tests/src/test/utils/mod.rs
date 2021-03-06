use crate::legacy::LegacyNodeController;
use crate::{
    node::{FragmentNode, NodeController},
    scenario::Controller,
    test::{ErrorKind, Result},
};
pub use jormungandr_testing_utils::testing::{SyncNode, SyncWaitParams};

use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{FragmentStatus, NodeState},
};
use jormungandr_testing_utils::{
    testing::{Speed, Thresholds},
    wallet::Wallet,
};
use std::time::Duration;

pub use jormungandr_testing_utils::testing::{
    assert, assert_equals,
    sync::{
        measure_and_log_sync_time, measure_fragment_propagation_speed,
        measure_how_many_nodes_are_running,
    },
    MeasurementReportInterval,
};

pub fn wait(seconds: u64) {
    std::thread::sleep(Duration::from_secs(seconds));
}

pub fn measure_single_transaction_propagation_speed<A: SyncNode + FragmentNode + Send + Sized>(
    controller: &mut Controller,
    mut wallet1: &mut Wallet,
    wallet2: &Wallet,
    leaders: &[&A],
    sync_wait: Thresholds<Speed>,
    info: &str,
    report_node_stats_interval: MeasurementReportInterval,
) -> Result<()> {
    let node = leaders.iter().next().unwrap();
    let check = controller.fragment_sender().send_transaction(
        &mut wallet1,
        &wallet2,
        *node,
        1_000.into(),
    )?;
    let fragment_id = check.fragment_id();
    Ok(measure_fragment_propagation_speed(
        *fragment_id,
        leaders,
        sync_wait,
        info,
        report_node_stats_interval,
    )?)
}

impl SyncNode for NodeController {
    fn alias(&self) -> &str {
        self.alias()
    }

    fn last_block_height(&self) -> u32 {
        self.stats()
            .unwrap()
            .stats
            .unwrap()
            .last_block_height
            .unwrap()
            .parse()
            .unwrap()
    }

    fn log_stats(&self) {
        println!("Node: {} -> {:?}", self.alias(), self.stats());
    }

    fn tip(&self) -> Hash {
        self.tip().expect("cannot get tip from rest")
    }

    fn is_running(&self) -> bool {
        self.stats().unwrap().state == NodeState::Running
    }

    fn log_content(&self) -> String {
        self.logger().get_log_content()
    }

    fn get_lines_with_error_and_invalid(&self) -> Vec<String> {
        self.logger().get_lines_with_error_and_invalid().collect()
    }
}

impl SyncNode for LegacyNodeController {
    fn alias(&self) -> &str {
        self.alias()
    }

    fn last_block_height(&self) -> u32 {
        self.stats().unwrap()["lastBlockHeight"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    fn log_stats(&self) {
        println!("Node: {} -> {:?}", self.alias(), self.stats());
    }

    fn tip(&self) -> Hash {
        self.tip().expect("cannot get tip from rest")
    }

    fn log_content(&self) -> String {
        self.logger().get_log_content()
    }

    fn get_lines_with_error_and_invalid(&self) -> Vec<String> {
        self.logger().get_lines_with_error_and_invalid().collect()
    }

    fn is_running(&self) -> bool {
        self.stats().unwrap()["state"].as_str().unwrap() == "Running"
    }
}

pub fn assert_is_in_block<A: SyncNode + ?Sized>(status: FragmentStatus, node: &A) -> Result<()> {
    if !status.is_in_a_block() {
        bail!(ErrorKind::AssertionFailed(format!(
            "fragment status sent to node: {} is not in block :({:?}). logs: {}",
            node.alias(),
            status,
            node.log_content()
        )))
    }
    Ok(())
}
