extern crate graph;
#[cfg(feature = "serde")]
extern crate ron;
#[cfg(feature = "serde")]
extern crate serde;

use graph::Graph;

#[test]
fn main() {
  let mut g: Graph<String, String> = Graph::new();

  let mut nodes = Vec::new();
  for i in 0..10 {
    let string = format!("node {i}");
    assert_eq!(g.nodes_count(), i);
    nodes.push(g.add_node(string));
    assert_eq!(g.nodes_count(), i + 1);
    test_neighbors(&g);
  };

  for (i, value) in nodes.windows(2).take(5).enumerate() {
    let &[n0, n1] = value else { unreachable!() };
    let string = format!("link {i}");
    assert_eq!(g.links_count(), i);
    assert!(g.add_link(string, (n0, n1)).is_none());
    assert_eq!(g.links_count(), i + 1);
    test_neighbors(&g);
  };

  g.remove_node(nodes[1]).unwrap();
  test_neighbors(&g);

  assert_eq!(g.nodes_count(), 9);
  assert_eq!(g.links_count(), 3);

  assert!(!g.contains_link((nodes[0], nodes[1])));
  assert!(!g.contains_link((nodes[1], nodes[2])));

  #[cfg(feature = "serde")]
  test_serde_roundtrip(&g);
}

/// Ensures all of the internal invariants have been upheld.
fn test_neighbors<Node, Link>(graph: &Graph<Node, Link>) {
  for id in graph.nodes_ids() {
    let neighbors = graph.get_node_neighbors(id).unwrap();
    let actual_neighbors = graph.links_ids()
      .filter_map(|pair| pair.other(&id).copied())
      .collect::<graph::IdSet<Node>>();
    assert_eq!(*neighbors, actual_neighbors);
  };
}

#[cfg(feature = "serde")]
fn test_serde_roundtrip(graph1: &Graph<String, String>) {
  use std::collections::HashMap;
  let string1 = ron::to_string(graph1).expect("failed to serialize");
  let graph2: Graph<String, String> = ron::from_str(&string1).expect("failed to deserialize");

  let nodes1 = graph1.nodes().collect::<HashMap<_, _>>();
  let nodes2 = graph2.nodes().collect::<HashMap<_, _>>();
  assert_eq!(nodes1, nodes2);

  let links1 = graph1.links().collect::<HashMap<_, _>>();
  let links2 = graph2.links().collect::<HashMap<_, _>>();
  assert_eq!(links1, links2);
}
