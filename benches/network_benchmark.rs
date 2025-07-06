use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use uuid::Uuid;

/// Benchmark network layer performance
fn network_throughput_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_throughput");
    
    // Set measurement time for network benchmarks
    group.measurement_time(Duration::from_secs(15));
    
    // Benchmark message throughput with different message sizes
    for message_size in [64, 256, 1024, 4096, 16384].iter() {
        group.bench_with_input(
            BenchmarkId::new("message_throughput", message_size),
            message_size,
            |b, &message_size| {
                let messages = create_test_messages(100, message_size);
                b.iter(|| {
                    process_message_batch(black_box(&messages))
                });
            },
        );
    }
    
    // Benchmark peer connection handling
    for peer_count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("peer_connections", peer_count),
            peer_count,
            |b, &peer_count| {
                b.iter(|| {
                    simulate_peer_connections(black_box(*peer_count))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark gossip protocol performance
fn gossip_protocol_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("gossip_protocol");
    
    // Benchmark gossip fanout performance
    for fanout in [3, 6, 9, 12, 15].iter() {
        group.bench_with_input(
            BenchmarkId::new("gossip_fanout", fanout),
            fanout,
            |b, &fanout| {
                let message = create_test_message(1024);
                b.iter(|| {
                    simulate_gossip_fanout(black_box(&message), black_box(fanout))
                });
            },
        );
    }
    
    // Benchmark message deduplication
    group.bench_function("message_deduplication", |b| {
        let messages = create_duplicate_messages(1000);
        b.iter(|| {
            deduplicate_messages(black_box(&messages))
        });
    });
    
    // Benchmark gossip message validation
    group.bench_function("message_validation", |b| {
        let message = create_test_message(1024);
        b.iter(|| {
            validate_gossip_message(black_box(&message))
        });
    });
    
    group.finish();
}

/// Benchmark peer discovery performance
fn peer_discovery_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("peer_discovery");
    
    // Benchmark mDNS discovery
    group.bench_function("mdns_discovery", |b| {
        b.iter(|| {
            simulate_mdns_discovery(black_box(10))
        });
    });
    
    // Benchmark Kademlia DHT operations
    for node_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("kademlia_lookup", node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    simulate_kademlia_lookup(black_box(node_count))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark network latency simulation
fn network_latency_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_latency");
    
    // Benchmark different network conditions
    let latencies = [1, 10, 50, 100, 500]; // milliseconds
    
    for latency in latencies.iter() {
        group.bench_with_input(
            BenchmarkId::new("simulated_latency", latency),
            latency,
            |b, &latency| {
                let message = create_test_message(1024);
                b.iter(|| {
                    simulate_network_latency(black_box(&message), black_box(latency))
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for benchmarks

fn create_test_messages(count: usize, size: usize) -> Vec<TestMessage> {
    (0..count)
        .map(|_| create_test_message(size))
        .collect()
}

fn create_test_message(size: usize) -> TestMessage {
    TestMessage {
        id: Uuid::new_v4(),
        payload: vec![0u8; size],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        sender: format!("peer-{}", rand::random::<u32>()),
    }
}

fn create_duplicate_messages(count: usize) -> Vec<TestMessage> {
    let base_message = create_test_message(512);
    let mut messages = Vec::with_capacity(count);
    
    // Create some duplicates
    for i in 0..count {
        if i % 10 == 0 {
            // Every 10th message is a duplicate
            messages.push(base_message.clone());
        } else {
            messages.push(create_test_message(512));
        }
    }
    
    messages
}

fn process_message_batch(messages: &[TestMessage]) -> usize {
    // Simulate processing time based on message count and size
    let total_size: usize = messages.iter().map(|m| m.payload.len()).sum();
    let processing_time = (total_size / 1024) + messages.len(); // microseconds
    std::thread::sleep(Duration::from_micros(processing_time as u64));
    messages.len()
}

fn simulate_peer_connections(peer_count: usize) -> usize {
    // Simulate connection establishment time
    let time_per_peer = 100; // microseconds
    std::thread::sleep(Duration::from_micros((peer_count * time_per_peer) as u64));
    peer_count
}

fn simulate_gossip_fanout(message: &TestMessage, fanout: usize) -> usize {
    // Simulate gossip propagation time
    let base_time = 50; // microseconds
    let fanout_time = fanout * 10;
    let message_time = message.payload.len() / 1024;
    std::thread::sleep(Duration::from_micros((base_time + fanout_time + message_time) as u64));
    fanout
}

fn deduplicate_messages(messages: &[TestMessage]) -> Vec<TestMessage> {
    // Simulate deduplication processing
    let processing_time = messages.len() * 2; // microseconds
    std::thread::sleep(Duration::from_micros(processing_time as u64));
    
    // Simple deduplication simulation
    let mut unique_messages = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    
    for message in messages {
        if seen_ids.insert(message.id) {
            unique_messages.push(message.clone());
        }
    }
    
    unique_messages
}

fn validate_gossip_message(message: &TestMessage) -> bool {
    // Simulate message validation time
    let validation_time = 5 + (message.payload.len() / 1024); // microseconds
    std::thread::sleep(Duration::from_micros(validation_time as u64));
    
    // Simple validation: check if message is not empty and timestamp is reasonable
    !message.payload.is_empty() && message.timestamp > 0
}

fn simulate_mdns_discovery(expected_peers: usize) -> Vec<String> {
    // Simulate mDNS discovery time
    let discovery_time = 1000 + (expected_peers * 10); // microseconds
    std::thread::sleep(Duration::from_micros(discovery_time as u64));
    
    (0..expected_peers)
        .map(|i| format!("peer-{}.local", i))
        .collect()
}

fn simulate_kademlia_lookup(node_count: usize) -> Vec<String> {
    // Simulate Kademlia lookup time (logarithmic complexity)
    let lookup_steps = (node_count as f64).log2() as usize;
    let lookup_time = lookup_steps * 100; // microseconds per step
    std::thread::sleep(Duration::from_micros(lookup_time as u64));
    
    // Return closest nodes (simplified)
    (0..std::cmp::min(20, node_count))
        .map(|i| format!("node-{}", i))
        .collect()
}

fn simulate_network_latency(message: &TestMessage, latency_ms: u64) -> bool {
    // Simulate network latency
    std::thread::sleep(Duration::from_millis(latency_ms));
    
    // Simulate packet loss (1% chance)
    rand::random::<u8>() > 2 // 99% success rate
}

// Test data structures
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct TestMessage {
    id: Uuid,
    payload: Vec<u8>,
    timestamp: u64,
    sender: String,
}

criterion_group!(
    network_benches,
    network_throughput_benchmark,
    gossip_protocol_benchmark,
    peer_discovery_benchmark,
    network_latency_benchmark
);
criterion_main!(network_benches);
