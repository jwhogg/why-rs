use linfa::DatasetBase;
use polars::prelude::{DataFrame, DataType, Series};
use crate::dag::{Value, Variable};
use linfa::traits::{Fit, Predict};
use linfa_linear::{FittedLinearRegression, LinearRegression as LR};
use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use rand_distr::{Normal, Distribution};

pub type Function = Box<dyn FnMut(&[Value]) -> Value>;

pub trait Mechanism {
    fn predict(&self, inputs: Vec<Value>) -> Value;
    fn fit(&mut self, df: DataFrame, variable: Variable); //needs to know which variable it is
}

#[derive(Debug, Clone)]
pub struct LinearRegression {
    weights: Option<Array1<f64>>,  // predefined or learned
    bias: f64,                      // predefined bias
    noise: f64,                     // standard deviation of noise
    model: Option<FittedLinearRegression<f64>>,         // actual trained model
}

impl LinearRegression {
    // Constructor with predefined coefficients
    pub fn from<T: Into<f64>>(weights: Vec<f64>, bias: T, noise: T) -> Self {
        Self {
            weights: Some(Array1::from(weights)),
            bias: bias.into(),
            noise: noise.into(),
            model: None,
        }
    }

    // Constructor for an empty model to fit later
    pub fn new() -> Self {
        Self {
            weights: None,
            bias: 0.0,
            noise: 0.0,
            model: None,
        }
    }

    pub fn fit(&mut self, x: Array2<f64>, y: &Array1<f64>) {
        let db = DatasetBase::new(x, y.to_owned());
        let model = LR::default().fit(&db).unwrap();//bad error handling
        self.weights = Some(model.params().clone());
        self.model = Some(model);

    }

    pub fn predict(&self, x: &Array2<f64>) -> Array1<f64> {
        // Case 1: use trained linfa model
        if let Some(model) = &self.model {
            return model.predict(x);
        }

        // Case 2: manual linear regression
        let weights = self
            .weights
            .as_ref()
            .expect("No trained model and no weights provided");

        assert_eq!(
            x.ncols(),
            weights.len(),
            "Feature count mismatch between X and weights"
        );

        let mut rng = rand::thread_rng();
        let noise_dist = Normal::new(0.0, self.noise).unwrap();

        let mut y = Array1::<f64>::zeros(x.nrows());

        for (i, row) in x.outer_iter().enumerate() {
            let mut value = row.dot(weights) + self.bias;

            // Add noise (self.noise == 0.0 ⇒ deterministic)
            if self.noise > 0.0 {
                value += noise_dist.sample(&mut rng);
            }

            y[i] = value;
        }

        y
    }
}

impl Mechanism for LinearRegression {
    fn predict(&self, parents: Vec<Value>) -> Value {
        let parents = Array1::from(parents);
        let parents_as_2d = parents.insert_axis(Axis(0));
        self.predict(&parents_as_2d)[0]
    }

    fn fit(&mut self, mut df: DataFrame, variable: Variable) {
        //find out which variable it is from the dataset
        let target_series: &Series = df.column(&variable)
                                .expect("Error indexing variable name in provided df!")
                                .as_series()
                                .unwrap();

        //convert target series to a 1d ndarray
        let y: Array1<f64> = match target_series.dtype() {
            DataType::Float64 => target_series.f64().unwrap().into_no_null_iter().collect(),
            DataType::Float32 => target_series.f32().unwrap().into_no_null_iter().map(|v| v as f64).collect(),
            _ => panic!("Target column must be numeric"),
        };

        df.drop_in_place(&variable).expect("Error indexing variable name to drop from provided df!");

        //Convert df to 2D ndarray
        let n_rows = df.height();
        let n_cols = df.width();
        let mut x_vec = Vec::with_capacity(n_rows * n_cols);

        for col in df.get_columns() {
            match col.dtype() {
                DataType::Float64 => x_vec.extend(col.f64().unwrap().into_no_null_iter()),
                DataType::Float32 => x_vec.extend(col.f32().unwrap().into_no_null_iter().map(|v| v as f64)),
                _ => panic!("All feature columns must be numeric"),
            }
        }

        // Create Array2 in row-major order
        let x = Array2::from_shape_vec((n_rows, n_cols), x_vec).expect("Failed to create Array2");

        self.fit(x, &y);
    }
}

pub struct EmpiricalRoot { //randomly choose a value from the history of values for that variable
    history: Vec<Value>,
}

impl EmpiricalRoot {
    pub fn new(history: Vec<Value>) -> Self { EmpiricalRoot { history } }
}

impl Mechanism for EmpiricalRoot {
    fn predict(&self, _: Vec<Value>) -> Value {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.history.len());
        self.history[index]
    }

    fn fit(&mut self, df: DataFrame, variable: Variable) {
        todo!()
    }
}