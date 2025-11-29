use std::cell::RefCell;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};
use std::rc::Rc;

// The core building block: A Value stores data and its gradient.
// We use Rc<RefCell<...>> to allow shared ownership and mutation, 
// which is necessary for building a computation graph (DAG).
#[derive(Clone)]
pub struct Value(Rc<RefCell<ValueInternal>>);

struct ValueInternal {
    data: f64,
    grad: f64,
    _prev: Vec<Value>,
    _op: String,
    _backward: Option<Box<dyn Fn(&ValueInternal)>>, // Closure to propagate gradients
}

impl Value {
    pub fn new(data: f64) -> Value {
        Value(Rc::new(RefCell::new(ValueInternal {
            data,
            grad: 0.0,
            _prev: vec![],
            _op: "".to_string(),
            _backward: None,
        })))
    }

    pub fn data(&self) -> f64 {
        self.0.borrow().data
    }

    pub fn grad(&self) -> f64 {
        self.0.borrow().grad
    }

    pub fn zero_grad(&self) {
        self.0.borrow_mut().grad = 0.0;
    }

    // Optimization Step
    pub fn apply_gradient_descent(&self, learning_rate: f64) {
        let mut internal = self.0.borrow_mut();
        internal.data -= learning_rate * internal.grad;
    }

    // Backward pass: Topological sort to ensure we process dependencies after parents
    pub fn backward(&self) {
        let mut topo = vec![];
        let mut visited = std::collections::HashSet::new();
        
        fn build_topo(v: &Value, visited: &mut std::collections::HashSet<usize>, topo: &mut Vec<Value>) {
            let ptr = v.0.as_ptr() as usize;
            if !visited.contains(&ptr) {
                visited.insert(ptr);
                for child in &v.0.borrow()._prev {
                    build_topo(child, visited, topo);
                }
                topo.push(v.clone());
            }
        }

        build_topo(self, &mut visited, &mut topo);

        // Seed the gradient of the output node (usually loss) to 1.0
        self.0.borrow_mut().grad = 1.0;

        // Go backwards
        for node in topo.iter().rev() {
            let internal = node.0.borrow();
            if let Some(backward_fn) = &internal._backward {
                backward_fn(&internal);
            }
        }
    }

    // Operations -------------------------------------------------------------

    pub fn pow(&self, other: f64) -> Value {
        let out = Value::new(self.data().powf(other));
        
        let self_clone = self.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone()];
        out.0.borrow_mut()._op = format!("^{}", other);
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            let mut self_int = self_clone.0.borrow_mut();
            // d/dx (x^n) = n * x^(n-1)
            self_int.grad += other * self_int.data.powf(other - 1.0) * out_int.grad;
        }));
        
        out
    }

    pub fn tanh(&self) -> Value {
        let x = self.data();
        let t = (x.exp() - (-x).exp()) / (x.exp() + (-x).exp());
        let out = Value::new(t);

        let self_clone = self.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone()];
        out.0.borrow_mut()._op = "tanh".to_string();
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            let mut self_int = self_clone.0.borrow_mut();
            // d/dx tanh(x) = 1 - tanh(x)^2
            self_int.grad += (1.0 - out_int.data.powi(2)) * out_int.grad;
        }));

        out
    }
    
    pub fn relu(&self) -> Value {
        let out = Value::new(if self.data() < 0.0 { 0.0 } else { self.data() });
        let self_clone = self.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone()];
        out.0.borrow_mut()._op = "ReLU".to_string();
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            let mut self_int = self_clone.0.borrow_mut();
            if out_int.data > 0.0 {
                self_int.grad += out_int.grad;
            }
        }));
        
        out
    }

    pub fn exp(&self) -> Value {
        let out = Value::new(self.data().exp());
        let self_clone = self.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone()];
        out.0.borrow_mut()._op = "exp".to_string();
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            let mut self_int = self_clone.0.borrow_mut();
            // d/dx e^x = e^x
            self_int.grad += out_int.data * out_int.grad;
        }));
        
        out
    }

    pub fn log(&self) -> Value {
        let out = Value::new(self.data().ln());
        let self_clone = self.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone()];
        out.0.borrow_mut()._op = "log".to_string();
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            let mut self_int = self_clone.0.borrow_mut();
            // d/dx ln(x) = 1/x
            self_int.grad += (1.0 / self_int.data) * out_int.grad;
        }));
        
        out
    }

    pub fn sigmoid(&self) -> Value {
        // σ(x) = 1 / (1 + e^(-x))
        let s = 1.0 / (1.0 + (-self.data()).exp());
        let out = Value::new(s);
        let self_clone = self.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone()];
        out.0.borrow_mut()._op = "sigmoid".to_string();
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            let mut self_int = self_clone.0.borrow_mut();
            // d/dx σ(x) = σ(x) * (1 - σ(x))
            self_int.grad += out_int.data * (1.0 - out_int.data) * out_int.grad;
        }));
        
        out
    }

    /// Division: self / other
    pub fn div(&self, other: &Value) -> Value {
        // a/b = a * b^(-1)
        let inv = other.pow(-1.0);
        self.clone() * inv
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, other: Value) -> Value {
        let out = Value::new(self.data() + other.data());
        
        let self_clone = self.clone();
        let other_clone = other.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone(), other.clone()];
        out.0.borrow_mut()._op = "+".to_string();
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            self_clone.0.borrow_mut().grad += out_int.grad;
            other_clone.0.borrow_mut().grad += out_int.grad;
        }));
        
        out
    }
}

impl Add<f64> for Value {
    type Output = Value;
    fn add(self, other: f64) -> Value {
        self + Value::new(other)
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, other: Value) -> Value {
        let out = Value::new(self.data() * other.data());
        
        let self_clone = self.clone();
        let other_clone = other.clone();
        
        out.0.borrow_mut()._prev = vec![self.clone(), other.clone()];
        out.0.borrow_mut()._op = "*".to_string();
        let self_data = self_clone.data();
        let other_data = other_clone.data();
        out.0.borrow_mut()._backward = Some(Box::new(move |out_int| {
            self_clone.0.borrow_mut().grad += other_data * out_int.grad;
            other_clone.0.borrow_mut().grad += self_data * out_int.grad;
        }));
        
        out
    }
}

impl Mul<f64> for Value {
    type Output = Value;
    fn mul(self, other: f64) -> Value {
        self * Value::new(other)
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Value {
        self * -1.0
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, other: Value) -> Value {
        self + (-other)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Value(data={:.4}, grad={:.4})", self.data(), self.grad())
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_value_creation() {
        let v = Value::new(3.14);
        assert!(approx_eq(v.data(), 3.14));
        assert!(approx_eq(v.grad(), 0.0));
    }

    #[test]
    fn test_addition() {
        let a = Value::new(2.0);
        let b = Value::new(3.0);
        let c = a + b;
        assert!(approx_eq(c.data(), 5.0));
    }

    #[test]
    fn test_multiplication() {
        let a = Value::new(2.0);
        let b = Value::new(3.0);
        let c = a * b;
        assert!(approx_eq(c.data(), 6.0));
    }

    #[test]
    fn test_subtraction() {
        let a = Value::new(5.0);
        let b = Value::new(3.0);
        let c = a - b;
        assert!(approx_eq(c.data(), 2.0));
    }

    #[test]
    fn test_negation() {
        let a = Value::new(5.0);
        let b = -a;
        assert!(approx_eq(b.data(), -5.0));
    }

    #[test]
    fn test_pow() {
        let a = Value::new(2.0);
        let b = a.pow(3.0);
        assert!(approx_eq(b.data(), 8.0));
    }

    #[test]
    fn test_tanh() {
        let a = Value::new(0.0);
        let b = a.tanh();
        assert!(approx_eq(b.data(), 0.0));
        
        let c = Value::new(1.0);
        let d = c.tanh();
        assert!(d.data() > 0.7 && d.data() < 0.8); // tanh(1) ≈ 0.7616
    }

    #[test]
    fn test_relu() {
        let a = Value::new(-5.0);
        let b = a.relu();
        assert!(approx_eq(b.data(), 0.0));
        
        let c = Value::new(5.0);
        let d = c.relu();
        assert!(approx_eq(d.data(), 5.0));
    }

    #[test]
    fn test_backward_simple() {
        // f = a * b + c
        // df/da = b, df/db = a, df/dc = 1
        let a = Value::new(2.0);
        let b = Value::new(3.0);
        let c = Value::new(4.0);
        
        let ab = a.clone() * b.clone();
        let f = ab + c.clone();
        
        f.backward();
        
        assert!(approx_eq(a.grad(), 3.0), "da = {} expected 3.0", a.grad());
        assert!(approx_eq(b.grad(), 2.0), "db = {} expected 2.0", b.grad());
        assert!(approx_eq(c.grad(), 1.0), "dc = {} expected 1.0", c.grad());
    }

    #[test]
    fn test_backward_chain() {
        // f = (a + b) * c
        // df/da = c, df/db = c, df/dc = (a + b)
        let a = Value::new(2.0);
        let b = Value::new(3.0);
        let c = Value::new(4.0);
        
        let sum = a.clone() + b.clone();
        let f = sum * c.clone();
        
        f.backward();
        
        assert!(approx_eq(a.grad(), 4.0), "da = {} expected 4.0", a.grad());
        assert!(approx_eq(b.grad(), 4.0), "db = {} expected 4.0", b.grad());
        assert!(approx_eq(c.grad(), 5.0), "dc = {} expected 5.0", c.grad());
    }

    #[test]
    fn test_backward_pow() {
        // f = a^2
        // df/da = 2a
        let a = Value::new(3.0);
        let f = a.clone().pow(2.0);
        
        f.backward();
        
        assert!(approx_eq(f.data(), 9.0));
        assert!(approx_eq(a.grad(), 6.0), "da = {} expected 6.0", a.grad());
    }

    #[test]
    fn test_backward_tanh() {
        // f = tanh(a)
        // df/da = 1 - tanh(a)^2
        let a = Value::new(1.0);
        let f = a.clone().tanh();
        
        f.backward();
        
        let expected_grad = 1.0 - f.data().powi(2);
        assert!(approx_eq(a.grad(), expected_grad), 
                "da = {} expected {}", a.grad(), expected_grad);
    }

    #[test]
    fn test_zero_grad() {
        let a = Value::new(2.0);
        let b = Value::new(3.0);
        let c = a.clone() * b.clone();
        c.backward();
        
        assert!(approx_eq(a.grad(), 3.0));
        
        a.zero_grad();
        assert!(approx_eq(a.grad(), 0.0));
    }

    #[test]
    fn test_gradient_descent_step() {
        let w = Value::new(5.0);
        
        // Simulate: loss = (w - 3)^2, we want w -> 3
        let target = Value::new(3.0);
        let diff = w.clone() - target;
        let loss = diff.pow(2.0);
        
        loss.backward();
        
        // grad = 2 * (w - 3) = 2 * 2 = 4
        assert!(approx_eq(w.grad(), 4.0));
        
        // Apply gradient descent: w = w - lr * grad
        w.apply_gradient_descent(0.1);
        
        // new_w = 5 - 0.1 * 4 = 4.6
        assert!(approx_eq(w.data(), 4.6), "w = {} expected 4.6", w.data());
    }

    #[test]
    fn test_mlp_forward_backward() {
        // Simple 2-layer network: input -> hidden -> output
        // This mimics what lesson_03 does
        let x = Value::new(1.0);
        let w1 = Value::new(0.5);
        let b1 = Value::new(0.1);
        let w2 = Value::new(0.3);
        let b2 = Value::new(0.2);
        
        // Forward
        let hidden = (x.clone() * w1.clone() + b1.clone()).tanh();
        let output = hidden.clone() * w2.clone() + b2.clone();
        
        // Loss (simple MSE with target 1.0)
        let target = Value::new(1.0);
        let loss = (output - target).pow(2.0);
        
        // Backward
        loss.backward();
        
        // All weights should have gradients
        assert!(w1.grad() != 0.0, "w1 should have gradient");
        assert!(b1.grad() != 0.0, "b1 should have gradient");
        assert!(w2.grad() != 0.0, "w2 should have gradient");
        assert!(b2.grad() != 0.0, "b2 should have gradient");
    }
}
