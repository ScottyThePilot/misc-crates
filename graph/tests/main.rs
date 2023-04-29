extern crate graph;

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
