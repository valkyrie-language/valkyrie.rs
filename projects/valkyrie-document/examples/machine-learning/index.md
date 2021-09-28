# 机器学习特性

Valkyrie 为机器学习提供了丰富的内置支持，包括数据处理、特征工程、模型训练、评估和部署等完整的机器学习工作流。

## 数据处理

### 数据加载和预处理

```valkyrie
use ml::data::*

# 从CSV加载数据
let dataset = Dataset::from_csv("data/iris.csv")
    .with_header(true)
    .with_separator(",")
    .load()

# 数据预览
dataset.head(5).show()
dataset.info()  # 显示数据类型和统计信息

# 数据清洗
let cleaned = dataset
    .drop_na()  # 删除缺失值
    .drop_duplicates()  # 删除重复行
    .filter(|row| row["age"] > 0)  # 条件过滤

# 数据类型转换
let processed = cleaned
    .cast_column("age", DataType::Integer)
    .cast_column("income", DataType::Decimal)
```

### 特征工程

```valkyrie
# 特征选择
let feature_selector = FeatureSelector::new()
    .variance_threshold(0.01)  # 方差阈值
    .correlation_threshold(0.95)  # 相关性阈值
    .mutual_info_threshold(0.1)  # 互信息阈值

let selected_features = feature_selector.fit_transform(dataset)

# 特征缩放
let scaler = StandardScaler::new()
let scaled_data = scaler.fit_transform(selected_features)

# 或使用其他缩放方法
let min_max_scaler = MinMaxScaler::new(0.0, 1.0)
let normalized_data = min_max_scaler.fit_transform(selected_features)

# 特征编码
let encoder = OneHotEncoder::new()
let encoded_categorical = encoder.fit_transform(dataset.select(["category", "type"]))

# 标签编码
let label_encoder = LabelEncoder::new()
let encoded_labels = label_encoder.fit_transform(dataset["target"])
```

### 数据分割

```valkyrie
# 训练测试分割
let (train_data, test_data) = dataset.train_test_split(0.8, random_state: 42)

# 交叉验证分割
let cv_folds = dataset.k_fold_split(k: 5, shuffle: true, random_state: 42)

# 分层抽样
let (stratified_train, stratified_test) = dataset
    .stratified_split(0.8, target_column: "class", random_state: 42)
```

## 传统机器学习算法

### 线性模型

```valkyrie
# 线性回归
let linear_reg = LinearRegression::new()
    .fit_intercept(true)
    .normalize(true)

linear_reg.fit(X_train, y_train)
let predictions = linear_reg.predict(X_test)

# 岭回归
let ridge = RidgeRegression::new(alpha: 1.0)
ridge.fit(X_train, y_train)

# Lasso回归
let lasso = LassoRegression::new(alpha: 0.1)
lasso.fit(X_train, y_train)

# 弹性网络
let elastic_net = ElasticNet::new(alpha: 0.1, l1_ratio: 0.5)
elastic_net.fit(X_train, y_train)

# 逻辑回归
let logistic = LogisticRegression::new()
    .max_iter(1000)
    .solver(Solver::LBFGS)
    .regularization(Regularization::L2(1.0))

logistic.fit(X_train, y_train)
let class_predictions = logistic.predict(X_test)
let probabilities = logistic.predict_proba(X_test)
```

### 树模型

```valkyrie
# 决策树
let decision_tree = DecisionTreeClassifier::new()
    .max_depth(10)
    .min_samples_split(2)
    .min_samples_leaf(1)
    .criterion(Criterion::Gini)

decision_tree.fit(X_train, y_train)

# 随机森林
let random_forest = RandomForestClassifier::new()
    .n_estimators(100)
    .max_depth(10)
    .max_features(MaxFeatures::Sqrt)
    .bootstrap(true)
    .random_state(42)

random_forest.fit(X_train, y_train)

# 梯度提升
let gradient_boosting = GradientBoostingClassifier::new()
    .n_estimators(100)
    .learning_rate(0.1)
    .max_depth(3)
    .subsample(0.8)

gradient_boosting.fit(X_train, y_train)

# XGBoost集成
let xgboost = XGBoostClassifier::new()
    .n_estimators(100)
    .max_depth(6)
    .learning_rate(0.1)
    .subsample(0.8)
    .colsample_bytree(0.8)
    .reg_alpha(0.1)
    .reg_lambda(1.0)

xgboost.fit(X_train, y_train)
```

### 支持向量机

```valkyrie
# SVM分类
let svm_classifier = SVC::new()
    .kernel(Kernel::RBF)
    .C(1.0)
    .gamma(Gamma::Scale)
    .probability(true)

svm_classifier.fit(X_train, y_train)

# SVM回归
let svm_regressor = SVR::new()
    .kernel(Kernel::RBF)
    .C(1.0)
    .epsilon(0.1)
    .gamma(Gamma::Auto)

svm_regressor.fit(X_train, y_train)

# 自定义核函数
let custom_kernel = |x1: &[f64], x2: &[f64]| {
    let dot_product = x1.iter().zip(x2).map(|(a, b)| a * b).sum::<f64>()
    (dot_product + 1.0).pow(3.0)  # 多项式核
}

let custom_svm = SVC::new()
    .kernel(Kernel::Custom(custom_kernel))
    .C(1.0)
```

### 聚类算法

```valkyrie
# K-Means聚类
let kmeans = KMeans::new(n_clusters: 3)
    .init(Init::KMeansPlusPlus)
    .max_iter(300)
    .tol(1e-4)
    .random_state(42)

let cluster_labels = kmeans.fit_predict(X)
let centroids = kmeans.cluster_centers()

# 层次聚类
let hierarchical = AgglomerativeClustering::new(n_clusters: 3)
    .linkage(Linkage::Ward)
    .distance_threshold(None)

let hier_labels = hierarchical.fit_predict(X)

# DBSCAN
let dbscan = DBSCAN::new(eps: 0.5, min_samples: 5)
let dbscan_labels = dbscan.fit_predict(X)

# 高斯混合模型
let gmm = GaussianMixture::new(n_components: 3)
    .covariance_type(CovarianceType::Full)
    .max_iter(100)
    .tol(1e-3)

gmm.fit(X)
let gmm_labels = gmm.predict(X)
let probabilities = gmm.predict_proba(X)
```

## 降维算法

```valkyrie
# 主成分分析
let pca = PCA::new(n_components: 2)
    .whiten(false)
    .svd_solver(SVDSolver::Auto)

let X_pca = pca.fit_transform(X)
let explained_variance = pca.explained_variance_ratio()

# t-SNE
let tsne = TSNE::new(n_components: 2)
    .perplexity(30.0)
    .learning_rate(200.0)
    .n_iter(1000)
    .random_state(42)

let X_tsne = tsne.fit_transform(X)

# UMAP
let umap = UMAP::new(n_components: 2)
    .n_neighbors(15)
    .min_dist(0.1)
    .metric(Metric::Euclidean)
    .random_state(42)

let X_umap = umap.fit_transform(X)

# 线性判别分析
let lda = LinearDiscriminantAnalysis::new(n_components: 2)
let X_lda = lda.fit_transform(X, y)
```

## 模型评估

### 分类指标

```valkyrie
# 基本分类指标
let accuracy = accuracy_score(y_true, y_pred)
let precision = precision_score(y_true, y_pred, average: Average::Weighted)
let recall = recall_score(y_true, y_pred, average: Average::Weighted)
let f1 = f1_score(y_true, y_pred, average: Average::Weighted)

# 混淆矩阵
let cm = confusion_matrix(y_true, y_pred)
cm.plot()  # 可视化混淆矩阵

# 分类报告
let report = classification_report(y_true, y_pred, target_names: class_names)
println!("{}", report)

# ROC曲线和AUC
let (fpr, tpr, thresholds) = roc_curve(y_true, y_scores)
let auc = auc_score(fpr, tpr)

# 绘制ROC曲线
plot_roc_curve(fpr, tpr, auc, title: "ROC Curve")

# PR曲线
let (precision, recall, thresholds) = precision_recall_curve(y_true, y_scores)
let ap = average_precision_score(y_true, y_scores)
```

### 回归指标

```valkyrie
# 回归评估指标
let mae = mean_absolute_error(y_true, y_pred)
let mse = mean_squared_error(y_true, y_pred)
let rmse = mse.sqrt()
let r2 = r2_score(y_true, y_pred)
let mape = mean_absolute_percentage_error(y_true, y_pred)

# 残差分析
let residuals = y_true - y_pred
plot_residuals(y_pred, residuals, title: "Residual Plot")
```

### 交叉验证

```valkyrie
# K折交叉验证
let cv_scores = cross_val_score(model, X, y, cv: 5, scoring: Scoring::Accuracy)
let mean_score = cv_scores.mean()
let std_score = cv_scores.std()

# 分层K折交叉验证
let stratified_scores = cross_val_score(
    model, X, y, 
    cv: StratifiedKFold::new(n_splits: 5, shuffle: true, random_state: 42),
    scoring: Scoring::F1Weighted
)

# 留一交叉验证
let loo_scores = cross_val_score(model, X, y, cv: LeaveOneOut::new())
```

## 超参数优化

### 网格搜索

```valkyrie
# 网格搜索
let param_grid = HashMap::from([
    ("C", vec![0.1, 1.0, 10.0, 100.0]),
    ("gamma", vec![0.001, 0.01, 0.1, 1.0]),
    ("kernel", vec!["rbf", "poly", "sigmoid"])
])

let grid_search = GridSearchCV::new(SVC::new())
    .param_grid(param_grid)
    .cv(5)
    .scoring(Scoring::Accuracy)
    .n_jobs(-1)  # 并行搜索

grid_search.fit(X_train, y_train)

let best_params = grid_search.best_params()
let best_score = grid_search.best_score()
let best_model = grid_search.best_estimator()
```

### 随机搜索

```valkyrie
# 随机搜索
let param_distributions = HashMap::from([
    ("C", Distribution::LogUniform(0.01, 100.0)),
    ("gamma", Distribution::LogUniform(0.0001, 1.0)),
    ("kernel", Distribution::Choice(vec!["rbf", "poly", "sigmoid"]))
])

let random_search = RandomizedSearchCV::new(SVC::new())
    .param_distributions(param_distributions)
    .n_iter(100)
    .cv(5)
    .scoring(Scoring::Accuracy)
    .random_state(42)

random_search.fit(X_train, y_train)
```

### 贝叶斯优化

```valkyrie
# 贝叶斯优化
let bayesian_opt = BayesianOptimization::new()
    .objective_function(|params| {
        let model = SVC::new()
            .C(params["C"])
            .gamma(params["gamma"])
        
        cross_val_score(model, X_train, y_train, cv: 5).mean()
    })
    .parameter_bounds(HashMap::from([
        ("C", (0.01, 100.0)),
        ("gamma", (0.0001, 1.0))
    ]))
    .n_calls(50)
    .acquisition_function(AcquisitionFunction::ExpectedImprovement)

let best_params = bayesian_opt.optimize()
```

## 集成学习

### 投票分类器

```valkyrie
# 硬投票
let voting_classifier = VotingClassifier::new()
    .estimators(vec![
        ("rf", RandomForestClassifier::new().n_estimators(100)),
        ("svm", SVC::new().probability(true)),
        ("nb", GaussianNB::new())
    ])
    .voting(Voting::Hard)

voting_classifier.fit(X_train, y_train)

# 软投票
let soft_voting = VotingClassifier::new()
    .estimators(vec![
        ("rf", RandomForestClassifier::new()),
        ("svm", SVC::new().probability(true)),
        ("lr", LogisticRegression::new())
    ])
    .voting(Voting::Soft)
```

### Stacking

```valkyrie
# 堆叠集成
let stacking_classifier = StackingClassifier::new()
    .estimators(vec![
        ("rf", RandomForestClassifier::new()),
        ("svm", SVC::new().probability(true)),
        ("nb", GaussianNB::new())
    ])
    .final_estimator(LogisticRegression::new())
    .cv(5)
    .stack_method(StackMethod::PredictProba)

stacking_classifier.fit(X_train, y_train)
```

### Bagging

```valkyrie
# Bagging分类器
let bagging = BaggingClassifier::new()
    .base_estimator(DecisionTreeClassifier::new())
    .n_estimators(100)
    .max_samples(0.8)
    .max_features(0.8)
    .bootstrap(true)
    .random_state(42)

bagging.fit(X_train, y_train)
```

## 特征重要性和模型解释

```valkyrie
# 特征重要性
let feature_importance = random_forest.feature_importances()
let feature_names = dataset.feature_names()

# 排序并可视化
let sorted_indices = feature_importance.argsort().reverse()
plot_feature_importance(feature_importance[sorted_indices], 
                       feature_names[sorted_indices])

# SHAP值解释
let explainer = TreeExplainer::new(random_forest)
let shap_values = explainer.shap_values(X_test)

# 绘制SHAP图
plot_shap_summary(shap_values, X_test, feature_names)
plot_shap_waterfall(shap_values[0], X_test[0], feature_names)

# 排列重要性
let perm_importance = permutation_importance(
    random_forest, X_test, y_test, 
    n_repeats: 10, random_state: 42
)
```

## 模型持久化

```valkyrie
# 保存模型
model.save("models/trained_model.vk")

# 保存预处理器
scaler.save("models/scaler.vk")
encoder.save("models/encoder.vk")

# 加载模型
let loaded_model = Model::load("models/trained_model.vk")
let loaded_scaler = StandardScaler::load("models/scaler.vk")

# 完整的预测管道
struct PredictionPipeline {
    scaler: StandardScaler,
    encoder: OneHotEncoder,
    model: RandomForestClassifier,
}

impl PredictionPipeline {
    fn predict(&self, raw_data: DataFrame) -> Vec<String> {
        let scaled_data = self.scaler.transform(raw_data.select_numeric())
        let encoded_data = self.encoder.transform(raw_data.select_categorical())
        let features = concat([scaled_data, encoded_data], axis: 1)
        
        self.model.predict(features)
    }
    
    fn save(&self, path: &str) {
        let pipeline_data = PipelineData {
            scaler: self.scaler.clone(),
            encoder: self.encoder.clone(),
            model: self.model.clone(),
        }
        pipeline_data.save(path)
    }
    
    fn load(path: &str) -> Self {
        let pipeline_data = PipelineData::load(path)
        Self {
            scaler: pipeline_data.scaler,
            encoder: pipeline_data.encoder,
            model: pipeline_data.model,
        }
    }
}
```

## 在线学习

```valkyrie
# 增量学习
let sgd_classifier = SGDClassifier::new()
    .loss(Loss::Hinge)
    .learning_rate(LearningRate::Constant)
    .eta0(0.01)

# 初始训练
sgd_classifier.fit(X_initial, y_initial)

# 增量更新
for (X_batch, y_batch) in data_stream {
    sgd_classifier.partial_fit(X_batch, y_batch)
}

# 在线被动攻击算法
let passive_aggressive = PassiveAggressiveClassifier::new()
    .C(1.0)
    .max_iter(1000)
    .tol(1e-3)

# 流式数据处理
struct StreamProcessor {
    model: SGDClassifier,
    scaler: IncrementalStandardScaler,
    window_size: usize,
    buffer: Vec<(Vec<f64>, i32)>,
}

impl StreamProcessor {
    fn process_sample(&mut self, sample: Vec<f64>, label: i32) {
        # 增量标准化
        let scaled_sample = self.scaler.partial_fit_transform(vec![sample])[0].clone()
        
        # 添加到缓冲区
        self.buffer.push((scaled_sample, label))
        
        # 当缓冲区满时进行批量更新
        if self.buffer.len() >= self.window_size {
            let (X_batch, y_batch): (Vec<_>, Vec<_>) = self.buffer.drain(..).unzip()
            self.model.partial_fit(X_batch, y_batch)
        }
    }
}
```

## 多任务学习

```valkyrie
# 多任务回归
let multi_task_lasso = MultiTaskLasso::new(alpha: 0.1)
multi_task_lasso.fit(X, Y)  # Y是多个目标的矩阵

# 多输出分类
let multi_output_classifier = MultiOutputClassifier::new(
    RandomForestClassifier::new().n_estimators(100)
)
multi_output_classifier.fit(X, Y_multi)

# 分类器链
let classifier_chain = ClassifierChain::new(
    LogisticRegression::new(),
    order: vec![0, 1, 2]  # 标签顺序
)
classifier_chain.fit(X, Y_multi)
```

## 异常检测

```valkyrie
# 孤立森林
let isolation_forest = IsolationForest::new()
    .n_estimators(100)
    .contamination(0.1)
    .random_state(42)

let anomaly_scores = isolation_forest.fit_predict(X)
let outliers = X[anomaly_scores == -1]  # -1表示异常

# 一类SVM
let one_class_svm = OneClassSVM::new()
    .kernel(Kernel::RBF)
    .gamma(Gamma::Scale)
    .nu(0.05)

one_class_svm.fit(X_normal)  # 只用正常数据训练
let predictions = one_class_svm.predict(X_test)

# 局部异常因子
let lof = LocalOutlierFactor::new()
    .n_neighbors(20)
    .contamination(0.1)

let lof_scores = lof.fit_predict(X)
```

## 时间序列分析

```valkyrie
# 时间序列分解
let ts_data = TimeSeries::from_csv("data/sales.csv", date_column: "date")
let decomposition = ts_data.seasonal_decompose(model: "additive", period: 12)

# ARIMA模型
let arima = ARIMA::new(order: (2, 1, 2))
arima.fit(ts_data)
let forecast = arima.forecast(steps: 12)

# 指数平滑
let exp_smoothing = ExponentialSmoothing::new()
    .trend("add")
    .seasonal("add")
    .seasonal_periods(12)

exp_smoothing.fit(ts_data)
let predictions = exp_smoothing.forecast(12)

# 时间序列交叉验证
let tscv = TimeSeriesSplit::new(n_splits: 5)
let cv_scores = cross_val_score(arima, ts_data, cv: tscv)
```

## 强化学习 (Reinforcement Learning)

### Q-Learning 算法

```valkyrie
struct QLearning {
    q_table: Array<f64, 2>,  # 状态-动作价值表
    α: f64,                  # 学习率
    γ: f64,                  # 折扣因子
    ε: f64,                  # 探索率
    state_size: usize,
    action_size: usize,
}

impl QLearning {
    fn new(state_size: usize, action_size: usize, α: f64, γ: f64, ε: f64) -> Self {
        Self {
            q_table: Array::zeros([state_size, action_size]),
            α, γ, ε, state_size, action_size,
        }
    }
    
    # ε-贪婪策略选择动作
    fn choose_action(&self, state: usize) -> usize {
        if random::<f64>() < self.ε {
            # 探索：随机选择动作
            random::<usize>() % self.action_size
        } else {
            # 利用：选择最优动作
            self.q_table.row(state).argmax()
        }
    }
    
    # 更新Q值
    fn update(&mut self, state: usize, action: usize, reward: f64, next_state: usize) {
        let current_q = self.q_table[[state, action]]
        let max_next_q = self.q_table.row(next_state).max()
        let target = reward + self.γ * max_next_q
        
        # Q-learning更新规则
        self.q_table[[state, action]] = current_q + self.α * (target - current_q)
    }
    
    # 训练智能体
    fn train<E: Environment>(&mut self, env: &mut E, episodes: usize) {
        for episode in 0..episodes {
            let mut state = env.reset()
            let mut total_reward = 0.0
            
            loop {
                let action = self.choose_action(state)
                let (next_state, reward, done) = env.step(action)
                
                self.update(state, action, reward, next_state)
                
                state = next_state
                total_reward += reward
                
                if done { break }
            }
            
            # 衰减探索率
            self.ε *= 0.995
            
            if episode % 100 == 0 {
                println!("Episode {}: Total Reward = {:.2}", episode, total_reward)
            }
        }
    }
}
```

### 深度Q网络 (DQN)

```valkyrie
struct DQN {
    network: NeuralNetwork,
    target_network: NeuralNetwork,
    replay_buffer: ReplayBuffer,
    γ: f64,
    ε: f64,
    ε_decay: f64,
    ε_min: f64,
    batch_size: usize,
    update_frequency: usize,
}

struct Experience {
    state: Tensor,
    action: usize,
    reward: f64,
    next_state: Tensor,
    done: bool,
}

struct ReplayBuffer {
    buffer: VecDeque<Experience>,
    capacity: usize,
}

impl ReplayBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    fn push(&mut self, experience: Experience) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front()
        }
        self.buffer.push_back(experience)
    }
    
    fn sample(&self, batch_size: usize) -> Vec<Experience> {
        let mut rng = thread_rng()
        self.buffer.iter()
            .choose_multiple(&mut rng, batch_size)
            .into_iter()
            .cloned()
            .collect()
    }
}

impl DQN {
    fn new(state_dim: usize, action_dim: usize, hidden_dims: &[usize]) -> Self {
        let network = NeuralNetwork::new()
            .add_layer(Dense::new(state_dim, hidden_dims[0]).activation(ReLU))
            .add_layer(Dense::new(hidden_dims[0], hidden_dims[1]).activation(ReLU))
            .add_layer(Dense::new(hidden_dims[1], action_dim))
        
        let target_network = network.clone()
        
        Self {
            network,
            target_network,
            replay_buffer: ReplayBuffer::new(10000),
            γ: 0.99,
            ε: 1.0,
            ε_decay: 0.995,
            ε_min: 0.01,
            batch_size: 32,
            update_frequency: 100,
        }
    }
    
    fn choose_action(&mut self, state: &Tensor) -> usize {
        if random::<f64>() < self.ε {
            random::<usize>() % self.network.output_size()
        } else {
            let q_values = self.network.forward(state)
            q_values.argmax()
        }
    }
    
    fn train_step(&mut self) {
        if self.replay_buffer.buffer.len() < self.batch_size {
            return
        }
        
        let batch = self.replay_buffer.sample(self.batch_size)
        
        let states = Tensor::stack(&batch.iter().map(|e| &e.state).collect::<Vec<_>>())
        let actions = Tensor::from_vec(batch.iter().map(|e| e.action as f64).collect())
        let rewards = Tensor::from_vec(batch.iter().map(|e| e.reward).collect())
        let next_states = Tensor::stack(&batch.iter().map(|e| &e.next_state).collect::<Vec<_>>())
        let dones = Tensor::from_vec(batch.iter().map(|e| if e.done { 1.0 } else { 0.0 }).collect())
        
        # 计算当前Q值
        let current_q_values = self.network.forward(&states)
        let current_q = current_q_values.gather(&actions)
        
        # 计算目标Q值
        let next_q_values = self.target_network.forward(&next_states)
        let max_next_q = next_q_values.max(dim: 1)
        let target_q = rewards + self.γ * max_next_q * (1.0 - dones)
        
        # 计算损失并反向传播
        let loss = mse_loss(&current_q, &target_q)
        self.network.backward(&loss)
        self.network.step()
        
        # 衰减探索率
        if self.ε > self.ε_min {
            self.ε *= self.ε_decay
        }
    }
    
    fn update_target_network(&mut self) {
        self.target_network = self.network.clone()
    }
}
```

### 策略梯度算法 (REINFORCE)

```valkyrie
struct PolicyGradient {
    policy_network: NeuralNetwork,
    γ: f64,
    α: f64,  # 学习率
}

struct Episode {
    states: Vec<Tensor>,
    actions: Vec<usize>,
    rewards: Vec<f64>,
}

impl PolicyGradient {
    fn new(state_dim: usize, action_dim: usize, hidden_dim: usize) -> Self {
        let policy_network = NeuralNetwork::new()
            .add_layer(Dense::new(state_dim, hidden_dim).activation(ReLU))
            .add_layer(Dense::new(hidden_dim, hidden_dim).activation(ReLU))
            .add_layer(Dense::new(hidden_dim, action_dim).activation(Softmax))
        
        Self {
            policy_network,
            γ: 0.99,
            α: 0.001,
        }
    }
    
    fn choose_action(&self, state: &Tensor) -> usize {
        let action_probs = self.policy_network.forward(state)
        
        # 根据概率分布采样动作
        let mut rng = thread_rng()
        let dist = WeightedIndex::new(&action_probs.to_vec()).unwrap()
        dist.sample(&mut rng)
    }
    
    fn compute_returns(&self, rewards: &[f64]) -> Vec<f64> {
        let mut returns = vec![0.0; rewards.len()]
        let mut running_return = 0.0
        
        # 从后往前计算折扣回报
        for i in (0..rewards.len()).rev() {
            running_return = rewards[i] + self.γ * running_return
            returns[i] = running_return
        }
        
        returns
    }
    
    fn train(&mut self, episode: Episode) {
        let returns = self.compute_returns(&episode.rewards)
        
        # 标准化回报
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64
        let std_return = (returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / returns.len() as f64).sqrt()
        let normalized_returns: Vec<f64> = returns.iter().map(|r| (r - mean_return) / (std_return + 1e-8)).collect()
        
        let mut total_loss = 0.0
        
        for (i, (state, action)) in episode.states.iter().zip(&episode.actions).enumerate() {
            let action_probs = self.policy_network.forward(state)
            let log_prob = action_probs[*action].ln()
            
            # 策略梯度损失
            let loss = -log_prob * normalized_returns[i]
            total_loss += loss
        }
        
        # 反向传播
        self.policy_network.backward(&Tensor::scalar(total_loss))
        self.policy_network.step()
    }
}
```

### Actor-Critic 算法

```valkyrie
struct ActorCritic {
    actor: NeuralNetwork,   # 策略网络
    critic: NeuralNetwork,  # 价值网络
    γ: f64,
    α_actor: f64,
    α_critic: f64,
}

impl ActorCritic {
    fn new(state_dim: usize, action_dim: usize, hidden_dim: usize) -> Self {
        let actor = NeuralNetwork::new()
            .add_layer(Dense::new(state_dim, hidden_dim).activation(ReLU))
            .add_layer(Dense::new(hidden_dim, action_dim).activation(Softmax))
        
        let critic = NeuralNetwork::new()
            .add_layer(Dense::new(state_dim, hidden_dim).activation(ReLU))
            .add_layer(Dense::new(hidden_dim, 1))
        
        Self {
            actor,
            critic,
            γ: 0.99,
            α_actor: 0.001,
            α_critic: 0.005,
        }
    }
    
    fn choose_action(&self, state: &Tensor) -> usize {
        let action_probs = self.actor.forward(state)
        let mut rng = thread_rng()
        let dist = WeightedIndex::new(&action_probs.to_vec()).unwrap()
        dist.sample(&mut rng)
    }
    
    fn train_step(&mut self, state: &Tensor, action: usize, reward: f64, next_state: &Tensor, done: bool) {
        # 计算TD误差
        let current_value = self.critic.forward(state)[0]
        let next_value = if done { 0.0 } else { self.critic.forward(next_state)[0] }
        let td_target = reward + self.γ * next_value
        let td_error = td_target - current_value
        
        # 更新Critic
        let critic_loss = td_error.powi(2)
        self.critic.backward(&Tensor::scalar(critic_loss))
        self.critic.step()
        
        # 更新Actor
        let action_probs = self.actor.forward(state)
        let log_prob = action_probs[action].ln()
        let actor_loss = -log_prob * td_error  # 使用TD误差作为优势函数
        
        self.actor.backward(&Tensor::scalar(actor_loss))
        self.actor.step()
    }
}
```

### 多智能体强化学习

```valkyrie
struct MultiAgentEnvironment {
    agents: Vec<Box<dyn Agent>>,
    state_size: usize,
    action_sizes: Vec<usize>,
    current_state: Tensor,
}

trait Agent {
    fn choose_action(&mut self, state: &Tensor, agent_id: usize) -> usize
    fn update(&mut self, experience: &MultiAgentExperience)
}

struct MultiAgentExperience {
    states: Vec<Tensor>,
    actions: Vec<usize>,
    rewards: Vec<f64>,
    next_states: Vec<Tensor>,
    done: bool,
}

struct MADDPG {
    actors: Vec<NeuralNetwork>,
    critics: Vec<NeuralNetwork>,
    target_actors: Vec<NeuralNetwork>,
    target_critics: Vec<NeuralNetwork>,
    replay_buffer: MultiAgentReplayBuffer,
}

impl MADDPG {
    fn new(state_dims: &[usize], action_dims: &[usize]) -> Self {
        let num_agents = state_dims.len()
        let mut actors = Vec::new()
        let mut critics = Vec::new()
        
        for i in 0..num_agents {
            # Actor网络：输入单个智能体状态，输出动作
            let actor = NeuralNetwork::new()
                .add_layer(Dense::new(state_dims[i], 128).activation(ReLU))
                .add_layer(Dense::new(128, 64).activation(ReLU))
                .add_layer(Dense::new(64, action_dims[i]).activation(Tanh))
            
            # Critic网络：输入所有智能体的状态和动作
            let total_state_dim: usize = state_dims.iter().sum()
            let total_action_dim: usize = action_dims.iter().sum()
            let critic = NeuralNetwork::new()
                .add_layer(Dense::new(total_state_dim + total_action_dim, 128).activation(ReLU))
                .add_layer(Dense::new(128, 64).activation(ReLU))
                .add_layer(Dense::new(64, 1))
            
            actors.push(actor)
            critics.push(critic)
        }
        
        let target_actors = actors.clone()
        let target_critics = critics.clone()
        
        Self {
            actors,
            critics,
            target_actors,
            target_critics,
            replay_buffer: MultiAgentReplayBuffer::new(100000),
        }
    }
    
    fn choose_actions(&mut self, states: &[Tensor]) -> Vec<Tensor> {
        self.actors.iter().zip(states)
            .map(|(actor, state)| actor.forward(state))
            .collect()
    }
    
    fn train(&mut self) {
        let batch = self.replay_buffer.sample(64)
        
        for agent_id in 0..self.actors.len() {
            self.train_agent(agent_id, &batch)
        }
        
        # 软更新目标网络
        self.soft_update_targets(0.01)
    }
    
    fn train_agent(&mut self, agent_id: usize, batch: &[MultiAgentExperience]) {
        # 训练Critic
        let states = self.concat_states(&batch.iter().map(|e| &e.states).collect::<Vec<_>>())
        let actions = self.concat_actions(&batch.iter().map(|e| &e.actions).collect::<Vec<_>>())
        let rewards = Tensor::from_vec(batch.iter().map(|e| e.rewards[agent_id]).collect())
        let next_states = self.concat_states(&batch.iter().map(|e| &e.next_states).collect::<Vec<_>>())
        
        # 使用目标网络计算目标Q值
        let next_actions = self.get_target_actions(&next_states)
        let next_state_actions = Tensor::concat(&[next_states, next_actions], dim: 1)
        let target_q = self.target_critics[agent_id].forward(&next_state_actions)
        let y = rewards + 0.99 * target_q
        
        let current_state_actions = Tensor::concat(&[states, actions], dim: 1)
        let current_q = self.critics[agent_id].forward(&current_state_actions)
        
        let critic_loss = mse_loss(&current_q, &y)
        self.critics[agent_id].backward(&critic_loss)
        self.critics[agent_id].step()
        
        # 训练Actor
        let predicted_actions = self.get_predicted_actions(&states, agent_id)
        let actor_loss = -self.critics[agent_id].forward(&Tensor::concat(&[states, predicted_actions], dim: 1)).mean()
        
        self.actors[agent_id].backward(&actor_loss)
        self.actors[agent_id].step()
    }
}
```

### 环境接口

```valkyrie
trait Environment {
    type State
    type Action
    type Reward
    
    fn reset(&mut self) -> Self::State
    fn step(&mut self, action: Self::Action) -> (Self::State, Self::Reward, bool)
    fn render(&self)
}

# 经典的CartPole环境
struct CartPole {
    x: f64,           # 小车位置
    x_dot: f64,       # 小车速度
    θ: f64,           # 杆子角度
    θ_dot: f64,       # 杆子角速度
    steps: usize,
    max_steps: usize,
}

impl Environment for CartPole {
    type State = Tensor
    type Action = usize  # 0: 左, 1: 右
    type Reward = f64
    
    fn reset(&mut self) -> Self::State {
        self.x = random::<f64>() * 0.1 - 0.05
        self.x_dot = random::<f64>() * 0.1 - 0.05
        self.θ = random::<f64>() * 0.1 - 0.05
        self.θ_dot = random::<f64>() * 0.1 - 0.05
        self.steps = 0
        
        Tensor::from_vec(vec![self.x, self.x_dot, self.θ, self.θ_dot])
    }
    
    fn step(&mut self, action: Self::Action) -> (Self::State, Self::Reward, bool) {
        let force = if action == 0 { -10.0 } else { 10.0 }
        
        # 物理仿真
        let cos_θ = self.θ.cos()
        let sin_θ = self.θ.sin()
        
        let temp = (force + 0.1 * self.θ_dot * self.θ_dot * sin_θ) / 1.1
        let θ_acc = (9.8 * sin_θ - cos_θ * temp) / (0.5 * (4.0/3.0 - 0.1 * cos_θ * cos_θ / 1.1))
        let x_acc = temp - 0.1 * θ_acc * cos_θ / 1.1
        
        # 更新状态
        self.x += 0.02 * self.x_dot
        self.x_dot += 0.02 * x_acc
        self.θ += 0.02 * self.θ_dot
        self.θ_dot += 0.02 * θ_acc
        
        self.steps += 1
        
        let state = Tensor::from_vec(vec![self.x, self.x_dot, self.θ, self.θ_dot])
        
        # 检查终止条件
        let done = self.x.abs() > 2.4 || self.θ.abs() > 0.2 || self.steps >= self.max_steps
        let reward = if done { 0.0 } else { 1.0 }
        
        (state, reward, done)
    }
    
    fn render(&self) {
        println!("Cart: x={:.2}, θ={:.2}°", self.x, self.θ.to_degrees())
    }
}
```

## 最佳实践

### 1. 数据预处理管道

```valkyrie
# 创建预处理管道
let preprocessing_pipeline = Pipeline::new()
    .add_step("imputer", SimpleImputer::new(strategy: Strategy::Mean))
    .add_step("scaler", StandardScaler::new())
    .add_step("selector", SelectKBest::new(k: 10))

# 完整的机器学习管道
let ml_pipeline = Pipeline::new()
    .add_step("preprocessing", preprocessing_pipeline)
    .add_step("classifier", RandomForestClassifier::new())

ml_pipeline.fit(X_train, y_train)
let predictions = ml_pipeline.predict(X_test)
```

### 2. 模型选择和验证

```valkyrie
# 模型比较
let models = vec![
    ("Logistic Regression", LogisticRegression::new()),
    ("Random Forest", RandomForestClassifier::new()),
    ("SVM", SVC::new()),
    ("Gradient Boosting", GradientBoostingClassifier::new())
]

let mut results = HashMap::new()
for (name, model) in models {
    let scores = cross_val_score(model, X, y, cv: 5)
    results.insert(name, (scores.mean(), scores.std()))
}

# 打印结果
for (name, (mean, std)) in results {
    println!("{}: {:.3} (+/- {:.3})", name, mean, std * 2)
}
```

### 3. 特征工程自动化

```valkyrie
# 自动特征工程
let feature_engineer = AutoFeatureEngineering::new()
    .polynomial_features(degree: 2)
    .interaction_features(true)
    .log_transform(columns: vec!["income", "age"])
    .binning(column: "age", bins: 5)

let engineered_features = feature_engineer.fit_transform(X)
```

Valkyrie 的机器学习特性提供了完整的工具链，从数据预处理到模型部署，支持传统机器学习和现代深度学习方法，为数据科学家和机器学习工程师提供了强大而灵活的开发环境。