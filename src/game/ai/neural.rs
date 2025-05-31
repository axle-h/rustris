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

pub type TetrisNeuralNetwork = NeuralNetwork<12, 2, 1, 8>;

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
        let weights = [-1.056502, -0.796098, -0.053648, -0.071565, 0.083271, -0.099626, -0.699513, 0.031525, 0.390448, -0.810285, -0.593718, -0.271802, -0.388339, 0.481532, -0.469494, -0.062165, -0.126453, -0.739544, 0.487065, -0.426797, 0.537255, -0.659045, -0.702951, -0.325840, 0.646044, -0.858864, 0.242812, -0.667524, 1.075694, 0.799047, -0.507226, 0.965966, -0.409226, -0.439192, -0.607686, -0.425626, 0.793657, -0.655457, -1.159878, -0.900606, -0.111132, 0.276609, -0.412893, -0.818341, -0.540972, -0.981110, -0.858268, 0.797144, 0.012155, 0.780454, -0.277203, 0.225857, -0.155646, -0.832602, 0.984284, -0.174425, 0.715473, 0.984140, 0.036445, 0.004618, 0.208223, -0.175752, -0.159903, 0.317458, -0.028611, 0.402672, 0.179024, 0.503937, -0.986495, -0.030628, -0.829780, -0.972241, 0.220687, 0.408690, -0.955459, 0.693065, -0.907354, 1.031030, -0.662447, -0.920645, -0.514211, 0.259468, -0.756119, -0.393044, -1.023034, 0.282969, 0.177461, 0.591409, 0.392982, -0.323641, -0.174774, -0.517332, 0.840305, 0.486442, -0.385808, 0.185334, 0.992079, -0.391447, -0.961981, -0.877411, -0.842355, -0.654291, -0.492961, 0.078545, -1.021563, 0.557856, -0.320775, 0.322292, 0.899088, -0.297075, 0.591504, 0.249283, -1.017994, -0.106234, -0.140403, -0.505209, 0.665385, 0.148742, -0.444840, 0.928351, 0.391248, 0.047255, 0.684281, 0.276429, -0.018718, -0.048387, 0.049562, -0.551123, 0.519659, 0.107659, -0.561146, 0.086873, -0.534457, -0.752945, -0.139511, -0.450041, 0.338714, -0.509794, -0.492961, -1.025787, -0.503774, -0.648824, -0.916174, -0.998842, 0.081754, -0.905768, 0.640945, 0.476688, -0.236073, 0.314308, -0.027294, -0.382147, -0.382222, 0.546820, -0.600452, -0.310468, -0.162458, 0.634087, -0.971095, -0.669232, -0.721461, 0.711010, -0.223608, 0.993250, -0.702583, -0.183765, -0.291351, 0.097875, -0.037718, 0.677666, 0.275051, 0.794521, -0.055689, 0.671304, -0.528407, 0.570049, 0.346303, 0.278980, 0.415343, -0.510400, -0.689614, 0.135139, 1.065554, 0.365030, -0.729294, 0.796826, -0.075664, -0.637062, -0.738646, -0.480627, -0.949761, -0.163118, -0.858783, -0.220768, -0.406520, 0.070973, 0.807334, 0.823452, 0.111880, -0.425820, 0.356093, -0.298487, 0.602199, 0.756252, 0.215868, 0.360387, -0.819726, -0.024571, 1.019055, -0.531909, -0.107887, 0.268430, 0.449944, 0.515003, -0.526843, 0.578167, 0.531330, -0.343960, 0.493863, -0.857138, 0.609539, -0.528394, -0.197799, -0.760333, 0.534011];
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
