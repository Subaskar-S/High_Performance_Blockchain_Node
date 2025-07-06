use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use tokio::sync::mpsc;

use crate::types::{ConsensusMessage, NodeId, Hash};
use super::ConsensusConfig;

/// View change state
#[derive(Debug, Clone, PartialEq)]
pub enum ViewChangeState {
    Normal,
    ViewChanging,
    NewViewReceived,
}

/// View change timeout configuration
#[derive(Debug, Clone)]
pub struct ViewChangeTimeout {
    pub base_timeout_ms: u64,
    pub timeout_multiplier: f64,
    pub max_timeout_ms: u64,
}

impl Default for ViewChangeTimeout {
    fn default() -> Self {
        Self {
            base_timeout_ms: 5000,  // 5 seconds base timeout
            timeout_multiplier: 1.5, // Exponential backoff
            max_timeout_ms: 60000,   // 1 minute max timeout
        }
    }
}

/// View change manager for handling view changes in BFT consensus
#[derive(Clone)]
pub struct ViewChangeManager {
    config: ConsensusConfig,
    timeout_config: ViewChangeTimeout,
    
    // View change state
    current_view: Arc<RwLock<u64>>,
    state: Arc<RwLock<ViewChangeState>>,
    
    // View change messages
    view_change_messages: Arc<RwLock<HashMap<u64, HashMap<NodeId, ConsensusMessage>>>>,
    new_view_messages: Arc<RwLock<HashMap<u64, ConsensusMessage>>>,
    
    // Timeout tracking
    view_start_time: Arc<RwLock<Instant>>,
    timeout_duration: Arc<RwLock<Duration>>,
    
    // Message sender for broadcasting
    message_sender: Arc<RwLock<Option<mpsc::UnboundedSender<ConsensusMessage>>>>,
}

impl ViewChangeManager {
    /// Create a new view change manager
    pub fn new(config: ConsensusConfig) -> Self {
        Self {
            config,
            timeout_config: ViewChangeTimeout::default(),
            current_view: Arc::new(RwLock::new(0)),
            state: Arc::new(RwLock::new(ViewChangeState::Normal)),
            view_change_messages: Arc::new(RwLock::new(HashMap::new())),
            new_view_messages: Arc::new(RwLock::new(HashMap::new())),
            view_start_time: Arc::new(RwLock::new(Instant::now())),
            timeout_duration: Arc::new(RwLock::new(Duration::from_millis(
                ViewChangeTimeout::default().base_timeout_ms
            ))),
            message_sender: Arc::new(RwLock::new(None)),
        }
    }

    /// Set message sender for broadcasting view change messages
    pub fn set_message_sender(&self, sender: mpsc::UnboundedSender<ConsensusMessage>) {
        let mut message_sender = self.message_sender.write().unwrap();
        *message_sender = Some(sender);
    }

    /// Start a new view
    pub fn start_view(&self, view: u64) -> Result<()> {
        {
            let mut current_view = self.current_view.write().unwrap();
            *current_view = view;
        }

        {
            let mut state = self.state.write().unwrap();
            *state = ViewChangeState::Normal;
        }

        {
            let mut view_start_time = self.view_start_time.write().unwrap();
            *view_start_time = Instant::now();
        }

        // Reset timeout to base value
        {
            let mut timeout_duration = self.timeout_duration.write().unwrap();
            *timeout_duration = Duration::from_millis(self.timeout_config.base_timeout_ms);
        }

        tracing::info!("Started new view: {}", view);
        Ok(())
    }

    /// Check if view change timeout has occurred
    pub fn is_timeout(&self) -> bool {
        let view_start_time = *self.view_start_time.read().unwrap();
        let timeout_duration = *self.timeout_duration.read().unwrap();
        let state = self.state.read().unwrap();
        
        *state == ViewChangeState::Normal && view_start_time.elapsed() > timeout_duration
    }

    /// Trigger a view change
    pub async fn trigger_view_change(&self, new_view: u64) -> Result<()> {
        let current_view = *self.current_view.read().unwrap();
        
        if new_view <= current_view {
            return Err(anyhow!("New view must be greater than current view"));
        }

        // Update state
        {
            let mut state = self.state.write().unwrap();
            *state = ViewChangeState::ViewChanging;
        }

        // Create and broadcast VIEW-CHANGE message
        let view_change_message = ConsensusMessage::ViewChange {
            new_view,
            validator_id: self.config.node_id.clone(),
            signature: [0; 64], // Simplified signature
        };

        self.broadcast_message(view_change_message.clone()).await?;

        // Store our own view change message
        {
            let mut view_change_messages = self.view_change_messages.write().unwrap();
            view_change_messages
                .entry(new_view)
                .or_insert_with(HashMap::new)
                .insert(self.config.node_id.clone(), view_change_message);
        }

        tracing::info!("Triggered view change to view: {}", new_view);
        Ok(())
    }

    /// Handle incoming VIEW-CHANGE message
    pub async fn handle_view_change(
        &self,
        new_view: u64,
        validator_id: NodeId,
        signature: crate::types::Signature,
    ) -> Result<()> {
        // Verify validator is in validator set
        if !self.config.validator_set.contains(&validator_id) {
            return Ok(()); // Ignore invalid validators
        }

        let current_view = *self.current_view.read().unwrap();
        
        // Only accept view changes for higher views
        if new_view <= current_view {
            return Ok(());
        }

        // Store the view change message
        let view_change_message = ConsensusMessage::ViewChange {
            new_view,
            validator_id: validator_id.clone(),
            signature,
        };

        {
            let mut view_change_messages = self.view_change_messages.write().unwrap();
            view_change_messages
                .entry(new_view)
                .or_insert_with(HashMap::new)
                .insert(validator_id, view_change_message);
        }

        // Check if we have enough view change messages
        let view_change_count = {
            let view_change_messages = self.view_change_messages.read().unwrap();
            view_change_messages
                .get(&new_view)
                .map(|messages| messages.len())
                .unwrap_or(0)
        };

        let required_count = self.byzantine_threshold();

        if view_change_count >= required_count {
            // We have enough view change messages
            if self.is_new_primary(new_view) {
                // We are the new primary, send NEW-VIEW message
                self.send_new_view(new_view).await?;
            } else {
                // Wait for NEW-VIEW message from new primary
                self.wait_for_new_view(new_view).await?;
            }
        }

        Ok(())
    }

    /// Handle incoming NEW-VIEW message
    pub async fn handle_new_view(
        &self,
        view: u64,
        view_change_messages: Vec<ConsensusMessage>,
    ) -> Result<()> {
        let current_view = *self.current_view.read().unwrap();
        
        // Only accept new view for higher views
        if view <= current_view {
            return Ok(());
        }

        // Verify the NEW-VIEW message contains enough VIEW-CHANGE messages
        if view_change_messages.len() < self.byzantine_threshold() {
            return Err(anyhow!("Insufficient view change messages in NEW-VIEW"));
        }

        // Verify all view change messages are valid
        for msg in &view_change_messages {
            if let ConsensusMessage::ViewChange { new_view, validator_id, .. } = msg {
                if *new_view != view {
                    return Err(anyhow!("Invalid view in view change message"));
                }
                if !self.config.validator_set.contains(validator_id) {
                    return Err(anyhow!("Invalid validator in view change message"));
                }
            } else {
                return Err(anyhow!("Invalid message type in NEW-VIEW"));
            }
        }

        // Store the NEW-VIEW message
        let new_view_message = ConsensusMessage::NewView {
            view,
            view_change_messages,
        };

        {
            let mut new_view_messages = self.new_view_messages.write().unwrap();
            new_view_messages.insert(view, new_view_message);
        }

        // Start the new view
        self.start_view(view)?;

        tracing::info!("Accepted new view: {}", view);
        Ok(())
    }

    /// Send NEW-VIEW message as the new primary
    async fn send_new_view(&self, view: u64) -> Result<()> {
        // Collect view change messages for this view
        let view_change_messages = {
            let view_change_messages = self.view_change_messages.read().unwrap();
            view_change_messages
                .get(&view)
                .map(|messages| messages.values().cloned().collect())
                .unwrap_or_default()
        };

        if view_change_messages.len() < self.byzantine_threshold() {
            return Err(anyhow!("Insufficient view change messages to send NEW-VIEW"));
        }

        let new_view_message = ConsensusMessage::NewView {
            view,
            view_change_messages,
        };

        self.broadcast_message(new_view_message).await?;

        // Start the new view
        self.start_view(view)?;

        tracing::info!("Sent NEW-VIEW message for view: {}", view);
        Ok(())
    }

    /// Wait for NEW-VIEW message from new primary
    async fn wait_for_new_view(&self, view: u64) -> Result<()> {
        // Set state to waiting for new view
        {
            let mut state = self.state.write().unwrap();
            *state = ViewChangeState::NewViewReceived;
        }

        // In a real implementation, this would set up a timeout
        // For now, we just log that we're waiting
        tracing::info!("Waiting for NEW-VIEW message for view: {}", view);
        Ok(())
    }

    /// Check if this node is the new primary for the given view
    fn is_new_primary(&self, view: u64) -> bool {
        if self.config.validator_set.is_empty() {
            return false;
        }
        
        let primary_index = (view as usize) % self.config.validator_set.len();
        &self.config.validator_set[primary_index] == &self.config.node_id
    }

    /// Get Byzantine fault threshold
    fn byzantine_threshold(&self) -> usize {
        (self.config.validator_set.len() * 2 / 3) + 1
    }

    /// Broadcast a consensus message
    async fn broadcast_message(&self, message: ConsensusMessage) -> Result<()> {
        let message_sender = self.message_sender.read().unwrap();
        if let Some(sender) = message_sender.as_ref() {
            sender.send(message)
                .map_err(|e| anyhow!("Failed to broadcast message: {}", e))?;
        }
        Ok(())
    }

    /// Update timeout duration with exponential backoff
    pub fn update_timeout(&self) {
        let mut timeout_duration = self.timeout_duration.write().unwrap();
        let new_duration = Duration::from_millis(
            (timeout_duration.as_millis() as f64 * self.timeout_config.timeout_multiplier) as u64
        );
        
        let max_duration = Duration::from_millis(self.timeout_config.max_timeout_ms);
        *timeout_duration = new_duration.min(max_duration);
    }

    /// Get current view
    pub fn get_current_view(&self) -> u64 {
        *self.current_view.read().unwrap()
    }

    /// Get current state
    pub fn get_state(&self) -> ViewChangeState {
        self.state.read().unwrap().clone()
    }

    /// Clean up old view change messages
    pub fn cleanup_old_messages(&self, keep_last_n_views: u64) {
        let current_view = *self.current_view.read().unwrap();
        
        if current_view <= keep_last_n_views {
            return;
        }

        let cutoff_view = current_view - keep_last_n_views;

        {
            let mut view_change_messages = self.view_change_messages.write().unwrap();
            view_change_messages.retain(|&view, _| view >= cutoff_view);
        }

        {
            let mut new_view_messages = self.new_view_messages.write().unwrap();
            new_view_messages.retain(|&view, _| view >= cutoff_view);
        }
    }
}
