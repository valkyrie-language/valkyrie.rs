# Machine Learning

Valkyrie provides comprehensive machine learning support, including traditional machine learning algorithms, deep learning frameworks, data processing tools, and more. Through high-performance computing and automatic differentiation systems, Valkyrie offers an efficient development experience for machine learning.

## Core Features

- **Automatic Differentiation**: Built-in autodiff system supporting forward and reverse modes
- **Tensor Operations**: High-performance multidimensional array operations
- **Neural Network Layers**: Rich set of predefined layers and custom layer support
- **Optimizers**: Multiple optimization algorithm implementations
- **Data Processing**: Data loading, preprocessing, and augmentation

## Tensor Operations

### Tensor Basics

```valkyrie
use valkyrie::tensor::*

# Create tensors
let a = Tensor::from([1.0, 2.0, 3.0, 4.0])
let b = Tensor::zeros([3, 4])
let c = Tensor::ones([2, 3, 4])
let d = Tensor::random([100, 50])

# Tensor operations
let sum = a + b
let product = a * b
let matmul = a.matmul(&b)

# Indexing and slicing
let row = b[0]      # First row
let col = b[:, 0]   # First column
let sub = b[0..2, 1..3]  # Submatrix

# Shape operations
let reshaped = b.reshape([6, 2])
let transposed = b.transpose()
let flattened = b.flatten()
```

### Tensor Operations

```valkyrie
# Mathematical operations
let x = Tensor::random([32, 128])

let exp_x = x.exp()
let log_x = x.log()
let sin_x = x.sin()
let sqrt_x = x.sqrt()

# Reduction operations
let sum_all = x.sum()
let mean_all = x.mean()
let max_val = x.max()
let min_val = x.min()

# Reduction along specific axis
let sum_axis0 = x.sum(axis: 0)  # Sum along first dimension
let mean_axis1 = x.mean(axis: 1)  # Mean along second dimension

# Broadcasting
let a = Tensor::from([1.0, 2.0, 3.0])  # Shape: [3]
let b = Tensor::ones([4, 3])            # Shape: [4, 3]
let c = a + b  # Broadcasting: [3] -> [4, 3]
```

## Data Processing

### Data Loading

```valkyrie
class DataLoader {
    dataset: Dataset,
    batch_size: usize,
    shuffle: bool,
    drop_last: bool,
}

imply DataLoader {
    micro new(dataset: Dataset, batch_size: usize, shuffle: bool = true) -> Self {
        DataLoader {
            dataset,
            batch_size,
            shuffle,
            drop_last: true,
        }
    }
    
    micro iter(mut self) -> DataLoaderIterator {
        let indices = if self.shuffle {
            self.dataset.indices().shuffle()
        } else {
            self.dataset.indices()
        }
        
        DataLoaderIterator {
            dataloader: self,
            indices,
            current: 0,
        }
    }
}

class DataLoaderIterator {
    dataloader: DataLoader,
    indices: [usize],
    current: usize,
}

imply DataLoaderIterator {
    micro next(mut self) -> Option<(Tensor, Tensor)> {
        if self.current >= self.indices.length {
            return None
        }
        
        let end = min(self.current + self.dataloader.batch_size, self.indices.length)
        let batch_indices = self.indices[self.current..end]
        
        self.current = end
        
        let (inputs, targets) = self.dataloader.dataset.get_batch(batch_indices)
        Some((inputs, targets))
    }
}
```

### Data Preprocessing

```valkyrie
class DataPreprocessor {
    normalizers: [Normalizer],
    augmentations: [Augmentation],
}

imply DataPreprocessor {
    micro normalize(mut self, data: Tensor) -> Tensor {
        for normalizer in self.normalizers {
            data = normalizer.apply(data)
        }
        data
    }
    
    micro augment(mut self, data: Tensor) -> Tensor {
        for augmentation in self.augmentations {
            data = augmentation.apply(data)
        }
        data
    }
}

class StandardScaler {
    mean: Tensor,
    std: Tensor,
}

imply StandardScaler {
    micro fit(mut self, data: Tensor) {
        self.mean = data.mean(axis: 0)
        self.std = data.std(axis: 0)
    }
    
    micro transform(self, data: Tensor) -> Tensor {
        (data - self.mean) / self.std
    }
}

class MinMaxScaler {
    min: Tensor,
    max: Tensor,
    feature_range: (f32, f32),
}

imply MinMaxScaler {
    micro fit(mut self, data: Tensor) {
        self.min = data.min(axis: 0)
        self.max = data.max(axis: 0)
    }
    
    micro transform(self, data: Tensor) -> Tensor {
        let (min_val, max_val) = self.feature_range
        let scale = (max_val - min_val) / (self.max - self.min)
        (data - self.min) * scale + min_val
    }
}
```

## Traditional Machine Learning

### Linear Regression

```valkyrie
class LinearRegression {
    weights: Tensor,
    bias: f32,
    learning_rate: f32,
}

imply LinearRegression {
    micro new(input_size: usize, learning_rate: f32 = 0.01) -> Self {
        LinearRegression {
            weights: Tensor::zeros([input_size]),
            bias: 0.0,
            learning_rate,
        }
    }
    
    micro predict(self, x: Tensor) -> Tensor {
        x.matmul(&self.weights) + self.bias
    }
    
    micro fit(mut self, x: Tensor, y: Tensor, epochs: usize) {
        let n = x.shape()[0] as f32
        
        for _ in 0..epochs {
            let predictions = self.predict(&x)
            let errors = predictions - y
            
            # Compute gradients
            let dw = x.transpose().matmul(&errors) / n
            let db = errors.mean()
            
            # Update parameters
            self.weights = self.weights - self.learning_rate * dw
            self.bias = self.bias - self.learning_rate * db
        }
    }
}
```

### Logistic Regression

```valkyrie
class LogisticRegression {
    weights: Tensor,
    bias: f32,
    learning_rate: f32,
}

imply LogisticRegression {
    micro sigmoid(x: Tensor) -> Tensor {
        1.0 / (1.0 + (-x).exp())
    }
    
    micro predict_proba(self, x: Tensor) -> Tensor {
        Self::sigmoid(x.matmul(&self.weights) + self.bias)
    }
    
    micro predict(self, x: Tensor) -> Tensor {
        let proba = self.predict_proba(&x)
        (proba > 0.5).cast::<i32>()
    }
    
    micro fit(mut self, x: Tensor, y: Tensor, epochs: usize) {
        let n = x.shape()[0] as f32
        
        for _ in 0..epochs {
            let proba = self.predict_proba(&x)
            let errors = proba - y
            
            # Compute gradients
            let dw = x.transpose().matmul(&errors) / n
            let db = errors.mean()
            
            # Update parameters
            self.weights = self.weights - self.learning_rate * dw
            self.bias = self.bias - self.learning_rate * db
        }
    }
}
```

### Decision Tree

```valkyrie
class DecisionTreeClassifier {
    max_depth: usize,
    min_samples_split: usize,
    tree: TreeNode,
}

union TreeNode {
    Leaf { class: i32 },
    Internal {
        feature: usize,
        threshold: f32,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
}

imply DecisionTreeClassifier {
    micro new(max_depth: usize = 10, min_samples_split: usize = 2) -> Self {
        DecisionTreeClassifier {
            max_depth,
            min_samples_split,
            tree: None,
        }
    }
    
    micro fit(mut self, x: Tensor, y: Tensor) {
        self.tree = self.build_tree(x, y, 0)
    }
    
    micro build_tree(self, x: Tensor, y: Tensor, depth: usize) -> TreeNode {
        # Stopping conditions
        if depth >= self.max_depth || x.shape()[0] < self.min_samples_split {
            return TreeNode::Leaf { class: self.most_common_class(y) }
        }
        
        # Find best split
        let (best_feature, best_threshold, best_gain) = self.find_best_split(x, y)
        
        if best_gain <= 0.0 {
            return TreeNode::Leaf { class: self.most_common_class(y) }
        }
        
        # Split data
        let (left_x, left_y, right_x, right_y) = self.split_data(x, y, best_feature, best_threshold)
        
        TreeNode::Internal {
            feature: best_feature,
            threshold: best_threshold,
            left: Box::new(self.build_tree(left_x, left_y, depth + 1)),
            right: Box::new(self.build_tree(right_x, right_y, depth + 1)),
        }
    }
    
    micro predict(self, x: Tensor) -> Tensor {
        let mut predictions = Tensor::zeros([x.shape()[0]])
        
        for i in 0..x.shape()[0] {
            predictions[i] = self.predict_single(x[i], &self.tree)
        }
        
        predictions
    }
    
    micro predict_single(self, sample: Tensor, node: TreeNode) -> i32 {
        match node {
            TreeNode::Leaf { class } => class,
            TreeNode::Internal { feature, threshold, left, right } => {
                if sample[feature] <= threshold {
                    self.predict_single(sample, left)
                } else {
                    self.predict_single(sample, right)
                }
            }
        }
    }
}
```

## Model Evaluation

```valkyrie
class ModelEvaluator;

imply ModelEvaluator {
    micro accuracy(predictions: Tensor, targets: Tensor) -> f32 {
        let correct = (predictions == targets).sum()
        correct as f32 / predictions.shape()[0] as f32
    }
    
    micro precision(predictions: Tensor, targets: Tensor, positive_class: i32) -> f32 {
        let true_positives = ((predictions == positive_class) & (targets == positive_class)).sum()
        let predicted_positives = (predictions == positive_class).sum()
        
        if predicted_positives == 0 {
            0.0
        } else {
            true_positives as f32 / predicted_positives as f32
        }
    }
    
    micro recall(predictions: Tensor, targets: Tensor, positive_class: i32) -> f32 {
        let true_positives = ((predictions == positive_class) & (targets == positive_class)).sum()
        let actual_positives = (targets == positive_class).sum()
        
        if actual_positives == 0 {
            0.0
        } else {
            true_positives as f32 / actual_positives as f32
        }
    }
    
    micro f1_score(predictions: Tensor, targets: Tensor, positive_class: i32) -> f32 {
        let p = Self::precision(&predictions, &targets, positive_class)
        let r = Self::recall(&predictions, &targets, positive_class)
        
        if p + r == 0.0 {
            0.0
        } else {
            2.0 * p * r / (p + r)
        }
    }
    
    micro confusion_matrix(predictions: Tensor, targets: Tensor, num_classes: usize) -> Tensor {
        let mut matrix = Tensor::zeros([num_classes, num_classes])
        
        for i in 0..predictions.shape()[0] {
            let pred = predictions[i] as usize
            let target = targets[i] as usize
            matrix[pred, target] += 1
        }
        
        matrix
    }
    
    micro mean_squared_error(predictions: Tensor, targets: Tensor) -> f32 {
        ((predictions - targets) ^ 2).mean()
    }
    
    micro r2_score(predictions: Tensor, targets: Tensor) -> f32 {
        let ss_res = ((targets - predictions) ^ 2).sum()
        let ss_tot = ((targets - targets.mean()) ^ 2).sum()
        1.0 - ss_res / ss_tot
    }
}
```

## Document Navigation

### [Heterogeneous Computing](heterogeneous-computing.md)
Learn how to use Valkyrie for GPU-accelerated machine learning computing, including CUDA and OpenCL integration.

### [NDArray Operations](ndarray.md)
Detailed introduction to Valkyrie's multidimensional array operations, including tensor operations, broadcasting, indexing, and more.

### [Web Crawler](web-crawler.md)
Learn how to use Valkyrie for data collection and web crawling, including asynchronous HTTP requests, HTML parsing, and data extraction.

## Summary

Valkyrie provides comprehensive machine learning support:

- **Tensor Operations**: High-performance multidimensional array operations
- **Data Processing**: Data loading, preprocessing, and augmentation
- **Traditional ML**: Linear regression, logistic regression, decision trees, etc.
- **Model Evaluation**: Multiple evaluation metrics and validation methods
- **Deep Learning**: Neural network construction and training

Through these tools, Valkyrie offers an efficient development experience for machine learning practitioners.
