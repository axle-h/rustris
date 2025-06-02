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
        let weights = [0.014221, 0.196371, -0.519972, -0.687836, 0.491826, 0.083057, 0.804071, 0.366862, 0.449301, -0.092942, 0.111290, -0.117743, 0.719734, 0.076960, -0.050059, 0.373466, 1.052553, 0.466046, 0.995127, 1.396157, 0.774764, -0.323014, 0.491714, 0.466619, -0.592883, 0.116033, -0.842174, -0.269453, 0.507198, 0.394632, 0.467178, 0.580793, 0.517937, -0.731222, 0.432154, -0.149729, 0.701156, 0.762523, -0.522608, -0.750479, -0.121323, -0.059789, 0.174706, 0.980512, -0.712510, -0.722521, 0.890123, 0.909904, 0.537127, 0.536344, -0.662542, -0.826439, 0.760746, 0.533474, 0.672621, 0.431103, 0.934090, -1.073290, 0.369822, 0.456271, 0.806399, -0.001529, 0.110056, 0.504864, 0.218955, 0.535488, -0.042738, 0.021191, -0.510973, 0.241044, 0.032625, 0.058169, -0.060441, 0.223064, 0.278424, 0.272576, 0.599554, -0.214345, -1.064583, -0.033588, 0.336591, -0.014331, -0.677627, 0.392318, -0.064443, -0.920246, 0.460099, -0.724259, -0.576332, -0.210911, 0.064286, 0.830603, 0.588695, -0.578248, 0.303386, 0.502529, 0.179193, 0.497803, 0.961047, 0.434169, 0.665037, 0.197284, 0.836098, 0.861664, 0.246485, -0.380809, 0.218532, 0.090363, 0.934470, 0.888854, -0.653779, -0.088016, 0.621891, -0.220698, -0.719277, -0.302752, -0.504095, 0.517550, -0.566380, -1.050309, 0.189782, -0.128207, -0.916762, -1.388439, 0.523895, -0.799172, 0.960857, -0.528679, 0.930359, 1.017823, -0.879644, -0.307906, -0.663272, -0.467943, -0.260593, 0.562521, -0.718670, -0.113654, -0.218088, 0.464580, 0.877928, 0.608214, -0.350296, 0.004955, 0.678670, -0.889420, -1.167614, 0.992364, 0.356650, 1.124027, 0.572896, -0.608731, 0.462600, 0.771149, 0.722378, 0.758096, -0.171630, -0.230322, -0.104559, -1.014183, -0.717465, 0.417704, -0.135484, 0.431218, 1.355849, -0.401280, -0.403895, -0.195242, -0.092762, -0.684936, -0.273213, -0.024098, -0.591690, 0.164091, 0.931319, -0.952742, 0.437499, -1.027098, -0.124029, 0.874769, -0.729961, 0.911621, -0.297900, -0.984520, -0.593371, -0.009693, 0.739124, 0.103262, -0.478865, 0.474621, -0.780262, 0.748805, -0.787955, 0.050923, -0.041811, 0.742508, 0.309933, -0.516169, 0.848043, -0.314138, 0.317813, -0.457331, -0.159448, 0.214445, -0.371265, -0.561276, -0.598798, -0.964726, 0.448621, 0.712034, -0.733141, -0.642462, 0.853621, 0.382279, 0.085525, -0.150238, -0.862965, -0.857184, 0.489831, -0.893524, 0.614685, -0.333870, -0.659463, 0.166089, 0.454030, -0.062414, 0.432538, -0.747462, 0.021515, 0.765967, -0.545504, -0.771505, -0.006119, 0.932966, -1.104343, 0.930963, -0.213687, -0.235967, -0.370001, 0.554925, 0.530900, -0.324167, -0.681841, 0.688199, -0.530423, 0.607147, 0.210525, -0.845430, 0.348742, -0.165219, 0.341024, 0.712464, -0.177078, -0.092281, -0.794123, 0.839958, -0.521162];
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
