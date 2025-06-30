use std::array::from_fn;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign};
use rand::distr::{Distribution, StandardUniform};
use rand::Rng;
use crate::game::ai::coefficient::Coefficient;
use crate::game::ai::genome::Genome;

#[derive(Copy, Clone, PartialEq)]
pub struct Tensor<const R: usize, const C: usize = 1> {
    data: [[f64; C]; R],
}

impl<const R: usize, const C: usize> Tensor<R, C> {

    pub fn rows(&self) -> usize {
        R
    }

    pub fn cols(&self) -> usize {
        C
    }


    const ZEROS: Self = Self {
        data: [[0.0; C]; R],
    };

    const ONES: Self = Self {
        data: [[1.0; C]; R],
    };

    pub fn new(data: [[f64; C]; R]) -> Self {
        Self { data }
    }

    pub const TOTAL_SIZE: usize = R * C;

    pub fn from_slice(data: &[f64]) -> Self {
        debug_assert_eq!(data.len(), Self::TOTAL_SIZE,
           "Invalid data length for Tensor<{}, {}>: expected {}, got {}",
           R, C, Self::TOTAL_SIZE, data.len()
        );
        let mut result = Self::ZEROS;
        for i in 0..R {
            for j in 0..C {
                result.data[i][j] = data[i * C + j];
            }
        }
        result
    }

    pub fn flatten(&self) -> Vec<f64> {
        let mut result = Vec::with_capacity(Self::TOTAL_SIZE);
        for row in self.data.iter() {
            for col in row.iter() {
                result.push(*col);
            }
        }
        result
    }
    
    pub fn dot<const R2: usize, const C2: usize>(&self, other: &Tensor<R2, C2>) -> Tensor<R, C2> {
        debug_assert_eq!(C, R2, "Cannot multiply tensors with incompatible dimensions");

        let mut result = Tensor::ZEROS;

        for i in 0..self.rows() {
            for j in 0..other.cols() {
                for k in 0..self.cols() {
                    result.data[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }

        result
    }
    fn relu_mut(&mut self) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self.data[i][j] = relu(self.data[i][j]);
            }
        }
    }

    fn sigmoid_mut(&mut self) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self.data[i][j] = sigmoid(self.data[i][j]);
            }
        }
    }

    fn mcculloch_pitts_mut(&mut self, threshold: f64) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self.data[i][j] = mcculloch_pitts(self.data[i][j], threshold);
            }
        }
    }

    fn fmt(&self, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result {
        let mut formatted_nums = Vec::with_capacity(R * C);
        let mut col_widths = vec![0; C];
        for row in self.data.iter() {
            for (col_idx, val) in row.iter().enumerate() {
                let formatted = format!("{:.6}", val);
                col_widths[col_idx] = col_widths[col_idx].max(formatted.len());
                formatted_nums.push(formatted);
            }
        }

        let indent_str = " ".repeat(indent);
        for i in 0..R {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}[", indent_str)?;
            for j in 0..C {
                if j > 0 {
                    write!(f, " ")?;
                }
                let num = &formatted_nums[i * C + j];
                write!(f, "{:>width$}", num, width = col_widths[j])?;
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}


fn relu(x: f64) -> f64 {
    x.max(0.0)
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn mcculloch_pitts(x: f64, threshold: f64) -> f64 {
    if x > threshold { 1.0 } else { 0.0 }
}

fn activate(x: f64, activation: ActivationFunction) -> f64 {
    match activation {
        ActivationFunction::Identity => x,
        ActivationFunction::ReLU => relu(x),
        ActivationFunction::Sigmoid => sigmoid(x),
        ActivationFunction::McCullochPitt(threshold) => mcculloch_pitts(x, threshold),
    }
}

impl<const SIZE: usize> Tensor<SIZE> {
    pub fn vector(data: [f64; SIZE]) -> Self {
        let mut result = Self::ZEROS;
        for i in 0..SIZE {
            result.data[i][0] = data[i]
        }
        result
    }

    pub fn into_diagonal(self) -> Tensor<SIZE, SIZE> {
        let mut result = Tensor::ZEROS;
        for i in 0..SIZE {
            result.data[i][i] = self.data[i][0]
        }
        result
    }

    fn activate_mut(&mut self, activation: [ActivationFunction; SIZE]) {
        for i in 0..SIZE {
            self.data[i][0] = activate(self.data[i][0], activation[i])
        }
    }
}

impl<const SIZE: usize> Tensor<SIZE, SIZE> {
    pub fn diagonal(data: [f64; SIZE]) -> Self {
        let mut result = Self::ZEROS;
        for i in 0..SIZE {
            result.data[i][i] = data[i]
        }
        result
    }
}

impl Tensor<1, 1> {
    pub fn value(&self) -> f64 {
        self.data[0][0]
    }
}

impl<const R: usize, const C: usize> Add for Tensor<R, C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Self::ZEROS;
        for i in 0..R {
            for j in 0..C {
                result.data[i][j] = self.data[i][j] + rhs.data[i][j];
            }
        }
        result
    }
}

impl<const R: usize, const C: usize> AddAssign for Tensor<R, C> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..R {
            for j in 0..C {
                self.data[i][j] += rhs.data[i][j];
            }
        }
    }
}


impl<const R: usize, const C: usize> Display for Tensor<R, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}

impl<const R: usize, const C: usize> Debug for Tensor<R, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}


impl<const R: usize, const C: usize> Default for Tensor<R, C> {
    fn default() -> Self {
        Self::ZEROS
    }
}

impl<const R: usize, const C: usize> Distribution<Tensor<R, C>> for StandardUniform {
    fn sample<RNG: Rng + ?Sized>(&self, rng: &mut RNG) -> Tensor<R, C> {
        let mut result = Tensor::ZEROS;
        for i in 0..R {
            for j in 0..C {
                result.data[i][j] = rng.random_range(0.0 ..= 1.0);
            }
        }
        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActivationFunction {
    Identity,
    Sigmoid,
    ReLU,
    McCullochPitt(f64)
}

impl Default for ActivationFunction {
    fn default() -> Self {
        ActivationFunction::Sigmoid
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Layer<const IN: usize, const SIZE: usize> {
    weights: Tensor<SIZE, IN>,
    bias: Tensor<SIZE>,
    activation: [ActivationFunction; SIZE],
}

impl<const IN: usize, const SIZE: usize> Display for Layer<IN, SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Layer<{}, {}>:", IN, SIZE)?;
        writeln!(f, "  Weights:")?;
        self.weights.fmt(f, 4)?;
        writeln!(f, "\n  Bias:")?;
        self.bias.fmt(f, 4)?;
        write!(f, "\n  Activations: {:?}", self.activation)
    }
}


impl<const IN: usize, const SIZE: usize> Layer<IN, SIZE> {
    pub fn new(weights: Tensor<SIZE, IN>, bias: Tensor<SIZE>, activation: [ActivationFunction; SIZE]) -> Self {
        Self { weights, bias, activation }
    }

    pub fn fully_connected(weights: Tensor<SIZE, IN>, bias: Tensor<SIZE>, activation: ActivationFunction) -> Self {
        Self::new(weights, bias, [activation; SIZE])
    }

    pub fn mcculloch_pitt(weights: Tensor<SIZE, IN>, thresholds: [f64; SIZE]) -> Self {
        Self::new(weights, Tensor::ZEROS, thresholds.map(ActivationFunction::McCullochPitt))
    }

    pub fn set_activation(&mut self, activation: ActivationFunction) {
        self.activation = [activation; SIZE]
    }

    const WEIGHTS_SIZE: usize = IN * SIZE;
    pub const TOTAL_SIZE: usize = Self::WEIGHTS_SIZE + SIZE; // weights + biases

    pub fn flatten(&self) -> Vec<f64> {
        let mut result = Vec::with_capacity(Self::TOTAL_SIZE);
        result.extend(self.weights.flatten());
        result.extend(self.bias.flatten());
        debug_assert_eq!(result.len(), Self::TOTAL_SIZE, "Layer flattened size mismatch");
        result
    }

    pub fn from_slice(data: &[f64]) -> Self {
        debug_assert_eq!(data.len(), Self::TOTAL_SIZE,
           "Invalid data length for Layer<{}, {}>: expected {}, got {}",
           IN, SIZE, Self::TOTAL_SIZE, data.len()
        );
        Self {
            // First WEIGHTS_SIZE elements are weights
            weights: Tensor::from_slice(&data[..Self::WEIGHTS_SIZE]),
            // Remaining SIZE elements are biases
            bias: Tensor::from_slice(&data[Self::WEIGHTS_SIZE..]),
            // Use default activation function
            activation: [Default::default(); SIZE]
        }
    }

    fn forward(&self, input: &Tensor<IN>) -> Tensor<SIZE> {
        // Perform forward propagation: output = (weights · input) + bias
        let mut result = self.weights.dot(input);
        result += self.bias;
        result.activate_mut(self.activation);
        result
    }

    pub fn backward(&self,
                    input: &Tensor<IN>,
                    output: &Tensor<SIZE>,
                    upstream_gradient: &Tensor<SIZE>
    ) -> (Tensor<SIZE, IN>, Tensor<SIZE>, Tensor<IN>) {
        // First apply activation function derivative
        let mut activation_gradient = *upstream_gradient;
        for i in 0..SIZE {
            activation_gradient.data[i][0] *= match self.activation[i] {
                ActivationFunction::Identity => 1.0,
                ActivationFunction::ReLU => if output.data[i][0] > 0.0 { 1.0 } else { 0.0 },
                ActivationFunction::Sigmoid => {
                    let s = output.data[i][0];
                    s * (1.0 - s) // derivative of sigmoid
                },
                ActivationFunction::McCullochPitt(_) => 0.0, // Not differentiable, treated as 0
            };
        }


        // Calculate gradients
        // dL/dW = dL/dY * X^T
        let mut weight_gradient = Tensor::ZEROS;
        for i in 0..SIZE {
            for j in 0..IN {
                weight_gradient.data[i][j] = activation_gradient.data[i][0] * input.data[j][0];
            }
        }

        // dL/db = dL/dY
        let bias_gradient = activation_gradient;

        // dL/dX = W^T * dL/dY
        let mut input_gradient = Tensor::ZEROS;
        for i in 0..IN {
            for j in 0..SIZE {
                input_gradient.data[i][0] += self.weights.data[j][i] * activation_gradient.data[j][0];
            }
        }

        // TODO type this
        (weight_gradient, bias_gradient, input_gradient)
    }

    pub fn update(&mut self, weight_gradient: &Tensor<SIZE, IN>, bias_gradient: &Tensor<SIZE>, learning_rate: f64) {
        // Update weights: W = W - learning_rate * dL/dW
        for i in 0..SIZE {
            for j in 0..IN {
                self.weights.data[i][j] -= learning_rate * weight_gradient.data[i][j];
            }
        }

        // Update biases: b = b - learning_rate * dL/db
        for i in 0..SIZE {
            self.bias.data[i][0] -= learning_rate * bias_gradient.data[i][0];
        }
    }


}

impl<const IN: usize, const SIZE: usize> Distribution<Layer<IN, SIZE>> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Layer<IN, SIZE> {
        let scale = (2.0 / (IN + SIZE) as f64).sqrt();
        let mut weights = Tensor::ZEROS;
        let mut bias = Tensor::ZEROS;

        // Xavier/Glorot initialization
        for i in 0..SIZE {
            for j in 0..IN {
                weights.data[i][j] = (rng.random::<f64>() * 2.0 - 1.0) * scale;
            }
            bias.data[i][0] = (rng.random::<f64>() * 2.0 - 1.0) * 0.1;
        }
        Layer { weights, bias, activation: [Default::default(); SIZE] }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NeuralNetwork<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> {
    input: Layer<IN, WIDTH>,
    hidden: [Layer<WIDTH, WIDTH>; HIDDEN],
    output: Layer<WIDTH, OUT>,
}

impl<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> Display for NeuralNetwork<IN, HIDDEN, OUT, WIDTH> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "NeuralNetwork<{}, {}, {}, {}>", IN, OUT, WIDTH, HIDDEN)?;
        writeln!(f, "Input {}", self.input)?;

        for (i, layer) in self.hidden.iter().enumerate() {
            writeln!(f, "Hidden[{}] {}", i + 1, layer)?;
        }

        write!(f, "Output {}", self.output)
    }
}

impl<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> NeuralNetwork<IN, HIDDEN, OUT, WIDTH> {
    const INPUT_LAYER_SIZE: usize = Layer::<IN, WIDTH>::TOTAL_SIZE;
    const HIDDEN_LAYER_SIZE: usize = Layer::<WIDTH, WIDTH>::TOTAL_SIZE;
    const OUTPUT_LAYER_SIZE: usize = Layer::<WIDTH, OUT>::TOTAL_SIZE;
    pub const TOTAL_SIZE: usize = Self::INPUT_LAYER_SIZE + HIDDEN * Self::HIDDEN_LAYER_SIZE + Self::OUTPUT_LAYER_SIZE;

    pub fn flatten(&self) -> Vec<f64> {
        let mut result = Vec::with_capacity(Self::TOTAL_SIZE);

        // Flatten input layer
        result.extend(self.input.flatten());

        // Flatten hidden layers
        for layer in self.hidden.iter() {
            result.extend(layer.flatten());
        }

        // Flatten output layer
        result.extend(self.output.flatten());

        debug_assert_eq!(result.len(), Self::TOTAL_SIZE, "Network flattened size mismatch");
        result
    }

    pub fn from_slice(data: &[f64]) -> Self {
        debug_assert_eq!(data.len(), Self::TOTAL_SIZE,
             "Invalid data length for NeuralNetwork<{}, {}, {}, {}>: expected {}, got {}",
             IN, HIDDEN, OUT, WIDTH, Self::TOTAL_SIZE, data.len()
        );

        let mut offset = 0;

        // Create input layer
        let input = Layer::from_slice(&data[offset..offset + Self::INPUT_LAYER_SIZE]);
        offset += Self::INPUT_LAYER_SIZE;

        // Create hidden layers
        let mut hidden = Vec::with_capacity(HIDDEN);
        for _ in 0..HIDDEN {
            hidden.push(Layer::from_slice(&data[offset..offset + Self::HIDDEN_LAYER_SIZE]));
            offset += Self::HIDDEN_LAYER_SIZE;
        }
        let hidden = hidden.try_into().unwrap();

        // Create output layer
        let output = Layer::from_slice(&data[offset..offset + Self::OUTPUT_LAYER_SIZE]);

        Self { input, hidden, output }
    }


    pub fn set_input_activation(&mut self, activation: ActivationFunction) {
        self.input.set_activation(activation)
    }

    pub fn set_hidden_activation(&mut self, activation: ActivationFunction) {
        for layer in self.hidden.iter_mut() {
            layer.set_activation(activation);
        }
    }

    pub fn set_output_activation(&mut self, activation: ActivationFunction) {
        self.output.set_activation(activation)
    }

    pub fn set_activation(&mut self, activation: ActivationFunction) {
        self.set_input_activation(activation);
        self.set_hidden_activation(activation);
    }

    pub fn set_default_activation(&mut self) {
        self.set_activation(ActivationFunction::Sigmoid);
        self.set_output_activation(ActivationFunction::Identity);
    }

    pub fn forward(&self, input: &Tensor<IN>) -> Tensor<OUT> {
        let mut current = self.input.forward(input);
        for layer in self.hidden.iter() {
            current = layer.forward(&current);
        }
        self.output.forward(&current)
    }

    pub fn train_step(&mut self, input: &Tensor<IN>, target: &Tensor<OUT>, learning_rate: f64) -> f64 {
        // Store activations during forward pass
        let mut hidden_activations = Vec::with_capacity(HIDDEN);
        let mut hidden_outputs = Vec::with_capacity(HIDDEN);

        // Forward pass

        // input layer
        let initial_activation = *input;
        let mut current = self.input.forward(input);
        let initial_output = current;

        // hidden layers
        for layer in self.hidden.iter() {
            hidden_activations.push(current);
            current = layer.forward(&current);
            hidden_outputs.push(current);
        }

        // output layer
        let final_activation = current;
        let final_output = self.output.forward(&current);

        // Calculate loss and initial gradient
        let mut loss = 0.0;
        let mut output_gradient = Tensor::ZEROS;
        for i in 0..OUT {
            let diff = final_output.data[i][0] - target.data[i][0];
            loss += 0.5 * diff * diff; // MSE loss
            output_gradient.data[i][0] = diff; // derivative of MSE
        }

        // Backward pass
        let (w_grad, b_grad, mut upstream_grad) = self.output.backward(
            &final_activation,
            &final_output,
            &output_gradient
        );
        self.output.update(&w_grad, &b_grad, learning_rate);

        // Backpropagate through hidden layers
        for i in (0..HIDDEN).rev() {
            let (w_grad, b_grad, grad) = self.hidden[i].backward(
                &hidden_activations[i],
                &hidden_outputs[i],
                &upstream_grad
            );
            self.hidden[i].update(&w_grad, &b_grad, learning_rate);
            upstream_grad = grad;
        }

        // Input layer
        let (w_grad, b_grad, _) = self.input.backward(
            &initial_activation,
            &initial_output,
            &upstream_grad
        );
        self.input.update(&w_grad, &b_grad, learning_rate);

        loss
    }

    pub fn train(&mut self,
                 inputs: &[Tensor<IN>],
                 targets: &[Tensor<OUT>],
                 epochs: usize,
                 learning_rate: f64
    ) -> Vec<f64> {
        assert_eq!(inputs.len(), targets.len(), "Number of inputs and targets must match");
        let mut losses = Vec::with_capacity(epochs);

        for _ in 0..epochs {
            let mut epoch_loss = 0.0;

            for (input, target) in inputs.iter().zip(targets.iter()) {
                epoch_loss += self.train_step(input, target, learning_rate);
            }

            epoch_loss /= inputs.len() as f64;
            losses.push(epoch_loss);
        }

        losses
    }

}

impl<const IN: usize, const HIDDEN: usize, const OUT: usize, const WIDTH: usize> Distribution<NeuralNetwork<IN, HIDDEN, OUT, WIDTH>> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NeuralNetwork<IN, HIDDEN, OUT, WIDTH> {
        let mut network = NeuralNetwork {
            input: rng.random(),
            hidden: from_fn(|_| rng.random()),
            output: rng.random(),
        };
        network.set_default_activation();
        network
    }
}

pub type TetrisNeuralNetwork = NeuralNetwork<14, 2, 1, 14>;

pub const NEURAL_GENOME_SIZE: usize = TetrisNeuralNetwork::TOTAL_SIZE;
pub type NeuralGenome = Genome<NEURAL_GENOME_SIZE>;

impl Into<NeuralGenome> for TetrisNeuralNetwork {
    fn into(self) -> NeuralGenome {
        let array: [f64; NEURAL_GENOME_SIZE] = self.flatten().try_into().unwrap();
        array.into()
    }
}

impl From<NeuralGenome> for TetrisNeuralNetwork {
    fn from(genome: NeuralGenome) -> Self {
        Self::from_slice(&genome.chromosome().map(Coefficient::into_f64))
    }
}

impl Default for TetrisNeuralNetwork {
    fn default() -> Self {
        let weights = [-0.573496, -0.397385, -0.125694, -0.578120, 0.529914, 0.651939, 0.067630, 0.459326, 0.321204, 0.030117, 0.808952, 1.199330, 0.666438, 0.400337, 0.025976, 1.022147, 0.145951, 0.938055, -0.882071, -0.060782, -0.074916, 0.500057, 0.851837, 0.563589, 0.225444, -0.007735, 0.981288, -0.280504, 1.241916, -0.297652, 0.480647, 0.344778, 0.643780, 0.258160, 0.721047, 0.067764, -0.229536, -0.433096, 0.312117, -0.189486, 0.134483, 0.243515, 0.787046, -0.182634, -0.740265, 0.357388, -0.850713, -0.352260, -0.313771, 0.030578, 0.418359, -0.021581, -0.474711, 0.118136, 0.974727, 0.820125, 0.996931, 0.162040, 0.180819, 0.185826, 0.469017, -0.333567, -0.519101, 0.578333, 0.685596, 0.467839, 0.426612, 0.594481, -0.450611, 0.468214, 0.197505, -0.479479, 0.290821, 0.511132, 0.194233, 0.047656, 0.662832, 0.994508, 0.545407, 0.571185, 0.351535, -0.313564, 0.790643, 0.604548, 0.462424, 0.005265, -0.440009, 0.570571, 0.953712, -0.203545, 0.219699, 0.576354, 1.030751, 0.948164, -0.463304, -0.065227, 0.813119, 1.216707, 0.413285, 0.283367, -0.775769, -0.018677, -0.434824, -0.969838, 0.080575, 1.087581, 0.429615, -0.759285, -0.225167, 0.105063, 0.267991, 0.755934, 0.555224, 0.986713, 0.511353, 0.168190, 0.630711, -0.802799, 0.601674, 0.319665, 0.635904, -0.161475, -0.054914, 0.834434, -0.580332, 0.002352, 0.148657, -0.009915, -0.887424, 0.423063, 0.289617, 0.224700, -0.637871, 0.694859, 0.890746, 0.165355, 0.898045, 0.679337, -0.246549, 0.919187, 0.656059, 0.644526, 0.729578, -1.099081, -0.179080, 0.441735, 0.342221, 0.244856, 0.576126, 0.033211, -0.792453, 0.062588, 0.332228, 0.764084, 0.724749, 0.180800, 0.165417, 0.533110, -0.105739, 0.302243, 0.854081, 0.571360, -0.312798, 0.661863, 0.884231, 0.996556, 0.994363, -0.369681, -0.175269, -0.774563, 0.864201, 0.269476, 0.376435, 0.232095, 0.292526, 0.641543, 0.673069, -0.574519, 0.648352, 0.502073, 0.433956, 0.963458, 0.502847, -0.600045, 0.528203, 0.524007, -0.037964, 0.608262, 0.301130, 0.772954, 0.577165, -0.717260, -0.582160, 0.611896, -0.342353, 0.627415, -0.071409, 0.444412, 0.652168, 0.586706, -0.917648, 0.639409, -0.498833, 0.435439, 0.618190, 0.585450, 0.606821, -0.510800, -0.742434, 0.760447, -1.020874, -0.891879, -0.542986, 0.403694, 0.243224, -0.534299, -0.810722, 0.430763, -0.550600, -0.932669, 0.444631, 1.375506, -0.031953, 0.538202, -0.834493, -0.090234, -0.558999, 0.224502, 0.396911, 0.046677, -0.749290, -0.411403, -0.221069, -0.844092, 0.924038, -0.994089, 0.404233, 0.100506, -0.482884, -0.784374, 0.256183, 0.406465, 0.262712, -0.168213, 0.167664, -0.220458, 0.682936, -0.061982, -1.013839, 0.714286, -1.108632, -0.175853, -0.534337, 0.012326, -0.687209, 0.326917, -0.479734, 0.399171, -0.740648, -0.955869, 0.222728, -0.666484, -1.116715, 0.605602, -0.850374, 0.041822, 0.112724, -0.504994, -0.182150, 0.452391, 0.748713, 0.917131, -0.509266, -0.222869, 0.999219, -0.400490, 0.313445, 0.139765, 0.065319, 0.120191, 0.101454, 0.956555, -0.413433, 0.590297, 0.444842, -0.688677, 0.025373, -0.308205, 0.477668, 0.835615, -0.777824, 0.733136, 0.513803, 0.109486, 0.986309, 0.618222, 1.101976, -0.023147, -0.281888, -0.048623, 0.026529, -0.515352, 0.458256, -0.649918, -0.264320, -0.133694, 0.189606, 0.575846, -0.568982, -0.256196, 0.214352, 0.058854, -0.533808, 0.564057, -0.061143, -0.612078, 0.249678, -0.333314, -0.047233, -0.082773, 0.753523, 0.608199, 0.248964, 0.908776, -1.274026, 0.029460, -0.566601, 0.775109, 1.086082, -0.461751, -1.068113, -0.222197, 0.214468, 0.079635, -0.406168, -0.793091, 1.010816, 0.788299, 0.081032, -0.897617, 0.721205, -0.840994, -0.778295, 0.748101, -0.054547, -0.214717, -0.510210, 0.161265, -0.001429, -0.283815, -0.030242, 0.495749, -0.861531, 0.152362, -0.515696, -0.408709, -0.806399, 0.608003, 0.583570, -0.621298, 0.732073, -0.930444, -0.907544, -0.645834, -0.610068, -0.823172, -0.258096, 0.779411, -0.842675, 0.431282, 0.068303, -0.036018, 0.198298, 0.281544, -0.108822, 0.633012, -0.914975, -0.100847, -0.399534, -0.438440, 0.993671, 0.344555, 0.792983, 0.900146, 0.541669, 0.004140, -0.558925, -0.024502, -0.406527, 0.241072, 0.238221, -0.388227, 0.026979, -0.332542, -0.790145, -0.214443, -0.239419, 0.666505, -0.531268, -0.190419, -0.895654, -0.740467, 0.269799, 0.463871, 0.229404, 0.311364, -0.058684, -1.173483, 0.904021, 0.464088, 0.083971, 0.227500, 0.104917, -0.415722, 0.341896, 0.435586, 0.846461, 0.653169, -0.352454, -0.628608, 0.673301, 0.889873, -0.578189, 0.892923, 0.219330, 0.635818, 0.919184, 0.718558, 0.392697, -1.009412, 0.092583, -0.059465, 1.228924, 0.434582, 0.379925, 0.231279, 0.586928, 0.272719, -0.535962, 0.460174, -0.080890, -0.614235, -0.636396, -0.004061, 0.694421, -0.883968, 0.301575, -0.345150, 1.033992, 0.063037, 0.322276, 0.068291, 0.156623, -0.227074, -0.573592, -0.059187, 0.945411, 0.112732, -0.743155, -0.404737, 0.431632, 0.237990, 0.423007, -0.610882, -0.930840, 0.283663, -0.627034, -0.811520, 0.441715, 0.155000, -0.399546, -0.452698, 0.882870, -0.562274, -0.754724, -0.417450, 0.549599, 0.749976, -0.024914, 0.169300, -0.049538, 0.545418, 0.018073, -0.435847, 0.641273, -0.889888, 0.380741, 0.407971, 0.513607, 0.410894, -0.983899, 0.112542, -0.407492, 0.273437, 0.427834, 1.011792, 0.023888, -0.344443, 0.846997, -0.160696, -0.181001, -0.833531, 0.743399, -0.464432, -0.451740, 0.037094, 0.362486, 0.920646, -0.479757, 0.569471, 0.457449, 0.415851, 0.549766, 0.692348, 0.290963, 0.691648, -0.587043, 0.366054, -0.132577, 0.744003, 0.650505, 0.539819, -0.396233, -0.706160, 0.164246, -0.080837, -0.087412, 0.795942, 0.981461, -0.717166, -0.799893, -0.491582, -0.360970, 0.382206, 0.300535, 0.849009, 0.094594, -0.750707, -0.712306, 0.132871, 0.969480, -1.004964, 0.762189, -0.835004, -0.376563, -0.612476, 0.068316, 0.144712, 0.806111, 0.437636, -0.586821, 0.157653, 0.131392, 0.418306, 0.736240, -0.934611, 0.850350, 0.487894, 0.567641, 0.139110, 0.431381, -0.033697, 0.730916, -0.627010, -0.051924, 0.823444, -0.586361, 0.587541, -0.432121, 0.292068, -0.142785, 0.244390, 0.568108, 0.404363, 0.746368, -0.167819, 0.858227, 0.156505, 0.914770, -0.374297, -0.101495, -0.387064, 1.293997, -0.433538, -0.601663, -0.109562, -0.203688, 0.855053, 0.703778, -0.944275, -0.979159, -0.618606, -0.765221, -0.382291, 0.446595, -0.311882, 0.954013, 0.974401, 0.091883, 0.350613, -0.245828, -1.043798, 0.639231, -0.250286, -0.092590, -0.726716, -0.274778, -0.633970, 0.895483, -0.875700, 0.648405, 0.726026, -0.612290, -0.894031, 0.039203, 0.070386, -0.700805, -0.294895, 0.715106, -0.132778, -0.986759, -0.832652, 0.102500, -0.473752, -0.728587, -0.131571, 0.310271, 0.767580, -0.630190, 0.480153, -0.374058, 1.178690, 0.386878, -0.021964, -0.003422, 0.354621, 1.213435, -0.433406, 0.680911, -0.202533, -0.186906, -0.310082, 0.705797, -0.061854, -0.084191];
        let mut network = TetrisNeuralNetwork::from_slice(&weights);
        network.set_default_activation();
        network
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;
    use super::*;

    #[test]
    fn flatten_tensor() {
        let tensor = Tensor::new([[1., 2., 3.], [4., 5., 6.]]);
        let flat = tensor.flatten();
        let from_flat = Tensor::from_slice(&flat);
        assert_eq!(tensor, from_flat);
    }

    #[test]
    fn flatten_layer() {
        let layer = Layer::fully_connected(
            Tensor::new([[1., 2., 3.], [4., 5., 6.]]),
            Tensor::new([[1.], [2.]]),
            ActivationFunction::default(),
        );
        let flat = layer.flatten();
        let from_flat = Layer::from_slice(&flat);
        assert_eq!(layer, from_flat);
    }

    #[test]
    fn flatten_network() {
        let network = NeuralNetwork {
            input: Layer::fully_connected(
                Tensor::new([[1., 2., 3.], [4., 5., 6.]]),
                Tensor::new([[0.1], [0.2]]),
                ActivationFunction::default(),
            ),
            hidden: [
                Layer::fully_connected(
                    Tensor::new([[7., 8.], [9., 10.]]),
                    Tensor::new([[0.3], [0.4]]),
                    ActivationFunction::default(),
                ),
                Layer::fully_connected(
                    Tensor::new([[11., 12.], [13., 14.]]),
                    Tensor::new([[0.5], [0.6]]),
                    ActivationFunction::default(),
                )
            ],
            output: Layer::fully_connected(
                Tensor::new([[15., 16.]]),
                Tensor::new([[0.7]]),
                ActivationFunction::default(),
            )
        };
        let flattened = network.flatten();
        let expected = vec![
            1., 2., 3., 4., 5., 6., 0.1, 0.2, // input
            7., 8., 9., 10., 0.3, 0.4, // hidden 1
            11., 12., 13., 14., 0.5, 0.6, // hidden 2
            15., 16., 0.7 // output
        ];
        assert_eq!(flattened, expected);

        // Reconstruct network from flattened vector
        let reconstructed = NeuralNetwork::from_slice(&flattened);
        assert_eq!(reconstructed, network);
    }

    #[test]
    fn dot_product() {
        let t1 = Tensor::new([[1., 2., 3.], [4., 5., 6.]]);
        let t2 = Tensor::new([[7., 8.], [9., 10.], [11., 12.]]);
        let result = t1.dot(&t2);
        assert_eq!(result, Tensor::new([[58., 64.], [139., 154.]]));
    }

    #[test]
    fn relu() {
        let mut result = Tensor::new([[-1., 2., 3.], [4., -5., 6.]]);
        result.relu_mut();
        assert_eq!(result, Tensor::new([[0., 2., 3.], [4., 0., 6.]]));
    }

    #[test]
    fn add() {
        let t1 = Tensor::new([[1., 2., 3.], [4., 5., 6.]]);
        let t2 = Tensor::new([[7., 8., 9.], [10., 11., 12.]]);
        let result = t1 + t2;
        assert_eq!(result, Tensor::new([[8., 10., 12.], [14., 16., 18.]]));
    }

    #[test]
    fn fully_connected_layer_forward() {
        let layer = Layer::fully_connected(
            Tensor::new([[1., 2., 3.], [4., 5., 6.]]),
            Tensor::new([[1.], [2.]]),
            ActivationFunction::ReLU,
        );

        let ones = Tensor::ONES;
        let observed = layer.forward(&ones);
        assert_eq!(observed, Tensor::vector([7., 17.]));
    }

    #[test]
    fn test_mcculloch_pitt_network() {
        // network from https://blog.abhranil.net/2015/03/03/training-neural-networks-with-genetic-algorithms/
        let network: NeuralNetwork<2, 0, 1, 2> = NeuralNetwork {
            input: Layer::mcculloch_pitt(
                Tensor::new([[1.0, 1.0], [-1.0, -1.0]]),
                [0.5,-1.5]
            ),
            hidden: [],
            output: Layer::mcculloch_pitt(
                Tensor::new([[1.0, 1.0]]),
                [1.5],
            ),
        };

        for x in [0, 1] {
            for y in [0, 1] {
                let expected = if x == y { 0.0 } else { 1.0 };
                let observed = network.forward(&Tensor::vector([x as f64, y as f64]));
                assert_eq!(observed.value(), expected, "x={}, y={}", x, y);
            }
        }
    }

    #[test]
    fn test_train_x_plus_y() {
        let mut rng = ChaChaRng::seed_from_u64(100);
        let network = train_network::<0, 2>(&mut rng, 100, 1500, |x, y| x + y);
        validate_network(&mut rng, network, 100, |x, y| x + y);
    }

    #[test]
    fn test_train_x_mul_y() {
        let mut rng = ChaChaRng::seed_from_u64(100);
        let network = train_network::<0, 8>(&mut rng, 500, 5000, |x, y| x * y);
        validate_network(&mut rng, network, 100, |x, y| x * y);
    }

    fn random_xy(rng: &mut ChaChaRng) -> (f64, f64) {
        let x = rng.random_range(0. .. 1.);
        let y = rng.random_range(0. .. 1.);
        (x, y)
    }

    fn train_network<const HIDDEN: usize, const WIDTH: usize>(
        rng: &mut ChaChaRng,
        training_set_size: usize,
        epochs: usize,
        function: impl Fn(f64, f64) -> f64
    ) -> NeuralNetwork<2, HIDDEN, 1, WIDTH> {
        // Create a simple network: 2 inputs, 1 output
        let mut network: NeuralNetwork<2, HIDDEN, 1, WIDTH> = rng.random();
        network.set_activation(ActivationFunction::Sigmoid);


        // build training data from random numbers
        let mut inputs = vec![];
        let mut targets = vec![];
        for _ in 0..training_set_size {
            let (x, y) = random_xy(rng);
            inputs.push(Tensor::vector([x, y]));
            targets.push(Tensor::vector([function(x, y)]))
        }

        // Train the network
        network.train(&inputs, &targets, epochs, 0.01);

        network
    }

    fn validate_network<const HIDDEN: usize, const WIDTH: usize>(
        rng: &mut ChaChaRng,
        network: NeuralNetwork<2, HIDDEN, 1, WIDTH>,
        validation_set_size: usize,
        function: impl Fn(f64, f64) -> f64
    ) {
        let mut sum_error = 0.0;
        for _ in 0..validation_set_size {
            let (x, y) = random_xy(rng);
            let expected = function(x, y);
            let observed = network.forward(&Tensor::vector([x, y]));
            sum_error += (expected - observed.value()).abs();
        }

        let mean_error = sum_error / validation_set_size as f64;
        assert_relative_eq!(
            mean_error,
            0.0,
            epsilon = 0.01, // within 1%
        );
    }

}
