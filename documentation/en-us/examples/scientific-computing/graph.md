# Graph Algorithms

Valkyrie provides powerful support for graph algorithms, combining functional programming with efficient memory management.

## Graph Representation

### Adjacency List

```valkyrie
class Graph⟨V, E⟩ {
    vertices: HashMap⟨VertexId, V⟩
    edges: HashMap⟨VertexId, [(VertexId, E)]⟩
}

imply Graph⟨V, E⟩ {
    micro new() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
        }
    }
    
    micro add_vertex(mut self, id: VertexId, value: V) {
        self.vertices.insert(id, value)
        self.edges.insert(id, [])
    }
    
    micro add_edge(mut self, from: VertexId, to: VertexId, edge: E) {
        self.edges.get_mut(from)?.push((to, edge))
    }
    
    micro neighbors(self, id: VertexId) -> [(VertexId, E)] {
        self.edges.get(id)?.clone()
    }
}
```

## Traversal Algorithms

### Breadth-First Search (BFS)

```valkyrie
micro bfs⟨V, E⟩(graph: Graph⟨V, E⟩, start: VertexId) -> [VertexId] {
    let mut visited = HashSet::new()
    let mut queue = VecDeque::new()
    let mut result = []
    
    queue.push_back(start)
    visited.insert(start)
    
    while let Some(current) = queue.pop_front() {
        result.push(current)
        
        for (neighbor, _) in graph.neighbors(current) {
            if !visited.contains(neighbor) {
                visited.insert(neighbor)
                queue.push_back(neighbor)
            }
        }
    }
    
    result
}
```

### Depth-First Search (DFS)

```valkyrie
micro dfs⟨V, E⟩(graph: Graph⟨V, E⟩, start: VertexId) -> [VertexId] {
    let mut visited = HashSet::new()
    let mut result = []
    
    micro visit(node: VertexId) {
        if visited.contains(node) {
            return
        }
        
        visited.insert(node)
        result.push(node)
        
        for (neighbor, _) in graph.neighbors(node) {
            visit(neighbor)
        }
    }
    
    visit(start)
    result
}
```

## Shortest Path Algorithms

### Dijkstra's Algorithm

```valkyrie
micro dijkstra⟨V⟩(
    graph: Graph⟨V, f64⟩,
    start: VertexId,
    end: VertexId
) -> Option⟨([VertexId], f64)⟩ {
    let mut distances = HashMap::new()
    let mut previous = HashMap::new()
    let mut heap = BinaryHeap::new()
    
    # Initialize distances
    for (id, _) in graph.vertices {
        distances.insert(id, f64::infinity())
    }
    distances.insert(start, 0.0)
    
    # Priority queue: (distance, vertex)
    heap.push((0.0, start))
    
    while let Some((dist, current)) = heap.pop() {
        if current == end {
            # Reconstruct path
            let mut path = []
            let mut node = end
            while let Some(prev) = previous.get(node) {
                path.push(node)
                node = *prev
            }
            path.push(start)
            path.reverse()
            return Some((path, dist))
        }
        
        if dist > distances.get(current)? {
            continue
        }
        
        for (neighbor, weight) in graph.neighbors(current) {
            let new_dist = dist + weight
            if new_dist < distances.get(neighbor)? {
                distances.insert(neighbor, new_dist)
                previous.insert(neighbor, current)
                heap.push((new_dist, neighbor))
            }
        }
    }
    
    None
}
```

## Minimum Spanning Tree

### Kruskal's Algorithm

```valkyrie
micro kruskal⟨V⟩(graph: Graph⟨V, f64⟩) -> [(VertexId, VertexId, f64)] {
    let mut edges = []
    
    # Collect all edges
    for (from, neighbors) in graph.edges {
        for (to, weight) in neighbors {
            edges.push((from, to, weight))
        }
    }
    
    # Sort by weight
    edges.sort_by { |a, b| a.2.partial_cmp(b.2) }
    
    # Union-Find for cycle detection
    let mut uf = UnionFind::new(graph.vertices.keys())
    let mut mst = []
    
    for (from, to, weight) in edges {
        if uf.find(from) != uf.find(to) {
            uf.union(from, to)
            mst.push((from, to, weight))
        }
    }
    
    mst
}
```

## Topological Sort

```valkyrie
micro topological_sort⟨V, E⟩(graph: Graph⟨V, E⟩) -> Option⟨[VertexId]⟩ {
    let mut in_degree = HashMap::new()
    let mut result = []
    let mut queue = VecDeque::new()
    
    # Calculate in-degrees
    for (id, _) in graph.vertices {
        in_degree.insert(id, 0)
    }
    
    for (_, neighbors) in graph.edges {
        for (to, _) in neighbors {
            *in_degree.get_mut(to)? += 1
        }
    }
    
    # Start with vertices having no incoming edges
    for (id, degree) in in_degree {
        if degree == 0 {
            queue.push_back(id)
        }
    }
    
    # Process vertices
    while let Some(current) = queue.pop_front() {
        result.push(current)
        
        for (neighbor, _) in graph.neighbors(current) {
            let degree = in_degree.get_mut(neighbor)?
            *degree -= 1
            if *degree == 0 {
                queue.push_back(neighbor)
            }
        }
    }
    
    # Check for cycles
    if result.length == graph.vertices.length {
        Some(result)
    } else {
        None
    }
}
```

## Graph Visualization

Valkyrie can generate visual representations of graphs:

```valkyrie
micro to_dot⟨V, E⟩(graph: Graph⟨V, E⟩) -> string
where V: Display, E: Display
{
    let mut dot = "digraph G {\n"
    
    for (id, value) in graph.vertices {
        dot += "  {} [label=\"{}\"]\n".format(id, value)
    }
    
    for (from, neighbors) in graph.edges {
        for (to, edge) in neighbors {
            dot += "  {} -> {} [label=\"{}\"]\n".format(from, to, edge)
        }
    }
    
    dot += "}\n"
    dot
}
```

## Performance Considerations

1. **Memory Layout**: Use adjacency lists for sparse graphs, adjacency matrices for dense graphs
2. **Parallel Processing**: Many graph algorithms can be parallelized
3. **Caching**: Cache frequently accessed neighbors and distances
4. **Lazy Evaluation**: Use iterators for on-demand traversal
