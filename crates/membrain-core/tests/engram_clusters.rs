use membrain_core::graph::EngramStore;
use membrain_core::types::MemoryId;

#[test]
fn create_vs_join_threshold_behavior_is_explicit() {
    let mut store = EngramStore::new(0.95).with_lookup_cap(3);

    let first = store.assign_memory(MemoryId(1), vec![1.0, 0.0], 100, "embed.v1");
    let join = store.assign_memory(MemoryId(2), vec![0.999, 0.001], 101, "embed.v1");
    let create = store.assign_memory(MemoryId(3), vec![0.0, 1.0], 102, "embed.v1");

    assert!(first.created_new_cluster);
    assert!(!join.created_new_cluster);
    assert_eq!(join.engram_id, first.engram_id);
    assert!(create.created_new_cluster);
    assert_ne!(create.engram_id, first.engram_id);
}

#[test]
fn rebuild_recovers_authoritative_centroid_and_membership() {
    let mut store = EngramStore::new(0.90).with_lookup_cap(3);
    let cluster = store.assign_memory(MemoryId(10), vec![1.0, 0.0], 200, "embed.v1");
    store.assign_memory(MemoryId(11), vec![0.95, 0.05], 201, "embed.v1");

    store.refresh_cluster(cluster.engram_id, 202);
    let rebuilt = store.rebuild_from_memberships();
    let rebuilt_cluster = rebuilt.cluster(cluster.engram_id);

    assert!(rebuilt_cluster.is_some());
    assert_eq!(rebuilt_cluster.map(|cluster| cluster.member_count), Some(2));
    assert_eq!(
        rebuilt_cluster.map(|cluster| cluster.centroid.clone()),
        Some(vec![0.975, 0.025])
    );
    assert_eq!(rebuilt.lookup_for_memory(MemoryId(10)), Some(cluster.engram_id));
    assert_eq!(rebuilt.lookup_for_memory(MemoryId(11)), Some(cluster.engram_id));
}

#[test]
fn bounded_lookup_returns_only_top_three_similar_engrams() {
    let mut store = EngramStore::new(0.999).with_lookup_cap(3);
    store.assign_memory(MemoryId(1), vec![1.0, 0.0], 1, "embed.v1");
    store.assign_memory(MemoryId(2), vec![0.0, 1.0], 2, "embed.v1");
    store.assign_memory(MemoryId(3), vec![-1.0, 0.0], 3, "embed.v1");
    store.assign_memory(MemoryId(4), vec![0.0, -1.0], 4, "embed.v1");

    let candidates = store.similar_engrams(&[0.9, 0.1]);

    assert_eq!(candidates.len(), 3);
    assert!(candidates[0].similarity >= candidates[1].similarity);
    assert!(candidates[1].similarity >= candidates[2].similarity);
}

#[test]
fn rebuild_preserves_lookup_cap_and_next_cluster_identity() {
    let mut store = EngramStore::new(0.999).with_lookup_cap(2);
    let first = store.assign_memory(MemoryId(1), vec![1.0, 0.0], 10, "embed.v1");
    let second = store.assign_memory(MemoryId(2), vec![0.0, 1.0], 11, "embed.v1");
    store.assign_memory(MemoryId(3), vec![-1.0, 0.0], 12, "embed.v1");

    let rebuilt = store.rebuild_from_memberships();
    let candidates = rebuilt.similar_engrams(&[0.8, 0.2]);
    let created_after_rebuild = rebuilt
        .clone()
        .assign_memory(MemoryId(4), vec![0.0, -1.0], 13, "embed.v1");

    assert_eq!(candidates.len(), 2);
    assert_eq!(created_after_rebuild.engram_id.0, second.engram_id.0 + 2);
    assert_ne!(created_after_rebuild.engram_id, first.engram_id);
    assert_ne!(created_after_rebuild.engram_id, second.engram_id);
}
