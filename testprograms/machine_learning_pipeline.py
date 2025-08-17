# Complex Machine Learning Pipeline - Python
# This demonstrates advanced Python patterns for comprehensive test case generation

import asyncio
import concurrent.futures
import json
import logging
import pickle
import threading
import time
from abc import ABC, abstractmethod
from collections import defaultdict, namedtuple
from contextlib import contextmanager
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum, IntEnum
from functools import wraps, lru_cache, singledispatch
from pathlib import Path
from typing import (
    Any, Dict, List, Optional, Union, Tuple, Set, Callable, 
    Generic, TypeVar, Protocol, Iterator, AsyncIterator, Coroutine
)
from uuid import uuid4
import weakref


# Type definitions
T = TypeVar('T')
ModelType = TypeVar('ModelType', bound='BaseModel')


class ModelStatus(Enum):
    TRAINING = "training"
    TRAINED = "trained"
    EVALUATING = "evaluating"
    DEPLOYED = "deployed"
    FAILED = "failed"
    ARCHIVED = "archived"


class DataType(IntEnum):
    NUMERICAL = 1
    CATEGORICAL = 2
    TEXT = 3
    IMAGE = 4
    AUDIO = 5
    VIDEO = 6


class MetricType(Enum):
    ACCURACY = "accuracy"
    PRECISION = "precision"
    RECALL = "recall"
    F1_SCORE = "f1_score"
    AUC_ROC = "auc_roc"
    MSE = "mse"
    RMSE = "rmse"
    MAE = "mae"


# Custom exceptions
class MLPipelineException(Exception):
    """Base exception for ML pipeline"""
    pass


class ModelNotTrainedException(MLPipelineException):
    """Raised when attempting to use an untrained model"""
    pass


class InsufficientDataException(MLPipelineException):
    """Raised when there's insufficient data for training"""
    def __init__(self, required_samples: int, available_samples: int):
        self.required_samples = required_samples
        self.available_samples = available_samples
        super().__init__(
            f"Insufficient data: required {required_samples}, got {available_samples}"
        )


class ModelValidationException(MLPipelineException):
    """Raised when model validation fails"""
    pass


# Data structures
@dataclass
class DataPoint:
    """Represents a single data point in the pipeline"""
    id: str
    features: Dict[str, Any]
    label: Optional[Any] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    timestamp: datetime = field(default_factory=datetime.now)
    data_type: DataType = DataType.NUMERICAL


@dataclass
class TrainingConfig:
    """Configuration for model training"""
    model_type: str
    hyperparameters: Dict[str, Any] = field(default_factory=dict)
    validation_split: float = 0.2
    batch_size: int = 32
    epochs: int = 100
    early_stopping_patience: int = 10
    learning_rate: float = 0.001
    random_seed: int = 42


@dataclass
class ModelMetrics:
    """Container for model performance metrics"""
    accuracy: Optional[float] = None
    precision: Optional[float] = None
    recall: Optional[float] = None
    f1_score: Optional[float] = None
    auc_roc: Optional[float] = None
    mse: Optional[float] = None
    rmse: Optional[float] = None
    mae: Optional[float] = None
    training_time: Optional[float] = None
    evaluation_time: Optional[float] = None
    custom_metrics: Dict[str, float] = field(default_factory=dict)


# Protocol definitions
class Trainable(Protocol):
    """Protocol for trainable models"""
    def fit(self, X: List[Dict[str, Any]], y: List[Any]) -> None: ...
    def predict(self, X: List[Dict[str, Any]]) -> List[Any]: ...


class Evaluatable(Protocol):
    """Protocol for evaluatable models"""
    def evaluate(self, X: List[Dict[str, Any]], y: List[Any]) -> ModelMetrics: ...


# Abstract base classes
class BasePreprocessor(ABC):
    """Abstract base class for data preprocessors"""
    
    def __init__(self, name: str):
        self.name = name
        self.is_fitted = False
        self.transform_history: List[Dict[str, Any]] = []
    
    @abstractmethod
    def fit(self, data: List[DataPoint]) -> 'BasePreprocessor':
        """Fit the preprocessor to the data"""
        pass
    
    @abstractmethod
    def transform(self, data: List[DataPoint]) -> List[DataPoint]:
        """Transform the data"""
        pass
    
    def fit_transform(self, data: List[DataPoint]) -> List[DataPoint]:
        """Fit and transform the data in one step"""
        return self.fit(data).transform(data)


class BaseModel(ABC, Generic[T]):
    """Abstract base class for ML models"""
    
    def __init__(self, model_id: str, config: TrainingConfig):
        self.model_id = model_id
        self.config = config
        self.status = ModelStatus.TRAINING
        self.metrics = ModelMetrics()
        self.created_at = datetime.now()
        self.last_updated = datetime.now()
        self.version = 1
        self.artifacts_path: Optional[Path] = None
        self._lock = threading.RLock()
    
    @abstractmethod
    async def train(self, data: List[DataPoint]) -> None:
        """Train the model"""
        pass
    
    @abstractmethod
    def predict(self, data: List[DataPoint]) -> List[T]:
        """Make predictions"""
        pass
    
    @abstractmethod
    def evaluate(self, data: List[DataPoint]) -> ModelMetrics:
        """Evaluate model performance"""
        pass
    
    def save(self, path: Path) -> None:
        """Save the model to disk"""
        with self._lock:
            path.mkdir(parents=True, exist_ok=True)
            model_data = {
                'model_id': self.model_id,
                'config': self.config,
                'status': self.status.value,
                'metrics': self.metrics,
                'created_at': self.created_at.isoformat(),
                'last_updated': self.last_updated.isoformat(),
                'version': self.version
            }
            
            with open(path / 'model_metadata.json', 'w') as f:
                json.dump(model_data, f, default=str, indent=2)
            
            with open(path / 'model.pkl', 'wb') as f:
                pickle.dump(self, f)
            
            self.artifacts_path = path


class BaseValidator(ABC):
    """Abstract base class for model validators"""
    
    @abstractmethod
    def validate(self, model: BaseModel, data: List[DataPoint]) -> bool:
        """Validate the model"""
        pass


# Concrete implementations
class NumericPreprocessor(BasePreprocessor):
    """Preprocessor for numerical features"""
    
    def __init__(self, name: str = "numeric_preprocessor", normalize: bool = True):
        super().__init__(name)
        self.normalize = normalize
        self.feature_stats: Dict[str, Dict[str, float]] = {}
    
    def fit(self, data: List[DataPoint]) -> 'NumericPreprocessor':
        """Calculate statistics for normalization"""
        if not data:
            raise InsufficientDataException(1, 0)
        
        # Collect all numerical features
        all_features = defaultdict(list)
        for point in data:
            for key, value in point.features.items():
                if isinstance(value, (int, float)):
                    all_features[key].append(value)
        
        # Calculate statistics
        for feature_name, values in all_features.items():
            if values:
                self.feature_stats[feature_name] = {
                    'mean': sum(values) / len(values),
                    'std': (sum((x - sum(values) / len(values)) ** 2 for x in values) / len(values)) ** 0.5,
                    'min': min(values),
                    'max': max(values)
                }
        
        self.is_fitted = True
        return self
    
    def transform(self, data: List[DataPoint]) -> List[DataPoint]:
        """Normalize numerical features"""
        if not self.is_fitted:
            raise ModelNotTrainedException("Preprocessor must be fitted before transformation")
        
        transformed_data = []
        for point in data:
            new_features = {}
            for key, value in point.features.items():
                if key in self.feature_stats and isinstance(value, (int, float)):
                    stats = self.feature_stats[key]
                    if self.normalize and stats['std'] > 0:
                        new_features[key] = (value - stats['mean']) / stats['std']
                    else:
                        new_features[key] = value
                else:
                    new_features[key] = value
            
            transformed_data.append(DataPoint(
                id=point.id,
                features=new_features,
                label=point.label,
                metadata=point.metadata,
                timestamp=point.timestamp,
                data_type=point.data_type
            ))
        
        return transformed_data


class LinearRegressionModel(BaseModel[float]):
    """Simple linear regression model implementation"""
    
    def __init__(self, model_id: str, config: TrainingConfig):
        super().__init__(model_id, config)
        self.weights: Optional[Dict[str, float]] = None
        self.bias: float = 0.0
        self.feature_names: List[str] = []
        self.training_losses: List[float] = []
    
    async def train(self, data: List[DataPoint]) -> None:
        """Train the linear regression model using gradient descent"""
        with self._lock:
            if len(data) < 10:
                raise InsufficientDataException(10, len(data))
            
            self.status = ModelStatus.TRAINING
            start_time = time.time()
            
            # Extract features and labels
            X, y = self._prepare_data(data)
            if not X or not y:
                raise InsufficientDataException(1, 0)
            
            self.feature_names = list(X[0].keys())
            n_features = len(self.feature_names)
            
            # Initialize weights
            self.weights = {name: 0.0 for name in self.feature_names}
            self.bias = 0.0
            
            # Gradient descent
            learning_rate = self.config.learning_rate
            n_samples = len(X)
            
            for epoch in range(self.config.epochs):
                # Forward pass
                predictions = []
                for features in X:
                    pred = self.bias + sum(self.weights[name] * features.get(name, 0) 
                                         for name in self.feature_names)
                    predictions.append(pred)
                
                # Calculate loss
                mse_loss = sum((pred - actual) ** 2 for pred, actual in zip(predictions, y)) / n_samples
                self.training_losses.append(mse_loss)
                
                # Backward pass
                weight_gradients = {name: 0.0 for name in self.feature_names}
                bias_gradient = 0.0
                
                for features, pred, actual in zip(X, predictions, y):
                    error = pred - actual
                    bias_gradient += error
                    for name in self.feature_names:
                        weight_gradients[name] += error * features.get(name, 0)
                
                # Update weights
                for name in self.feature_names:
                    self.weights[name] -= learning_rate * weight_gradients[name] / n_samples
                self.bias -= learning_rate * bias_gradient / n_samples
                
                # Early stopping check
                if epoch > self.config.early_stopping_patience:
                    recent_losses = self.training_losses[-self.config.early_stopping_patience:]
                    if all(abs(recent_losses[i] - recent_losses[i-1]) < 1e-6 
                           for i in range(1, len(recent_losses))):
                        break
                
                # Simulate async behavior
                if epoch % 10 == 0:
                    await asyncio.sleep(0.01)
            
            training_time = time.time() - start_time
            self.metrics.training_time = training_time
            self.status = ModelStatus.TRAINED
            self.last_updated = datetime.now()
    
    def predict(self, data: List[DataPoint]) -> List[float]:
        """Make predictions using the trained model"""
        if self.status != ModelStatus.TRAINED or self.weights is None:
            raise ModelNotTrainedException("Model must be trained before making predictions")
        
        predictions = []
        for point in data:
            pred = self.bias + sum(self.weights[name] * point.features.get(name, 0) 
                                 for name in self.feature_names)
            predictions.append(pred)
        
        return predictions
    
    def evaluate(self, data: List[DataPoint]) -> ModelMetrics:
        """Evaluate model performance"""
        start_time = time.time()
        
        X, y_true = self._prepare_data(data)
        y_pred = [self.bias + sum(self.weights[name] * features.get(name, 0) 
                                for name in self.feature_names) for features in X]
        
        # Calculate metrics
        n = len(y_true)
        mse = sum((pred - actual) ** 2 for pred, actual in zip(y_pred, y_true)) / n
        rmse = mse ** 0.5
        mae = sum(abs(pred - actual) for pred, actual in zip(y_pred, y_true)) / n
        
        evaluation_time = time.time() - start_time
        
        metrics = ModelMetrics(
            mse=mse,
            rmse=rmse,
            mae=mae,
            evaluation_time=evaluation_time
        )
        
        return metrics
    
    def _prepare_data(self, data: List[DataPoint]) -> Tuple[List[Dict[str, float]], List[float]]:
        """Extract features and labels from data points"""
        X, y = [], []
        for point in data:
            if point.label is not None:
                # Only include numerical features
                features = {k: v for k, v in point.features.items() 
                           if isinstance(v, (int, float))}
                if features:
                    X.append(features)
                    y.append(float(point.label))
        return X, y


class CrossValidator(BaseValidator):
    """K-fold cross validation"""
    
    def __init__(self, k_folds: int = 5, min_accuracy: float = 0.7):
        self.k_folds = k_folds
        self.min_accuracy = min_accuracy
    
    def validate(self, model: BaseModel, data: List[DataPoint]) -> bool:
        """Perform k-fold cross validation"""
        if len(data) < self.k_folds:
            raise InsufficientDataException(self.k_folds, len(data))
        
        fold_size = len(data) // self.k_folds
        accuracies = []
        
        for i in range(self.k_folds):
            # Split data
            start_idx = i * fold_size
            end_idx = start_idx + fold_size if i < self.k_folds - 1 else len(data)
            
            test_data = data[start_idx:end_idx]
            train_data = data[:start_idx] + data[end_idx:]
            
            # Train on fold
            temp_model = type(model)(f"{model.model_id}_fold_{i}", model.config)
            asyncio.run(temp_model.train(train_data))
            
            # Evaluate on test fold
            metrics = temp_model.evaluate(test_data)
            
            # Calculate accuracy (for regression, use R²)
            if hasattr(metrics, 'mse') and metrics.mse is not None:
                # For regression: pseudo R² = 1 - (SSres / SStot)
                y_true = [point.label for point in test_data if point.label is not None]
                y_mean = sum(y_true) / len(y_true)
                ss_tot = sum((y - y_mean) ** 2 for y in y_true)
                ss_res = metrics.mse * len(y_true)
                r_squared = 1 - (ss_res / ss_tot) if ss_tot > 0 else 0
                accuracies.append(max(0, r_squared))
            else:
                accuracies.append(metrics.accuracy or 0)
        
        avg_accuracy = sum(accuracies) / len(accuracies)
        return avg_accuracy >= self.min_accuracy


# Main pipeline class
class MLPipeline:
    """Main machine learning pipeline orchestrator"""
    
    def __init__(self, pipeline_id: str = None):
        self.pipeline_id = pipeline_id or str(uuid4())
        self.preprocessors: List[BasePreprocessor] = []
        self.models: Dict[str, BaseModel] = {}
        self.validators: List[BaseValidator] = []
        self.data_store: List[DataPoint] = []
        self.pipeline_config = {}
        self.execution_history: List[Dict[str, Any]] = []
        self._lock = threading.RLock()
        self._model_registry = weakref.WeakValueDictionary()
        
        # Setup logging
        self.logger = logging.getLogger(f"MLPipeline-{self.pipeline_id}")
        self.logger.setLevel(logging.INFO)
        if not self.logger.handlers:
            handler = logging.StreamHandler()
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
            handler.setFormatter(formatter)
            self.logger.addHandler(handler)
    
    def add_preprocessor(self, preprocessor: BasePreprocessor) -> 'MLPipeline':
        """Add a preprocessor to the pipeline"""
        with self._lock:
            self.preprocessors.append(preprocessor)
            self.logger.info(f"Added preprocessor: {preprocessor.name}")
        return self
    
    def add_model(self, model: BaseModel) -> 'MLPipeline':
        """Add a model to the pipeline"""
        with self._lock:
            self.models[model.model_id] = model
            self._model_registry[model.model_id] = model
            self.logger.info(f"Added model: {model.model_id}")
        return self
    
    def add_validator(self, validator: BaseValidator) -> 'MLPipeline':
        """Add a validator to the pipeline"""
        with self._lock:
            self.validators.append(validator)
            self.logger.info(f"Added validator: {type(validator).__name__}")
        return self
    
    def load_data(self, data: List[DataPoint]) -> 'MLPipeline':
        """Load data into the pipeline"""
        with self._lock:
            self.data_store.extend(data)
            self.logger.info(f"Loaded {len(data)} data points")
        return self
    
    async def preprocess_data(self, data: Optional[List[DataPoint]] = None) -> List[DataPoint]:
        """Apply all preprocessors to the data"""
        data_to_process = data or self.data_store
        
        if not data_to_process:
            raise InsufficientDataException(1, 0)
        
        processed_data = data_to_process
        
        for preprocessor in self.preprocessors:
            self.logger.info(f"Applying preprocessor: {preprocessor.name}")
            
            if not preprocessor.is_fitted:
                preprocessor.fit(processed_data)
            
            processed_data = preprocessor.transform(processed_data)
            
            # Simulate async processing
            await asyncio.sleep(0.01)
        
        return processed_data
    
    async def train_models(self, data: Optional[List[DataPoint]] = None) -> Dict[str, ModelMetrics]:
        """Train all models in the pipeline"""
        training_data = data or await self.preprocess_data()
        
        if not training_data:
            raise InsufficientDataException(1, 0)
        
        results = {}
        
        # Use concurrent training for independent models
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
            future_to_model = {}
            
            for model_id, model in self.models.items():
                future = executor.submit(asyncio.run, model.train(training_data))
                future_to_model[future] = model_id
            
            for future in concurrent.futures.as_completed(future_to_model):
                model_id = future_to_model[future]
                try:
                    future.result()  # This will raise any exceptions from training
                    model = self.models[model_id]
                    results[model_id] = model.metrics
                    self.logger.info(f"Successfully trained model: {model_id}")
                except Exception as e:
                    self.logger.error(f"Failed to train model {model_id}: {e}")
                    results[model_id] = ModelMetrics()
        
        return results
    
    async def evaluate_models(self, test_data: List[DataPoint]) -> Dict[str, ModelMetrics]:
        """Evaluate all trained models"""
        results = {}
        
        for model_id, model in self.models.items():
            if model.status == ModelStatus.TRAINED:
                try:
                    metrics = model.evaluate(test_data)
                    results[model_id] = metrics
                    self.logger.info(f"Evaluated model {model_id}")
                except Exception as e:
                    self.logger.error(f"Failed to evaluate model {model_id}: {e}")
                    results[model_id] = ModelMetrics()
            
            # Simulate async processing
            await asyncio.sleep(0.01)
        
        return results
    
    async def validate_models(self, validation_data: List[DataPoint]) -> Dict[str, bool]:
        """Validate all models using registered validators"""
        results = {}
        
        for model_id, model in self.models.items():
            if model.status == ModelStatus.TRAINED:
                model_results = []
                
                for validator in self.validators:
                    try:
                        is_valid = validator.validate(model, validation_data)
                        model_results.append(is_valid)
                        self.logger.info(f"Validated model {model_id} with {type(validator).__name__}: {is_valid}")
                    except Exception as e:
                        self.logger.error(f"Validation failed for model {model_id}: {e}")
                        model_results.append(False)
                
                # Model passes validation if all validators pass
                results[model_id] = all(model_results) if model_results else False
            else:
                results[model_id] = False
        
        return results
    
    async def run_full_pipeline(self, train_data: List[DataPoint], 
                               test_data: List[DataPoint]) -> Dict[str, Any]:
        """Run the complete ML pipeline"""
        pipeline_start = time.time()
        
        try:
            self.logger.info("Starting full ML pipeline execution")
            
            # Preprocess data
            processed_train_data = await self.preprocess_data(train_data)
            processed_test_data = await self.preprocess_data(test_data)
            
            # Train models
            training_results = await self.train_models(processed_train_data)
            
            # Evaluate models
            evaluation_results = await self.evaluate_models(processed_test_data)
            
            # Validate models
            validation_results = await self.validate_models(processed_test_data)
            
            # Determine best model
            best_model_id = self._select_best_model(evaluation_results, validation_results)
            
            pipeline_duration = time.time() - pipeline_start
            
            # Record execution
            execution_record = {
                'timestamp': datetime.now().isoformat(),
                'duration': pipeline_duration,
                'training_results': training_results,
                'evaluation_results': evaluation_results,
                'validation_results': validation_results,
                'best_model': best_model_id,
                'data_points_processed': len(processed_train_data) + len(processed_test_data)
            }
            
            with self._lock:
                self.execution_history.append(execution_record)
            
            self.logger.info(f"Pipeline completed in {pipeline_duration:.2f}s")
            
            return execution_record
            
        except Exception as e:
            self.logger.error(f"Pipeline execution failed: {e}")
            raise MLPipelineException(f"Pipeline execution failed: {e}") from e
    
    def _select_best_model(self, evaluation_results: Dict[str, ModelMetrics], 
                          validation_results: Dict[str, bool]) -> Optional[str]:
        """Select the best performing model"""
        valid_models = [model_id for model_id, is_valid in validation_results.items() if is_valid]
        
        if not valid_models:
            return None
        
        # For regression, use RMSE (lower is better)
        best_model_id = None
        best_score = float('inf')
        
        for model_id in valid_models:
            metrics = evaluation_results.get(model_id)
            if metrics and metrics.rmse is not None:
                if metrics.rmse < best_score:
                    best_score = metrics.rmse
                    best_model_id = model_id
        
        return best_model_id
    
    @contextmanager
    def model_context(self, model_id: str):
        """Context manager for safe model operations"""
        model = self.models.get(model_id)
        if not model:
            raise ValueError(f"Model {model_id} not found")
        
        try:
            yield model
        finally:
            model.last_updated = datetime.now()
    
    def get_pipeline_stats(self) -> Dict[str, Any]:
        """Get comprehensive pipeline statistics"""
        with self._lock:
            return {
                'pipeline_id': self.pipeline_id,
                'num_preprocessors': len(self.preprocessors),
                'num_models': len(self.models),
                'num_validators': len(self.validators),
                'data_points': len(self.data_store),
                'executions': len(self.execution_history),
                'models_by_status': {
                    status.value: len([m for m in self.models.values() if m.status == status])
                    for status in ModelStatus
                },
                'last_execution': self.execution_history[-1] if self.execution_history else None
            }
    
    # Decorators and utility functions
    @staticmethod
    def retry_on_failure(max_retries: int = 3, delay: float = 1.0):
        """Decorator for retrying failed operations"""
        def decorator(func):
            @wraps(func)
            async def wrapper(*args, **kwargs):
                last_exception = None
                for attempt in range(max_retries):
                    try:
                        if asyncio.iscoroutinefunction(func):
                            return await func(*args, **kwargs)
                        else:
                            return func(*args, **kwargs)
                    except Exception as e:
                        last_exception = e
                        if attempt < max_retries - 1:
                            await asyncio.sleep(delay * (2 ** attempt))
                        continue
                raise last_exception
            return wrapper
        return decorator
    
    @lru_cache(maxsize=128)
    def get_cached_prediction(self, model_id: str, feature_hash: str) -> Any:
        """Cached predictions for repeated inputs"""
        # In a real implementation, this would cache actual predictions
        return f"cached_prediction_{model_id}_{feature_hash}"


# Factory functions and utilities
@singledispatch
def create_data_point(data) -> DataPoint:
    """Create data point from various input types"""
    raise NotImplementedError(f"Cannot create DataPoint from {type(data)}")


@create_data_point.register
def _(data: dict) -> DataPoint:
    return DataPoint(
        id=data.get('id', str(uuid4())),
        features=data.get('features', {}),
        label=data.get('label'),
        metadata=data.get('metadata', {}),
        data_type=DataType(data.get('data_type', DataType.NUMERICAL))
    )


@create_data_point.register
def _(data: tuple) -> DataPoint:
    """Create from (features, label) tuple"""
    features, label = data
    return DataPoint(
        id=str(uuid4()),
        features=features if isinstance(features, dict) else {'feature_0': features},
        label=label
    )


def create_sample_data(n_samples: int = 100) -> List[DataPoint]:
    """Generate sample data for testing"""
    import random
    
    data_points = []
    for i in range(n_samples):
        # Generate synthetic regression data
        x1 = random.uniform(-10, 10)
        x2 = random.uniform(-5, 5)
        x3 = random.uniform(0, 20)
        
        # y = 2*x1 + 3*x2 - 0.5*x3 + noise
        noise = random.gauss(0, 0.5)
        y = 2 * x1 + 3 * x2 - 0.5 * x3 + noise
        
        data_point = DataPoint(
            id=f"sample_{i}",
            features={'x1': x1, 'x2': x2, 'x3': x3},
            label=y,
            metadata={'source': 'synthetic', 'batch': i // 20}
        )
        data_points.append(data_point)
    
    return data_points


# Example usage and demonstration
async def main():
    """Demonstrate the ML pipeline"""
    print("Creating ML Pipeline...")
    
    # Create pipeline
    pipeline = MLPipeline("demo_pipeline")
    
    # Add components
    pipeline.add_preprocessor(NumericPreprocessor())
    
    # Create model
    config = TrainingConfig(
        model_type="linear_regression",
        learning_rate=0.01,
        epochs=100,
        early_stopping_patience=10
    )
    model = LinearRegressionModel("regression_model_1", config)
    pipeline.add_model(model)
    
    # Add validator
    pipeline.add_validator(CrossValidator(k_folds=3, min_accuracy=0.5))
    
    # Generate sample data
    all_data = create_sample_data(200)
    train_data = all_data[:150]
    test_data = all_data[150:]
    
    # Run pipeline
    try:
        results = await pipeline.run_full_pipeline(train_data, test_data)
        print(f"Pipeline completed successfully!")
        print(f"Best model: {results['best_model']}")
        print(f"Duration: {results['duration']:.2f}s")
        
        # Get pipeline stats
        stats = pipeline.get_pipeline_stats()
        print(f"Pipeline stats: {stats}")
        
    except Exception as e:
        print(f"Pipeline failed: {e}")


# Advanced features demonstration
class ModelEnsemble:
    """Ensemble of multiple models for better predictions"""
    
    def __init__(self, models: List[BaseModel]):
        self.models = models
        self.weights = [1.0 / len(models)] * len(models)  # Equal weights initially
    
    def predict(self, data: List[DataPoint]) -> List[float]:
        """Make ensemble predictions"""
        all_predictions = []
        
        for model in self.models:
            if model.status == ModelStatus.TRAINED:
                predictions = model.predict(data)
                all_predictions.append(predictions)
        
        if not all_predictions:
            raise ModelNotTrainedException("No trained models in ensemble")
        
        # Weighted average of predictions
        ensemble_predictions = []
        for i in range(len(data)):
            weighted_sum = sum(predictions[i] * weight 
                             for predictions, weight in zip(all_predictions, self.weights))
            ensemble_predictions.append(weighted_sum)
        
        return ensemble_predictions


class AutoML:
    """Automated machine learning pipeline"""
    
    def __init__(self):
        self.best_pipeline = None
        self.trial_results = []
    
    async def auto_train(self, data: List[DataPoint], 
                        model_types: List[str] = None) -> MLPipeline:
        """Automatically find the best model configuration"""
        model_types = model_types or ["linear_regression"]
        
        best_score = float('inf')
        best_pipeline = None
        
        for model_type in model_types:
            for lr in [0.001, 0.01, 0.1]:
                for epochs in [50, 100, 200]:
                    config = TrainingConfig(
                        model_type=model_type,
                        learning_rate=lr,
                        epochs=epochs
                    )
                    
                    # Create and run pipeline
                    pipeline = MLPipeline(f"auto_{model_type}_{lr}_{epochs}")
                    pipeline.add_preprocessor(NumericPreprocessor())
                    
                    model = LinearRegressionModel(f"auto_model_{lr}_{epochs}", config)
                    pipeline.add_model(model)
                    pipeline.add_validator(CrossValidator())
                    
                    # Split data
                    split_idx = int(0.8 * len(data))
                    train_split = data[:split_idx]
                    test_split = data[split_idx:]
                    
                    try:
                        results = await pipeline.run_full_pipeline(train_split, test_split)
                        
                        # Get performance metric
                        if results['best_model']:
                            eval_results = results['evaluation_results']
                            model_metrics = eval_results[results['best_model']]
                            score = model_metrics.rmse or float('inf')
                            
                            if score < best_score:
                                best_score = score
                                best_pipeline = pipeline
                        
                        self.trial_results.append({
                            'config': config,
                            'score': score if 'score' in locals() else float('inf'),
                            'results': results
                        })
                        
                    except Exception as e:
                        print(f"Trial failed: {e}")
                        continue
        
        self.best_pipeline = best_pipeline
        return best_pipeline


if __name__ == "__main__":
    # Run the demonstration
    asyncio.run(main())