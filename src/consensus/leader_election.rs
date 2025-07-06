use std::collections::HashMap;
use crate::types::NodeId;

/// Leader election mechanism for BFT consensus
#[derive(Clone)]
pub struct LeaderElection {
    validator_set: Vec<NodeId>,
    leader_history: HashMap<u64, NodeId>,
}

impl LeaderElection {
    /// Create a new leader election instance
    pub fn new(validator_set: Vec<NodeId>) -> Self {
        Self {
            validator_set,
            leader_history: HashMap::new(),
        }
    }

    /// Get the leader for a specific view using round-robin selection
    pub fn get_leader(&self, view: u64) -> NodeId {
        if self.validator_set.is_empty() {
            return "unknown".to_string();
        }

        let leader_index = (view as usize) % self.validator_set.len();
        self.validator_set[leader_index].clone()
    }

    /// Check if a node is the leader for a specific view
    pub fn is_leader(&self, node_id: &NodeId, view: u64) -> bool {
        self.get_leader(view) == *node_id
    }

    /// Get the next leader after the current view
    pub fn get_next_leader(&self, current_view: u64) -> NodeId {
        self.get_leader(current_view + 1)
    }

    /// Get leader rotation schedule for next N views
    pub fn get_leader_schedule(&self, start_view: u64, count: usize) -> Vec<(u64, NodeId)> {
        let mut schedule = Vec::with_capacity(count);
        
        for i in 0..count {
            let view = start_view + i as u64;
            let leader = self.get_leader(view);
            schedule.push((view, leader));
        }
        
        schedule
    }

    /// Update validator set (for dynamic validator changes)
    pub fn update_validator_set(&mut self, new_validator_set: Vec<NodeId>) {
        self.validator_set = new_validator_set;
        // Clear history as validator set changed
        self.leader_history.clear();
    }

    /// Get current validator set
    pub fn get_validator_set(&self) -> &[NodeId] {
        &self.validator_set
    }

    /// Check if a node is a validator
    pub fn is_validator(&self, node_id: &NodeId) -> bool {
        self.validator_set.contains(node_id)
    }

    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.validator_set.len()
    }

    /// Calculate Byzantine fault tolerance threshold
    pub fn byzantine_threshold(&self) -> usize {
        if self.validator_set.is_empty() {
            return 0;
        }
        (self.validator_set.len() * 2 / 3) + 1
    }

    /// Get maximum number of Byzantine faults tolerated
    pub fn max_byzantine_faults(&self) -> usize {
        if self.validator_set.len() < 4 {
            return 0;
        }
        (self.validator_set.len() - 1) / 3
    }

    /// Record leader for a view (for history tracking)
    pub fn record_leader(&mut self, view: u64, leader: NodeId) {
        self.leader_history.insert(view, leader);
    }

    /// Get leader history
    pub fn get_leader_history(&self) -> &HashMap<u64, NodeId> {
        &self.leader_history
    }

    /// Check if the validator set satisfies BFT requirements
    pub fn is_bft_capable(&self) -> bool {
        self.validator_set.len() >= 4 // Need at least 4 nodes for BFT (3f+1 where f=1)
    }

    /// Get validator index
    pub fn get_validator_index(&self, node_id: &NodeId) -> Option<usize> {
        self.validator_set.iter().position(|v| v == node_id)
    }

    /// Get validator by index
    pub fn get_validator_by_index(&self, index: usize) -> Option<&NodeId> {
        self.validator_set.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validators() -> Vec<NodeId> {
        vec![
            "validator-1".to_string(),
            "validator-2".to_string(),
            "validator-3".to_string(),
            "validator-4".to_string(),
        ]
    }

    #[test]
    fn test_leader_election_round_robin() {
        let validators = create_test_validators();
        let leader_election = LeaderElection::new(validators.clone());

        // Test round-robin selection
        assert_eq!(leader_election.get_leader(0), validators[0]);
        assert_eq!(leader_election.get_leader(1), validators[1]);
        assert_eq!(leader_election.get_leader(2), validators[2]);
        assert_eq!(leader_election.get_leader(3), validators[3]);
        assert_eq!(leader_election.get_leader(4), validators[0]); // Wraps around
    }

    #[test]
    fn test_is_leader() {
        let validators = create_test_validators();
        let leader_election = LeaderElection::new(validators.clone());

        assert!(leader_election.is_leader(&validators[0], 0));
        assert!(leader_election.is_leader(&validators[1], 1));
        assert!(!leader_election.is_leader(&validators[0], 1));
    }

    #[test]
    fn test_byzantine_threshold() {
        let validators = create_test_validators();
        let leader_election = LeaderElection::new(validators);

        // For 4 validators: (4 * 2 / 3) + 1 = 2 + 1 = 3
        assert_eq!(leader_election.byzantine_threshold(), 3);
        assert_eq!(leader_election.max_byzantine_faults(), 1);
    }

    #[test]
    fn test_bft_capability() {
        let leader_election_small = LeaderElection::new(vec!["v1".to_string(), "v2".to_string()]);
        assert!(!leader_election_small.is_bft_capable());

        let leader_election_large = LeaderElection::new(create_test_validators());
        assert!(leader_election_large.is_bft_capable());
    }

    #[test]
    fn test_leader_schedule() {
        let validators = create_test_validators();
        let leader_election = LeaderElection::new(validators.clone());

        let schedule = leader_election.get_leader_schedule(0, 6);
        assert_eq!(schedule.len(), 6);
        
        // Check the schedule follows round-robin
        assert_eq!(schedule[0], (0, validators[0].clone()));
        assert_eq!(schedule[1], (1, validators[1].clone()));
        assert_eq!(schedule[4], (4, validators[0].clone())); // Wraps around
    }
}
