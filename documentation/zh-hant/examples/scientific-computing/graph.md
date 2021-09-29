# 圖計算 (Graph Computing)

Valkyrie 提供了完整的圖數據結構和算法庫，支持各種圖計算任務，從簡單的圖遍歷到複雜的網絡分析。

## 基本圖類型

### 無向圖

```valkyrie
use graph::*

# 創建無向圖
let mut 𝐆 = UndirectedGraph::new()

# 添加节點
let 𝐯₁ = 𝐆.add_node("A")
let 𝐯₂ = 𝐆.add_node("B")
let 𝐯₃ = 𝐆.add_node("C")

# 添加邊
𝐆.add_edge(𝐯₁, 𝐯₂, 1.0)  # 權重為1.0的邊
𝐆.add_edge(𝐯₂, 𝐯₃, 2.0)
𝐆.add_edge(𝐯₁, 𝐯₃, 3.0)

# 圖的基本信息
let n = 𝐆.node_count()
let m = 𝐆.edge_count()
let neighbors = 𝐆.neighbors(𝐯₁)
```

### 有向圖

```valkyrie
# 創建有向圖
let mut 𝐃 = DirectedGraph::new()

# 添加节點和邊
let start = 𝐃.add_node("start")
let middle = 𝐃.add_node("middle")
let end = 𝐃.add_node("end")

𝐃.add_edge(start, middle, 1.0)
𝐃.add_edge(middle, end, 2.0)
𝐃.add_edge(start, end, 5.0)  # 直接路徑

# 有向圖特有操作
let in_degree = 𝐃.in_degree(end)
let out_degree = 𝐃.out_degree(start)
let predecessors = 𝐃.predecessors(end)
let successors = 𝐃.successors(start)
```

### 加權圖

```valkyrie
# 自定義邊權重類型
structure EdgeWeight {
    distance: f64,
    cost: f64,
    capacity: i32,
}

let mut 𝐖 = WeightedGraph⟨utf8, EdgeWeight⟩::new()

let city_a = 𝐖.add_node("CityA")
let city_b = 𝐖.add_node("CityB")

𝐖.add_edge(city_a, city_b, EdgeWeight {
    distance: 100.0,
    cost: 50.0,
    capacity: 1000
})
```

## 圖算法

### 遍歷算法

```valkyrie
# 深度優先搜索 (DFS)
let dfs_result = 𝐆.dfs(start_node)
let dfs_tree = 𝐆.dfs_tree(start_node)

# 廣度優先搜索 (BFS)
let bfs_result = 𝐆.bfs(start_node)
let bfs_levels = 𝐆.bfs_levels(start_node)

# 自定義遍歷
let custom_traversal = 𝐆.traverse(start_node) { $1, $2 ->
    print("Visiting node {} at depth {}", $1, $2)
    $2 < 5  # 限制遍歷深度
}
```

### 最短路徑算法

```valkyrie
# Dijkstra算法
let shortest_paths = 𝐆.dijkstra(start_node)
let path_to_target = 𝐆.shortest_path(start_node, target_node)

# A*算法（需要啟發式函數）
let heuristic = { $node -> estimate_distance_to_goal($node) }
let astar_path = graph.astar(start_node, target_node, heuristic)

# Floyd-Warshall算法（所有节點對之間的最短路徑）
let all_pairs_shortest = graph.floyd_warshall()

# Bellman-Ford算法（處理負權重）
let bellman_ford_result = graph.bellman_ford(start_node)
match result {
    Fine { value: distances } => print("Shortest distances: {:?}", distances),
    Fail { error: NegativeCycle } => print("Graph contains negative cycle")
}
```

### 連通性算法

```valkyrie
# 連通分量
let connected_components = graph.connected_components()
let is_connected = graph.is_connected()

# 強連通分量（有向圖）
let strongly_connected = digraph.strongly_connected_components()
let condensation = digraph.condensation()  # 強連通分量的DAG

# 橋和割點
let bridges = graph.find_bridges()
let articulation_points = graph.find_articulation_points()
```

### 最小生成樹

```valkyrie
# Kruskal算法
let mst_kruskal = graph.minimum_spanning_tree_kruskal()

# Prim算法
let mst_prim = graph.minimum_spanning_tree_prim(start_node)

# 最小生成森林
let msf = graph.minimum_spanning_forest()
```

### 網絡流算法

```valkyrie
# 最大流算法
let max_flow = graph.max_flow(source, sink)
let flow_value = max_flow.value
let flow_edges = max_flow.edges

# Ford-Fulkerson算法
let ford_fulkerson = graph.ford_fulkerson(source, sink)

# 最小割
let min_cut = graph.min_cut(source, sink)

# 最小費用最大流
let min_cost_max_flow = graph.min_cost_max_flow(source, sink)
```

## 圖分析

### 中心性度量

```valkyrie
# 度中心性
let degree_centrality = graph.degree_centrality()

# 接近中心性
let closeness_centrality = graph.closeness_centrality()

# 介數中心性
let betweenness_centrality = graph.betweenness_centrality()

# PageRank
let pagerank = graph.pagerank(0.85, 100)  # 阻尼因子0.85，最大迭代100次

# 特徵向量中心性
let eigenvector_centrality = graph.eigenvector_centrality()
```

### 社區檢測

```valkyrie
# Louvain算法
let communities_louvain = graph.louvain_communities()

# 標籤傳播算法
let communities_label_prop = graph.label_propagation_communities()

# 模組度優化
let modularity = graph.modularity(communities_louvain)

# 層次聚類
let dendrogram = graph.hierarchical_clustering()
let communities_at_level = dendrogram.communities_at_level(0.5)
```

### 圖統計

```valkyrie
# 基本統計
let density = graph.density()
let diameter = graph.diameter()
let radius = graph.radius()
let average_path_length = graph.average_path_length()

# 度分佈
let degree_distribution = graph.degree_distribution()
let degree_histogram = graph.degree_histogram()

# 聚類係數
let clustering_coefficient = graph.clustering_coefficient()
let local_clustering = graph.local_clustering_coefficient(node)

# 同配性
let assortativity = graph.assortativity()
```

## 特殊圖類型

### 二分圖

```valkyrie
# 創建二分圖
let mut bipartite = BipartiteGraph::new()

# 添加两個集合的节點
let left_nodes = bipartite.add_left_nodes(["L1", "L2", "L3"])
let right_nodes = bipartite.add_right_nodes(["R1", "R2"])

# 添加邊（只能在两個集合之間）
bipartite.add_edge(left_nodes[0], right_nodes[0], 1.0)
bipartite.add_edge(left_nodes[1], right_nodes[1], 1.0)

# 二分圖算法
let is_bipartite = graph.is_bipartite()
let bipartite_matching = bipartite.maximum_matching()
let vertex_cover = bipartite.minimum_vertex_cover()
```

### 平面圖

```valkyrie
# 平面性檢測
let is_planar = graph.is_planar()

# 平面嵌入
let planar_embedding = graph.planar_embedding()

# 面的計算
let faces = planar_embedding.faces()
let outer_face = planar_embedding.outer_face()
```

### 樹和森林

```valkyrie
# 樹的特殊操作
let tree = Tree::from_edges(edges)

# 樹的遍歷
let preorder = tree.preorder_traversal(root)
let postorder = tree.postorder_traversal(root)
let inorder = tree.inorder_traversal(root)  # 對于二叉樹

# 樹的性質
let height = tree.height()
let leaves = tree.leaves()
let internal_nodes = tree.internal_nodes()

# 最近公共祖先
let lca = tree.lowest_common_ancestor(node1, node2)
```

## 動態圖

```valkyrie
# 支持動態更新的圖
let mut dynamic_graph = DynamicGraph::new()

# 時間戳邊
dynamic_graph.add_temporal_edge(node_a, node_b, 1.0, timestamp: 100)
dynamic_graph.add_temporal_edge(node_b, node_c, 2.0, timestamp: 200)

# 查詢特定時間的圖狀態
let snapshot_at_150 = dynamic_graph.snapshot_at(150)

# 時間窗口查詢
let active_edges = dynamic_graph.edges_in_window(100, 300)

# 動態算法
let temporal_paths = dynamic_graph.temporal_paths(start, end, start_time, end_time)
```

## 圖的可視化

```valkyrie
use graph::visualization::*

# 佈局算法
let spring_layout = graph.spring_layout(iterations: 1000)
let circular_layout = graph.circular_layout()
let hierarchical_layout = digraph.hierarchical_layout()

# 導出為可視化格式
let dot_format = graph.to_dot()
let graphml_format = graph.to_graphml()
let json_format = graph.to_json()

# 交互式可視化
let visualization = GraphVisualization::new(graph)
visualization.set_node_color({ $node -> 
    if communities[0].contains($node) { "red" } else { "blue" }
})
visualization.set_edge_width { $edge -> $edge.weight * 2.0 }
visualization.render("output.svg")
```

## 並行圖算法

```valkyrie
use graph::parallel::*

# 並行BFS
let parallel_bfs = graph.parallel_bfs(start_node, num_threads: 8)

# 並行PageRank
let parallel_pagerank = graph.parallel_pagerank(0.85, 100, num_threads: 8)

# 並行連通分量
let parallel_components = graph.parallel_connected_components()

# 分佈式圖計算
let distributed_graph = DistributedGraph::from_partitions(partitions)
let distributed_pagerank = distributed_graph.distributed_pagerank()
```

## 圖數據庫接口

```valkyrie
# 圖查詢語言
let query_result = graph.query("
    MATCH (a)-[r]->(b)
    WHERE a.type = 'Person' AND r.weight > 0.5
    RETURN a.name, b.name, r.weight
")

# 路徑查詢
let path_query = graph.find_paths(
    start: { type: "City", name: "Beijing" },
    end: { type: "City", name: "Shanghai" },
    max_hops: 3
)

# 子圖匹配
let pattern = Graph::from_edges⟨[utf8; 3]⟩([
        ("A", "B", "friend"),
        ("B", "C", "colleague")
    ])
let matches = graph.subgraph_isomorphism(pattern)
```

## 性能優化

### 內存優化

```valkyrie
# 壓縮圖表示
let compressed_graph = graph.compress()  # 使用壓縮存儲格式

# 稀疏圖優化
let sparse_graph = SparseGraph::from_coo(rows, cols, values)  # COO格式
let csr_graph = sparse_graph.to_csr()  # 轉換為CSR格式

# 內存映射大圖
let memory_mapped = Graph::from_file_mmap("large_graph.bin")
```

### 緩存優化

```valkyrie
# 預計算常用查詢
let mut cached_graph = CachedGraph::new(graph)
cached_graph.precompute_shortest_paths()  # 預計算最短路徑
cached_graph.precompute_centrality()      # 預計算中心性

# 查詢緩存
let cached_result = cached_graph.cached_query("shortest_path", (start, end))
```

## 最佳實踐

### 1. 選擇合適的圖表示

```valkyrie
# 稠密圖使用鄰接矩陣
let dense_graph = AdjacencyMatrixGraph::new(node_count)

# 稀疏圖使用鄰接列表
let sparse_graph = AdjacencyListGraph::new()

# 大規模圖使用壓縮格式
let large_graph = CompressedGraph::new()
```

### 2. 算法選擇

```valkyrie
# 根據圖的特性選擇算法
micro choose_shortest_path_algorithm(graph: &Graph) -> ShortestPathResult {
    if graph.has_negative_weights() {
        graph.bellman_ford(start)
    } else if graph.is_sparse() {
        graph.dijkstra(start)
    } else {
        graph.floyd_warshall()
    }
}
```

### 3. 內存管理

```valkyrie
# 流式處理大圖
micro process_large_graph(graph_stream: GraphStream) {
    loop chunk in graph_stream.chunks(1000) {
        let subgraph = Graph::from_edges(chunk)
        let result = subgraph.compute_metrics()
        // 處理結果，釋放內存
    }
}
```

圖計算為 Valkyrie 提供了處理複雜網絡數據的強大能力，支持從社交網絡分析到生物信息學等各種應用場景。