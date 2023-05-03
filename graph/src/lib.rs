extern crate ids;
extern crate uord;
extern crate nohash_hasher;

use ids::IdContext;
use nohash_hasher::{IntMap, IntSet};

pub use ids::Id;
pub use uord::UOrd;

use std::collections::HashMap;
use std::iter::Extend;
use std::fmt;

pub type IdSet<F> = IntSet<Id<F>>;
pub type IdPair<F> = UOrd<Id<F>>;



#[derive(Clone)]
pub struct Graph<Node, Link> {
  id_context: IdContext<Node>,
  nodes: IntMap<Id<Node>, NodeInner<Node>>,
  links: HashMap<UOrd<Id<Node>>, Link>
}

impl<Node, Link> Graph<Node, Link> {
  pub fn new() -> Self {
    Graph {
      id_context: IdContext::new(),
      nodes: IntMap::default(),
      links: HashMap::new()
    }
  }

  /// Adds a new node (with no links) to the graph, placing the given value inside it.
  pub fn add_node(&mut self, value: Node) -> Id<Node> {
    let id = self.id_context.next_id();
    self.nodes.insert(id, NodeInner {
      value, neighbors: IntSet::default()
    });

    id
  }

  /// Adds a new link to the graph, placing the given value inside it.
  /// Returns the value of the previous link if the link already existed.
  /// Panics when attempting to link a node to itself.
  pub fn add_link(&mut self, value: Link, nodes: impl Into<IdPair<Node>>) -> Option<Link> {
    let nodes = nodes.into();
    assert!(nodes.is_distinct(), "graph node may not link to itself");
    if let Some(previous) = self.links.insert(nodes, value) {
      Some(previous)
    } else {
      self.link_node_neighbors(nodes);
      None
    }
  }

  /// Tries to remove a node from the graph, as well as all links to it.
  pub fn remove_node(&mut self, id: Id<Node>) -> Option<(Node, Vec<Link>)> {
    self.nodes.remove(&id).map(|NodeInner { value, neighbors }| {
      let links = neighbors.into_iter()
        .map(|linked_node| {
          let nodes = UOrd::new(linked_node, id);
          // only this half needs to be removed
          self.unlink_node_neighbor(linked_node, id);
          self.links.remove(&nodes).unwrap()
        })
        .collect();
      (value, links)
    })
  }

  /// Tries to remove a link from the graph.
  pub fn remove_link(&mut self, nodes: impl Into<IdPair<Node>>) -> Option<Link> {
    let nodes = nodes.into();
    self.links.remove(&nodes).map(|value| {
      self.unlink_node_neighbors(nodes);
      value
    })
  }

  /// Returns the number of nodes that a given node is linked to.
  /// Returns `None` if no node exists for the given `Id`.
  pub fn node_neighbors_count(&self, id: Id<Node>) -> Option<usize> {
    self.get_node_neighbors(id).map(IdSet::len)
  }

  /// Returns a list of nodes that the given node is linked to.
  /// Returns `None` if no node exists for the given `Id`.
  pub fn get_node_neighbors(&self, id: Id<Node>) -> Option<&IdSet<Node>> {
    self.nodes.get(&id).map(|inner_node| &inner_node.neighbors)
  }

  fn get_node_neighbors_mut(&mut self, id: Id<Node>) -> Option<&mut IdSet<Node>> {
    self.nodes.get_mut(&id).map(|inner_node| &mut inner_node.neighbors)
  }

  /// Links both halves of a node pair in their `neighbors` list.
  fn link_node_neighbors(&mut self, nodes: UOrd<Id<Node>>) {
    let (node1, node2) = nodes.into_tuple();
    self.get_node_neighbors_mut(node1).unwrap().insert(node2);
    self.get_node_neighbors_mut(node2).unwrap().insert(node1);
  }

  /// Unlinks one half of a node pair in the node's `neighbors` list.
  fn unlink_node_neighbor(&mut self, node: Id<Node>, neighbor: Id<Node>) {
    self.get_node_neighbors_mut(node).unwrap().remove(&neighbor);
  }

  /// Unlinks both halves of a node pair in the node's `neighbors` list.
  fn unlink_node_neighbors(&mut self, nodes: UOrd<Id<Node>>) {
    let (node1, node2) = nodes.into_tuple();
    self.get_node_neighbors_mut(node1).unwrap().remove(&node2);
    self.get_node_neighbors_mut(node2).unwrap().remove(&node1);
  }

  /// Removes all nodes that have no links.
  pub fn remove_orphaned_nodes(&mut self) {
    self.nodes.retain(|_, inner_node| !inner_node.neighbors.is_empty())
  }

  /// Returns true if a node exists with the given ID, false otherwise.
  #[inline]
  pub fn contains_node(&self, id: Id<Node>) -> bool {
    self.nodes.contains_key(&id)
  }

  /// Returns true if the given nodes are linked, false otherwise.
  #[inline]
  pub fn contains_link(&self, id: impl Into<IdPair<Node>>) -> bool {
    self.links.contains_key(&id.into())
  }

  /// Gets the number of nodes in this graph.
  #[inline]
  pub fn nodes_count(&self) -> usize {
    self.nodes.len()
  }

  /// Gets the number of links in this graph.
  #[inline]
  pub fn links_count(&self) -> usize {
    self.links.len()
  }

  /// Gets a reference to the value of a node.
  #[inline]
  pub fn get_node_value(&self, id: Id<Node>) -> Option<&Node> {
    self.nodes.get(&id).map(|inner_node| &inner_node.value)
  }

  /// Gets a mutable reference to the value of a node.
  #[inline]
  pub fn get_node_value_mut(&mut self, id: Id<Node>) -> Option<&mut Node> {
    self.nodes.get_mut(&id).map(|inner_node| &mut inner_node.value)
  }

  /// Gets a reference to the value of a link.
  #[inline]
  pub fn get_link_value(&self, id: impl Into<IdPair<Node>>) -> Option<&Link> {
    self.links.get(&id.into())
  }

  /// Gets a mutable reference to the value of a link.
  #[inline]
  pub fn get_link_value_mut(&mut self, id: impl Into<IdPair<Node>>) -> Option<&mut Link> {
    self.links.get_mut(&id.into())
  }

  #[inline]
  pub fn nodes(&self) -> Nodes<Node> {
    Nodes { inner: self.nodes.iter() }
  }

  #[inline]
  pub fn nodes_mut(&mut self) -> NodesMut<Node> {
    NodesMut { inner: self.nodes.iter_mut() }
  }

  #[inline]
  pub fn nodes_values(&self) -> NodesValues<Node> {
    NodesValues { inner: self.nodes.values() }
  }

  #[inline]
  pub fn nodes_values_mut(&mut self) -> NodesValuesMut<Node> {
    NodesValuesMut { inner: self.nodes.values_mut() }
  }

  #[inline]
  pub fn nodes_ids(&self) -> NodesIds<Node> {
    NodesIds { inner: self.nodes.keys() }
  }

  #[inline]
  pub fn links(&self) -> Links<Node, Link> {
    Links { inner: self.links.iter() }
  }

  #[inline]
  pub fn links_mut(&mut self) -> LinksMut<Node, Link> {
    LinksMut { inner: self.links.iter_mut() }
  }

  #[inline]
  pub fn links_values(&self) -> LinksValues<Node, Link> {
    LinksValues { inner: self.links.values() }
  }

  #[inline]
  pub fn links_values_mut(&mut self) -> LinksValuesMut<Node, Link> {
    LinksValuesMut { inner: self.links.values_mut() }
  }

  #[inline]
  pub fn links_ids(&self) -> LinksIds<Node, Link> {
    LinksIds { inner: self.links.keys() }
  }

  #[cfg(feature = "serde")]
  fn from_raw(mut nodes: IntMap<Id<Node>, NodeInner<Node>>, links: HashMap<UOrd<Id<Node>>, Link>) -> Self {
    fn maximize(acc: &mut Option<u64>, value: u64) {
      let acc = acc.get_or_insert(value);
      if value > *acc { *acc = value };
    }

    let mut highest_id = None;
    for (&id, node_inner) in nodes.iter_mut() {
      let neighbors = links.keys().copied()
        .filter_map(|pair| pair.other(&id).copied());
      node_inner.neighbors.clear();
      node_inner.neighbors.extend(neighbors);
      maximize(&mut highest_id, id.into_raw());
    };

    for id in links.keys().copied().flatten() {
      maximize(&mut highest_id, id.into_raw());
    };

    let current_id = nodes.keys().copied()
      .chain(links.keys().copied().flatten())
      .map(Id::into_raw).max().map_or(0, |max| max + 1);
    let id_context = IdContext::with_current_id(current_id);

    Graph {
      id_context,
      nodes,
      links
    }
  }
}

impl<Node, Link> Extend<(UOrd<Id<Node>>, Link)> for Graph<Node, Link> {
  fn extend<T: IntoIterator<Item = (UOrd<Id<Node>>, Link)>>(&mut self, iter: T) {
    for (nodes, value) in iter {
      self.add_link(value, nodes);
    };
  }
}

impl<Node, Link> Default for Graph<Node, Link> {
  #[inline]
  fn default() -> Self {
    Graph::new()
  }
}

impl<Node: fmt::Debug, Link: fmt::Debug> fmt::Debug for Graph<Node, Link> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("Graph")
      .field("nodes", &self.nodes)
      .field("links", &self.links)
      .finish()
  }
}

#[derive(Clone)]
struct NodeInner<Node> {
  value: Node,
  neighbors: IntSet<Id<Node>>
}

impl<Node: fmt::Debug> fmt::Debug for NodeInner<Node> {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    <Node as fmt::Debug>::fmt(&self.value, f)
  }
}

#[cfg(feature = "serde")]
impl<Node> serde::Serialize for NodeInner<Node>
where Node: serde::Serialize {
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.value.serialize(serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, Node> serde::Deserialize<'de> for NodeInner<Node>
where Node: serde::Deserialize<'de> {
  #[inline]
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    Node::deserialize(deserializer).map(|value| NodeInner {
      value, neighbors: IntSet::default()
    })
  }
}

#[cfg(feature = "serde")]
impl<Node, Link> serde::Serialize for Graph<Node, Link>
where Node: serde::Serialize, Link: serde::Serialize {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("Graph", 2)?;
    serde::ser::SerializeStruct::serialize_field(&mut state, "nodes", &self.nodes)?;
    serde::ser::SerializeStruct::serialize_field(&mut state, "links", &self.links)?;
    serde::ser::SerializeStruct::end(state)
  }
}

#[cfg(feature = "serde")]
impl<'de, Node, Link> serde::Deserialize<'de> for Graph<Node, Link>
where Node: serde::Deserialize<'de>, Link: serde::Deserialize<'de> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    enum GraphField {
      Nodes,
      Links
    }

    impl<'de> serde::de::Deserialize<'de> for GraphField {
      #[inline]
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where D: serde::Deserializer<'de> {
        deserializer.deserialize_identifier(GraphFieldVisitor)
      }
    }

    struct GraphFieldVisitor;

    impl<'de> serde::de::Visitor<'de> for GraphFieldVisitor {
      type Value = GraphField;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("field identifier")
      }

      fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
      where E: serde::de::Error {
        match value {
          0u64 => Ok(GraphField::Nodes),
          1u64 => Ok(GraphField::Links),
          _ => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(value),
            &"field index 0 <= i < 2"
          ))
        }
      }

      #[inline]
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where E: serde::de::Error {
        match value {
          "nodes" => Ok(GraphField::Nodes),
          "links" => Ok(GraphField::Links),
          _ => Err(serde::de::Error::unknown_field(&value, FIELDS))
        }
      }

      fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
      where E: serde::de::Error {
        match value {
          b"nodes" => Ok(GraphField::Nodes),
          b"links" => Ok(GraphField::Links),
          _ => {
            let value = String::from_utf8_lossy(value);
            Err(serde::de::Error::unknown_field(&value, FIELDS))
          }
        }
      }
    }

    type GraphNodes<Node> = IntMap<Id<Node>, NodeInner<Node>>;
    type GraphLinks<Node, Link> = HashMap<UOrd<Id<Node>>, Link>;

    struct GraphVisitor<'de, Node, Link> {
      marker: std::marker::PhantomData<Graph<Node, Link>>,
      lifetime: std::marker::PhantomData<&'de ()>
    }

    impl<'de, Node, Link> serde::de::Visitor<'de> for GraphVisitor<'de, Node, Link>
    where Node: serde::Deserialize<'de>, Link: serde::Deserialize<'de> {
      type Value = Graph<Node, Link>;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("struct Graph")
      }

      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where A: serde::de::SeqAccess<'de> {
        let nodes = seq.next_element::<GraphNodes<Node>>()?
          .ok_or(serde::de::Error::invalid_length(0usize, &"struct Graph with 2 elements"))?;
        let links = seq.next_element::<GraphLinks<Node, Link>>()?
          .ok_or(serde::de::Error::invalid_length(1usize, &"struct Graph with 2 elements"))?;
        Ok(Graph::from_raw(nodes, links))
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where A: serde::de::MapAccess<'de> {
        let mut nodes = None;
        let mut links = None;
        while let Some(key) = map.next_key::<GraphField>()? {
          match key {
            GraphField::Nodes => {
              if nodes.is_some() { return Err(serde::de::Error::duplicate_field("nodes")) };
              nodes = Some(map.next_value::<GraphNodes<Node>>()?);
            },
            GraphField::Links => {
              if links.is_some() { return Err(serde::de::Error::duplicate_field("links")) };
              links = Some(map.next_value::<GraphLinks<Node, Link>>()?);
            }
          }
        };

        let nodes = nodes.ok_or(serde::de::Error::missing_field("nodes"))?;
        let links = links.ok_or(serde::de::Error::missing_field("links"))?;
        Ok(Graph::from_raw(nodes, links))
      }
    }

    const FIELDS: &[&str] = &["nodes", "links"];
    deserializer.deserialize_struct("Graph", FIELDS, GraphVisitor {
      marker: std::marker::PhantomData,
      lifetime: std::marker::PhantomData
    })
  }
}

macro_rules! impl_iterator {
  {
    $(#[$attr:meta])*
    $vis:vis struct $Type:ident <$($lt:lifetime),* $(,)? $($gn:ident),* $(,)?>,
    $inner:ident: $InnerType:ty, $Item:ty, $map:expr
    $(, where $($w:tt)*)? $(,)?
  } => {
    $(#[$attr])*
    $vis struct $Type<$($lt,)* $($gn,)*> {
      $inner: $InnerType
    }

    impl<$($lt,)* $($gn,)*> std::iter::Iterator for $Type<$($lt,)* $($gn,)*> $(where $($w)*)? {
      type Item = $Item;

      #[inline]
      fn next(&mut self) -> Option<Self::Item> {
        self.$inner.next().map($map)
      }

      #[inline]
      fn size_hint(&self) -> (usize, Option<usize>) {
        self.$inner.size_hint()
      }
    }

    impl<$($lt,)* $($gn,)*> std::iter::ExactSizeIterator for $Type<$($lt,)* $($gn,)*> $(where $($w)*)? {
      #[inline]
      fn len(&self) -> usize {
        self.$inner.len()
      }
    }

    impl<$($lt,)* $($gn,)*> std::iter::FusedIterator for $Type<$($lt,)* $($gn,)*> $(where $($w)*)? {}
  };
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct Nodes<'a, Node>,
  inner: std::collections::hash_map::Iter<'a, Id<Node>, NodeInner<Node>>,
  (Id<Node>, &'a Node), |(&id, NodeInner { value, .. })| (id, value)
}

impl_iterator! {
  #[derive(Debug)] pub struct NodesMut<'a, Node>,
  inner: std::collections::hash_map::IterMut<'a, Id<Node>, NodeInner<Node>>,
  (Id<Node>, &'a mut Node), |(&id, NodeInner { value, .. })| (id, value)
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct NodesValues<'a, Node>,
  inner: std::collections::hash_map::Values<'a, Id<Node>, NodeInner<Node>>,
  &'a Node, |NodeInner { value, .. }| value
}

impl_iterator! {
  #[derive(Debug)] pub struct NodesValuesMut<'a, Node>,
  inner: std::collections::hash_map::ValuesMut<'a, Id<Node>, NodeInner<Node>>,
  &'a mut Node, |NodeInner { value, .. }| value
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct NodesIds<'a, Node>,
  inner: std::collections::hash_map::Keys<'a, Id<Node>, NodeInner<Node>>,
  Id<Node>, |&id| id
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct Links<'a, Node, Link>,
  inner: std::collections::hash_map::Iter<'a, UOrd<Id<Node>>, Link>,
  (UOrd<Id<Node>>, &'a Link), |(&id, value)| (id, value)
}

impl_iterator! {
  #[derive(Debug)] pub struct LinksMut<'a, Node, Link>,
  inner: std::collections::hash_map::IterMut<'a, UOrd<Id<Node>>, Link>,
  (UOrd<Id<Node>>, &'a mut Link), |(&id, value)| (id, value)
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct LinksValues<'a, Node, Link>,
  inner: std::collections::hash_map::Values<'a, UOrd<Id<Node>>, Link>,
  &'a Link, std::convert::identity
}

impl_iterator! {
  #[derive(Debug)] pub struct LinksValuesMut<'a, Node, Link>,
  inner: std::collections::hash_map::ValuesMut<'a, UOrd<Id<Node>>, Link>,
  &'a mut Link, std::convert::identity
}

impl_iterator! {
  #[derive(Debug, Clone)] pub struct LinksIds<'a, Node, Link>,
  inner: std::collections::hash_map::Keys<'a, UOrd<Id<Node>>, Link>,
  UOrd<Id<Node>>, |&id| id
}
