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